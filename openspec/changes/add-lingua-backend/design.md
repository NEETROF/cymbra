# Design — add-lingua-backend

## Context

On the backend, everything needed already exists: `check_audience`
(`backend/auth/src/module.rs:126`) iterates `CYMBRA_ALLOWED_AUDIENCES`; refresh-token
rotation with reuse detection and family revocation is in place (tested:
`refresh_rotates_then_reuse_revokes_family`); Google **and** Apple OIDC are verified
server-side (`CYMBRA_GOOGLE_AUDIENCE`/`CYMBRA_APPLE_AUDIENCE`, CSV audiences); the
"product module" pattern is established by `backend/music` (schema + role + pinned
`search_path` + MIGRATOR + inert without env); and the `purge_user` job
(`#[sqlxmq::job]`, `admin_svc` pool) already carries cross-schema erasure. The delta is
therefore narrow and mostly **configuration** — keeping it narrow is the point of this
design.

Constraint inherited from the local stack: every client surface has a **versioned local
state** whose schemas share the `lingua-core` types (decided in `add-lingua-decks-review`
and `add-lingua-apple`) — "so that merging is mechanical": the protocol defined here is
that merge, and the clients plug into it in `add-lingua-connected-clients` (which
carries the client-side decisions: gRPC-web bearer transport, token storage, OIDC in the
extension, Sign in with Apple, pre-account store merge).

## Goals / Non-Goals

**Goals:**
- A complete Lingua backend that is **deployable inert**: crate, migrations, roles,
  protos, services — nothing changes for anyone as long as
  `CYMBRA_LINGUA_DATABASE_URL` is absent.
- The server sees **only** what the spec allows: lemma statuses, cards, aggregates.
  Never reading history.
- Zero new role, zero new scope, zero new HTTP route: the footprint on the platform is
  two env variables and one CORS list.
- A convergent sync protocol testable server-side alone (two simulated clients
  converge).

**Non-Goals:**
- The connected clients (account UI, client outbox, pre-account store merge, stats
  screen) — the `add-lingua-connected-clients` change.
- Claude Code plugin sync (`~/.lingua/`) — transcripts are confidential; a later change
  with its own design (loopback PKCE CLI auth).
- Card media/images (v1: the schema's `media` field does not sync; encrypted sync comes
  with image capture).
- Lingua back-office console, `lingua` role/scope, moderation — nothing admin in this
  change.
- Fine-grained conflict resolution (CRDT, per-field history) — see D3.

## Decisions

### D1 — The `lingua` audience: one configuration entry, zero roles
`CYMBRA_ALLOWED_AUDIENCES=music,live,back-office,web,lingua` — that is the whole
identity delta. `check_audience` accepts the audience at issuance and at refresh;
`lingua` tokens cross the strict auth interceptor exactly like `music` ones.
**`SCOPES`/`APP_SCOPES` (`backend/platform/src/lib.rs`) do not move**: a scope exists
only to carry scoped administration roles, and this change has no admin surface.
Rejected alternative: adding `LINGUA_SCOPE` "for later" — that would drag back-office
session aggregation and scope-aware admin along for zero v1 use, and the
separation-of-powers audit showed what declared-but-untested scopes cost. The scope
arrives with the first Lingua admin surface (`add-lingua-back-office`).

### D2 — CORS: `CYMBRA_ALLOWED_WEB_ORIGINS`, the union rather than a widening
A new `CYMBRA_ALLOWED_WEB_ORIGINS` variable: a general list of browser origins admitted
on the **bearer-only** gRPC-web surface. The tonic CORS layer (today fed by
`cfg.back_office_origins` alone) moves to the union
`back_office_origins ∪ allowed_web_origins`. That is where `chrome-extension://<id>`
goes (a stable id, derived from the key published to the store). Rejected alternatives:
widening `CYMBRA_BACK_OFFICE_ORIGINS` — its name is a contract ("console only", says the
env comment), and mixing a product origin into the admin console's list muddies the
audit; reusing `CYMBRA_WEB_ORIGINS` — it governs the **credentialed cookie** surface
(`/web/auth/*` + site), exactly what an extension must not touch. CORS remains defence
in depth: the auth interceptor is the authorisation, not the origin.

### D3 — Sync protocol: client op-log, LWW per entity, cursor pull — no CRDT
Each client keeps an **outbox** (a local op-log of mutations: status set, card
created/edited/deleted) drained to the server in idempotent batches; every op carries the
client timestamp and the `device_id`. Resolution: **last-write-wins per (language,
lemma)** for statuses and **per card** (client UUID) for decks — the most recent
timestamp wins, with a deterministic `device_id` tie-break. The pull is a **cursor
delta** (a monotonic per-user server change sequence); bootstrap or an invalid cursor
goes through a **snapshot** guarded by ETag/version (no re-download if nothing moved).
Why no CRDT: the state is a growing set plus counters — two devices each setting a
status on the same lemma within the same minute is the realistic worst case, and "the
last gesture wins" is exactly the semantics the user expects. A per-field CRDT would
cost the format, the documentation and the tests of a research protocol for a conflict a
single click resolves. **The first-sign-in merge is not a special case**: the client
pushes its pre-account state as one large outbox with the original timestamps (a
client-side decision in the next change) — the server applies ordinary LWW.

### D4 — What the server stores: three data tables, one allow-list
The `lingua` schema holds: statuses per (user, language, lemma, status, provenance,
timestamp, device); complete cards at the local stack's schema **minus** the contents of
the `media` field (the slot stays, nothing goes up) — a card's source sentence and
source do go up: they are **personal data the user explicitly captured**, not history;
and stat aggregates (D5). The allow-list is the spec's contract: any data outside these
three families does not go up — in particular no URL of a merely-read page, no page
text, no per-site counter. The local stack's exposure counters stay **local** in v1
(bulky, low multi-device value, and the data closest to reading history — keeping it
local is also a product position).

