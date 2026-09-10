# add-lingua-backend — Cymbra Lingua: audience, backend module and sync protocol

## Why

The Lingua stack shipped a deliberately local product: every device carries its own
state, seeded by calibration or a LingQ import, and nothing ever converges. From the
second device on — the founder's real case, Chrome on macOS + Safari on iOS — statuses
drift apart. This change ships the first **server** brick of Lingua, on its own: the
Cymbra ID `lingua` audience (the "almost free" integration identified during
exploration — one configuration entry), a `backend/lingua` crate modelled on
`backend/music`, the three sync services and their protocol, and the GDPR purge. The
whole thing is **deployable inert** (nothing connects to it yet): the clients arrive in
the next change.

**Position in the stack** (12 changes): **11th** — after `add-lingua-agent`, before
`add-lingua-connected-clients`. **Explicit prerequisite: `add-lingua-decks-review`**
(the server schemas reuse the shared `lingua-core` types — statuses, cards, FSRS state
— so that merging is mechanical). **Independent of the extension branch**
(`add-lingua-extension-reading` → `add-lingua-apple`) and of the plugin
(`add-lingua-agent`): parallelisable with both. Consumed by
`add-lingua-connected-clients` (12th) and, outside the stack, by
`add-lingua-back-office`.

## What Changes

- **The `lingua` Cymbra ID audience is one configuration entry.**
  `CYMBRA_ALLOWED_AUDIENCES` (backend/.env.example) goes from
  `music,live,back-office,web` to `music,live,back-office,web,lingua`;
  `check_audience` (`backend/auth/src/module.rs`) admits it with no further code.
  **No `lingua` role and no `lingua` scope in v1**: `SCOPES`/`APP_SCOPES`
  (`backend/platform/src/lib.rs`) do not move — there is no Lingua back-office console
  in this change. The extension's Google OAuth client (redirect `chromiumapp.org`) is
  created and added to the `CYMBRA_GOOGLE_AUDIENCE` CSV (the desktop client is the
  precedent) — its client-side flow arrives in the next change.
- **CORS**: a new `CYMBRA_ALLOWED_WEB_ORIGINS` variable (a general list of bearer-only
  gRPC-web origins) — the tonic CORS layer serves the union with
  `CYMBRA_BACK_OFFICE_ORIGINS`, which is **not** widened (the admin console keeps its
  own list). This is where `chrome-extension://<id>` will live (bearer, never
  credentialed).
- **New `backend/lingua` crate** modelled on `backend/music`: a `lingua` Postgres
  schema, a `lingua_svc` role with a pinned `search_path`, its own migrations, an
  injected `UserPort`, a **module that stays inert without
  `CYMBRA_LINGUA_DATABASE_URL`**, and `cymbra.lingua.v1` protos in
  `backend/lingua/proto` (`build_client(false)`, as everywhere).
- **Three services**: `KnownWordsService` (syncs statuses per (language, lemma):
  client op-log (outbox), last-write-wins per lemma by timestamp, cursor-based delta
  pull, initial snapshot guarded by ETag/version); `DeckService` (complete cards
  **including** the source sentence — the user's own personal data; media/images
  **excluded** from v1); `StatsService` (learning aggregates per day and per language:
  exposures, words learned, reviews done). The first-sign-in merge is **not a special
  case of the protocol**: it is ordinary LWW applied to a large outbox.
- **Privacy — hard rules in spec**: only lemma statuses, cards and stat aggregates go
  up. **Never** a browsing URL, never page text, never web reading history; the only
  URL the server sees is the one carried by a card the user explicitly created.
  `DeleteAccount` → `lingua.*` purge via the existing `purge_user` job (extended worker
  handler).
- **Platform consumed, not redeclared**: feature flags (the `EvalContext` derives the
  app from the token audience — `lingua` comes for free), analytics (`UsageService`,
  whose `platform` values are already in the contract), existing observability.
  **Caddy: nothing to change** — `/cymbra.lingua.v1.*/…` paths fall through to the
  default branch → tonic (the `@http` matcher trap only concerns Axum HTTP routes, and
  this change adds none).

## Capabilities

### New Capabilities
- `lingua-sync` (server and protocol half): the `lingua` audience by configuration,
  general gRPC-web origins, an isolated and inert backend module, the sync protocol
  (op-log, LWW, cursor, snapshot) for statuses and cards, hard server-side privacy
  rules, purge on account deletion, and the platform consumed as-is. _The client half
  (extension/app sign-in, client outbox, pre-account store merge, opt-in) is the
  `add-lingua-connected-clients` change._
- `lingua-stats` (server half): learning aggregates — per day, per language and per
  device (exposures, words learned, reviews done) — idempotent upsert, server-side
  consolidation on read, aggregates only (never fine-grained timestamped events). _The
  clients' stats screen is the `add-lingua-connected-clients` change._

### Modified Capabilities
_None. The audience is configuration, not a spec delta: `backend-auth`
(issuance/refresh/OIDC), `user-account` (`DeleteAccount`), `runtime-feature-flags`,
`feature-usage-analytics` and `job-infrastructure` are **consumed as-is** — a product
consumes the platform, it does not redeclare it. The local stack's `lingua-*`
capabilities are untouched._

## Impact

- **Products**: Lingua backend (all new); **Cymbra ID consumed** (audience in config,
  server-verified OIDC, rotating refresh, `DeleteAccount`); **platform consumed**
  (flags, analytics, jobs, observability); Music / Live / back office / site:
  **untouched** (no existing proto modified, no scope added). No Lingua client changes
  in this change.
- **Tree**: `backend/lingua` (crate + `proto/` + `migrations/`),
  `backend/db/init/roles.sql.tpl` (`lingua` schema, `lingua_svc` role, admin role
  `search_path` extended), `backend/deploy/provision-lingua-role.sql`, `backend/worker`
  (`lingua.*` purge), `backend/server` (module wiring + CORS union).
- **Env/deploy**: `CYMBRA_LINGUA_DATABASE_URL` (new, module inert without it),
  `CYMBRA_ALLOWED_AUDIENCES` +`lingua`, `CYMBRA_ALLOWED_WEB_ORIGINS` (new),
  `CYMBRA_GOOGLE_AUDIENCE` + the extension client id; `.env.example`,
  `.env.prod.example`, `DEPLOY.md` and `provision-optional-modules.sh` updated.
- **CI**: the `rust` lane (repo-wide, `cargo --workspace`) covers `backend/lingua`
  automatically; llvm-cov ≥ 80% under the existing convention (logic in `*_core.rs`,
  `pg*.rs`/`grpc.rs` adapters excluded by the shared regex); **`buf breaking` does not
  apply** (the protos are entirely new — the gate bites from the next change on);
  `ci-units`: no new unit under `apps/`.
- **Out of scope (later changes)**: the connected clients — extension/app sign-in,
  client outbox, pre-account store merge, stats screen
  (`add-lingua-connected-clients`); Claude Code plugin sync (confidential transcripts —
  restated there); card media/image sync; the Lingua back-office console and the
  `lingua` role/scope (`add-lingua-back-office`); TextProfile/book catalogue.
