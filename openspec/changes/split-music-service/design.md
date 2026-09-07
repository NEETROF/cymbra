# Design — split-music-service

## Context

`cymbra-music` (32 363 LOC) is a **leaf** of the crate graph: nothing depends on it but
`server`, `worker` and `crates/score-crawler`, and it reaches its neighbours through
`Arc<dyn UserPort>` and `Arc<dyn PlanSource>`. At the crate level the separation is already
done — there is nothing to extract. What extraction changes is that **17 in-process calls
become remote**, and that is the whole cost.

Costed after `harden-module-boundaries`: **6–9 days** for a pure-RPC contract, or **3–4,5
days** if some seams are resolved as scoped read grants instead. The credit from the
hardening change is ~2–2,5 days, essentially all of it from emptying `backend/server` of
music (the real work there is the 301 contiguous lines of `main.rs:332-632`, not the file
moves).

Measurements that bound the design:

- `cargo tree -p cymbra-music` = 371 of the workspace's 446 crates (83 % shared). A
  touch-rebuild of `server` + `worker` is **4,46 s**. Build time is not a motivation.
- The worker links music regardless (`backend/worker/Cargo.toml`, `handlers.rs`). The split
  decouples the **request** path only.
- Six of the eleven outbound seams to `user` discard the error by construction
  (`backend/music/src/grpc.rs:602`, `:792`; `module.rs:947`, `:967`;
  `leaderboard_module.rs:297`; `global_leaderboard_module.rs:242`).
- `AppError` (`backend/platform/src/error.rs`) has no `Unavailable` and no
  `DeadlineExceeded`, no `From<Status>`, and `Internal` flattens to `"internal error"`.

**This change is dormant.** It is designed now and implemented only when a trigger in the
proposal fires.

## Goals / Non-Goals

**Goals:**
- Serve `music` from its own process against the same Postgres, with no client-visible
  contract change.
- Make a transport failure expressible and never silent.
- Keep account erasure atomic — one database, one transaction.
- Leave a template that `lingua` can follow if it, not music, is what needs to split first.

**Non-Goals:**
- Splitting the database. Costed at V1 + 5–8 days for no benefit: the eight database URLs
  already point at one `cymbra` database with eight distinct roles, one container, one
  `pg_dump`. Splitting loses atomicity at the *database* boundary without gaining isolation
  not already held at the *role* boundary, and forces a second `sqlxmq` queue plus a third
  permanent service.
- Splitting the worker. It keeps one database and one queue.
- Any change to the client contract, the back-office, or the site.

## Decisions

### D1 — One process boundary, one database

`music` gets its own binary and its own listener; the database stays shared, reached
through `music_svc` as today. This is the decision that keeps `purge_user`
(`backend/worker/src/lib.rs:43-270`, 21 tables / 4 schemas under one commit) untouched —
the worker still has one database, so erasure stays a single transaction and the GDPR path
does not become a saga.

*Alternative rejected:* splitting the database at the same time. It converts a solved
problem into a distributed one, on the most safety-critical path in the system, for no
operational gain on a single-host deployment.

### D2 — Each seam is resolved deliberately: served call or granted read

The 17 outbound seams do **not** all become RPCs. For each, the choice is recorded:

- **Served call** where the callee owns logic the caller must not duplicate — the
  visibility and age-eligibility predicates (`listable_profiles`, `age_eligible_profiles`,
  `get_player_profile`) encode a privacy rule that must have exactly one implementation.
- **Granted read** where the call is a plain projection on a path that must not become
  chatty or fallible — the worker's streak sweep already does exactly this
  (`backend/music/src/pg_streak.rs:194`, `LEFT JOIN user_account.users`, two columns).

The repo already reads across schemas on the worker path, so a narrow grant is not a new
category of thing. It is bounded by the `ops-db-access` requirement in this change: named
tables, read-only, with the reason recorded.

*Alternative rejected:* pure RPC everywhere. It is the tidier boundary and it costs roughly
double, for seams that are projections rather than decisions.

### D3 — No RPC exists for the internal seams, and none should be published

None of the four `UserPort` methods music calls has a usable RPC:
`GetAccountRequest` is empty (`backend/user-port/proto/user.proto:28`) while music resolves
an arbitrary id; `GetPlayerProfile` derives the viewer from the token while music passes
the `"public"` sentinel (`backend/music/src/module.rs:967`); `listable_profiles` and
`age_eligible_profiles` have no RPC at all.

They are served on an **internal surface**, not added to the audience-facing contract.
Publishing `resolve_or_provision` or `effective_roles` on the public façade to make a split
possible would be the worst outcome of this change — hence the requirement in
`platform-service-identity`.

### D4 — Service identity before any seam moves

