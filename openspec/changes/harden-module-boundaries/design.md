# Design — harden-module-boundaries

## Context

The backend is a modular monolith (`backend/*`: `platform`, `storage`, `jobs`;
`auth-port`, `user-port`; `auth`, `user`, `music`, `plans`, `feature-flags`, `analytics`,
`notifications`; `server`, `worker`) that has only ever served one product. A second
product scope (`lingua`, then `live`) is coming, and an audit of the boundaries turned up
work that is cheap now and expensive later.

Three findings set the shape of this change:

- **The trigger is a database row, not a commit.** `validate_scope_role`
  (`backend/user/src/module.rs:490`) tests `SCOPES.contains(&scope)` and `SCOPES` already
  contains `live` (`:28`). The admin console therefore accepts
  `grant_role(x, "live", "moderator")` today. Meanwhile `require_moderator_or_admin`
  (`backend/platform/src/guard.rs:45`) tests the **flat** role set, and `access_claims`
  (`backend/auth/src/module.rs:141-148`) gives a `back-office` token `global ∪ music ∪ live`.
  One grant opens 23 music gates.
- **The dependency graph is already right.** `cymbra-music` is a leaf — nothing depends
  on it but `server`, `worker` and `score-crawler` — and it consumes its neighbours as
  `Arc<dyn UserPort>` / `Arc<dyn PlanSource>`. There is nothing to extract, which is why
  this change hardens boundaries instead of splitting anything.
- **The design record contradicts the code.** The archived `add-cymbra-id` design
  prescribes a port↔gRPC-service 1:1 mapping and a contract test across two adapters.
  Neither exists: `GrpcUserClient` (`backend/user-port/src/lib.rs:352-377`) does not
  implement `UserPort` — the only `impl UserPort for` is `backend/user/src/module.rs:90` —
  and no `GrpcAuthClient` exists at all despite `auth-port`'s doc-comment.

Constraints: solo developer, <50 users, one server binary plus one worker on a VPS
(docker-compose + Caddy). Coverage ≥ 80 % both ecosystems. Conventional Commits.

## Goals / Non-Goals

**Goals:**
- Close the authorization hole before a second product scope can be granted.
- Close the account-erasure gap on `music.user_soundfonts` (rows and stored objects).
- Make premium unlocks belong to the product they were paid for, using the `product`
  column that already exists and is inert.
- Empty the composition root of product code, so the second product is not added by
  mimicry to the wrong crate.
- Put a compatibility net under the served protobuf surface.
- Write down the boundary rule so the design record stops prescribing something that does
  not exist.

**Non-Goals:**
- Splitting any module into a separate process or database.
- A BFF per app.
- Making the four central registries contributive — deferred to *while* Lingua is
  written, when a second real consumer exists to validate the split.
- Turning `purge_user` into a saga.
- Splitting protos into admin/app services with a dedicated audience interceptor.
- Changing the worker's database pool.

## Decisions

### D1 — Lock the grant path first, fix the guards second

Two steps, not one. The lock (`role == "moderator" && scope != "music"` → refused) goes
into `grant_role` (`backend/user/src/module.rs:220`) and buys the window; the guard
migration follows. The lock is removed in the same change once the guards are
scope-matched, so it never becomes permanent.

It goes in `grant_role`, **not** in `validate_scope_role`, because that helper is shared
with `revoke_role` (`:243`): revoking a bad grant must stay possible in every scope. This
is the one placement mistake that would turn a safety lock into a trap.

*Alternative rejected:* fix the guards first and skip the lock. It leaves the window open
for the duration of a 23-site refactor, for no saving — the lock is five lines.

### D2 — Scope-matched guard modelled on the existing one

Add `require_moderator_or_admin_in_scope(id, scope)` next to `require_admin_in_scope`
(`backend/platform/src/guard.rs:28`), built on `has_role_in_scope`
(`backend/platform/src/identity.rs:41`), which already implements the `global`
break-glass. The migration is parameter propagation across 23 sites: 16
`require_moderator_or_admin`, 2 inline flat tests (`backend/music/src/grpc.rs:380`,
`:1450`), and 5 coarse `require_admin`. Line numbers in `music/src/grpc.rs` drift with
every feature — re-derive them with grep before starting.

The doc-comment at `guard.rs:38-44` justifies the flat test by claiming the role set is
the audience's effective set. That reasoning holds for a single-scope app audience and
fails for `back-office` — the audience these gates actually serve. It is corrected, not
deleted: the reason it was wrong is the reason the fix is needed.

*Alternative rejected:* an audience-requiring interceptor on an admin service. It needs
the protos split first, and no RPC requires the `back-office` audience today — the
prerequisite decision has not been made. Deferred, not dismissed.

