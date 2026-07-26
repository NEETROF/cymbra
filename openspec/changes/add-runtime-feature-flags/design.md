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

## Decisions

### D1 — One store, two kinds: boolean flags + typed config; DB source, code defaults
A single key/value store holds **boolean feature flags** and **typed config values** (int,
number, string, small JSON). The **DB is the source of truth**; **code declares every key with a
typed default**. An absent key resolves to its code default, so the store only overrides.
- **Why one store**: flags and tunables share the same lifecycle (declare, default, override,
  audit, hot-eval). Splitting them would duplicate machinery.

### D2 — Hot evaluation: short-TTL cache + Redis invalidation, fail-safe to defaults
The service reads effective values through a small **in-memory cache with a short TTL**
(default ~15 s) so per-request reads are cheap; an admin edit **publishes an invalidation via
Redis** so all server/worker instances refresh within seconds (not just on TTL expiry). If the
DB/Redis is unreachable, the service **falls back to code defaults** (fail-safe) rather than
erroring.
- **Why cache + invalidation**: hot means "seconds, no redeploy" across horizontally-scaled
  instances; TTL bounds staleness even if an invalidation is missed. Redis is already present.
- **Fail direction is per-key**: kill-switches default to the **safe** state (usually off), so a
  store outage never silently enables something risky.

### D3 — Backend enforces; the UI toggle is defense-in-depth
Where a feature is gated, the **backend** checks the flag and, when off, **rejects the RPC /
withholds the data**. The app hiding a button is **not** the gate. Turning a flag off SHALL
cleanly disable the capability without deleting data (it's gated, not destroyed).
- **Why**: a flag must actually disable a capability even against a modified client; hiding UI
  alone is bypassable.

### D4 — App consumes: fetch on launch + resume; backend authoritative
A client-facing read returns the caller's **effective** flags (respecting scope/identity). The
app fetches them on **launch** and on **resume** (plus a lightweight refresh), exposes them via a
small provider, and shows/hides feature entry points accordingly. Truly instant on-device would
need push; fetch-on-launch/resume + short cache is "hot enough", and the backend enforces
regardless of client state.
- **Why not push**: disproportionate now; the backend gate makes eventual client refresh safe.

### D5 — Scope: global by default, optional staff-only; no percentage rollout yet
A flag's scope is **global** or **staff-only** (visible/enabled only for admin/moderator
identities), so a feature can be dogfooded by staff before a global flip. Percentage/cohort
targeting is deferred (the user base is tiny; it adds real complexity).
- **Why staff-only**: it's the highest-value targeting for a small team — try it internally,
  then flip global — without an experimentation framework.

### D6 — Admin editing from the BO, audited
Flags/config are viewed and changed from an **admin-only** BO panel (guarded by `require_admin`,
reusing #3). Every change appends to a **`feature_flag_changes`** audit (key, old→new, actor,
time), mirroring `role_grants`.
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

- **TTL value** and whether to require Redis invalidation or rely on TTL only (default: both,
  TTL ~15 s).
- **Which keys are flags vs fixed** — confirm the initial registry (all #2/#4/#6/#7 tunables as
  config; on/off flags per major feature: rating, rewards/shop, profiles, per-piece boards, global
  board, onboarding).
- **Kill-switch confirm** — require a typed confirmation in the BO for disabling a live feature?
