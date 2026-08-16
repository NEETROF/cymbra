## Why

Every catalog browse, search, and score-download RPC on `ScoreService` runs behind a
verified per-user identity but has **no throttling of any kind**. A single signed-in
user can therefore point a bot at their own valid token and enumerate the entire
catalog via `SearchCatalog` (empty query = browse-all) and then bulk-pull every
MusicXML file through `GetCatalogScoreBytes` — siphoning the whole corpus, including
copyrighted, author-attested uploads, at machine speed. Upload abuse is already
capped by a per-owner quota; the read/egress side has no equivalent guardrail. We
need per-user access limits so a token cannot be weaponised to scrape the catalog.

## What Changes

- Introduce **per-user rate limits on catalog egress**, keyed on the authenticated
  `AuthIdentity.user_id` (the token subject), enforced inside `ScoreService`
  handlers:
  - A **download guardrail** on `GetCatalogScoreBytes` and `GetRatingPreviewBytes`
    (the raw-MusicXML paths) with two tiers: a short-window **burst cap** (pure
    rate — nobody opens hundreds of files in seconds), and an **engagement-aware
    volume allowance** instead of a flat daily cap. The volume allowance is a base
    floor plus headroom earned from the user's actual engagement — **play sessions
    plus score ratings** (the swipe deck's preview returns the score's bytes, so
    rating legitimately involves a download) — under a high hard ceiling. A user
    whose download volume tracks how much they play or rate is never blocked; only
    the "downloads a lot, engages with nothing" profile (the bot signature) falls
    back to the floor.
  - An **enumeration guardrail** on `SearchCatalog`, `GetCatalogScore`, and
    `ListRatingDeck` — a per-window request cap to slow catalog walk-through
    (the existing page-size clamp stays; this adds a request-rate ceiling).
- Reuse the existing `cymbra_platform::ratelimit::check` primitive (windowed Redis
  `INCR`+TTL) and wire the always-on `Cache` handle into `ScoreModule`, which does
  not currently receive it.
- **Exemptions**: the **back-office audience** (the trusted, CORS-gated curator
  console, which reuses `GetCatalogScoreBytes` and whose users never play) and
  **music-scope admins** (`has_role_in_scope("music", "admin")` — a `music/admin` or
  the `global/admin` break-glass) bypass all catalog access limits. On the music-app
  audience, **moderators are not exempt**, and neither is an `admin` held only in an
  unrelated scope (e.g. `live`). (Aligns with the scope-aware role model, change
  `scope-aware-role-admin`.)
- On breach, return gRPC `RESOURCE_EXHAUSTED` (`AppError::ResourceExhausted`) — the
  same contract the auth throttles already use.
- **Configurable, operator-tunable thresholds** (window + max per tier) via the
  platform config, adjustable at runtime through the existing feature-flags/config
  platform so limits can be tightened or relaxed without a redeploy, and disabled
  as a kill-switch if they misfire.
- **Observability**: count rate-limit rejections so scrape attempts are visible to
  operators (a scraping user shows up as sustained `RESOURCE_EXHAUSTED` on the
  download methods).
- **Graceful client handling**: the Flutter app surfaces a localized "slow down /
  limit reached" message on `RESOURCE_EXHAUSTED` instead of a raw gRPC error, and
  does not retry-storm.

## Capabilities

### New Capabilities
- `catalog-access-limits`: Per-user rate limiting on catalog browse/search/download
  RPCs — download burst + daily caps, enumeration request caps, `RESOURCE_EXHAUSTED`
  semantics, operator-tunable thresholds with a runtime kill-switch, rejection
  metrics, and graceful client degradation.

### Modified Capabilities
<!-- No existing spec's requirements change; the guardrail is additive new behaviour
     layered onto the existing ScoreService handlers. -->

## Impact

- **Backend (Rust)**:
  - `backend/music/src/grpc.rs` — enforce limits in `search_catalog`,
    `get_catalog_score_bytes`, `get_rating_preview_bytes`, `get_catalog_score`,
    `list_rating_deck` at the point where identity is already extracted
    (`owner()` / `identity()`).
  - `backend/music/src/module.rs` + call site `backend/server/src/main.rs` —
    inject `Arc<dyn Cache>` into `ScoreModule::new` (current wiring gap).
  - `backend/platform/src/config.rs` — new threshold knobs (window/max per tier),
    alongside the existing `email_max` / `signin_max_attempts` config.
  - Reuses `cymbra_platform::ratelimit::check` and `Cache::incr_with_ttl`
    (`backend/platform/src/ratelimit.rs`, `cache.rs`) — no new dependency.
  - Runtime tunability via the `cymbra-feature-flags` / config platform.
- **Client (Flutter, `apps/music`)**: map `RESOURCE_EXHAUSTED` from the score
  gRPC client to a localized, non-technical message; no retry storms.
- **Infra**: Redis (already always-on) holds the per-user windowed counters; no new
  service. Metrics flow through the existing `ObserveLayer` RED metrics.
- **Non-goals**: per-IP/edge rate limiting, CAPTCHA/bot-detection, and DRM/watermarking
  of downloaded scores are out of scope for this change.
