## MODIFIED Requirements

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
