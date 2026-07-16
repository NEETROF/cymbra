# Tasks — score-catalog-facets

> **Delivered scope:** the facets derivable from the existing (app-bridged) parse model —
> smallest note value, chords/tuplets/dotted, ambitus, staff/note counts, tempo, dynamics.
> **Deferred** (need model + app-FFI/bridge changes): `is_minor` (mode) and the
> ornaments/articulations/pedal flags. Existing rows are populated by a **re-read backfill**,
> not by threading facets through the crawler/upload insert paths (also deferred).

## 1. Engine — facet derivation (`musicxml_core`)

- [x] 1.1 (n/a) The needed note signals (`type`/`chord`/`dot`/`tuplet`/`pitch`/`staff`/
  `metronome`/`dynamics`) are ALREADY parsed — no model change. Mode/ornaments/articulations/
  pedal parsing deferred.
- [x] 1.2 New `ScoreFacets` struct (kept separate from the app-bridged `ScoreSummary`):
  `min_note_value`, `has_tuplets`, `has_dotted`, `has_chords`, `lowest_midi`/`highest_midi`,
  `staff_count`, `note_count`, `tempo_bpm`, `has_dynamics`. (`is_minor` deferred.)
- [x] 1.3 Compute them in `ScoreFacets::from_document`: smallest note value from `<type>`
  (name→denominator), falling back to `duration/divisions` when untyped; ignore rests + grace
  notes; pitch→MIDI for ambitus; tempo from the first `<metronome>` per-minute, `None` when
  unmarked (never the playback default).
- [x] 1.4 Unit tests: sixteenth vs eighth min-value, `<type>`-absent duration fallback,
  chords/tuplets/dotted flags, ambitus min/max, tempo + null-when-unmarked, dynamics.

## 2. Database & migrations

- [x] 2.1 Module migration: add the nullable facet columns to `music.catalog_scores`
  (`min_note_value`, `has_tuplets`, `has_dotted`, `has_chords`, `lowest_midi`, `highest_midi`,
  `staff_count`, `note_count`, `is_minor`, `tempo_bpm`, `has_dynamics`, `has_ornaments`,
  `has_articulations`, `has_pedal`), guarded + fully-qualified.
- [x] 2.2 Mirror the same columns on `music.user_scores` (parity for "mes partitions").
- [x] 2.3 Indexes for the filter columns: btree on `min_note_value`, `staff_count`, `is_minor`,
  `tempo_bpm`, `lowest_midi`/`highest_midi`; leave low-cardinality booleans to bitmap/partial indexes.

## 2b. Backfill existing rows (re-read stored objects, no re-crawl)

- [x] 2b.1 Add a `UserLibraryRepo`-style facet-update method (or a dedicated maintenance repo):
  `update_facets(id, facets)` for `catalog_scores` and `user_scores`, plus a `rows_missing_facets()`
  stream of `(id, object_key)`.
- [x] 2b.2 Maintenance command (crawler subcommand or a small `cymbra-music` bin): for each row
  missing facets, read the object via `ObjectStorage`, `decode_canonical` + parse, derive facets
  via `ScoreSummary`, and `UPDATE` the row. Idempotent + resumable; log-and-skip a missing/unreadable
  object without aborting the run.
- [x] 2b.3 Wire it against the dev store (`/tmp/cymbra/scores`) + DB; document the invocation.

## 3. Crawler + upload — carry facets into rows (DEFERRED — the re-read backfill populates rows instead)

- [ ] 3.1 Extend `ScoreMetadata` (crawler `metadata.rs`) + the manifest to carry the new facets
  from `ScoreSummary`.
- [ ] 3.2 Extend `CatalogEntry` (backend `repo.rs`) with the facet fields; set them in the
  crawler's `to_catalog_entry` and bind them in `PgCatalogRepo::insert`.
- [ ] 3.3 Extend `UserScore` + the upload persistence (`module.rs` upload, `pg_user_scores.rs`)
  to store the facets from the re-derived summary.
- [ ] 3.4 Update the fakes/tests that build `CatalogEntry`/`UserScore` for the new fields.

## 4. Backend — search filters

- [x] 4.1 Extend `CatalogSearchParams` + `CatalogHit` (if a facet must show in results) with the
  optional filters: `is_piano`, `max_note_value`, `has_chords`, `has_tuplets`, `has_dotted`,
  `max_ambitus_semitones`, `staff_count`, `is_minor`, `min_bpm`/`max_bpm`.
- [x] 4.2 `ScoreModule::search_catalog`: validate `max_note_value`/`staff_count` against the
  allowed sets, pass the filters through; keep clamping/validation here (host-testable).
- [x] 4.3 `PgCatalogSearchRepo::search`: add `AND (…)` predicates for each supplied filter,
  excluding rows whose facet is null under an active filter; compose with the existing predicates.
- [x] 4.4 `FakeCatalogSearchRepo`: mirror the same filter semantics for unit tests.
- [x] 4.5 Unit-test module + fake: each facet filter narrows correctly, composes conjunctively,
  excludes null facets, and rejects invalid values.

## 5. gRPC surface

- [x] 5.1 Extend `score.proto` `SearchCatalogRequest` with the optional facet fields (and
  `CatalogHit` if a facet is displayed); regenerate Rust + Dart stubs.
- [x] 5.2 Map the new request fields in `grpc.rs` `search_catalog`; gRPC test covering a
  facet-filtered search and an invalid-value rejection.

## 6. App — service, state & advanced-filters UI

- [x] 6.1 `CatalogService.search`: add the facet params; the production `GrpcCatalogService`
  always sends `is_piano: true` (corpus is piano-only) while keeping the param overridable.
- [x] 6.2 `CatalogSearch` notifier + `CatalogSearchState`: hold the advanced-filter values,
  re-query on change (immediate), and apply them client-side in the "mes partitions" source.
- [x] 6.3 Score Hub UI: an expandable "Filtres avancés" panel — rhythmic-granularity chips
  (≤ noire / ≤ croche / ≤ double / tout), toggles (accords / triolets / pointés), ambitus chips
  (≤ 1 / ≤ 2 octaves / tout), mode (majeur / mineur / tout), tempo band (Lent / Modéré / Rapide /
  tout → min/max BPM); active filters visible + clearable.
- [x] 6.4 Localized strings for the advanced-filter labels across en/fr/es/it.
- [x] 6.5 Run `build_runner`; `custom_lint`/`riverpod_lint` clean.

## 7. Tests, coverage & pre-PR

- [x] 7.1 Widget test: advanced-filters panel drives the notifier (granularity/toggles/ambitus/
  mode) with an in-memory `CatalogService`; verify the production call pins `is_piano = true`.
- [x] 7.2 Rust ≥ 80% (`cargo llvm-cov … --fail-under-lines 80`) and Flutter ≥ 80%
  (`flutter test --coverage`); pure logic kept in host-testable seams.
- [x] 7.3 `melos run analyze` + `dart format`; `cargo fmt --all --check` +
  `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 7.4 Run the backfill on the dev corpus (re-read stored objects), smoke-test the filters
  end-to-end; `openspec validate score-catalog-facets --strict` passes.
