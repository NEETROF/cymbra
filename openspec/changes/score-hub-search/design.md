## Context

The crawler already ingests a public, redistributable corpus into
`music.catalog_scores` (see `backend/music/migrations/0001_catalog.sql`), with the
search/musical metadata captured at ingest — including `title_norm` (accent/case-folded
title), `composer`, `level`, `work_key`, and facet indexes. The migration explicitly
states the fuzzy/full-text `pg_trgm` GIN index on `title_norm`/composer is *"deferred to
that change and must enable the extension from an admin/ops migration, not this module
role."* This change is that change.

Today the `music` module's gRPC `ScoreService` (`backend/music/proto/score.proto`,
`grpc.rs`, `module.rs`) only serves a signed-in user's **own uploads** (`user_scores`):
`UploadScore`, `ListMyScores`, `DeleteScore`, `GetScoreBytes`. The caller's identity
comes from an internal-token interceptor set as a request extension (`owner()` in
`grpc.rs`), never the body. The app talks to it through `GrpcScoreUploadService`
(`apps/music/lib/services/score_upload_service.dart`) over the shared channel in
`grpc_client.dart`.

On the app side, `LibraryScreen` (`apps/music/lib/screens/library_screen.dart`) is the
home route. It renders the bundled `scoreCatalog` grouped by `PracticeLevel`, plus a
"MES CONTRIBUTIONS" section fed by `myContributedScoresProvider`
(`state/contributed_scores.dart`), which maps backend records to `CatalogEntry`. Tapping
a tile calls `selectedScoreProvider.select(entry)` and pushes `PlayerScreen`. Contributed
entries are byte-sourced (they carry `contributedId`, no `assetPath`).

**Product decisions locked with the user:** the hub is a **dedicated screen** reached from
the home app bar (like the contribution icon); **sign-in is required** to search and to
save; saved selections are **persisted backend-side and synced across devices** (a save
on one device appears on another; a removal clears it everywhere), and **removing a save
never deletes the public catalog entry**; search is backed by **pg_trgm**. The hub
filters results by **author (composer)** and offers a **"mes partitions" quick-filter that
scopes the hub to the user's own uploaded scores** (`user_scores`, via the existing
`ListMyScores`) — *not* the saved-catalog selection.

## Goals / Non-Goals

**Goals:**
- Expose the public `catalog_scores` corpus to signed-in users via a searchable, filterable
  hub, and let them pin scores that then appear on the home screen alongside bundled and
  contributed scores, opening in the player through the existing notation/playback path.
- Full-text search on **title and composer** tolerant of case/accents/typos, composable with
  a difficulty filter, paginated, index-backed by `pg_trgm`.
- Reuse the existing identity interceptor, object-store byte-source pattern, and
  `CatalogEntry`/`selectedScoreProvider` app plumbing — minimal new surface.

**Non-Goals:**
- No unauthenticated/public search surface (auth required, per decision).
- No new ranking/relevance beyond trigram similarity + a deterministic tiebreak; no facets
  beyond difficulty (source/licence/instrument filters are out of scope here).
- No offline caching of catalog bytes beyond what the player already does per-session; no
  local (device-only) saved list — persistence is backend-only.
- No changes to how bundled or contributed scores work.
- No new corpus curation or difficulty re-estimation (uses `level` as ingested).

## Decisions

### D1 — Search engine: pg_trgm trigram similarity (not tsvector FTS)
Use `pg_trgm` with a GIN index over the normalised title and a normalised composer, matching
with `ILIKE '%q%'` / `similarity()` and ordering by similarity then a deterministic key.
- **Why:** the corpus is multilingual public-domain works searched by *name fragments* (a
  composer surname, part of a title). Trigram substring/fuzzy matching tolerates typos and
  partial words and suits search-as-you-type — and it is the exact index the schema comment
  already earmarked. tsvector/tsquery adds stemming/lexeme weighting better suited to prose,
  is more brittle on short name queries, and needs per-language configs the corpus can't
  guarantee.
- **Alternative considered:** Postgres FTS (`to_tsvector('simple', title||' '||composer)` +
  GIN). Rejected for the above; can be layered later if word-relevance ranking is wanted.
- **Normalisation:** `title_norm` already exists. Composer has no normalised column; either
  add a generated/normalised `composer_norm` (accent/case-folded) covered by the trigram index,
  or index `lower(unaccent(composer))`. Prefer a persisted `composer_norm` column populated at
  ingest-parity (backfill for existing rows) so the index is a plain column GIN and the crawler
  keeps it fresh — keeps queries simple and index-only.

### D2 — Extension + index migration split by privilege
`CREATE EXTENSION pg_trgm` requires elevated privilege the least-privilege `music_svc` role
lacks (same reason `CREATE SCHEMA` is done by ops, per the migration header). So:
- an **ops/admin migration** enables `pg_trgm` (alongside where schemas/roles are provisioned);
- a **module migration** (`backend/music/migrations/000X_*.sql`) creates the trigram GIN
  index(es) and the `composer_norm` column + backfill, guarded (`IF NOT EXISTS`) and
  fully-qualified, matching the existing migration conventions.

