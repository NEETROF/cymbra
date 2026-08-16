## Context

`ScoreService` (proto `backend/music/proto/score.proto`, impl
`backend/music/src/grpc.rs`) already runs behind the strict `AuthInterceptor`
(`backend/platform/src/interceptor.rs`), so every handler has a verified
`AuthIdentity { user_id, audience, roles }` in its request extensions, pulled out
today via the `owner()` / `identity()` helpers (`grpc.rs:63-77`). What's missing is
any ceiling on how much a single identity can pull:

- `SearchCatalog` (empty query = browse-all, paginated) — the enumeration surface.
- `GetCatalogScoreBytes` / `GetRatingPreviewBytes` — the raw-MusicXML egress, the
  primary siphoning target. Bytes are streamed through the backend from
  `LocalFirstStore`; there is no signed-URL indirection, so each handler is a single
  choke point.

Two guardrail precedents already exist in the codebase:
- **`cymbra_platform::ratelimit::check(cache, scope, subject, max, window)`**
  (`backend/platform/src/ratelimit.rs`) — a windowed Redis `INCR`+TTL counter keyed
  `rl:{scope}:{subject}`, returning `AppError::ResourceExhausted` on breach. Used
  today only for auth email/sign-in throttles (`backend/auth/src/module.rs`).
- **The upload quota** (`backend/music/src/module.rs:151-161`) — a per-owner
  `count_recent(owner_id, window_days) >= quota_max` DB check. This is the read-side
  analogue we're mirroring on egress.

Redis (`Cache` trait, `RedisCache`) is always-on and constructed unconditionally in
`backend/server/src/main.rs:61`, but is **not currently injected into
`ScoreModule`** — that is the one wiring gap. Config knobs live beside the existing
throttle config in `backend/platform/src/config.rs`. Runtime tunability rides the
existing feature-flags / config platform (`cymbra-feature-flags`).

## Goals / Non-Goals

**Goals:**
- Make it infeasible for one authenticated token to bulk-download the catalog,
  whether via a fast burst or a slow sustained drip.
- **Never penalise a legitimate user whose download volume tracks their engagement**
  (playing **or** rating) — the guardrail targets the "download a lot, engage with
  nothing" profile, not engaged users.
- Reuse the existing rate-limit primitive and Redis counter store — no new
  dependency, no new service.
- Keep thresholds operator-tunable at runtime, with a kill-switch, and with safe
  non-zero defaults.
- Fail closed on egress (reject on breach) but degrade gracefully in the UI.

**Non-Goals:**
- Per-IP / edge rate limiting, WAF, CAPTCHA, or bot fingerprinting.
- DRM, watermarking, or per-download licensing enforcement of score content.
- Owner-scoped RPCs that only expose the caller's *own* data (`ListMyScores`,
  `GetScoreBytes`, `ListSavedCatalogScores`) — bounded by construction, out of scope.
- Retroactive detection/banning of past scrapers (this is prevention, not forensics).

## Decisions

### Decision 1: Enforce per-user, keyed on the token subject (`user_id`)
The threat in the request is explicitly "avec son token d'identification" — a valid
identity driving a bot. The natural and abuse-resistant key is therefore
`AuthIdentity.user_id`, already available in every handler. Per-IP was rejected:
users legitimately share NATs/CG-NAT, and a determined scraper rotates IPs but
cannot cheaply rotate authenticated identities (account creation is gated). Per-IP
can be layered at the edge later without conflicting with this.

### Decision 2: Reuse `ratelimit::check` + Redis, not a new tower layer
A tonic/tower layer would be generic but has no access to the decoded per-user
identity or per-method policy without extra plumbing, and would sit awkwardly beside
the interceptor. Calling `ratelimit::check` **inside the handlers** (where identity
is already extracted) is the smallest, most legible change and matches how auth
throttles are already written. Alternative considered: a middleware keyed on the
interceptor-injected identity — deferred as over-engineering for five methods.

### Decision 3: Download guardrail = burst cap + engagement-aware volume allowance
A flat daily download cap penalises the wrong people: a genuinely engaged user who
opens many scores looks the same to a fixed counter as a scraper. The distinguishing
signal is **engagement** — a legitimate user downloads a score in order to *use* it
(play it, or rate it in the swipe deck), a bot downloads and does neither. So the
volume guardrail is adossed to engagement, not a blind cap:

- **Tier 1 — burst cap** (pure rate): a short-window `ratelimit::check`
  (scope `cat_dl_burst`) that applies regardless of engagement. Nobody legitimately
  opens hundreds of files in seconds; this stops the fast flood cheaply.