### D5 — Stats: additive aggregates per (day, language, device), consolidated on read
Each device upserts its aggregate rows `(UTC day, language, device_id) → {exposures,
words learned, reviews done}` — idempotent (upsert by key), never a fine-grained
timestamped event. The read (`StatsService.GetStats`) sums across devices and returns
series per day × language, consumed by the clients' stats screen (next change). Rejected
alternative: absolute LWW counters per (day, language) — two devices active on the same
day would overwrite each other; keying by device makes addition correct by construction.
Privacy: day × language is the finest grain allowed — no hour, no source, no site.

### D6 — Backend module: `backend/lingua` modelled on `backend/music`, inert without env
The `cymbra-lingua` crate: a `lingua` Postgres schema owned by the `lingua_svc` role with
a pinned `search_path` (`roles.sql.tpl` + `provision-lingua-role.sql` for production,
the `provision-music-role.sql` pattern), its own MIGRATOR, `cymbra.lingua.v1` protos in
`backend/lingua/proto` (`build_client(false)` — no internal transport, per the
three-objects rule). The server wires the three services only if
`CYMBRA_LINGUA_DATABASE_URL` is set — otherwise
`tracing::info!("lingua services disabled")`, like music. `UserPort` is injected (the
`user-port` crate's trait, declared by the consumer) for account existence/state — never
a read of the `user_account` schema. Merge/cursor logic lives in host-tested
`lingua_sync_core.rs`; the `pg*.rs`/`grpc.rs` adapters are covered by the existing
exclusion regex.

### D7 — Purge: extend `purge_user`, do not create a job
`DeleteAccount` already triggers the `purge_user` job (worker, `admin_svc` pool,
idempotent). The delta: `purge_user_with` also erases `lingua.*` for the user, and the
admin role's `search_path` (`roles.sql.tpl`) gains the `lingua` schema. An account with
no Lingua data is a no-op (the job stays idempotent). Rejected alternative: a separate
`purge_lingua` job — two jobs to sequence for one GDPR semantic, and the precedent
(plans, analytics) is extending the single job.

### D8 — Platform consumed as-is: flags by audience, existing analytics, Caddy unchanged
Feature flags: the `EvalContext` derives `app` from the token audience — a `lingua` token
is scoped `lingua` automatically, with no declaration; future Lingua betas are managed in
the existing flags console. Analytics: the clients will emit usage events through
`UsageService` with the `platform` values already in the proto contract (`web`, `ios`,
`macos`) — no new pipeline. Observability: the `lingua` services sit under the existing
`ObserveLayer` like any tonic service. **Caddy: checked against the matcher** — the gRPC
paths `/cymbra.lingua.v1.<Service>/<Method>` are disjoint from the `@http` list
(`/.well-known/*`, `/healthz`, `/web/*`, …) and fall through to the default branch →
tonic h2c, which also carries gRPC-web and the CORS preflight. The documented trap
("prefix missing from the matcher → tonic answers 200 + an empty grpc-status 12") only
bites **Axum HTTP** routes; this change adds none, so the Caddyfile does not move.

## Risks / Trade-offs

- [LWW + wrong client clocks: a device with a skewed clock "wins" conflicts] →
  deterministic `device_id` tie-break, the server receipt timestamp kept for audit, and
  future timestamps clamped on receipt; the worst case is still fixable with one click
  (set the status again).
- [`chrome-extension://<id>` origin: the id differs between dev (unpacked) and store] →
  the published id is stable (key in the manifest); in dev, the local origin is added to
  `CYMBRA_ALLOWED_WEB_ORIGINS` in the dev environment only. Firefox
  (`moz-extension://<uuid>`, random per install): not blocking — fetch requests from a
  Firefox MV3 event page do not carry an origin subject to this CORS in the same way; to
  be confirmed at client integration, the documented fallback being a fixed origin id via
  `browser_specific_settings`.
- [A bulky initial push at first sign-in (years of statuses)] → bounded batches plus
  outbox-offset resumption, carried by the protocol (per-batch idempotence); op order
  preserves timestamps, so an interruption resumes without corruption.
- [Two increasingly similar origin lists (`WEB_ORIGINS`, `ALLOWED_WEB_ORIGINS`)] →
  explicit env names and comments ("credentialed cookie" vs "gRPC-web bearer"); a
  deliberate refusal to merge them while their semantics differ.
- [The lingua module on the same server: shared blast radius] → the same trade-off as
  music/plans, accepted by the agreed backend trajectory (no split before the need); the
  crate keeps the boundary (schema + role + port) so that the split stays a small job.
- [Backend shipped before any client: risk of an ill-fitting contract] → the protocol is
  tested by two **simulated** converging clients (an end-to-end integration test), and
  the types come from `lingua-core` — the same vocabulary as the real client stores.

## Migration Plan

1. Backend first (deployable inert): crate + migrations + roles + protos + services,
   `CYMBRA_LINGUA_DATABASE_URL` absent in production → nothing changes for anyone.
2. Provisioning: `provision-lingua-role.sql` + the admin `search_path` extension + prod
   env (`ALLOWED_AUDIENCES`, `ALLOWED_WEB_ORIGINS`, the Google client id); then enable
   the DB variable.
3. The clients arrive in `add-lingua-connected-clients` (extension, then the Apple app).
4. Rollback: removing `CYMBRA_LINGUA_DATABASE_URL` makes the module inert; no client
   depends on the server yet.

## Open Questions

- The exact snapshot format (a single message vs a paged stream) — to be settled at
  implementation against real sizes; the contract (ETag/version, "unchanged" response)
  does not move.