### D3 — Product-scoped unlocks on the column that already exists

`product TEXT NOT NULL DEFAULT 'music'` is already on `plan_entitlements` and
`beta_campaigns` (`backend/plans/migrations/0001_init.sql`), with the design note "carried
on every table so a Live plan is a new value, not a rename". It has zero occurrences in
`backend/plans/src/pg.rs` and `model.rs`.

Wire it end-to-end and give each `Unlock` variant an owning product, so resolving "does
the plan grant X" is answered for X's product. `PREMIUM_UNLOCKS` stops being a flat block.
The `DEFAULT 'music'` makes the backfill a no-op: every existing row is already correct.

Consumers to migrate: `backend/music/src/catalog_daily_access.rs:93`,
`backend/music/src/module.rs:357`, `backend/music/src/curation_rewards_module.rs:112`,
`backend/server/src/soundfont.rs:549` and `:886`.

The back-office plan visibility stays gated on `music` until the rest lands — it is the
last line of the work, not the first. Note that `/roles` was split into `/users` +
`/users/{id}` on 2026-09-06, so the gate now sits at four sites
(`apps/back-office/src/stores/roles.ts:54`, `views/UsersView.vue:32`,
`views/UserDetailView.vue:61`, `App.vue:62`) rather than two.

*Alternative rejected:* a separate unlock enum per product. It duplicates the resolution
machinery for a distinction that one field expresses.

### D4 — A second job kind for the private SoundFont bucket

The trap here is that the obvious move is wrong. `purge_user` already deletes
`music.user_scores … RETURNING object_key` and enqueues `PURGE_SCORE_OBJECT` in the same
transaction (`backend/worker/src/lib.rs:133-150`) — but that handler deletes from the
**score** store (`backend/worker/src/handlers.rs:33-35`), while private fonts live in a
dedicated private bucket (`backend/platform/src/config.rs:153`). Copying the block yields
a green fix that removes the row, logs success, and leaves the `.sf2` in place.

So: a new job kind (const + spec + channel, modelled on
`backend/jobs/src/registry.rs:27`), and `soundfont_store` wired into `WorkerCtx` — today
it is constructed only in `backend/worker/src/main.rs:100-113` for `ScorePreviewRenderer`.
The transactional-enqueue idiom is reused unchanged; only the target store differs.

**Implemented, and the audit widened the scope.** Looking for other tables in the same
state turned up three more, not zero:

| table | verdict |
|---|---|
| `music.user_score_collections` | unreached → now purged |
| `plans.sandbox_accounts` | unreached → now purged |
| `user_account.push_tokens`, `notification_prefs` | safe: `REFERENCES user_account.users ON DELETE CASCADE` |
| `music.user_score_takedowns` | **retention decision — see Open Questions** |

The audit is now a test rather than a one-off: `backend/worker/tests/erasure_coverage.rs`
reads the migrations for account-keyed tables and fails when one is neither purged, nor
cascaded, nor exempt with a written reason. It needs no database, and it was verified to
fail — removing the soundfont delete makes it report that table by name.

### D5 — Relocation, not rewrite, for the composition root

`backend/server` (~7 200 l.) carries ~3 700 l. of purely-music code: `soundfont.rs` 2 679,
`score_preview.rs` 385, four backfill binaries, and ~137 of `main.rs`'s 790. It moves to
`backend/music`, which exposes an `axum::Router` the server mounts.

The consequence is explicit rather than discovered: `backend/music` acquires `axum`,
`tower_http` and `jsonwebtoken`, none of which it declares today (zero occurrences in
`backend/music/src`). Making the largest crate in the workspace serve HTTP is defensible —
it already carries `tonic` — but it is a real change in what that crate is, and it is
assumed here rather than slipped in under a `git mv`.

*Alternative rejected:* a `music-http` crate between them. A third crate to avoid three
dependency lines, for a solo developer, is not worth its own `Cargo.toml`.

### D6 — Compatibility gate, with an honest expectation of overrides

A `buf breaking` job over the ten `backend/*/proto/*.proto`. No proto file is
admin-only — `plans.proto` and `score.proto` each declare a single service mixing app and
admin RPCs — so there is nothing to exempt: the gate is all-or-nothing.

Measured on history: ~42 commits touch a `.proto`, of which roughly one in eight carries a
removal or a re-typing. Expect an override roughly every 8–9 proto commits. That is the
cost, and it is worth paying — the incident it prevents already happened: `a42ab2a5`
removed `ReportStorePurchase` while `music-v1.23.0` and `music-v1.24.0` ship a client that
calls it, so a binary from those tags takes the store payment and then gets `UNIMPLEMENTED`
reporting it.