- **Tier 2 — engagement-aware volume allowance** over a rolling window:
  `effective = min(hard_ceiling, base_floor + k * engagement_in_window)`, where
  `engagement = play sessions + score ratings` in the window (`PlayRepo` +
  `ScoreRatingRepo`). The request passes only if the user's download count in the
  window is below `effective`. A user whose downloads track their engagement is
  never blocked; a download-heavy/engagement-light profile stops at `base_floor`;
  `hard_ceiling` is an absolute anti-abuse backstop.

**Why ratings count too (raised in review).** The rating deck ("Tinder for scores")
is a core engagement loop: a user listens to ~25 notes (or the whole score if
shorter), then rates it. That preview goes through `GetRatingPreviewBytes`, which
returns the **full** MusicXML — so it is both a legitimate rating action *and* a
full-corpus egress vector a bot could abuse. We therefore keep it under the download
guardrail, but fold ratings into the engagement signal: each rating spends one
download and earns `k` headroom, so a genuine rater is net-positive (self-sustaining
for `k ≥ 1`) while a preview-only scraper (downloads, never rates) is not. Counting
only plays would have wrongly throttled heavy raters who play little.

Rationale for `min(ceiling, floor + k·engagement)`: `base_floor` keeps brand-new
users unblocked; `k·engagement` makes the allowance elastic to real activity;
`hard_ceiling` bounds egress no matter how the signal is gamed (faking ratings means
driving real `SubmitScoreRating` calls — validated, one row per score, far costlier
and more detectable than raw downloads). `k`, `base_floor`, `hard_ceiling`, and the
window are config knobs (Decision 6).

Implementation: the burst tier is a `ratelimit::check`; the volume tier compares a
per-user rolling download counter against `effective`, computed from
`engagement_in_window` = in-window `PlayRepo::session_points` count +
`ScoreRatingRepo::count_recent_by_user`. To avoid DB reads on every download, the
combined engagement is cached per user in the shared cache with a short TTL (the
allowance only needs to be approximately fresh), failing safe toward 0 (the base
floor) on any error. Enumeration (its own requirement) stays a single request-rate
window; the page-size clamp already bounds per-request volume.

### Decision 4: Exempt the back-office audience + music-scope admins; music-app moderators subject
Two exemptions, both short-circuiting the guard (no limit) before any counter work,
since the identity is already in the handler:

1. **Back-office audience.** `ScoreService` is a single service mounted once, so the
   gRPC-web curator console (change `#155`, which downloads a catalog score's
   MusicXML by reusing `GetCatalogScoreBytes`) hits the same limiter as the music
   app. The back-office is a *different audience* — a trusted, CORS-gated admin
   surface — and its users never play, so the play-aware volume allowance would peg
   them at the base floor for no security benefit. The scrape threat model is the
   music-app token, not the console. So `id.audience == BACKOFFICE_AUDIENCE` is
   exempt regardless of role (including a back-office `moderator`).

2. **Music-scope admin.** Roles are **scope-aware** (change `scope-aware-role-admin`):
   `AuthIdentity` carries `roles_by_scope` + `has_role_in_scope(scope, role)` (true
   for the scope itself or the `global` break-glass). The catalog is a music-domain
   resource, so the guard exempts precisely `id.has_role_in_scope("music", "admin")`
   — a `music/admin` or a `global/admin` — and **not** a scope-agnostic `is_admin()`
   (which is true for an `admin` held in *any* scope, so a `live`-only admin would
   wrongly bypass; the scope-matched check closes that).

On the **music-app audience**, moderators are deliberately **not** exempt: their role
authorises content review, not corpus egress, and exempting them there would reopen
the exact scrape vector via a privileged account. (A moderator acting through the
back-office console is exempt by exemption #1, which is where moderation actually
happens.)

Note on token shape: `ScoreService` is mounted under the **music audience**, and a
music-audience access token is minted with `scopes = [global, music]`
(`backend/auth/src/module.rs:147`), so its `roles_by_scope` is populated with those
keys — `has_role_in_scope("music", …)` is reliable here (not a legacy flat token
where `roles_by_scope` is empty). Using the scope-matched primitive rather than the
audience-scoped flat `roles` also keeps the check correct if the handler is ever
reached under a multi-scope audience. Alternatives considered — `is_admin()`
(scope-agnostic, over-broad) and `require_moderator_or_admin` (widens the bypass to
moderators) — both rejected.

### Decision 5: A dedicated `CatalogAccessLimiter`; enforce in `grpc.rs`
Rather than thread `Arc<dyn Cache>` through `ScoreModule` (which owns only
score-storage logic and has no caller identity), a dedicated `CatalogAccessLimiter`
holds the `Cache` + play port + thresholds and exposes `check_download` /
`check_enumeration`. `ScoreGrpc` holds it optionally (`with_limiter`) and calls it
right after `identity()` resolves the caller, before any storage read — so a
rejected request never touches the object store. This keeps the identity-aware guard
at the gRPC layer (where identity lives) and leaves `ScoreModule` untouched. Unit
tests construct `ScoreGrpc` with no limiter (un-limited) and a separate limited
variant for the guardrail tests.