`Claims` (`backend/platform/src/token.rs`) carries `sub`/`aud`/`roles`/`roles_by_scope` and
nothing else. Internal calls have no caller credential, and some run before any user token
exists. The component credential lands **first**: a seam moved before it exists is either
unauthenticated or authenticated by the end user's token, and the second is worse than the
first because it looks correct.

### D5 — Error semantics before any seam moves, and they pay for themselves now

`AppError` gains `Unavailable`, `DeadlineExceeded` and `From<Status>`; the six silent seams
gain a `warn!`. Both are ordered **before** the split and both are worth doing even if the
split never happens: today the same seams degrade silently whenever the database is
unhealthy, and nothing records it.

The visible behaviour is deliberately unchanged — an enrichment that fails is still
omitted, because a private profile legitimately yields no credit. What changes is that a
failure that is *not* a domain outcome leaves a trace.

### D6 — Flags need no served call

Music holds a concrete `Arc<FlagService>` (`backend/music/src/module.rs:171`), which
suggests an RPC. It does not need one: the worker already builds a read-only `FlagService`
against the flags schema with `NoopResolver` + `NoopBus` and no user pool
(`backend/worker/src/flags.rs:29-56`), serving code defaults if the read fails.

Verified safe for music specifically: its single evaluation site,
`caller_may_see_percussion` (`backend/music/src/module.rs:319-343`), builds its
`EvalContext` from a caller-supplied `staff` flag plus `premium`/`betas` from `plans`. The
admin resolver is never consulted, so `NoopResolver` changes nothing. The `plans.snapshot`
call at the same site remains a real seam — and it is one of the *good* ones, already
logging on failure.

### D7 — Deployment: the health probe is part of the change, not a follow-up

`deploy.sh` probes a single `/healthz`, which would report a deployment green while the
music process is dead. Per-component readiness ships with the second service, not after it.

The Caddy configuration lives on the box and is deployed by `scp` + reload, outside CI. The
new route is therefore a **manual step per environment**, and the runbook says so rather
than assuming CI covers it.

### D8 — Unify the coverage regexes before adding a second binary

`.github/workflows/rust.yml` and `sonar.yml` maintain two `--ignore-filename-regex` lists
by hand, and they have **already diverged**. A second binary adds paths to both. They are
reconciled into one source before the split, not after — this is cheap now and compounding
later.

## Risks / Trade-offs

- **[Six seams become silently fallible over the network]** → D5 lands first, and is
  ordered before any seam moves. This is the single largest recurring cost of the split.
- **[The split is done for tidiness rather than a trigger]** → the proposal names three
  triggers and states that load, GDPR and architectural tidiness are not among them.
- **[An internal operation leaks onto the public contract to make the split work]** →
  forbidden by `platform-service-identity`; the internal surface is separate.
- **[A granted read becomes a blanket privilege]** → bounded by `ops-db-access`: named
  tables, read-only, reason recorded.
- **[A deploy reports green while music is down]** → D7; per-component readiness is in
  scope.
- **[Debugging degrades: a stack trace no longer spans the whole request]** → accepted, and
  it is the honest recurring cost. There is no trace propagator in `backend/` today; one is
  in scope, and it is the mitigation.
- **[The split is attempted before `harden-module-boundaries`]** → then it starts by
  untangling 301 lines of interleaved wiring in `backend/server/src/main.rs`. The hardening
  change is a hard precondition.

## Migration Plan

Ordered so the reversible, independently valuable work comes first:

1. **Error semantics + the six `warn!`** — ship independently, no split commitment.
2. **Unify the two coverage regexes.**
3. **Service identity** in `Claims`, with verification on both sides.
4. **Seam-by-seam resolution** — each seam becomes a served call or a granted read, still
   in-process. At the end of this step the code is split-ready and nothing has moved.
5. **The second binary** — composition root, listener, router; `server` loses its music
   wiring.
6. **Deployment** — compose service, Caddy route, per-component readiness, runbook.

Steps 1–4 are individually revertable and leave a working monolith. Step 5 is the only
irreversible-feeling one, and even it is a wiring change: the crate boundary already exists.

Rollback: until step 5, `git revert` per commit. After step 5, re-mounting the music
services in the main binary restores the previous topology — the code is unchanged, only
the composition root differs.

## Open Questions

- Which seams are served calls and which are granted reads? D2 gives the criterion;
  the per-seam decision is made in step 4, with the reason recorded in the grant or the
  proto.
- What form does the service credential take — a signed component token reusing the
  existing key material, or mTLS between containers? The former is far less operational
  work on a single-host docker-compose; decide when step 3 is written.
- Does the score crawler (`crates/score-crawler`, the third dependant of `cymbra-music`)
  follow music into the new process, stay with the server, or become a third thing? Not
  investigated; resolve before step 5.
- Does `deploy.sh` gain a per-component probe loop, or does docker-compose healthcheck
  ordering suffice? Decide when step 6 is written.