### D7 — Dead-code removal ordering

`.build_client(false)` must come **after** deleting `GrpcUserClient`, which is the only
consumer of a generated client in the whole backend — the reverse order breaks the build.
Note that `auth-port/build.rs` and `user-port/build.rs` use the `compile_protos` shorthand
with no builder, so they need `configure()` first; the other five already have it.

### D8 — The worker's ops privilege is documented, not corrected

Standing decision: the worker is an ops actor. The account-erasure job crosses four
schemas by design, and confining it would mean a distributed erasure for no benefit on a
single Postgres.

What is corrected is the contradiction: `backend/db/init/roles.sql.tpl:118-119` says the
ops role is "for OPERATIONS ONLY (runners/admins/psql); it MUST NEVER be wired into an
application module" without naming the worker, which is exactly what it is wired into.
Naming it there — and stating the consequence, that module boundaries inside the worker
are not database-enforced — turns an apparent violation into a stated invariant.

## Risks / Trade-offs

- **[The 22-site guard migration misses a site]** → the sites are enumerated in `tasks.md`
  from a counted audit, and the flat helper is removed once migrated so a missed site
  fails to compile rather than silently keeping the old behaviour.
- **[The grant lock is placed in the shared validator and blocks revocation]** → D1 names
  the placement explicitly; a test asserts revocation succeeds in a non-music scope while
  the lock is active.
- **[The SoundFont purge is written against the wrong bucket]** → D4 names the trap; the
  test asserts the object is gone from the private bucket, not merely that the row is
  gone.
- **[Product-scoped unlocks silently downgrade an existing subscriber]** → `DEFAULT
  'music'` means every existing row already resolves as before; a test covers a
  pre-existing entitlement resolving to the full music unlock set.
- **[The relocation changes what `cymbra-music` is]** → assumed in D5 and stated in the
  proposal's impact table, not discovered at review time.
- **[The compatibility gate becomes an obstacle that gets bypassed by habit]** → the
  expected override rate is stated up front (D6) so an override is a normal, recorded act
  rather than a sign the gate is wrong.
- **[Coverage drops when ~3 700 lines move between crates]** → the gate is a single
  workspace-wide run, so relocation is coverage-neutral; the ignore regex is checked for
  paths that name `server/` files being moved.

## Migration Plan

Ordered so each step is independently revertable and the risky window is shortest:

1. **Grant lock** (~5 l.). Revert = delete the block.
2. **SoundFont purge** — new job kind + `soundfont_store` in `WorkerCtx` + coverage test.
   Independent of everything else; ships alone if the rest slips.
3. **Scope-matched guards** across 23 sites; remove the flat helper; remove the grant lock
   from step 1.
4. **Product-scoped unlocks**; back-office plan visibility last.
5. **Relocation** of ~3 700 l. from `server` to `music`. Pure `git mv` plus three
   dependency lines; largest diff, lowest risk.
6. **Compatibility gate** + dead-code removal + `.build_client(false)` + design-record and
   role-comment corrections.

Rollback: every step is a separate commit; steps 1–4 are behavioural and revert cleanly.
Step 5 is a relocation with no behaviour change. No database migration is destructive —
the `product` work reads a column that already exists with a correct default.

## Open Questions

- **`music.user_score_takedowns` — erase, pseudonymise, or keep?** It holds `owner_id`,
  `admin_id`, the content `sha256` and the removal `reason` for an upload taken down by
  moderation. Erasing it with the account defeats its purpose: the `sha256` is what
  recognises the same file coming back under a new account. Keeping it retains an
  identifier naming a person after they asked to be forgotten. A third option is to null
  the `owner_id` (or set it to a nil UUID) and keep the rest, which satisfies both — at
  the cost of changing what a moderation record means, and of any code that reads that
  column. This is a product and legal call, not an implementation detail, so the table is
  currently EXEMPT with that reason written next to it.

- Should `require_admin` (the five coarse call sites) become scope-matched in this change,
  or stay coarse where it guards genuinely cross-product operations? Resolve per site
  during step 3 — some may legitimately be `global`-only.
- Which `buf` invocation shape: against the previous commit, or against the last shipped
  `music-v*` tag? The tag is the honest baseline for app-facing protos; the previous
  commit is simpler and catches more. Decide when writing the job.
- Does the private-SoundFont purge deserve its own job kind, or a generalised
  "purge object in bucket B" job that also subsumes `PURGE_SCORE_OBJECT`? The generalised
  form is tempting but changes an existing, working path — default to the dedicated kind.
