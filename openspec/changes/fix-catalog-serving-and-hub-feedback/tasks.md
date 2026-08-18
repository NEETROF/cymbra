## 1. Crawler: note-count gate (C)

- [x] 1.1 In `crates/score-crawler/src/convert.rs`, make `accept_mxl`/`verify_mxl` run `cymbra_musicxml_core::validate` (note_count > 0), matching `convert_native`, so `.mxl`/MuseScore-origin inputs are gated identically.
- [x] 1.2 Add a unit test: a parseable but noteless `.mxl` fixture is rejected with `RejectReason::NoNotes` and not counted as converted.
- [x] 1.3 `cargo test -p score-crawler`, `cargo clippy`, `cargo fmt` clean.

## 2. Backend: search total count (D)

- [x] 2.1 Add `int32 total = 3;` to `SearchCatalogResponse` in `backend/music/proto/score.proto`.
- [x] 2.2 In `backend/music/src/pg.rs`, add `COUNT(*) OVER() AS total_count` to the `PgCatalogSearchRepo::search` query (over the filtered set, before LIMIT/OFFSET); read the total from the first row.
- [x] 2.3 Change the `CatalogSearchRepo::search` trait (`catalog_search.rs`) to return `(Vec<CatalogHit>, i64)`; update `FakeCatalogSearchRepo` and existing tests.
- [x] 2.4 Propagate the total through `ScoreModule::search_catalog` (`module.rs`) and populate `SearchCatalogResponse.total` in `grpc.rs`.
- [x] 2.5 Add/extend a test asserting the total reflects the full filtered match count, independent of limit/offset.
- [x] 2.6 Regenerate gRPC bindings (Rust + Dart) via `melos run gen-grpc`.
- [x] 2.7 `cargo test -p cymbra-music`, `cargo clippy`, `cargo fmt` clean.

## 3. App: load feedback in all render modes (B)

- [x] 3.1 In `apps/music/lib/screens/player_screen.dart`, give the render area access to `notationProvider` (`ref.watch(notationProvider)` in the enclosing `Consumer`/`_buildRenderArea`).
- [x] 3.2 Synthesia branch: show a `CircularProgressIndicator` while `selectedScoreProvider != null && notation.error == null && !notation.hasDocument`, and an error banner (mirroring `_PartitionView`, `CymbraColors.error`) when `notation.error != null`.
- [x] 3.3 Staff branch: same loading indicator + error banner treatment.
- [x] 3.4 Add any needed l10n strings (loading / load-error) to `app_fr.arb` + `app_en.arb`; regen localizations.
- [x] 3.5 Widget test: `notationProvider` overridden → loading state shows a spinner in Synthesia; `error` state shows the error banner in Synthesia and Staff.

## 4. App: hub result count shows server total (D)

- [x] 4.1 In `apps/music/lib/services/catalog_service.dart`, add `total` to `CatalogSearchPage` and map `resp.total` in `search`.
- [x] 4.2 In `apps/music/lib/state/catalog_search_notifier.dart`, add `int? totalCount` to `CatalogSearchState` (Freezed) and set it from `page.total` in `_reload` and `loadMore`; leave it null in `myScoresOnly` mode. (Display via `displayCount` = uploads + server total.)
- [x] 4.3 In `apps/music/lib/screens/score_hub_screen.dart` (~line 145), bind the count label to the server total (`totalCount` + matching prepended uploads), falling back to `entries.length` when `totalCount == null` (my-scores mode).
- [x] 4.4 Run `dart run build_runner build --delete-conflicting-outputs` (Freezed/Riverpod codegen).
- [x] 4.5 Widget test: fake catalog service returns total=500 on a 20-item page → the hub label shows 500 (not 20); my-scores mode still shows the local count.

## 5. Infra: crawler writes into SCORES_DIR (A)

