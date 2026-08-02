## 1. Database: catalog table

- [x] 1.1 Add `backend/music/migrations/0012_soundfonts.sql`: create `music.soundfonts` (`id` text PK, `label` text, `object_key` text, `tier` text check in `('free','paid')`, `license` text, `attribution` text null, `size_bytes` bigint null, `created_at timestamptz default now()`); idempotent DDL, fully-qualified names (match existing migrations).
- [x] 1.2 Seed the CC0 default row in the same migration: `id='upright-piano-kw'`, `object_key='UprightPianoKW-20220221.sf2'`, `tier='free'`, `license='CC0-1.0'`, `attribution=NULL` (idempotent `INSERT ... ON CONFLICT (id) DO NOTHING`).

## 2. Backend: repo + shared catalog types (crate `cymbra-music`)

- [x] 2.1 Move `FontEntry` and `Tier` from `backend/server/src/soundfont.rs` into `cymbra-music` (e.g. `soundfont::{FontEntry, Tier}`), adding a `label` field; export from `music` lib.
- [x] 2.2 Add a `SoundFontRepo` trait (`list() -> Vec<FontEntry>`, `lookup(id) -> Option<FontEntry>`) + `PgSoundFontRepo` reading `music.soundfonts`; add a fake/in-memory repo for tests.
- [x] 2.3 Unit-test the Pg repo mapping (row → `FontEntry`, tier parse, null attribution) and `lookup` hit/miss (against the migrated schema, matching existing music pg tests).

## 3. Backend: delivery route resolves through the repo

- [x] 3.1 `backend/server/src/soundfont.rs`: replace the static `catalog()`/`lookup()` with a repo lookup — `decide` (or its caller) resolves id → entry via `SoundFontRepo`; keep `Entitlements`/`may_access`/`parse_range`/streaming/`store: None → 503` unchanged; import `FontEntry`/`Tier` from `cymbra-music`.
- [x] 3.2 Wire the repo into `SoundfontState` (and its construction in `backend/server/src/main.rs`); update the route's tests to seed the fake repo instead of the static array (401/404/403/206/200/416 cases preserved).

## 4. Backend: gRPC ListSoundFonts

- [x] 4.1 `backend/music/proto/score.proto`: add `message SoundFont { string id=1; string label=2; string license=3; string attribution=4; string tier=5; }`, `message ListSoundFontsRequest {}`, `message ListSoundFontsResponse { repeated SoundFont soundfonts=1; }`, and `rpc ListSoundFonts(ListSoundFontsRequest) returns (ListSoundFontsResponse);` on `ScoreService` (tonic regen is automatic via `build.rs`).
- [x] 4.2 Implement `list_soundfonts` in `backend/music/src/grpc.rs`: require identity (`identity(&req)?`), read the repo, map rows → proto `SoundFont`; wire the repo into `ScoreGrpc`.
- [x] 4.3 Unit-test `list_soundfonts`: authenticated returns the seeded rows; the handler maps fields (incl. empty attribution) correctly.

## 5. Flutter: server-driven download catalog (`apps/music`)

- [x] 5.1 Run `melos run gen-grpc` so `score.pbgrpc.dart` includes `ListSoundFonts`.
- [x] 5.2 Add an injectable `SoundFontCatalogService` seam (gRPC over `ScoreService.ListSoundFonts`) returning `download`-kind `PianoEntry`s, and a provider exposing them (async, degradable to empty on error); a fake for tests.
- [x] 5.3 `state/piano_catalog.dart`: reduce `builtInPianos` to **only** the bundled CC0 default (remove YDP/Salamander); make `pianoCatalog` union bundled default + server download list (excluding `defaultPianoId`) + user imports.
- [x] 5.4 Ensure graceful degradation: a failed/empty listing yields no download entries (picker shows bundled default + imports); selection/persistence unchanged.

## 6. Tests

- [x] 6.1 Flutter: the download list comes from the fake `SoundFontCatalogService` (server-listed fonts appear, the bundled default id is filtered out, an empty/error listing yields none).
- [x] 6.2 Flutter: update `selected_piano_test` / `piano_picker_test` to drive downloadable fonts from the fake catalog seam instead of the removed hardcoded grands.
- [x] 6.3 Backend: repo + `list_soundfonts` + delivery-route (repo-backed) tests green.

## 7. Verify & gate

- [x] 7.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov --workspace --fail-under-lines 80` pass (new `soundfont.rs`/glue kept in the coverage ignore regex as applicable; pure repo/mapping logic tested).
- [x] 7.2 `apps/music`: `dart run build_runner build`, `melos run analyze`, `dart format`, `dart run custom_lint` clean; `flutter test --coverage --exclude-tags golden` green and ≥ 80%.
- [x] 7.3 `apps/back-office`: `yarn gen` regenerates stubs; `yarn typecheck`/`lint` clean (no behavior change).
- [ ] 7.4 Manually confirm: with only the CC0 seed, the app picker shows just the bundled default + import (no fictional grands); inserting a font row + uploading its object makes it appear and download; a listing failure degrades to bundled + imports.
- [x] 7.5 `openspec validate add-soundfont-catalog-db --strict` passes.
