## 1. Database migration

- [x] 1.1 Add a migration under `backend/music/migrations/` that adds to `music.catalog_scores`: `moderation_status TEXT NOT NULL DEFAULT 'pending' CHECK (moderation_status IN ('pending','accepted','rejected'))`, `reviewed_by UUID`, `reviewed_at TIMESTAMPTZ`.
- [x] 1.2 Add an index on `moderation_status` (e.g. `CREATE INDEX ... ON music.catalog_scores (moderation_status)`) to back the hub's `WHERE moderation_status = 'accepted'` hot path.
- [x] 1.3 Backfill existing rows to `pending` (implicit via the `DEFAULT` on a populated table; add an explicit `UPDATE ... SET moderation_status='pending'` only if needed for clarity). Do NOT pre-accept any subset — every existing score stays `pending` and is reviewed individually.
- [x] 1.4 Confirm the migration runs cleanly under the `music` search_path / migrator role used by the crawler and server.

## 2. Storage model & insert default

- [x] 2.1 Extend the catalog row model / `CatalogEntry` and `PgCatalogRepo::insert` (`backend/music/src/repo.rs`, `backend/music/src/pg.rs`) so inserts always persist `moderation_status = 'pending'` (rely on the DB default and set it explicitly so it can't be overridden by the caller).
- [x] 2.2 Verify the crawler ingestion path (`crates/score-crawler/src/catalog.rs` → `ingest` → `PgCatalogRepo::insert`) compiles and ingests as `pending` with no auto-validation; no confidence-based validation.

## 3. Read-side gating in search & fetch

- [x] 3.1 In `PgCatalogSearchRepo::search` (`backend/music/src/pg.rs`), add a default `WHERE moderation_status = 'accepted'` for normal callers, composing with all existing filters. Keep `HIT_COLS` free of the status for normal callers.
- [x] 3.2 Add an optional privileged `moderation_status` parameter threaded from the request into `search` that, when present, replaces the accepted-only clause with the requested status.
- [x] 3.3 Gate catalog score **bytes** retrieval (the fetch-bytes handler in `backend/music/src/grpc.rs` / module) so a normal caller gets not-found for a non-`accepted` id, but an admin/moderator identity (admin-only until #3) is served the bytes of a `pending`/`rejected` score so a reviewer can open it.

## 4. gRPC surface & authorization

- [x] 4.1 Add an optional `moderation_status` field to `SearchCatalogRequest` in the score proto; regenerate Rust + Dart bindings (`flutter_rust_bridge_codegen` / proto codegen as applicable). The Dart app does not set this field.
- [x] 4.2 In `search_catalog` (`backend/music/src/grpc.rs`): when the request sets `moderation_status`, call `require_admin(identity)` (`backend/platform/src/guard.rs`) before querying; on failure return `PERMISSION_DENIED` and do not run the query. When unset, run the normal accepted-only path. (Note in code that #3 will widen the guard to admin-or-moderator.)
- [x] 4.3 Ensure the fetch-bytes and search handlers read `AuthIdentity` from request extensions consistently with existing handlers.

## 5. Tests (Rust ≥ 80% coverage)

- [x] 5.1 Unit-test the search repo: default returns only `accepted`; `pending`/`rejected` excluded; privileged status filter returns the requested status; filter composes with text/author/facet filters.
- [x] 5.2 Unit/handler-test the authorization gate: a non-admin request that sets `moderation_status` is rejected with `PERMISSION_DENIED` and runs no query; an admin request is honored; an absent field yields accepted-only.
- [x] 5.3 Test the insert default: a newly inserted catalog row is `pending` regardless of `confidence`.
- [x] 5.4 Test fetch-bytes gating: bytes of a `pending`/`rejected` score are refused (not-found) to a normal caller; `accepted` bytes are served to everyone; `pending`/`rejected` bytes are served to an admin/moderator caller.
- [x] 5.5 Run `cargo llvm-cov --workspace --fail-under-lines 80` (with the repo's ignore-filename-regex) and confirm the new logic is covered.

## 6. Verification & docs

- [x] 6.1 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [x] 6.2 Manually verify (or integration-check) that after migration the hub returns no scores until a row is set to `accepted`, and that flipping one row to `accepted` makes it appear.
- [x] 6.3 `openspec validate add-score-moderation-gating --strict` passes.
- [x] 6.4 Note the **prod deploy caveat** (empty hub until validation, no bulk-accept) in the change/PR description; ensure #3's review tooling is available before running the migration in production so moderators can work the backlog.
