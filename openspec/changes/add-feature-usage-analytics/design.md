## Context

Cymbra ships a Flutter app across iOS, Android, macOS, Windows, Linux and web,
plus a Vue back-office, backed by a Rust engine + backend on Postgres with an
`sqlxmq`/`cymbra-worker` job system and a `cymbra-feature-flags` config platform.
Product decisions are currently blind: we cannot say how many users are active on
a period, on which platforms/device classes, or which features they use — the only
signals are incidental logs and feature-specific tables that cannot be sliced
freely and miss purely client-side actions.

Constraints that shape the design:
- **EU/RGPD residency.** Data lives on OVH; the product markets EU data residency.
  A US-hosted third-party (Firebase/Google Analytics) is off the table on both
  residency and CNIL grounds.
- **Cross-platform including desktop.** FlutterFire analytics/crashlytics do not
  support Windows/Linux desktop, which we ship — so a Google SDK cannot even cover
  our platforms.
- **Small scale.** ~<50 users over 6 months today; volumetry is not a real
  constraint, but the design must stay bounded as it grows.
- **Existing seams to reuse.** `cymbra-feature-flags` (runtime config + kill
  switch), `job-infrastructure` (recurring jobs), the injectable-service pattern
  (Flutter Riverpod, Vue Pinia client seam), mockall/mockito test doubles.

The two product questions to answer over a filterable period: (1) how many unique
users, by platform and device class; (2) which actions were performed, filterable
in any combination.

## Goals / Non-Goals

**Goals:**
- First-party usage telemetry: capture → ingest → store → aggregate, entirely on
  our own infrastructure, no third-party SDK.
- Exact distinct-user counts over an arbitrary window, split by platform/device.
- Free-form cross-dimension filtering of action activity over a period.
- Bounded storage: short-lived raw detail + tiny permanent aggregates.
- Privacy-preserving by construction: pseudonymous, period-salted, retention-bounded.
- Reuse the feature-flag and worker platforms; no new infrastructure to operate.

**Non-Goals:**
- **Crash / error reporting** — a separate future concern (likely self-hosted or
  EU-cloud Sentry); explicitly out of scope here.
- Per-user behavioural profiling, funnels, cohorts, or cross-period user journeys.
- A general-purpose event lake / freeform logging. The taxonomy is curated (a
  client-owned registry), even though it travels as a validated string.
- A dedicated analytics datastore (ClickHouse etc.) — unnecessary at our scale.
- Real-time dashboards; daily aggregation granularity is sufficient.

## Decisions

### D1 — First-party pipeline over a third-party SDK

Build a small in-house pipeline rather than adopt Firebase/GA or self-host
PostHog/Aptabase.
- *Why:* residency (OVH/EU) and platform coverage (desktop) rule out Firebase.
  Self-hosting PostHog/Aptabase is *more* ops (a whole service + its DB + upgrades)
  and its dashboards live outside our back-office — contrary to the goal of "stats
  in the BO". A first-party pipeline is ~one migration + one RPC + two jobs + one
  BO screen, reusing platforms we already run.
- *Alternatives:* Firebase Analytics (rejected: US transfer, no desktop);
  Aptabase self-hosted (rejected: separate service + separate UI, not in the BO);
  PostHog self-hosted (rejected: heavy to operate, oversized for the questions).

### D2 — Period-salted pseudonymous `user_bucket`

Identify users by `hash(user_id, period_salt)` where the salt rotates monthly,
instead of a stable per-user hash.
- *Why:* both product questions are **per-period aggregates** — neither needs to
  follow a user across periods. Rotating the salt lets us count distinct users
  within a period while making cross-period correlation impossible, which is the
  lightest defensible RGPD posture and removes long-term profiling risk. The raw
  `user_id` is never stored on a usage row.
- *Alternatives:* stable hash (rejected: reconstructable individual timeline for
  no product benefit); per-install random UUID (rejected: user chose per-account
  for cross-device; also double-counts one user on two devices); no id at all
  (rejected: cannot answer "% / count of users").