- [x] 5.1 In `backend/deploy/docker-compose.crawler.prod.yml`, change each source service's output volume from `${CRAWL_OUT}/<source>:/work/output` to `${SCORES_DIR:-/var/lib/cymbra/scores}:/work/output`; keep `openscore`'s `docker.sock` + `SC_CONVTMP` mounts. Update the file header comment.
- [x] 5.2 Simplify `backend/deploy/sync-scores.sh`: remove the `CRAWL_OUT`→`SCORES_DIR` merge step, keep the S3 mirror; update its header docstring.
- [x] 5.3 Update `backend/deploy/DEPLOY.md` §11: crawler writes directly to `SCORES_DIR`; `sync-scores.sh` only mirrors to S3; note the `chown -R 1000:1000 $SCORES_DIR` write-permission requirement for the crawler UID.

## 6. Validation

- [x] 6.1 `openspec validate fix-catalog-serving-and-hub-feedback --strict` passes.
- [x] 6.2 Rust coverage: `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex 'frb_generated|/lib\.rs|/midi\.rs|/musicxml\.rs|/audio\.rs'` → PASS (lines 80.58%).
- [x] 6.3 Flutter: `flutter analyze` + `dart run custom_lint` clean; `flutter test --coverage --exclude-tags golden` → 442 passed. (The ≥80% gate is enforced in CI on the unit+widget coverage **merged with the integration run**; not reproducible locally without a device. New code — `_ScoreLoadOverlay`, `displayCount`, catalog `total` mapping — is covered by the new widget/unit tests.)
- [ ] 6.4 MANUAL / live env (A): smoke crawl with the new mount → `.mxl` appears under
      `SCORES_DIR/<prefix>/<shard>/<uuid>.mxl`; open a freshly crawled score in the hub
      **without** running `sync-scores.sh` → it plays; then run `sync-scores.sh` → S3 mirror
      OK.
      **Two of the three assertions are verified (2026-08-18, prod).** Crawls write straight
      into `SCORES_DIR` under `<prefix>/<shard>/<name>.mxl` and the corpus root gains nothing
      else, and `sync-scores.sh` mirrors it off-box successfully — both proven while
      validating `fix-crawler-corpus-isolation` (its tasks 7.3 and 7.5). Note the shard file
      name is now the content hash, not a UUID, since that change.
      **The live open is blocked on there being no new content to crawl.** Every source is
      exhausted against what is already ingested: musetrainer 68/68, eduardomourar 3/3,
      mutopia 1032 retained and 0 inserted on a full run. Only pdmx has upstream left
      (~142 769 of ~250 000 ingested). Forcing a fresh row by deleting an existing one was
      considered and **rejected**: nine tables cascade from `catalog_scores`, including
      `user_library`, `score_ratings` and `leaderboard_bests`, so it would destroy user data.
      Worth recording while the box is fresh in mind: the window this assertion exists to
      exclude can no longer occur by construction — the crawler writes every object before
      inserting any row, and `sync-scores.sh` was reduced to a mirror with no merge step at
      all. The live open remains worth doing as end-to-end proof.
      **Close it on the next pdmx crawl**, which will ingest ~100k genuinely new pieces. Do
      the object-key re-keying change first: re-crawling already-known content writes one
      orphan per piece (measured: +1032 objects, 0 rows, on the mutopia run above), so a pdmx
      run today would leave ~142k orphans to purge — 1h30 of deletions at the rate measured.
- [x] 6.5 MANUAL / device (B): with the backend unreachable, tapping a catalog score shows a spinner then an error banner (no silent blank) — **verified on device 2026-08-16**. Observed *before* entering the pre-play screen: the load fails at the hub boundary and never reaches Synthesia, so the user still gets loading + a localized error rather than a blank player. Two follow-ups, both out of this change's scope: the wait before the error is >30s because the app sets **no gRPC deadline** (`bearerOptions` builds a bare `CallOptions`, `ChannelOptions` sets no `connectionTimeout`), so failure is bounded only by the OS TCP timeout; and the error surfacing point is the hub, not the player overlay.
