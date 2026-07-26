## Why

This initiative ships a lot of new surface (rating, rewards/shop, profiles, leaderboards,
onboarding). Rolling all of it out big-bang is risky, and the many tuning knobs
(thresholds, weights, costs) shouldn't need a redeploy to adjust. We want **hot feature
flipping**: turn a feature on/off and change tunables **at runtime, without a redeploy** —
to roll out gradually, **kill-switch** a misbehaving feature, dogfood with staff first, and
tune the economy live. This also subsumes the runtime-config need for the scattered
straw-man values.

## What Changes

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
- **Admin editing from the back office** — an **admin-only** panel to view/toggle flags and
  edit config values, with every change **audited** (who/when/old→new), like `role_grants`.
- **Home for the tunables** — the straw-man values (#2 re-review thresholds, #4 reward
  points/costs/levels, #6/#7 best-N, difficulty weights, season length, …) register here with
  their code defaults; each feature reads them through the flag/config service.

Out of scope: percentage/cohort/gradual-rollout targeting and A/B experimentation (future);
promoting legal/infra values (e.g. the minimum public-sharing age, data retention) into
casually-editable flags — those stay code-reviewed unless deliberately promoted with audit.

## Capabilities

### New Capabilities
- `runtime-feature-flags`: the runtime store of boolean flags + typed config with DB source and
  code defaults; hot evaluation with a short-TTL/invalidated cache; backend enforcement (off =
  capability disabled); fail-safe fallback to defaults; the global/staff-only scope; the change
  audit; and the client fetch of relevant flags.
- `feature-flags-admin`: the back-office admin panel to view/toggle flags and edit config
  values, guarded to admins, with audited changes.

### Modified Capabilities
<!-- None. The store is additive; each feature reads it at implementation time. The BO panel is
     a new surface hosted by the existing back office (#3). -->

## Impact

- **Backend**: a `feature_flags` / `app_config` table (key, typed value, scope, updated_by,
  updated_at) + a `feature_flag_changes` audit; a flags/config **service** with code-declared
  defaults, a short-TTL/Redis-invalidated cache, and typed accessors; a client-facing read RPC
  returning the caller's effective flags; enforcement hooks where features are gated.
- **Back office** (`bo.cymbra.app`, #3): an admin-only flags/config panel (view, toggle, edit),
  audited.
- **App** (`apps/music`): fetch effective flags on launch/resume; a small provider exposing them;
  feature entry points read the flag to show/hide (backend still enforces).
- **Depends on #3** for the admin panel (and reuses its `require_admin` + audit pattern); is a
  **platform primitive consumed by** #2/#4/#5/#6/#7/#8 (their tunables + on/off). The backend
  store can land early; the BO panel needs #3.
- **Coverage**: Rust ≥ 80% for the store, defaults/fail-safe, hot cache, scope, enforcement, and
  audit; Flutter ≥ 80% for the flag fetch/provider via fakes; the Vue panel under its own tests.
