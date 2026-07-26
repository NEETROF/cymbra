## Context

A cross-cutting platform primitive requested to make the rollout safe and the tuning live:
**hot feature flipping** (on/off per feature) plus runtime config values, changeable without a
redeploy. It reuses the existing stack: gRPC/tonic, Postgres, **Redis** (already used for
rate-limit counters, a natural hot-invalidation channel), the back office (#3) with
`require_admin` + an audit pattern (`role_grants`), and the Riverpod app. It subsumes the
earlier "runtime config" option for the scattered straw-man tunables (#2/#4/#6/#7).

## Goals / Non-Goals

**Goals:**
- Turn features on/off and change tunables **at runtime, no redeploy**, effective within seconds.
- **Backend-enforced** flags (off = capability actually disabled), with the app reflecting them.
- Safe **defaults in code** and a fail-safe fallback if the store is unreachable.
- A **global / staff-only** scope for pre-rollout dogfooding.
- **Audited** admin edits from the BO.

**Non-Goals:**
- Percentage/cohort/gradual rollout and A/B experiments (future).
- Making legal/infra values (min public-sharing age, retention) casually flippable.
- **Purchase/subscription-based access (entitlements) — a separate, composable concern.** Flags
  are operator control ("is the feature enabled?"); **entitlements** are per-user grants from
  purchases/rewards ("is *this* user entitled?"), with their own source of truth (purchase/reward
  records, store-receipt validation, expiry, restore) and stronger server-side verification.
  Feature availability composes the two: **flag enabled AND user entitled**. Entitlement-like
  grants already exist (the #4 reward-shop unlocks; #5's future premium); real in-app purchases
  would feed an **entitlements** service (a possible future change), **not** the flag store — an
  admin must never be able to grant paid access by flipping a flag. The flag **evaluation context
  is kept extensible** so it can later *see* entitlements (as it already sees roles/app), but the
  entitlement check itself stays outside the flag system.

## Decisions

### D0 — A shared, app-agnostic platform crate, not part of `music`
The system is delivered as a **dedicated crate** `backend/feature-flags`
(`cymbra-feature-flags`), owning its own schema/migrations and service, on the model of
`backend/jobs` and `backend/observability`. It is consumed by the server, the worker, and every
app/module, and a **shared Flutter package** holds the client provider. It MUST NOT live inside
`backend/music`, so **Cymbra Live and future apps reuse it with zero re-coding**.
- **Why a crate, not a module method**: flags are cross-cutting platform infra (like jobs,
  observability, tokens), used by every app. Coupling them to `music` would force a re-implement
  for `live`.
- **Per-app key scoping (two orthogonal dimensions)**: each key has an **app scope** — `all`
  (shared across every Cymbra app) or a specific app (`music`, `live`, …) — resolved from the
  caller's app (token `aud`); and a **rollout scope** — `global` or `staff-only` (D5). A `music`
  build only ever sees `all` + `music` keys; a `live` build sees `all` + `live`. Shared keys (e.g.
  an "under maintenance" kill-switch) are defined once and apply everywhere.
- **Evaluation context** carries `{ app (aud), identity/roles }`, so the same service resolves the
  right values for any app.

### D1 — One store, two kinds: boolean flags + typed config; DB source, code defaults
A single key/value store holds **boolean feature flags** and **typed config values** (int,
number, string, small JSON). The **DB is the source of truth**; **code declares every key with a
typed default**. An absent key resolves to its code default, so the store only overrides.
- **Why one store**: flags and tunables share the same lifecycle (declare, default, override,
  audit, hot-eval). Splitting them would duplicate machinery.

### D2 — Two-tier cache: L1 in-process snapshot (hot path) + L2 Redis pub/sub invalidation
Flags are read on the hot path (potentially every request/gate), so evaluation MUST NOT do a
network hop per check.
- **L1 — in-process snapshot, per instance**: each server/worker instance keeps the **whole**
  flag/config snapshot in memory (tiny — dozens of keys), read with near-zero latency. It is the
  actual read cache. Refreshed **atomically** on a short **TTL** (~15 s, a safety net) or on an
  invalidation signal.
- **L2 — Redis pub/sub as the invalidation bus (chosen)**. On an admin edit, write Postgres and
  **publish** a coarse "flags changed" message on a Redis channel; every instance (server + worker)
  refreshes its L1 within milliseconds, well before the TTL.
  - **Why pub/sub over RESP3 client-side caching**: `CLIENT TRACKING` (BCAST) is the purpose-built
    server-assisted caching mechanism and would auto-invalidate with no hand-written wiring — but
    its main edge, **per-key precision**, is **neutralized by our whole-snapshot L1** (we reload the
    entire small set anyway). Pub/sub then wins on simplicity: **no client-lib dependency**
    (`fred` vs `redis-rs` RESP3 support is a non-issue), **no dedicated tracking connection / pool
    nuance**, RESP2-compatible, and Redis is already wired for rate-limit counters. RESP3 tracking
    stays a documented alternative to revisit only if the cache model ever becomes per-key.
  - **Redis's role here is ONLY this invalidation ping** — it is not the store (Postgres) and not
    the read cache (L1). System-wide, Redis also already backs rate-limit counters, so #9 adds no
    new dependency.
  - **Redis-free alternative — Postgres `LISTEN/NOTIFY`**: since Postgres is already the source of
    truth, the invalidation could ride on `LISTEN/NOTIFY` instead, removing Redis from the flags
    path entirely (Postgres does both truth and invalidation; Redis stays only for rate-limiting).
    Trade-off: a dedicated LISTEN connection per instance + reconnect handling (refresh L1 on
    reconnect). **Chosen Redis pub/sub** because Redis is already a hard dependency (zero marginal
    cost); switch to `LISTEN/NOTIFY` only if keeping flags Redis-independent is a goal.
- **Read path**: L1 → (miss/expired) Postgres (source of truth) → repopulate L1. **Write path**:
  Postgres → publish Redis invalidation → instances refresh L1.
- **Fail-safe**: if **Redis** is down, L1 keeps serving its last snapshot and TTL-polls Postgres;
  if **Postgres** is also unreachable, evaluation falls back to **code defaults**. Fail direction
  is per-key — kill-switches resolve to their **safe** (disabled) state so an outage never
  silently enables something risky.
- **Why L1 stays the hot path**: a per-flag Redis round-trip on the hot path would add latency and
  couple evaluation to Redis availability; L1 makes evaluation free and outage-tolerant. Redis
  pub/sub only drives **when to refresh L1**, giving the multi-instance "seconds, no redeploy".

### D8 — Build it in-house with an OpenFeature-shaped evaluation API (don't adopt OpenFeature now)
The service exposes an **OpenFeature-shaped** evaluation API — `bool(flag, default, ctx)`,
`number/string/json(key, default, ctx)` with an identity/scope context — but is implemented
**in-house**, not on the OpenFeature SDK.
- **Why not OpenFeature now**: it standardizes the *evaluation API + provider abstraction* only —
  it provides **no store, cache, BO panel, or audit**, so we'd build all of #9 anyway plus a
  custom provider. For a tiny self-hosted store at this scale that abstraction is **overkill**, and
  the **Rust/Dart OpenFeature SDKs are immature** relative to Go/Java — a young dependency for
  little gain.
- **Why keep the API OpenFeature-shaped**: if we later adopt OpenFeature or a managed/OSS backend
  (Unleash, Flagsmith, LaunchDarkly), the migration is a **thin adapter**, not a rewrite. We buy
  the option without paying the complexity.

### D3 — Backend enforces; the UI toggle is defense-in-depth
Where a feature is gated, the **backend** checks the flag and, when off, **rejects the RPC /
withholds the data**. The app hiding a button is **not** the gate. Turning a flag off SHALL
cleanly disable the capability without deleting data (it's gated, not destroyed).
- **Why**: a flag must actually disable a capability even against a modified client; hiding UI
  alone is bypassable.

### D4 — Client invalidation: lifecycle fetch + version/ETag; presentation-only, backend authoritative
The **client cannot subscribe to Redis**, so the shared Flutter package invalidates its local
flags differently — and the key that makes this simple is that the **backend is authoritative**,
so the app's flag cache is **presentation-only** (show/hide entry points), never the correctness
gate.
- **Lifecycle fetch (primary)**: fetch effective flags on **launch** and on **resume**
  (foreground), driven by the app lifecycle.
- **Version/ETag for cheap refresh**: `GetEffectiveFlags` returns a **version/hash**; the app
  sends its known version and the server replies "unchanged" (cheap) or the new set — so resume
  refreshes are near-free.
- **Optional foreground poll**: a light timer (~5–15 min), guarded by the version, catches a
  kill-switch during a long session without a restart.
- **Local persisted cache**: the last set is stored locally for a **flicker-free cold start**
  before the first fetch returns; fall back to code/bundled defaults if nothing was ever fetched
  (offline first launch).
- **Stale client is only cosmetic**: because the backend enforces (D3), a slightly-stale UI (a
  button still shown after an admin flips it off) just means the action **fails gracefully**
  server-side (a localized "unavailable" message, never a raw error) — not incorrect access. This
  is exactly why real-time client push is **not** required.
- **`WatchFlags` server-stream deferred**: near-real-time client updates via a gRPC stream are
  possible later but costly on mobile (long-lived connection, reconnect, backgrounding) and
  unnecessary given backend enforcement.
- **Whole-snapshot, not per-key network**: one `GetEffectiveFlags` returns the caller's entire
  effective set (small payload); the app caches it as a **snapshot** and reads are **per-key,
  local, synchronous** — never a per-key round trip.
- **Stale-while-revalidate**: a refresh is an **atomic swap** — the app keeps serving the last-good
  snapshot while a fetch is in flight and replaces it only on success; a **failed** refresh keeps
  the previous snapshot, never a gap/empty state.
- **Identity-scoped snapshot + auth-change reset (the security bit)**: the effective set depends on
  the caller's **roles + app**, so the snapshot is per-identity. The package **watches auth state**
  and: on **sign-out** discards the signed-in snapshot and reverts to the **anonymous/default**
  set; on **user switch** never reuses the previous user's snapshot — it refetches for the new
  identity (defaults meanwhile). Any **persisted** cache is **keyed by identity** (or cleared on
  sign-out), so one user's flags (e.g. staff-only features) never apply to another on a shared
  device. A pre-account app may fetch an **anonymous** effective set (global, non-staff keys) so
  onboarding respects kill-switches too; otherwise it uses code/bundled defaults.
- **All of this lives in the shared package** so music/live/future apps get identical behavior.

### D5 — Rollout scope: global by default, optional staff-only; no percentage rollout yet
Orthogonal to the **app scope** (D0), a flag's **rollout scope** is **global** or **staff-only**
(enabled only for admin/moderator identities), so a feature can be dogfooded by staff before a
global flip. Percentage/cohort targeting is deferred (the user base is tiny; it adds real
complexity).
- **Why staff-only**: it's the highest-value targeting for a small team — try it internally,
  then flip global — without an experimentation framework.
- **Two dimensions compose**: `app scope` picks *which apps* a key applies to; `rollout scope`
  picks *which users within an app*. Evaluation resolves both from the caller's `{app, roles}`.

### D6 — Admin editing: app-aware, platform + per-app admin, audited
Flags/config are viewed and changed from an **app-aware** admin panel (filter by app). A
**platform admin** (`global/admin`) manages keys across all apps; a **per-app admin** (e.g.
`music/admin`) manages only that app's keys (and `all`-scoped keys are platform-admin only). The
panel is hosted in the existing back office (#3) as a **platform-scoped section**, designed
app-agnostic so a future `live` back office reuses the same component. Every change appends to a
**`feature_flag_changes`** audit (key, app, old→new, actor, time), mirroring `role_grants`.
- **Why platform + per-app**: shared/`all` keys are cross-app and must not be flippable by a
  single app's admin; per-app keys stay with that app's admin. Reuses the scoped-roles model.
- **Why audited**: flipping a feature or an economy value is a sensitive, consequential action;
  "who changed what, when" must be durable and queryable.

### D7 — Key registry & defaults; the scattered tunables live here
Every flag/config key is **declared in code** (name, type, default, scope, fail direction, short
doc). The straw-man tunables register here — e.g. `rating.review.min_votes` (5),
`rating.review.threshold` (2.0), reward point bands/levels/costs, `leaderboard.global.best_n`
(20), difficulty weights, `leaderboard.season.length` — and each feature **reads them through the
service** instead of hardcoding. Legal/infra values (min public-sharing age, retention) MAY be
represented but are marked **not casually editable** (kept code-reviewed / require an explicit,
audited change).
- **Why a code registry**: defaults live with the code that uses them; the DB only overrides; the
  BO panel lists exactly the declared keys (no free-form typos).

## Risks / Trade-offs

- **Stale value after an edit** → short TTL + Redis invalidation bounds it to seconds; acceptable
  for these knobs. [Missed invalidation] → TTL still refreshes.
- **Flag off doesn't actually disable** → enforce in the backend (D3), not just the UI; test that
  a gated RPC is rejected when off.
- **Store outage** → fail-safe to code defaults (D2), per-key safe direction for kill-switches.
- **Accidental/harmful edit** → admin-only + audit (D6); legal/infra keys marked non-casual (D7);
  consider a confirm step for kill-switches in the BO.
- **Config drift** → the code registry is the canonical list; the BO edits only declared keys.
- **Coverage** → Rust tests for default fallback, cache/invalidation, scope resolution, enforcement
  (off rejects), audit; Flutter tests for the flag provider via fakes.

## Migration Plan

1. Backend: the `feature_flags`/`app_config` table + `feature_flag_changes` audit; the flags
   service (code registry + defaults + TTL cache + Redis invalidation + typed accessors); the
   client read RPC. Land early — features can start reading it.
2. Wire enforcement at each feature's gate as those features are implemented (read the flag/config
   via the service; reject/withhold when off).
3. BO panel (needs #3): list declared keys, toggle/edit, audited.
4. App: fetch effective flags on launch/resume; provider; feature entry points read them.
5. **Rollback**: additive; with no overrides set, everything runs on code defaults exactly as
   without the system.

## Open Questions

- **L1 TTL value** — a tuning detail (default ~15 s as a backstop). The tiering (L1 snapshot + L2
  Redis pub/sub) and the in-house-but-OpenFeature-shaped API are decided (D2, D8).
- **Which keys are flags vs fixed** — confirm the initial registry (all #2/#4/#6/#7 tunables as
  config; on/off flags per major feature: rating, rewards/shop, profiles, per-piece boards, global
  board, onboarding).
- **Kill-switch confirm** — require a typed confirmation in the BO for disabling a live feature?
