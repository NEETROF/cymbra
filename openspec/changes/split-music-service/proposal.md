## Why

**This change is deliberately dormant.** It is written now, while the analysis is fresh,
so that the day a trigger fires the decision is already made and costed — not so that it
is implemented next.

The `music` module is a leaf of the crate graph: nothing depends on `cymbra-music` but the
two composition roots and the score crawler, and it reaches its neighbours through trait
objects. Extracting it into its own process is therefore *possible* today. It is not
*justified* today: it costs 6–9 days once and buys a capability nothing currently needs,
while turning 17 in-process calls into network calls — six of which fail silently by
construction.

So this change records three things: what the split actually costs, the preconditions that
must hold before it is safe, and the **triggers** that would make it worth doing. It stays
unimplemented until one of the triggers fires.

**Triggers — implement only when one of these is true:**

1. **`lingua-sync` reaches the backend and needs a release cadence independent of music.**
   The most likely on the current trajectory. If it fires, split for *lingua* first; music
   follows only if it benefits, it does not motivate the split.
2. **A single module needs a second replica** (preview rendering or SoundFont delivery
   saturating the shared server) and duplicating auth/user alongside it is unacceptable.
3. **A second person works in the codebase** and merge windows become a measurable cost.

Load, GDPR, and architectural tidiness are explicitly *not* triggers. Erasure is safer in
one database, and the module boundary is already enforced at request time by the
per-module Postgres role.

## What Changes

**Precondition work — valuable on its own, do not wait for the split**
- Extend `AppError` with `Unavailable` and `DeadlineExceeded`, and add `From<Status>`. The
  type currently cannot express a transport failure, and `Internal` flattens to
  `"internal error"`, making a remote fault indistinguishable from a local one.
- Stop swallowing the six outbound seams. They discard the error by construction
  (`let Ok(…) =` / `.ok()`), which is correct in-process — a private profile legitimately
  yields no credit — but leaves no signal when the call fails for a different reason. A
  `warn!` preserves the visible behaviour and pays for itself today, whenever the database
  is unhealthy.

**The split itself**
- `music` becomes its own binary, serving its gRPC services and its Axum router, against
  **the same Postgres** through the `music_svc` role.
- The remaining server keeps auth, user, plans, flags, analytics, notifications and the
  site's BFF routes.
- The 17 outbound seams (11 → user, 4 → plans, 1 → flags, 1 arriving with
  `harden-module-boundaries`) are resolved as either a served RPC or a scoped read grant —
  see the design; the repo already reads across schemas on the worker path, so a grant is
  not a new category of thing.
- A service identity is added, because `resolve_or_provision`-style internal calls have no
  caller credential today: `Claims` carries only `sub`/`aud`/`roles`/`roles_by_scope`.
- Deployment gains a `music` service, a Caddy route, and a health probe that actually
  covers it — today one `/healthz` makes a deploy green while a component is dead.

**Explicitly NOT in this change**
- **Splitting the database (V2).** Costed at V1 + 5–8 days for no benefit on this
  deployment: the eight database URLs already point at one `cymbra` database with eight
  distinct roles, one Postgres container, and a single `pg_dump`. V2 loses atomicity at the
  *database* boundary without gaining any isolation not already held at the *role*
  boundary. It also forces a second `sqlxmq` queue and a third permanent service. The only
  legitimate trigger for V2 is a per-product data-residency obligation or a third-party
  host for music — neither exists.
- Changing the worker. It keeps one database and continues to link music; the split
  decouples the *request* path only.

## Capabilities

### New Capabilities
- `platform-service-identity`: a backend component can authenticate as itself when calling
  another backend component, distinctly from a user-scoped token.
- `platform-transport-failure`: a transport failure is expressible, distinguishable from a
  domain error, and never silently degrades a response.

### Modified Capabilities
- `backend-service`: the backend is served by more than one process; each declares its own
  readiness, and a deployment is healthy only when every component is.
- `ops-db-access`: an extracted module reaches the shared database through its own
  least-privilege role from its own process, and a deliberate cross-schema read grant is a
  named, documented exception rather than an accident.

## Impact

**Products**

| Product | Consumed (unchanged) | New / changed |
|---|---|---|
| **Cymbra ID** | accounts, roles, sessions | serves internal reads to the music process; issues service credentials |
| **Music** | its own schema and role | becomes its own binary; 17 in-process calls become remote or grant-based |
| **Lingua** | nothing today | if trigger 1 fires, lingua is the one that splits — this change is its template |
| **Live** | nothing | none |
| **Back-office** | moderation + admin RPCs | routed to a second upstream; no surface change |
| **Site** | web auth, plans, account | unchanged — the BFF routes stay on the main server |

**Code**
- `backend/platform/src/error.rs` — `Unavailable`, `DeadlineExceeded`, `From<Status>`.
- `backend/platform/src/token.rs` — service identity in `Claims`.
- `backend/music/src/{module,grpc,leaderboard_module,global_leaderboard_module}.rs` — the
  six silent seams, then the outbound adapters.
- `backend/music/src/bin/` — the new binary and its composition root.
- `backend/server/src/main.rs`, `flags.rs` — remove the music wiring and the six music
  trait adapters that `harden-module-boundaries` does not relocate.
- `backend/deploy/docker-compose.prod.yml`, the Caddyfile, `deploy.sh` — second service,
  route, per-component health.
- `.github/workflows/rust.yml`, `sonar.yml` — the two coverage regexes have already
  diverged; a second binary makes that worse unless they are unified first.

**Dependencies**: no new external dependency. `tonic` client generation is re-enabled for
the crates music calls — reversing one line of `harden-module-boundaries` task 7.3, by
design: that task removes stubs with no caller, and this change creates the caller.
