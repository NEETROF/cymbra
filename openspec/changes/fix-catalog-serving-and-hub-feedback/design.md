## Context

Crawler-imported catalog scores appear in the Score Hub but, when tapped, render no notes and no
error. The next day (after the nightly `sync-scores.sh`) they play. Investigation established the
end-to-end path and four defects (see `proposal.md`):

- The crawler (`crates/score-crawler/src/main.rs`) writes every `.mxl` (`OutputWriter::write`) and
  **then** inserts `catalog_scores` rows (`catalog::ingest`) — byte-before-row order is already
  correct. But bytes go to `CRAWL_OUT` (the crawler's own dir), while the server reads
  local-first from `SCORES_DIR` (`CYMBRA_SCORE_LOCAL_ROOT`, `LocalFirstStore`) then falls back to
  S3. The crawler's S3 output backend is not wired (`main.rs` `bail!`). Only `sync-scores.sh`
  (merge `CRAWL_OUT`→`SCORES_DIR` + S3 mirror, nightly) bridges the gap.
- The app fetches bytes per open (no on-disk cache); a fetch failure sets `NotationData.error`, but
  only `_PartitionView` renders it. The default Synthesia and Staff branches render from
  `PlayerData` and never observe `notation.error`.
- The crawler's `.mxl`/MuseScore acceptance (`accept_mxl`/`verify_mxl`) only re-parses; it skips the
  `note_count > 0` gate that `convert_native` (native validation) enforces.
- The hub count binds to `state.entries.length`; no layer (`SearchCatalogResponse`, SQL,
  `CatalogSearchState`) carries a server total.

Constraints: Riverpod 2 + Freezed codegen (no `setState`/`ChangeNotifier` for app state); ≥80% line
coverage both ecosystems; gRPC regen via `melos run gen-grpc` (protoc_plugin pinned); no DB schema
change intended. Solo-maintained small prod (single box, docker-compose + Caddy, OVH/Hetzner).

## Goals / Non-Goals

**Goals:**
- A catalog row visible in the hub is always immediately servable on the same box (no dependence on
  a deferred sync).
- Any load failure or in-flight fetch is visible in the default render mode (spinner + error),
  never a silent blank.
- Noteless scores never enter the catalog.
- The hub shows the true server-side total for the applied filters.

**Non-Goals:**
- Wiring the crawler's S3 output backend (`main.rs` `bail!`) — not needed once output lands directly
  in `SCORES_DIR`; S3 stays the DR origin + read fallback via `sync-scores.sh`.
- An on-disk score cache in the app (every open remains a network fetch) — a possible later perf/
  offline enhancement, out of scope here.
- Retroactively purging already-ingested noteless rows (the gate is forward-looking; a cleanup can
  be a separate ops task).
- DB schema / migration changes.

## Decisions

### D1 — Crawler writes directly into `SCORES_DIR` (infra, root cause)
Mount the served root into the crawler containers instead of a separate `CRAWL_OUT`: in
`backend/deploy/docker-compose.crawler.prod.yml`, each source service mounts
`${SCORES_DIR:-/var/lib/cymbra/scores}:/work/output` (keeping `openscore`'s `docker.sock` +
`SC_CONVTMP`). Because `OutputWriter::write` completes for all files before `catalog::ingest` runs,
a visible row implies its bytes are already at `SCORES_DIR/<prefix>/<shard>/<uuid>.mxl` — the exact
path the server resolves. Object keys are UUID v7, so concurrent source services never collide.
`sync-scores.sh` drops the merge step and keeps only the S3 mirror (DR + read-fallback); `DEPLOY.md`
§11 and the compose header are updated.

- *Alternative — automate `sync-scores.sh` at end of crawl:* keeps `CRAWL_OUT`, but relies on always
  running the extra step and preserves the merge as a correctness dependency. Rejected as more
  fragile.
- *Alternative — wire crawler S3 output:* heavier (implement the S3 backend), and local-first reads
  still need a warm copy or an S3 round-trip on first open. Rejected for now.

### D2 — Surface load state in all render modes (app UX)
`player_screen.dart`'s render area gains `ref.watch(notationProvider)`. Synthesia and Staff branches
add, inside their `Stack`: a `CircularProgressIndicator` while
`selectedScoreProvider != null && notation.error == null && !notation.hasDocument`, and an error
banner (mirroring `_PartitionView`'s `notation.error` block, `CymbraColors.error`) when
`notation.error != null`. `selectedScoreProvider != null` disambiguates loading from the neutral
no-selection state. No state-shape change is required — `NotationData` already models
loading (`document == null && error == null`), loaded, and error.

### D3 — Note-count gate on every acceptance path (crawler hardening)
In `crates/score-crawler/src/convert.rs`, make `accept_mxl`/`verify_mxl` run
`cymbra_musicxml_core::validate` (which returns `RejectReason::NoNotes` when `note_count == 0`),
matching `convert_native`. A single shared gate keeps native and `.mxl`/MuseScore paths symmetric.

### D4 — Thread a server total through the stack (count)
Add `int32 total = 3;` to `SearchCatalogResponse` (`score.proto`), regenerate Rust + Dart. In
`pg.rs`, compute the total with a `COUNT(*) OVER() AS total_count` window column on the existing
filtered `search` query (one round-trip, exact over the full filtered set before `LIMIT/OFFSET`);
change `CatalogSearchRepo::search` to return `(Vec<CatalogHit>, i64)` and update
`FakeCatalogSearchRepo`. `module.rs`/`grpc.rs` propagate and populate `total`. Dart:
`CatalogSearchPage` gains `total`; `CatalogSearchState` gains `int? totalCount` set in `_reload`/
`loadMore`; `score_hub_screen.dart` binds the label to `totalCount` (plus prepended matching
uploads), falling back to `entries.length` in my-scores mode.

- *Alternative — separate `SELECT COUNT(*)`:* a second query with a duplicated WHERE clause; more
  drift risk than the window count. Rejected.

## Risks / Trade-offs

- **Crawler writes into the live serving dir** → mitigation: rows are inserted only after all files
  are written, and files are UUID-named, so the server never resolves a row to a half-written or
  missing file; a mid-crawl abort leaves harmless orphan files (as `CRAWL_OUT` does today).
- **Filesystem permissions on `SCORES_DIR`** → the crawler container UID must be able to write it
  (`chown -R 1000:1000 $SCORES_DIR`, already noted for the server). Document in `DEPLOY.md`.
- **S3 mirror now lags crawl** → newly crawled bytes exist only in `SCORES_DIR` until the next
  `sync-scores.sh`; a box rebuild before the mirror loses the unsynced delta. Acceptable (DR window,
  same nightly cadence); mention running `sync-scores.sh` after large crawls.
- **`COUNT(*) OVER()` cost** on large corpora → acceptable at current scale; the count is over the
  already-filtered set and shares the single query. Revisit only if the corpus grows large.
- **Proto field addition** is backwards-compatible (new optional field `= 3`); older clients ignore
  it. Regen must run for both Rust and Dart or the field is invisible.

## Migration Plan

1. Land code + regen (gRPC) behind the existing (unreleased) catalog surface; no DB migration.
2. Deploy server/worker as usual.
3. Update the crawler deploy: set the new volume mounts, ensure `SCORES_DIR` is writable by the
   crawler UID, deploy the trimmed `sync-scores.sh`.
4. Run a smoke crawl (`LIMIT=5`) → verify `.mxl` under `SCORES_DIR/...` and that a fresh score plays
   without running `sync-scores.sh`; then run `sync-scores.sh` and confirm the S3 mirror.
- Rollback: revert the compose/script/proto+code changes; the additive proto field and the direct
  `SCORES_DIR` write are independently revertible.

## Open Questions

- Should already-ingested noteless rows (if any exist in prod) be purged now, or left to a separate
  ops cleanup? (Proposed: separate task.)
