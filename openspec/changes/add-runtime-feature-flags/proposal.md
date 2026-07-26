## Why

This initiative ships a lot of new surface (rating, rewards/shop, profiles, leaderboards,
onboarding). Rolling all of it out big-bang is risky, and the many tuning knobs
(thresholds, weights, costs) shouldn't need a redeploy to adjust. We want **hot feature
flipping**: turn a feature on/off and change tunables **at runtime, without a redeploy** —
to roll out gradually, **kill-switch** a misbehaving feature, dogfood with staff first, and
tune the economy live. This also subsumes the runtime-config need for the scattered
straw-man values.

Critically, this must be a **shared, app-agnostic platform component**, not something baked
into the `music` module: Cymbra is a multi-app suite (`music`, `live`, future apps), and the
flag system MUST be **reused across apps without re-coding**. So it lives in its own crate + a
shared client, and its keys are **scoped per app** (global across all Cymbra apps, or per app).

## What Changes

- **Shared platform component (reusable across Cymbra apps)** — delivered as a **dedicated
  crate** (`backend/feature-flags`, owning its schema/service, like `jobs`/`observability`) plus
  a **shared Flutter package**, consumed by the server, the worker, and **every app** (music,
  live, future). It is NOT part of the `music` module — a new app reuses it with zero re-coding.
- **Per-app key scoping** — a flag/config key is scoped to **all apps** (shared) or to a
  **specific app** (`music`, `live`, …), resolved by the caller's app (the token `aud`). This
  app-scope is orthogonal to the rollout scope (global vs staff-only).
- **Runtime flag & config store** — one store holding **boolean feature flags** (on/off per
  feature) and **typed config values** (the tunables), with the **source of truth in the DB**
  and **safe defaults in code** used when a key is absent or the store is unreachable.
- **Hot evaluation (no redeploy)** — flags/config are evaluated at request time via a
  **short-TTL cache** (and/or Redis invalidation, already in the stack), so a change takes
  effect within seconds across the horizontally-scaled server and worker.
- **Backend enforces the flag** — a feature turned **off actually disables the capability**
  (the RPC is rejected / data is withheld), not merely a hidden button; the UI toggle is
  defense-in-depth, not the gate.
- **App consumes flags** — the app fetches the **client-relevant** flags on launch and on
  resume and shows/hides features accordingly; the backend remains authoritative.
- **Rollout scope** — a flag is **global** by default, with an optional **staff-only**
  (admin/moderator) scope so a feature can be exercised by staff before opening to everyone.
  (Percentage/cohort rollout is a future extension.)
- **Admin editing from the back office** — an **app-aware** admin panel to view/toggle flags and
  edit config values, filtered by app; managed by a **platform (global) admin** across apps, and
  by a per-app admin (e.g. `music/admin`) for that app's keys. Every change **audited**
  (who/when/old→new), like `role_grants`.
- **Home for the tunables** — the straw-man values (#2 re-review thresholds, #4 reward
  points/costs/levels, #6/#7 best-N, difficulty weights, season length, …) register here with
  their code defaults; each feature reads them through the flag/config service.

Out of scope: percentage/cohort/gradual-rollout targeting and A/B experimentation (future);
promoting legal/infra values (e.g. the minimum public-sharing age, data retention) into
casually-editable flags — those stay code-reviewed unless deliberately promoted with audit.

## Capabilities

### New Capabilities
- `runtime-feature-flags`: the **shared, app-agnostic** runtime store of boolean flags + typed
  config with DB source and code defaults; **per-app key scoping** (all-apps or a specific app,
  resolved by the caller's app) orthogonal to the global/staff-only rollout scope; hot evaluation
  with an L1 snapshot + Redis pub/sub invalidation; backend enforcement (off = capability
  disabled); fail-safe fallback to defaults; the change audit; and the client fetch of the
  caller's effective flags. Delivered as a reusable crate + shared client, not tied to `music`.
- `feature-flags-admin`: the back-office admin panel to view/toggle flags and edit config
  values, guarded to admins, with audited changes.

### Modified Capabilities
<!-- None. The store is additive; each feature reads it at implementation time. The BO panel is
     a new surface hosted by the existing back office (#3). -->

## Impact

- **New dedicated crate** `backend/feature-flags` (`cymbra-feature-flags`), owning its schema and
  service — reusable by any app/module; NOT inside `backend/music`. A **shared Flutter package**
  holds the client provider so every app reuses it.
- **Backend**: a `feature_flags` / `app_config` table (key, **app** [all|music|live|…], typed
  value, rollout scope, sensitive, updated_by, updated_at) + a `feature_flag_changes` audit; a
  flags/config **service** with code-declared defaults, an L1 snapshot + Redis-pub/sub cache, and
  typed OpenFeature-shaped accessors taking an identity+app context; a client-facing read RPC
  returning the caller's effective flags (resolved by app + identity); enforcement hooks where
  features are gated.
- **Back office** (`bo.cymbra.app`, #3): an admin-only flags/config panel (view, toggle, edit),
  audited.
- **App** (`apps/music`): fetch effective flags on launch/resume; a small provider exposing them;
  feature entry points read the flag to show/hide (backend still enforces).
- **Depends on #3** for the admin panel (and reuses its `require_admin` + audit pattern); is a
  **platform primitive consumed by** #2/#4/#5/#6/#7/#8 (their tunables + on/off). The backend
  store can land early; the BO panel needs #3.
- **Coverage**: Rust ≥ 80% for the store, defaults/fail-safe, hot cache, scope, enforcement, and
  audit; Flutter ≥ 80% for the flag fetch/provider via fakes; the Vue panel under its own tests.