### D3 — Saved library: a new `music.user_library` table
Persist saves as owner-scoped rows referencing a catalog id:
```
music.user_library(
  owner_id  UUID NOT NULL,     -- caller's AuthIdentity.user_id; no cross-schema FK (module isolation)
  catalog_id UUID NOT NULL,    -- references music.catalog_scores(id) — same schema, FK allowed
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (owner_id, catalog_id)
)
```
- **Why a table, not JSON on a user row:** owner-scoped list/insert/delete are the only ops;
  a composite-PK join table gives idempotent save (`ON CONFLICT DO NOTHING`), O(1) remove, and
  cheap owner-scoped list — mirroring how `user_scores` models per-user ownership. `owner_id`
  stays a plain UUID (no cross-schema FK to the user module, per existing module-isolation
  convention); `catalog_id` **may** FK to `catalog_scores` since both live in the `music`
  schema, giving referential cleanup — but a crawler re-ingest can change ids, so the list read
  joins to `catalog_scores` and simply omits rows whose catalog entry is gone (spec: stale save
  not surfaced as broken). Choose `ON DELETE CASCADE` if the FK is added.
- **Account deletion:** the existing account-erasure path that purges `user_scores` also deletes
  `user_library` rows for the owner. Add it to that purge (owner-scoped `DELETE`).

### D4 — gRPC surface: extend `ScoreService`, identity from the interceptor
Add to `score.proto` (all authenticated; `owner()`/identity from the interceptor, never the body):
- `SearchCatalog(SearchCatalogRequest{ query, author?, level?, limit, offset }) -> SearchCatalogResponse{ repeated CatalogHit, next_offset?, total? }`
  where `CatalogHit{ id, title, composer, level, license, source }` (no bytes). `author`
  is an optional structured composer filter (trigram/`ILIKE` on `composer_norm`) that
  composes with the free-text `query` and the `level` filter — see D7.
- `SaveCatalogScore(SaveCatalogScoreRequest{ catalog_id }) -> SaveCatalogScoreResponse{}` (idempotent).
- `RemoveSavedCatalogScore(RemoveSavedCatalogScoreRequest{ catalog_id }) -> …{}` (idempotent no-op).
- `ListSavedCatalogScores(ListSavedCatalogScoresRequest{}) -> ListSavedCatalogScoresResponse{ repeated CatalogHit }`.
- `GetCatalogScoreBytes(GetCatalogScoreBytesRequest{ catalog_id }) -> GetCatalogScoreBytesResponse{ bytes data }`.
- **Why extend `ScoreService` rather than a new service:** same module, same object store, same
  interceptor; the app already wires `ScoreServiceClient`. `SearchCatalog`/`Save…` are catalog-wide
  but still require an authenticated identity, so no interceptor change is needed — they read the
  identity like the existing RPCs; search just doesn't *scope by* owner, save/list do.
- Regenerate FRB/gRPC stubs after the proto change (Dart `score.pbgrpc.dart`, Rust `proto`).

### D5 — Business logic in `ScoreModule`, ports in `repo.rs`/`pg.rs`
- Add a `CatalogSearchRepo` port (search + get-by-id-for-bytes) and a `UserLibraryRepo` port
  (save/remove/list), each with a `Fake*` for host tests and a `Pg*` adapter — matching the
  existing `CatalogRepo`/`UserScoreRepo` split so unit tests need no Postgres.
- `ScoreModule` gains `search_catalog`, `save_catalog_score`, `remove_saved_catalog_score`,
  `list_saved_catalog_scores`, `get_catalog_bytes` — clamping `limit` to a server max, validating
  `level` against the fixed set, and validating catalog-id existence on save.
- Bytes read from `ObjectStorage` under the public-corpus prefix using `catalog_scores.object_key`,
  the same local-first/S3 fallback the contributed-score bytes use.

### D6 — App: dedicated hub screen + reuse `CatalogEntry`
- `ScoreHubScreen` reached from a new app-bar action on `LibraryScreen`, gated by
  `canUseOnlineServicesProvider` (same guard as the contribution icon).
- State via Riverpod codegen (per CLAUDE.md): a `catalogSearchNotifier` (query + level +
  paging, debounced) calling a new `CatalogService` seam over `ScoreServiceClient`, and a
  `savedCatalogScoresProvider` (list saved → `CatalogEntry`s) that the home screen watches.
  `SaveCatalogScore`/`Remove…` mutate through the service and invalidate the saved provider.
- Map `CatalogHit` → the existing `CatalogEntry` (add a `catalogId` field paralleling
  `contributedId`; a byte source keyed by `catalogId` fetches via `GetCatalogScoreBytes`). This
  lets the home section and the player reuse the exact bundled/contributed rendering + selection
  path — a saved catalog score is "just another byte-sourced entry."
