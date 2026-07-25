## Why

Scores imported by the score-crawler show up in the Score Hub but, when tapped, render no notes
and surface no error — a blank player. The next day (after the nightly `sync-scores.sh`) the same
scores play. The root cause is that catalog rows become visible before their bytes are servable,
compounded by the app swallowing the resulting load error. Two adjacent defects surfaced while
diagnosing: noteless scores can slip into the catalog, and the hub's result count reflects only
the page loaded in memory, not the true server-side total for the applied filters.

## What Changes

- **Ingestion availability (root cause):** a `catalog_scores` row MUST only be discoverable once its
  bytes are resolvable by the app's read path (local `SCORES_DIR` or S3). The crawler deployment
  fan-out writes its output directly into the served directory (`SCORES_DIR`) instead of a separate
  `CRAWL_OUT` that needs a deferred merge; `sync-scores.sh` is reduced to the S3 mirror (DR +
  read-fallback).
- **Load feedback:** every player render mode (Synthesia default, Staff, Partition) MUST show a
  loading indicator while a score is being fetched/parsed and an error banner on failure —
  distinguishing "no score selected" from "loading". Today only the Partition view surfaces this.
- **Noteless-score gate (hardening):** the crawler's `.mxl`/MuseScore acceptance path MUST enforce
  the same "at least one playable note" gate as native MusicXML validation, rejecting noteless
  scores at ingest instead of admitting them to the catalog.
- **True result count:** the catalog search response MUST report the total number of rows matching
  the filters (independent of the page), and the hub MUST display that server total (plus matching
  local uploads) rather than the number of results currently loaded in memory. In "my scores" mode
  (no server total) the local list length still stands.

## Capabilities

### New Capabilities
<!-- None — all changes modify existing capabilities. -->

### Modified Capabilities
- `corpus-ingestion`: a catalog row is only discoverable once its bytes are resolvable via the app
  read path; the crawler deploy fan-out writes output directly into the served dir (no deferred
  merge).
- `score-notation`: rendering state (loading indicator + error banner, and no-selection vs loading)
  applies to all render modes, not only Partition.
- `score-conversion`: output verification requires at least one pitched, non-rest note; noteless
  scores are rejected at ingest.
- `catalog-search`: the search response reports the total count of matching rows, independent of the
  returned page.
- `score-hub`: the hub result count displays the server-side total for the applied filters (plus
  matching local uploads), not the in-memory loaded count.

## Impact

- **Backend (Rust):**
  - `crates/score-crawler` — `convert.rs` acceptance/verification path (note-count gate); deploy
    topology (`backend/deploy/docker-compose.crawler.prod.yml`, `sync-scores.sh`, `DEPLOY.md`).
  - `backend/music` — `proto/score.proto` (`SearchCatalogResponse.total`), `pg.rs`
    (`COUNT(*) OVER()`), `catalog_search.rs` trait + `FakeCatalogSearchRepo`, `module.rs`,
    `grpc.rs`. gRPC regen (Rust + Dart) via `melos run gen-grpc`.
- **Flutter app:**
  - `lib/state/notation_notifier.dart` / `lib/screens/player_screen.dart` — loading/error surfacing
    in Synthesia + Staff modes.
  - `lib/services/catalog_service.dart` / `lib/state/catalog_search_notifier.dart` /
    `lib/screens/score_hub_screen.dart` + l10n — thread and display the server total.
- **Ops:** deploy runbook change (crawler writes to `SCORES_DIR`); `sync-scores.sh` behavior scoped
  to the S3 mirror. No DB schema/migration change.
