## Why

The crawler has ingested a public, redistributable corpus into `music.catalog_scores`
(title/composer/level + search metadata already captured at ingest), but nothing in
the app surfaces it: users only ever see the handful of bundled scores and their own
uploads. Players want to *discover* new pieces to practise — search the corpus by
title or composer, narrow by difficulty, and pin the ones they like so they sit on the
home screen alongside the bundled and contributed scores. The `catalog_scores`
migration explicitly deferred the full-text index "to that change" — this is it.

## What Changes

- Add an authenticated **catalog search** to the backend `music` module: full-text
  matching on title **and** composer plus an optional **author (composer) filter** and
  an optional difficulty filter, all composable, paginated, returning
  attribution-complete results over the public corpus. Enable `pg_trgm` and add a
  trigram GIN index on `title_norm` + a normalised composer key (the deferred index
  from `0001_catalog.sql`).
- Add a per-user **saved-catalog library**: a signed-in user can save a catalog score
  to their personal library and remove it later. The saved set is persisted
  backend-side (owner-scoped) and therefore **syncs across every device of the account**
  — a save on one device shows on another, a removal on one clears it everywhere.
  **Removing a save deletes only the owner's save record, never the public catalog
  entry** (the score stays discoverable and re-savable from the hub). Saved catalog
  scores are erased with the account (extends the existing account-deletion purge).
- Serve **catalog score bytes** to open a saved/searched catalog score in the player,
  paralleling the existing contributed-score byte source (object store, public-corpus
  prefix).
- Add a **Score Hub** screen in the app: a search field (title/author), an **author
  filter**, a difficulty filter, and a **"mes partitions" quick-filter that scopes the
  hub to the user's own uploaded scores** (sourced from the existing `ListMyScores`, no
  new backend); paginated results with attribution, and an add/remove-from-library
  toggle on catalog results. Reached from the home app bar, gated to signed-in users
  (mirrors the existing "contribute" entry point).
- Surface **saved catalog scores on the home screen** as a section distinct from the
  bundled catalog and the user's own contributions; each opens in the player like any
  other entry and can be removed from the library.

## Capabilities

### New Capabilities
- `catalog-search`: authenticated full-text + author-filtered + difficulty-filtered,
  paginated search over the public `catalog_scores` corpus, and fetching a catalog
  score's bytes to play.
- `saved-catalog-library`: per-user persistence of saved catalog scores (save, remove,
  list), owner-scoped, synced across the account's devices, and erased with the account.
- `score-hub`: the app's Score Hub screen — search field, author filter, difficulty
  filter, a "mes partitions" quick-filter scoping to the user's uploads, paginated
  results with attribution, and add/remove-from-library, gated to signed-in users.

### Modified Capabilities
- `score-library`: the home screen additionally presents the signed-in user's saved
  catalog scores as a distinct section, byte-sourced, selectable to open in the player,
  and removable — and exposes the entry point to the Score Hub.

## Impact

- **Backend `music` module**: new `SearchCatalog` (with optional `author` + `level`
  filters) / `SaveCatalogScore` / `RemoveSavedCatalogScore` / `ListSavedCatalogScores` /
  `GetCatalogScoreBytes` RPCs on `score.proto`; new query paths in `repo.rs`/`pg.rs`;
  module logic in `module.rs`. The hub's "mes partitions" filter reuses the existing
  `ListMyScores` RPC — no new backend surface for it.
- **Migrations**: an ops/admin migration to `CREATE EXTENSION pg_trgm` + trigram GIN
  index on the catalog; a module migration for the `music.user_library` table.
- **App**: new hub screen + Riverpod state (search notifier, saved-library notifier),
  a catalog byte source, a Dart `CatalogService`/`ScoreService` client extension, and a
  new section + entry point on `LibraryScreen`. New localized strings.
- **Auth**: search/save/list/bytes RPCs go through the existing internal-token identity
  interceptor (all owner-aware or authenticated-only); no unauthenticated surface added.
- **Account deletion**: the account-erasure path also purges `user_library` rows.
- Coverage gates (Rust + Flutter ≥ 80%) apply to all new logic.