- *Consequence:* distinct users are exact **within** a period; a range spanning
  months sums per-month distincts (a user active in two months counts twice). This
  is an accepted, documented trade-off.

### D3 — Three-tier storage: raw + two permanent aggregates

`usage_events` (raw, all dimensions, TTL'd) + `usage_action_daily` (volume) +
`usage_user_daily` (per-user daily presence).
- *Why:* the critical pitfall is that **daily `COUNT(DISTINCT)` cannot be summed**
  into a period distinct count. A volume-only rollup answers "how many actions" but
  not "how many users". `usage_user_daily` stores presence at `(day, user_bucket,
  platform, device_class)` so a `COUNT(DISTINCT user_bucket)` over any window is
  exact and cheap. `usage_action_daily` answers action volumes filterable on its
  frozen dimensions forever. Raw powers truly arbitrary slicing within the
  retention window.
- *Aggregate grain is frozen now:* `action, platform, device_class, app_version,
  locale` — all low-cardinality, so aggregates stay tiny at any scale. Any
  dimension not in the grain is only slice-able on raw (within retention); adding a
  historical filter dimension later means widening the grain now.
- *Alternatives:* single volume rollup (rejected: no unique users); HyperLogLog
  sketches per day (rejected: `hll` extension may be unavailable on managed OVH PG,
  and exact is affordable at our scale); keep raw forever (rejected: unbounded
  growth + heavier RGPD retention posture).

### D4 — Retention TTL as a runtime config; purge + rollup as worker jobs

Raw retention is a `cymbra-feature-flags` config value (default 6 months), edited
in the BO. Two recurring jobs on the existing worker: a **rollup** that folds each
closed day into the aggregates, and a **purge** that deletes raw older than the
TTL. Rollup is ordered before purge.
- *Why:* reuses `job-infrastructure` (no new scheduler) and `runtime-feature-flags`
  (retune retention without redeploy). The multi-month margin between "day closed →
  aggregated" and "TTL expiry → purged" makes the ordering safe with wide slack.
  Rollup is idempotent (upsert keyed by day+grain) so re-runs never double-count.
- *Alternatives:* Postgres partition-drop by day (viable later for scale; more
  migration machinery than needed now — a `DELETE ... WHERE occurred_at < ...` is
  enough at our volume); hard-coded TTL (rejected: user wants to tune it in the BO).

### D5 — Batched, best-effort, kill-switchable ingestion

A single `ReportEvents` RPC takes a batch. The Flutter client buffers events in a
local offline queue and flushes periodically; failures are silent and retried; a
`cymbra-feature-flags` kill-switch disables collection without a release.
- *Why:* telemetry must never degrade UX or block on the network. Batching keeps
  RPC overhead negligible. The client derives `platform`/`device_class`. The server
  validates the action's *shape* (not membership) and skips malformed events without
  failing the batch. The kill-switch is the safety valve if anything misbehaves.
- *RPC placement:* a **dedicated `UsageService`** (decided), to keep telemetry
  concerns off `ScoreService`.
- *Alternatives:* per-action RPC (rejected: chatty); fire-and-forget UDP/logs
  (rejected: lossy and unstructured).

### D7 — Curated-but-not-frozen action taxonomy

`action` is a **shape-validated string**, governed by a single client-owned
registry (a shared constant), NOT a frozen proto enum. The server accepts any
well-formed action (even previously unseen) and rejects only malformed ones; the
BO filter list is derived from `SELECT DISTINCT action` in the aggregates.
- *Why:* an action only ever originates from an instrumented call site in client
  code, so it can never appear without a client release — freezing it in the wire
  contract buys nothing and forces a coordinated backend deploy for every new
  action. A shared client constant + code review gives governance; shape validation
  (length/charset) keeps the store clean without a hard allowlist; the data-driven
  BO list keeps the console current with zero redeploy.
- *Alternatives:* frozen proto enum (rejected: too rigid for something expected to
  churn; double-deploy friction); fully free-form strings with no validation
  (rejected: invites typos/variants → data swamp); server-side hard allowlist
  (rejected: same double-deploy friction as the enum). A soft server allowlist can
  be added later if abuse ever appears.

### D8 — Two optional event context fields: `subject_id` + `variant`

Some tracked actions need a parameter. Add two optional fields to the event rather
than a free-form JSON blob:
- `subject_id` (nullable) — a high-cardinality reference, the score UUID for
  `play_start`/`play_stop`/`score_upload`/`score_propose`/`favorite_add`/
  `favorite_remove`. Kept in **raw only** (queryable within the retention window);
  NOT in the aggregate grain, which it would explode. Long-term per-score stats are
  already served by the `play-activity` / leaderboard capabilities.
- `variant` (nullable, same shape-validated string rule) — a low-cardinality
  qualifier: the play mode (`fall_note`/`vertical_partition`/`full_partition`) for
  `play_mode_switch`, and the settings *category* (`piano_type`/`hand_left`/
  `hand_right`/…) for `settings_change`. Added to the `usage_action_daily` grain.
- *Privacy:* `variant` records the *category*, never the setting value — so
  "which settings people change" is measurable without capturing what they chose.
- *Alternatives:* a `props JSONB` (rejected: unstructured, un-indexable at the grain,
  invites value leakage); separate columns per action (rejected: sparse and rigid).

### D9 — Anonymous "test without account" is a deferred, isolated slice

Ingestion is authenticated (the anti-spam guarantee). A guest has no account, so
`user_bucket = hash(user_id, salt)` has no input — the `guest_session_start` action
cannot flow through the normal pipeline. Because it is the *only* action needing an
unauthenticated ingress (the sole abuse surface), the **slice is deferred** out of
this change and marked `pending` in the registry; the other 12 actions ship now.
The *approach*, however, is decided — only its sequencing is deferred.
- **Adopted approach (when picked up):** mint a short-lived, install-scoped,
  PII-free **anonymous guest token** at first launch, rate-limited at issuance, so
  guest events flow through the *same authenticated pipeline* (no separate
  unauthenticated ingress) with a pseudonymous guest `user_bucket` — which keeps the
  anti-spam guarantee **and** enables distinct-guest counts (answering "how many try
  the app without creating an account"). The guest bucket uses the same
  period-salted scheme, so guests are unlinkable across periods too.
- *Rejected alternatives:* a rate-limited unauthenticated endpoint with no identity
  (gives guest event counts but not distinct guests, and widens the abuse surface);
  tracking nothing for guests (loses the sign-up-funnel signal).

### D10 — Dedicated `analytics` Postgres schema

The three tables live in a dedicated `analytics` schema, not in `music`.
- *Why:* the data is deliberately decoupled from identity — it holds a hashed
  `user_bucket`, never a FK to `users`, so there is nothing to relate to `music`
  tables (and the repo already avoids cross-schema FKs). A separate schema gives a
  clean home for its own retention policy and targeted grants (an ingestion role, a
  worker role), and makes a future move to a separate analytics DB a clean cut.
- *Alternatives:* tables in `music` (rejected: clutters the domain schema, muddies
  grants/retention, no relational benefit since there are no FKs).

### D6 — Back-office reads aggregates, raw within window

The BO "Usage" screen queries the aggregates for historical periods and MAY hit
raw within the retention window for filter combinations the aggregates do not
pre-compute. It follows the vue-frontend-architecture rules (Pinia store behind the
injectable client seam; `ts-pattern` `Async<T>`).
- *Why:* aggregates make historical and long-range queries fast and always
  available; raw gives max flexibility on recent data. Consistent with how the repo
  already splits console vs backend capabilities.

## Risks / Trade-offs

- **Cross-period unique counts overcount** (D2 salt rotation) → document it in the
  BO UI ("distinct within period; ranges spanning months may double-count"); it is
  an accepted cost of the privacy posture.
- **Historical slicing limited to the aggregate grain** (D3) → freeze all
  foreseeable filter dimensions into the grain now; raw covers arbitrary slicing
  within the (default 6-month) window.
- **6-month raw retention is heavier on RGPD than a short window** → mitigated by
  period-salted pseudonymity (no cross-period linkage), first-party/no cross-site,
  a runtime-tunable TTL, and the opt-out consent control; the raw window is the
  only place near-personal detail lives and it self-expires.
- **Purge before rollup would lose data** → strict job ordering + wide multi-month
  margin + idempotent rollup; purge only touches days already aggregated.
- **Client clock skew on `occurred_at`** → accept client timestamps for period
  bucketing but clamp implausible values server-side (e.g. far future / far past)
  and stamp a server `received_at` for sanity.
- **Taxonomy drift** (client ships a new action) → by design the server accepts any
  well-formed action (D7); governance is the client-owned registry + code review,
  and the BO list is data-driven, so drift is a non-issue, not a failure mode.
- **Opt-out selection bias** → if too many users opt out, aggregates skew; monitor
  opt-out rate; this is inherent to any consent regime and informs the opt-out vs
  opt-in decision.

## Migration Plan

1. Land the migration creating `analytics.usage_events`, `analytics.usage_action_daily`,
   `analytics.usage_user_daily` + indexes (idempotent, additive; no backfill).
2. Add the config values to `cymbra-feature-flags` (retention TTL default 6 months,
   collection kill-switch default **on**) — surfaced in the BO flags panel.
3. Ship the backend ingestion + repos + period-salt hashing; deploy behind the
   kill-switch (can be shipped off, then turned on).
4. Register the rollup and purge worker jobs (rollup first in ordering).
5. Ship the Flutter tracking seam + offline buffer + consent toggle + taxonomy call
   sites; collection gated by the kill-switch and the user's consent.
6. Ship the BO "Usage" screen reading the aggregates.
- *Rollback:* flip the kill-switch to stop collection instantly; the tables and
  jobs are inert without inbound events. The migration is additive and safe to
  leave in place; drop tables only on a full revert.

## Resolved Decisions (from review)

- **Consent posture → opt-out with a user toggle.** Collection is on by default
  (first-party audience-measurement regime) and each user can disable it in
  settings. This per-user consent is distinct from the global `cymbra-feature-flags`
  kill-switch (an operator-side global off). Both suppress emission.
- **RPC home → dedicated `UsageService`.**
- **DB schema → dedicated `analytics` schema** (D10).
- **Action taxonomy → shape-validated string + client-owned registry, not a frozen
  enum** (D7); BO filter list derived from the data.
- **Salt storage/rotation → derive from a master secret (Option A):**
  `salt(month) = HMAC(master_secret, "YYYY-MM")`. Nothing to store; reproducible
  within a month; the analytics data *alone* cannot link a user across months.
  *Hardening (Option B), deferred:* generate a random per-month salt, store it, and
  delete salts older than retention so old buckets become permanently
  un-relinkable even to an operator holding the users table. A is sufficient given
  the tables never store `user_id`; revisit B only if the threat model tightens.

- **Initial taxonomy → decided (12 in scope + 1 deferred):** `auth_sign_in`,
  `auth_sign_up`, `play_start` (+subject), `play_stop` (+subject), `play_mode_switch`
  (+variant), `settings_change` (+variant, category only), `score_upload` (+subject),
  `score_propose` (+subject), `soundfont_upload`, `soundfont_propose`,
  `profile_view`, `favorite_add` (+subject), `favorite_remove` (+subject).
  `guest_session_start` is **deferred** (D9).
- **Action/variant shape rule → decided:** `^[a-z][a-z0-9_]{0,63}$`.

## Open Questions

- **None blocking.** The deferred `guest_session_start` slice (D9) has an adopted
  approach (signed anonymous guest token → authenticated pipeline → distinct-guest
  counts); only its scheduling is open.
