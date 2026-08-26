# corpus-manifest Specification

## Purpose
TBD - created by archiving change add-score-crawler. Update Purpose after archive.
## Requirements
### Requirement: Provenance is stored in the catalog

The system SHALL record every retained file's provenance as a row in the
`catalog_scores` table (the source of truth), with at least: `id`, `title`,
`composer` (author), `arranger` (when present), `source` name, `source_url`,
normalised `license` code (e.g. `CC-BY-4.0`), `license_url`, `confidence`
(`verified` | `unverified`), `sha256`, `origin_format`, `conversion_status`, and
`object_key` (the object-store key of the `.mxl`). These fields are the
attribution shown on re-distribution and SHALL be complete and accurate for
accepted items. `catalog_scores` lives in the backend `score` module's schema,
alongside `user_scores`, and carries no owner (public corpus).

#### Scenario: Row carries full attribution
- **WHEN** a file is retained and ingested
- **THEN** its `catalog_scores` row includes source, source URL, normalised
  licence code, licence URL, and author so the file can be lawfully re-published
  from the catalog alone

#### Scenario: Confidence recorded per row
- **WHEN** an `unverified` item is retained
- **THEN** its `catalog_scores` row's `confidence` field is `unverified`

### Requirement: Search-ready metadata captured at ingest

The system SHALL, at ingest time, populate the search/facet and musical-metadata
columns from the already-parsed score so that a later search API needs no
re-parse backfill: at least `work_key` (normalised composer+title for
dedup/grouping), `title_norm` (normalised for text search), the derived
**`instrument`** classification (`keyboard` | `percussion` | `unknown` — replacing
the former `is_piano` staff-count flag, which the crawler stops deriving and the
manifest stops carrying), `key_fifths`, `time_sig`, `measure_count`, and, when
available, `language` and `voicing`. Source-specific extras SHALL be stored in a
`metadata` JSON column rather than requiring a per-source migration.

#### Scenario: Musical metadata persisted without re-parse
- **WHEN** a score is ingested
- **THEN** its `key_fifths`, `time_sig`, and `measure_count` are written from the
  parsed `ScoreDocument`, not left for a later re-parse

#### Scenario: Facet columns present for filtering
- **WHEN** a `catalog_scores` row is written
- **THEN** it carries `source`, `license`, `confidence`, and `work_key` values
  suitable for equality-filtered faceting

#### Scenario: The manifest carries the instrument, not the retired flag
- **WHEN** the crawler writes a catalog entry or exports the manifest
- **THEN** the entry records the derived `instrument` and no `is_piano` field

### Requirement: Difficulty is provenance-tracked

The system SHALL record difficulty as a nullable `level` (Beginner /
Intermediate / Advanced) together with a `level_source` (`source` | `heuristic` |
`manual` | null). A source-declared grade SHALL set `level_source = source`;
otherwise a heuristic estimate computed from the parsed score MAY set
`level_source = heuristic`; `level` SHALL be null when neither applies. A
heuristic estimate SHALL NEVER be recorded as a source grade.

#### Scenario: Source grade recorded as authoritative
- **WHEN** the source provides an explicit difficulty grade
- **THEN** `level` is set from it and `level_source = source`

#### Scenario: Heuristic estimate flagged as such
- **WHEN** no source grade exists and the heuristic runs
- **THEN** `level` holds the estimate and `level_source = heuristic`, so a later
  curation pass can distinguish and overwrite it without clobbering real grades

### Requirement: Derived manifest export

The system SHALL be able to export the catalog to `manifest.csv` and
`manifest.json` (same content, one entry per retained file, consistent between
the two), derived from `catalog_scores`. The export is a convenience/audit
artefact, not the source of truth.

#### Scenario: CSV and JSON agree
- **WHEN** the manifest is exported in both formats
- **THEN** they contain the same entries with matching field values for each file

#### Scenario: Export reflects the catalog
- **WHEN** the manifest is exported
- **THEN** each entry corresponds to a `catalog_scores` row with the same field
  values

### Requirement: Rejection log

The system SHALL record every excluded item in a `rejected.log` with its source,
identifying URL, the raw licence signal observed, and the normalised reason, so
exclusions are auditable independently of the catalog.

#### Scenario: Rejected item is auditable
- **WHEN** an item is rejected (licence or conversion)
- **THEN** a line is appended to `rejected.log` with the source, item URL, raw
  licence signal, and reason

### Requirement: Run artefacts are written to the crawler work location

The derived manifest export and the rejection log SHALL be written to the crawler's work
location, never to the served corpus root: they are audit artefacts of a crawl run, not
servable corpus objects, so the corpus root holds only objects the application serves and the
off-box mirror never carries them.

#### Scenario: Manifest export lands outside the corpus

- **WHEN** a crawl run exports the manifest
- **THEN** `manifest.csv` and `manifest.json` are written under the crawler work location and
  the corpus root gains neither

#### Scenario: Rejection log lands outside the corpus

- **WHEN** a crawl run records excluded items
- **THEN** `rejected.log` is written under the crawler work location and the corpus root gains
  no such file

#### Scenario: Artefacts stay auditable after relocation

- **WHEN** an operator inspects a completed run's exclusions and exported manifest
- **THEN** both are found at the work location, with the content the existing manifest and
  rejection-log requirements define

