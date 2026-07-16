## 1. Database & migrations

- [x] 1.1 Ops/admin migration: `CREATE EXTENSION IF NOT EXISTS pg_trgm` (in the ops role/schema provisioning path, not the module role).
- [x] 1.2 Module migration: add `composer_norm` (accent/case-folded) to `music.catalog_scores`, backfill existing rows, guarded + fully-qualified.
- [x] 1.3 Module migration: create trigram GIN index(es) over `title_norm` and `composer_norm` (`IF NOT EXISTS`).
- [x] 1.4 Module migration: create `music.user_library(owner_id, catalog_id, created_at, PRIMARY KEY(owner_id, catalog_id))`, owner index; decide + apply optional FK `catalog_id -> catalog_scores(id) ON DELETE CASCADE`.
- [x] 1.5 Make the crawler ingest populate `composer_norm` going forward (parity with `title_norm`).

## 2. Backend — ports & adapters

- [x] 2.1 Add a `CatalogSearchRepo` port (`search(query, author, level, limit, offset) -> hits + paging`, `get_by_id -> object_key/metadata`) with a `FakeCatalogSearchRepo` for host tests.
- [x] 2.2 Implement `PgCatalogSearchRepo` in `pg.rs`: trigram similarity/`ILIKE` on `title_norm`/`composer_norm`, optional structured `author` (composer) filter and `level` equality applied conjunctively, clamp+offset paging, deterministic secondary sort.
- [x] 2.3 Add a `UserLibraryRepo` port (`save` idempotent, `remove` idempotent no-op, `list(owner) -> hits joined to catalog`) with a `FakeUserLibraryRepo`.
- [x] 2.4 Implement `PgUserLibraryRepo` in `pg_user_scores.rs`/new module: `ON CONFLICT DO NOTHING` save, owner-scoped delete, list joined to `catalog_scores` omitting missing entries.
- [x] 2.5 Unit-test both fakes/adapters: match on title & composer, accent/case tolerance, author filter compose, difficulty filter compose, paging, save idempotency, owner isolation, list reflects prior save/remove (sync source-of-truth).

## 3. Backend — module logic

- [x] 3.1 `ScoreModule::search_catalog`: validate `level` against the fixed set, pass through the optional `author` filter, clamp `limit` to server max, empty-query browse path.
- [x] 3.2 `ScoreModule::save_catalog_score`: validate catalog id exists, then idempotent save (owner-scoped).
- [x] 3.3 `ScoreModule::remove_saved_catalog_score` and `list_saved_catalog_scores` (owner-scoped).
- [x] 3.4 `ScoreModule::get_catalog_bytes`: resolve `object_key`, read from `ObjectStorage` public-corpus prefix (local-first + S3 fallback), typed not-found on unknown id.
- [x] 3.5 Extend the account-deletion purge to also delete the owner's `user_library` rows.
- [x] 3.6 Unit-test module logic: level validation, limit clamp, unknown-id save/bytes rejection, purge-on-account-deletion.

## 4. Backend — gRPC surface

- [x] 4.1 Extend `backend/music/proto/score.proto`: `SearchCatalog` (`SearchCatalogRequest` with `query`, optional `author`, optional `level`, `limit`, `offset`), `SaveCatalogScore`, `RemoveSavedCatalogScore`, `ListSavedCatalogScores`, `GetCatalogScoreBytes` + messages (`CatalogHit` with id/title/composer/level/license/source, no bytes).
- [x] 4.2 Regenerate Rust proto stubs (build.rs) and confirm compile.
- [x] 4.3 Implement the handlers in `grpc.rs`: identity via the interceptor (reject unauthenticated), map module results/errors to `Status`, never trust the body for identity.
- [x] 4.4 gRPC-level tests: unauthenticated rejection, search shape (query + author + level compose), save/list/remove round-trip, list reflects a prior save/remove for the same owner (cross-session sync), bytes for known/unknown id.

## 5. App — service seam & state (Riverpod + codegen)

- [ ] 5.1 Add a `catalogId` field to `CatalogEntry` (parallel to `contributedId`) and an `isCatalog` getter; keep bundled/contributed behavior unchanged.
- [ ] 5.2 Add a `CatalogService` seam (abstract + `GrpcCatalogService` over `ScoreServiceClient`, bearer-authed with transparent refresh like `GrpcScoreUploadService`): `search(query, author, level, page)`, `save`, `remove`, `listSaved`, `fetchBytes`.
- [ ] 5.3 `catalogSearchNotifier` (`@riverpod`): source mode (catalog vs. "mes partitions"), query + author + difficulty + paging, debounced; exposes results + load-more + saved-state. In catalog mode it calls `CatalogService.search`; in "mes partitions" mode it sources the user's uploads via the existing `ListMyScores` path (`myContributedScoresProvider`/upload service) and filters them client-side by query/author/level.
- [ ] 5.4 `savedCatalogScoresProvider` (`@riverpod`): list saved → `CatalogEntry`s; empty when signed out; invalidated on save/remove. (Note: "mes partitions" surfaces uploads, not this saved set — keep the two distinct.)
- [ ] 5.5 Catalog byte source keyed by `catalogId` (parallel to the contributed byte source) wired into the player's score-source path.
- [ ] 5.6 Run `build_runner`; ensure `custom_lint`/`riverpod_lint` clean.

## 6. App — Score Hub screen

- [ ] 6.1 `ScoreHubScreen`: search field (title/author), author filter control (clearable), difficulty filter (all-levels default), and a "mes partitions" quick-filter chip that scopes the hub to the user's uploads (signed-in only); paginated results with load-more, attribution shown per catalog result.
- [ ] 6.2 Add/remove-from-library toggle per catalog result reflecting saved state, mutating through `CatalogService` and invalidating `savedCatalogScoresProvider`; suppress the add/remove action in "mes partitions" mode (uploads are already owned) — tapping opens the player.
- [ ] 6.3 Empty-query browse, no-results, and empty-"mes partitions" states.
- [ ] 6.4 Add the hub entry point (app-bar action) on `LibraryScreen`, gated by `canUseOnlineServicesProvider`; not reachable when signed out.

## 7. App — home screen integration

- [ ] 7.1 Add a saved-catalog section to `LibraryScreen` distinct from bundled catalog and "MES CONTRIBUTIONS", watching `savedCatalogScoresProvider` (hidden when signed out/empty).
- [ ] 7.2 Saved tiles open in the player via `selectedScoreProvider` + the catalog byte source; retain keyboard/MIDI/transport.
- [ ] 7.3 Per-tile remove action (backend remove + invalidate), leaving the public catalog intact.
- [ ] 7.4 Add localized strings for the hub + saved section across locales (title, search hint, author-filter label, difficulty-filter labels, "mes partitions" chip, empty/no-results, add/remove, attribution).

## 8. Tests, coverage & pre-PR

- [ ] 8.1 Widget tests: hub search/author-filter/difficulty-filter/paging/add-remove with an in-memory `CatalogService` override; "mes partitions" chip switches source to uploads and hides add-to-library; home saved-section visibility (signed in/out/empty) and removal.
- [ ] 8.2 Confirm Rust ≥ 80% (`cargo llvm-cov ... --fail-under-lines 80`) and Flutter ≥ 80% (`flutter test --coverage`), pure logic kept in host-testable seams.
- [ ] 8.3 `melos run analyze` + `dart format`; `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 8.4 Regenerate gRPC/FRB stubs if the public API changed; `openspec validate score-hub-search --strict` passes.
