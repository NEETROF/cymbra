# Tasks — add-lingua-backend

## 1. Audience and configuration

- [x] 1.1 Add `lingua` to `CYMBRA_ALLOWED_AUDIENCES` in `backend/.env.example` and `backend/deploy/.env.prod.example` (+ comment); integration test: `SignInLocal`/`Refresh` with the `lingua` audience accepted, an audience absent from the config refused
- [ ] 1.2 Create the extension's Google OAuth client (Web type, redirect `https://<ext-id>.chromiumapp.org/`) and add it to the `CYMBRA_GOOGLE_AUDIENCE` CSV (env example + doc) — precedent: the desktop client
- [x] 1.3 Introduce `CYMBRA_ALLOWED_WEB_ORIGINS` (server config, CSV, empty by default) and serve the union `back_office_origins ∪ allowed_web_origins` in tonic's gRPC-web CORS layer (`backend/server/src/main.rs`); tests: an origin from the new list admitted, an unknown origin refused, `CYMBRA_BACK_OFFICE_ORIGINS` not widened
- [x] 1.4 Confirm on the evidence that `SCOPES`/`APP_SCOPES` (`backend/platform/src/lib.rs`) reference `lingua` nowhere after this change (no scope added — a review, not code)

## 2. The `backend/lingua` crate

- [x] 2.1 Scaffold `backend/lingua` (crate `cymbra-lingua`) modelled on `backend/music`: `Cargo.toml` (workspace), `build.rs` (`build_client(false)`), `src/` layout with the `*_core.rs` / `pg*.rs` / `grpc.rs` seam
- [x] 2.2 Schema + role: a `lingua_svc` entry (`lingua` schema, pinned `search_path`) in `backend/db/init/roles.sql.tpl`, `CYMBRA_LINGUA_DATABASE_URL` in the env examples, `backend/deploy/provision-lingua-role.sql` + wiring into `provision-optional-modules.sh` (the music pattern)
- [x] 2.3 Migrations: status table (user, language, lemma, status, provenance, timestamp, device, change sequence), card table (client id, the local stack's schema fields minus media content, sequence), stat aggregates (user, day, language, device); index on (user, sequence) for the cursor pull
- [x] 2.4 Server wiring: module inert without `CYMBRA_LINGUA_DATABASE_URL` (log "lingua services disabled"), its own pool + MIGRATOR, injected `UserPort`, services behind the strict auth interceptor with audience checking
- [x] 2.5 Update `DEPLOY.md` (enabling the module, variables, provisioning)

## 3. `cymbra.lingua.v1` protos and services

- [x] 3.1 `backend/lingua/proto/known_words.proto`: `KnownWordsService` — `PushOps` (idempotent timestamped batches), `PullChanges` (cursor delta), `GetSnapshot` (ETag/version, "unchanged" response); `buf lint` green (new protos: `buf breaking` does not apply)
- [x] 3.2 Host-tested sync logic (`known_words_core.rs`): LWW per (language, lemma) with a device tie-break, batch idempotence, monotonic sequence, offset resumption; convergence tests (conflict, replay, interleaving)
- [x] 3.3 `deck.proto`: `DeckService` — push/pull of complete cards by client id, LWW per card, deletions propagated, the media field never transported; `deck_core.rs` + tests
- [x] 3.4 `stats.proto`: `StatsService` — `UpsertDailyStats` (day/language/device key), `GetStats` (date range, series consolidated by SUM per day × language); `stats_core.rs` + tests (two devices, replayed upsert)
- [x] 3.5 `pg_known_words.rs` / `pg_deck.rs` / `pg_stats.rs` adapters + `grpc.rs` (auth: the token's user, never a client-supplied user_id); add the new `pg*.rs` files to the coverage exclusion regex if the existing pattern does not already cover them
- [x] 3.6 End-to-end integration test: two simulated clients converge (statuses + cards + stats) across the three services

## 4. Purge and privacy

- [x] 4.1 Extend `purge_user_with` (`backend/worker/src/lib.rs`): erase `lingua.*` for the user, idempotent, a no-op without data; extend the admin role's `search_path` (`roles.sql.tpl` admin line + prod migration doc)
- [x] 4.2 Worker handler tests: purge with data, replayed purge, an account with no Lingua data
- [x] 4.3 Allow-list contract test: the `cymbra.lingua.v1` proto messages carry no field for a read page's URL, page text or history (a schema review documented in the proto + an assertion on the descriptors if practical)

## 5. Gates and finishing

- [x] 5.1 `cargo fmt --all --check` + `clippy --workspace --all-targets -- -D warnings` + `cargo llvm-cov --workspace --fail-under-lines 80` (shared exclusion regex up to date for the new adapters)
- [x] 5.2 `buf lint` on `backend/lingua/proto`; confirm the `proto` lane (`buf breaking`) watches `backend/lingua/proto/**` for the following changes
- [x] 5.3 Env/deploy finalised: `.env.example`, `.env.prod.example`, `DEPLOY.md`, provisioning; rehearse the migration plan (inert backend → provisioning → enablement — the clients come in the next change)
- [x] 5.4 Final `openspec validate add-lingua-backend --strict` + spec updates if implementation moved a contract
