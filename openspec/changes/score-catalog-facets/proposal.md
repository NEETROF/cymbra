## Why

The Score Hub can filter by title/composer + difficulty, but players choose practice
material by *musical* traits the catalog already *could* expose: the fastest note value
(no sixteenths yet), presence of chords/tuplets/dotted rhythms, hand span (ambitus),
grand-staff vs single-staff, major/minor, length. Today only `is_piano`, `key_fifths`,
`time_sig`, `measure_count` are persisted — and none is surfaced as a filter. The parser
already sees note `type`, `chord`, `dot`, `pitch`, `staff`, `dynamics` per note, so these
facets are derivable **at ingest with no new source data**: the crawler already parses every
score, so it computes and persists the facets directly on the `catalog_scores` row. The
existing corpus is repopulated simply by **re-crawling** (no separate backfill pass).

## What Changes

- Extend the shared `ScoreSummary` (one derivation for app preview / upload / crawler) to
  compute a set of **musical facets** from the already-parsed document: smallest note
  value, has-tuplets / has-dotted / has-chords, pitch ambitus (lowest/highest MIDI),
  staff count, note count, mode (major/minor), tempo (BPM, when marked), and expressivity
  flags (dynamics / ornaments / articulations / pedal). Purely derived — never client-supplied.
- Persist those facets as new **nullable columns** on `music.catalog_scores` (+ btree/partial
  indexes for the ones used as filters), **populated at ingest** — by the crawler for the
  catalog and by the upload path for `music.user_scores` (surfaced via `ScoreRecord`), so
  contributed scores render a faithful cover too.
- Add optional **facet filters** to the backend `SearchCatalog` RPC — all composed
  conjunctively with the existing query/author/level: max-fastest-note-value (e.g. "nothing
  faster than an eighth"), chords/tuplets/dotted (include-only), max ambitus span, staff
  count, mode, a **tempo range** (BPM), plus an explicit **`is_piano` filter parameter** (the
  front forces it `true` for now, since the corpus is piano-only, but the search supports it later).
- Surface the new filters in the **Score Hub** UI behind an "advanced filters" affordance so
  the default view stays simple; the front always sends `is_piano = true`.

## Capabilities

### New Capabilities
- `score-facets`: musical facet metadata (smallest note value, tuplets/dotted/chords,
  ambitus, staff/note counts, mode, tempo, expressivity flags) derived at ingest by the shared
  `ScoreSummary` and persisted on catalog + user-score rows.

### Modified Capabilities
- `catalog-search`: the search operation accepts optional facet filters (rhythmic
  granularity, chords/tuplets/dotted, ambitus span, staff count, mode, tempo range) and an explicit
  `is_piano` filter, all composed conjunctively with the existing text/author/level filters.
- `score-hub`: the hub screen exposes the facet filters (advanced-filters panel) and always
  constrains results to piano scores for now.

## Impact

- **`musicxml_core`**: new fields on `ScoreSummary` + the note-level parsing needed for them
  (note-value min, tuplet/mode/ornament/articulation/pedal flags, pitch→MIDI min/max). Host-testable.
- **Crawler**: carry the new facets through `ScoreMetadata`/`CatalogEntry` into the ingest insert.
- **Migrations**: nullable facet columns + indexes on `catalog_scores` and `user_scores`.
- **Crawler**: derives the facets at ingest (`ScoreFacets::from_document`) and threads them
  through `ScoreMetadata → ManifestEntry → CatalogEntry` into the `catalog_scores` insert.
- **Backend `music`**: extend `CatalogEntry`, `PgCatalogRepo` insert, `UserScore`, the search
  port/params/adapter, `ScoreModule::search_catalog`, and the `score.proto` `SearchCatalogRequest`.
- **App**: `CatalogService.search` + `CatalogSearch` notifier + hub UI gain the facet filters;
  front pins `is_piano = true`.
- Coverage gates (Rust + Flutter ≥ 80%) apply to all new pure logic (facet derivation, filter
  validation/compose, UI state).
