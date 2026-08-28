## Why

The backend is a modular monolith that has only ever served **one product**. Several
invariants currently held by convention — "a moderator moderates one product", "premium
unlocks belong to the product you paid for", "deleting an account erases everything" —
are either already false or become false the moment a **second product scope exists**.
Lingua ships next (local-only MVP, then a sync change that reaches the backend); Live
follows it.

Two of these are armed **today**, before any second-product code is written:

- `validate_scope_role` already accepts the `live` scope, so the admin console accepts
  `grant_role(user, "live", "moderator")` right now — and the moderation guards read the
  **flat** role set, so that grant would open every music moderation gate.
- Account deletion never purges `music.user_soundfonts`: the rows **and** the private
  `.sf2` objects survive a deletion. That is a live GDPR gap, independent of any product
  roadmap.

This change makes the boundaries explicit and enforceable **before** the second product
lands. It splits nothing, adds no deployable, and introduces no new transport.

## What Changes

**Authorization — scope-matched moderation**
- Immediate fail-closed lock in `grant_role`: refuse `moderator` outside the `music`
  scope until the guards are fixed. Placed in `grant_role`, **not** in the shared
  `validate_scope_role` (revoking a bad grant must stay possible).
- Add `require_moderator_or_admin_in_scope`, modelled on the existing
  `require_admin_in_scope`, and migrate the **22 production call sites** (15
  `require_moderator_or_admin`, 2 inline flat checks, 5 coarse `require_admin`).
- Fix the `require_moderator_or_admin` doc-comment, which claims scope-matching that the
  code does not perform for the `back-office` audience.
- Remove the lock once the guards are scope-matched.

**Subscriptions — product-scoped unlocks**
- Wire the existing but **inert** `product` column (already on `plan_entitlements` and
  `beta_campaigns` since the first migration) end-to-end.
- Scope `Unlock` per product, so a Lingua-only subscriber never receives
  `soundfonts.library`. Today `PREMIUM_UNLOCKS` grants all five music unlocks as a block.

**Account erasure — close the GDPR gap**
- Purge `music.user_soundfonts` rows **and** their stored objects on account deletion,
  via a **new job kind** targeting the private SoundFont bucket.
- Add a test that fails when a table holding personal data is not covered by the purge.

**Responsibility — empty the composition root**
- Move the ~3 700 lines of purely-music code out of `backend/server` into
  `backend/music`, exposed as an `axum::Router` the server mounts.

**Authorization is resolved in one place**
- `PgAdminScopeResolver` answers "is this account a platform admin" with its own SQL, while
  `UserPort::scoped_effective_roles` already answers exactly that and the composition root
  already holds the port. This is not a privilege problem — the query runs on the user pool
  as `user_svc`, against a table that role owns — but it is a **second implementation of a
  role-resolution rule**, and it will silently diverge the day role resolution gains a
  nuance every other authorization path picks up.
- Record the two direct cross-schema reads as **named** exceptions rather than silent ones:
  the worker's streak sweep (assumed — the worker is an ops actor) and `notifications` (a
  schema-ownership debt, deliberately deferred to its own change).

**External contract — a safety net**
- Add a `buf breaking` CI job over the ten `backend/*/proto/*.proto`.

**Design cleanup — stop the rule from lying**
- Delete the dead `GrpcUserClient`, fix the two port-crate doc-comments that advertise a
  gRPC client adapter that does not exist, and amend the archived `add-cymbra-id` design
  rule that prescribes a 1:1 port-to-gRPC-service mapping.
- Add `.build_client(false)` to the seven `build.rs` (~3 700 lines of client stubs
  generated every build with no caller).

**Documented decisions**
- Record the three-object boundary rule (internal boundary / internal transport /
  external contract) as a convention.
- Name the **worker** as an ops actor in the `admin_svc` role comment, so the code stops
  contradicting the standing decision that the worker may reach every schema.

## Capabilities

### New Capabilities
- `platform-module-boundaries`: the three-object rule — an internal boundary is a Rust
  trait, no internal transport is written before a split, and the external contract is
  the served surface. Includes the invariant that the composition root carries no product
  code.
- `platform-proto-compatibility`: shipped clients keep working — the served protobuf
  surface is checked for breaking changes in CI.

### Modified Capabilities
- `moderation-access-control`: the moderator role is scope-matched at every gate, not
  only declared in the `music` scope. A moderator in one product scope has no authority
  in another.
- `music-plan-entitlements`: the premium unlock set is scoped to the product that was
  paid for, instead of a single fixed set granted as a block.
- `user-soundfont-library`: the private library and its stored objects are erased with
  the account.
- `ops-db-access`: the worker is an ops-tier actor, not an application service — an
  explicit exception to "the ops role MUST NOT be used by any application service".

## Impact

**Products**

| Product | Consumed (unchanged) | New / changed |
|---|---|---|
| **Cymbra ID** | accounts, roles, sessions, erasure job | `grant_role` refuses non-music `moderator` (temporary); erasure gains a SoundFont purge step |
| **Music** | everything | 22 guard call sites take a scope; unlocks become product-scoped; ~3 700 l. move from `server` into `music` (acquires `axum`, `tower_http`, `jsonwebtoken`) |
| **Lingua** | nothing — the MVP is local-only | nothing in this change; it inherits the boundaries |
| **Live** | nothing — no module exists | the `live` scope stops being a latent authorization hole |
| **Back-office** | moderation and admin RPCs | unchanged surface; plan visibility stays gated until unlocks are product-scoped |
| **Site** | web auth, plans, account | unchanged |

**Code**
- `backend/platform/src/guard.rs`, `identity.rs` — new scope-matched guard.
- `backend/music/src/grpc.rs`, `backend/server/src/soundfont.rs`, `score_preview.rs` — guard call sites.
- `backend/user/src/module.rs` — temporary grant lock.
- `backend/plans/src/model.rs`, `pg.rs`, migrations — product-scoped unlocks.
- `backend/jobs/src/registry.rs`, `backend/worker/` — new purge job kind, `soundfont_store` in `WorkerCtx`.
- `backend/server/` → `backend/music/` — file relocation.
- `backend/user-port/src/lib.rs`, `backend/auth-port/src/lib.rs`, seven `build.rs`.
- `backend/db/init/roles.sql.tpl` — comment only.
- `.github/workflows/` — one new job.

**Not in scope** (each deliberately excluded, with its reason)
- **No module split into a separate process.** `cymbra-music` is already a leaf of the
  crate graph — nothing depends on it but `server`, `worker` and `score-crawler` — and it
  consumes its neighbours through trait objects. There is nothing to extract; a split
  would only add a second deployment.
- **No per-app BFF.** 98 of the 108 RPCs already have exactly one client (58 Flutter, 40
  back-office, 9 shared, 1 orphan). The single existing BFF (the site's JSON routes)
  exists only because a static Astro site cannot embed generated gRPC stubs.
- **Making the four central registries contributive** (`config.rs`, the flag registry,
  the job registry, `plans/model.rs`): deferred to **while Lingua is written**, when a
  real second consumer exists to validate the split. Doing it now is blind design; doing
  it after is refactoring two modules instead of one.
- **Turning `purge_user` into a saga.** 21 tables on a single Postgres — ACID is free,
  and the existing runtime-guard pattern already supports adding a product.
- **Splitting protos into admin/app services** with a dedicated audience interceptor:
  deferred. No RPC requires the `back-office` audience today, so the prerequisite
  decision has not been made.
- **Changing the worker's database pool.** Standing decision: the worker is an ops actor.
  Only the role comment is corrected, so the code matches the decision.