### Decision 6: `RESOURCE_EXHAUSTED` contract + config placement
Reuse `AppError::ResourceExhausted` → gRPC `RESOURCE_EXHAUSTED`, identical to auth
throttles, so client error mapping is uniform. Thresholds (window + max per tier)
join the existing throttle knobs in `config.rs` and are overridable at runtime
through the config/feature-flags platform, including a boolean kill-switch. Defaults
are non-zero and generous enough for real human use (see Open Questions for values).

### Decision 7: Observability via the existing RED metrics
`ObserveLayer` already labels `rpc.server.requests` by method + status, so
`RESOURCE_EXHAUSTED` counts per method fall out for free — a scraping user surfaces
as a sustained rejection rate on the download methods. We add no per-user label
(cardinality/PII); operator alerting keys on the method-level rejection rate.

## Risks / Trade-offs

- **Thresholds too low → legitimate power users blocked** → The play-aware
  allowance is designed precisely to avoid this: engaged users earn headroom from
  play. Plus non-zero generous defaults, runtime-tunable, and the kill-switch;
  start permissive and tighten with real traffic data. Ship with dashboards on
  rejection rate before lowering.
- **Scraper games the engagement signal (fakes plays/ratings to earn headroom)**
  → Faking engagement means driving real `PlayService` sessions or `SubmitScoreRating`
  calls (validated, one row per score), far costlier and more detectable than raw
  downloads, and `hard_ceiling` caps total egress regardless. Keep `k` modest so a
  little fake activity can't unlock a lot of download.
- **Engagement data adds a dependency / staleness on the download path** → Read the
  combined `engagement_in_window` (plays + ratings) from a short-TTL cache, not live
  DB reads per download; the allowance only needs to be approximately fresh. If the
  data is unavailable, each source contributes 0 and the caller falls back toward the
  `base_floor` (fail-safe: legitimate light users still work, scrapers still capped)
  rather than blocking or opening wide.
- **Redis unavailable → what happens to egress?** → `ratelimit::check` depends on
  the cache; decide fail-open vs fail-closed on cache error explicitly (see Open
  Questions). Leaning fail-open for availability, since Redis is declared disposable
  and the guardrail is anti-abuse, not a security boundary — a brief Redis outage
  shouldn't take down catalog reads.
- **Counters are approximate** (fixed-window `INCR`+TTL allows a 2× burst across a
  window boundary) → Acceptable; the two-tier design bounds total volume regardless,
  and exactness isn't required for an anti-scrape guardrail.
- **Scraper spreads load across many accounts** → Out of scope here; mitigated by
  gated account creation and by the existing upload/behavioral signals. Per-user
  caps still raise the cost linearly with accounts.
- **Client retry logic could amplify load on rejection** → Spec requires no retry
  storm; verify the score client maps `RESOURCE_EXHAUSTED` to a terminal, non-retried
  UI state.

## Migration Plan

1. Add config knobs + defaults in `config.rs`; wire `Arc<dyn Cache>` into
   `ScoreModule::new` and `main.rs`.
2. Implement the guard helper(s) calling `ratelimit::check`; invoke in the five
   handlers. Ship **enabled with permissive defaults** (kill-switch on/available).
3. Map `RESOURCE_EXHAUSTED` in the Flutter score client to a localized message.
4. Observe rejection-rate metrics in prod; tighten thresholds at runtime as real
   traffic is understood.
5. Rollback: flip the kill-switch (runtime) — no redeploy — if the guardrail
   misbehaves.

## Open Questions

- **Default threshold values**: download burst (max/window), volume-allowance
  `base_floor`, engagement multiplier `k`, `hard_ceiling`, allowance window, and
  enumeration rate — pick starting numbers from expected human usage (e.g. a rating
  session rarely previews dozens of scores/minute). Needs a sanity check against
  per-user download / play / rating telemetry if available.
- **Engagement signal composition**: currently `play-session count + rating count`
  in the window. Open refinement: whether to weight ratings vs plays differently, or
  use distinct-scores rather than counts. Counts are the simplest defensible start.
- **Fail-open vs fail-closed on Redis error** — proposed fail-open (serve, log);
  confirm this matches the product's abuse-vs-availability posture.
- Should the enumeration guardrail also cover any other browse RPC not listed
  (e.g. facet endpoints) if they can enumerate?