- `LibraryScreen` gains a "Découvertes"/saved section between contributions and bundled (order TBD
  in implementation), each tile openable and removable (remove → backend + invalidate).
- New localized strings (hub title, search hint, filter labels, empty/no-results, add/remove,
  attribution) across the app's locales.

### D7 — Author filter (structured) and the "mes partitions" source scope
Two distinct filter concepts sit alongside the difficulty filter in the hub:
- **Author filter (backend):** `SearchCatalog` gains an optional `author` param, a
  *structured* composer filter distinct from the free-text `query`. It narrows to catalog
  scores whose composer matches (trigram/`ILIKE` on `composer_norm`, the same normalised
  column the text search uses), composing with `query` and `level` (all applied
  conjunctively). Kept separate from `query` so a user can, e.g., free-text a title *and*
  pin the composer. Empty `author` imposes no composer constraint. This is server-side and
  index-backed; no extra column beyond `composer_norm` (D1).
- **"Mes partitions" quick-filter (client, reuses `ListMyScores`):** a source toggle in
  the hub. Default source is the **public catalog** (`SearchCatalog`). When "mes
  partitions" is active, the source switches to the **caller's own uploaded scores**
  (`user_scores`) fetched via the **existing `ListMyScores` RPC** — *no new backend*. The
  author and difficulty filters (and the text query) still apply, filtered client-side over
  that bounded, already-owned list (uploads are few). Because these are the user's own
  scores, hub results in this mode **do not** offer the add/remove-to-library toggle (they
  are already owned; they are managed through the upload/contribution flow) — tapping one
  opens it in the player like any contributed entry.
- **Why uploads, not saved catalog:** locked with the user — "mes partitions" means *my
  uploads*. The saved-catalog selection already surfaces on the home screen (score-library);
  the hub's shortcut is a discovery convenience to find one's *own* uploaded pieces while in
  the hub, reusing plumbing that already exists (`myContributedScoresProvider`/`ListMyScores`).

## Risks / Trade-offs

- **[pg_trgm quality on very short queries]** → 1–2 char queries produce low-similarity noise;
  clamp with a minimum length before switching from prefix/`ILIKE` browse to similarity ranking,
  and always apply a deterministic secondary sort so paging is stable.
- **[Composer not normalised today]** → adding `composer_norm` + backfill touches every existing
  row; do it in one guarded migration and have the crawler populate it going forward, else search
  misses accented composer names. Mitigation: functional index `lower(unaccent(composer))` as a
  fallback if the column backfill is deferred.
- **[Extension privilege]** → `CREATE EXTENSION` from the module role will fail (least privilege);
  putting it in the ops migration is mandatory, not cosmetic. Mitigation: fail fast in an
  ops-preflight; document in deploy notes.
- **[Stale saves after crawler re-ingest]** → a re-ingest can change catalog ids/dedup rows,
  orphaning `user_library` rows. Mitigation: list read joins and omits missing entries (spec);
  optionally key saves by `content_fingerprint`/`work_key` instead of `id` — deferred, `id` is
  simpler and re-ingest is rare.
- **[Auth gate vs. public corpus]** → requiring sign-in to browse a *public* corpus adds friction;
  accepted per product decision (keeps one identity model, syncs saves). Revisit if a
  logged-out discovery surface is later wanted (would need an unauthenticated read RPC).
- **[Pagination drift]** → offset paging can skip/duplicate if the corpus changes mid-scroll; the
  corpus is near-static between crawls, so offset is acceptable; a `work_key`/id keyset cursor is
  the upgrade path if needed.
- **[Coverage gates]** → search SQL and byte I/O live behind the object-store/pg seams that the
  coverage config already excludes; keep the pure logic (clamping, level validation, hit mapping,
  save idempotency) in host-testable module/core code to stay ≥ 80% both ecosystems.

## Migration Plan

1. Ops/admin migration: `CREATE EXTENSION IF NOT EXISTS pg_trgm`.
2. Module migration: add `composer_norm` (+ backfill), trigram GIN index(es) on
   `title_norm`/`composer_norm`, and create `music.user_library`.
3. Extend `score.proto`; regenerate Rust proto + Dart gRPC stubs.
4. Implement repos/adapters + `ScoreModule` methods + gRPC handlers; wire account-deletion purge.
5. App: `CatalogService` seam, hub screen + state, home section + entry point, l10n.
6. **Rollback:** the new RPCs/screen are additive — reverting the app hides the hub; dropping
   `user_library` and the index is safe (no other consumer). Keep `pg_trgm` (harmless if unused).

## Open Questions

- **Saved-section placement/label** on the home screen (above or below "MES CONTRIBUTIONS"; label
  wording) — a UI detail, settle during implementation.
- **`composer_norm` column vs. functional index** — decide at migration time based on backfill cost;
  design leans to the persisted column for index-only queries.
- **Catalog id vs. fingerprint as the save key** — `id` for the POC; revisit only if crawler
  re-ingest churn proves to orphan saves in practice.
