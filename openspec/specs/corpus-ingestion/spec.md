# corpus-ingestion Specification

## Purpose
TBD - created by archiving change add-score-crawler. Update Purpose after archive.
## Requirements
### Requirement: Ingest into the shared score store

The system SHALL ingest each retained score into the same store the app reads:
its bytes into an object store abstracted by `object_store` (a local filesystem
folder in dev, S3/MinIO in prod, selected by config) and a provenance row into
the `catalog_scores` table in the backend `score` module's Postgres schema.
Ingestion SHALL write **directly** through the `score` module's repo/storage
with an admin/ingestion role (batch), NOT via per-file gRPC. The DB row SHALL be
the source of truth for a corpus entry.

#### Scenario: Retained score persisted to store and catalog
- **WHEN** a score passes the licence gate and conversion
- **THEN** its `.mxl` bytes are written to the object store under the public-corpus
  prefix and a `catalog_scores` row is inserted with its provenance fields

#### Scenario: Object store backend swapped by config
- **WHEN** the tool runs in dev with the local-filesystem backend and later in
  prod with the S3/MinIO backend
- **THEN** the same ingestion code writes to a local folder in dev and to the
  bucket in prod, with no code change

### Requirement: Title derived from the score, filename only as fallback

The system SHALL derive each `catalog_scores` row's display `title` from the
score's own embedded metadata (the parsed `<work-title>`), and SHALL fall back to
the source adapter's title ONLY when the score carries no embedded title. A source
adapter's title — for git corpora the file stem, which for OpenScore Lieder is an
opaque id such as `lc28971056` — SHALL NEVER shadow an embedded title.

The persisted search key SHALL stay consistent with the display title: `title`,
the normalised `title_norm`, and the `work_key` SHALL all be derived from the same
parsed title (via the shared `ScoreSummary` derivation), so a score is findable by
its real title through the catalog search (which matches `title_norm`, normalised
the same way as the query).

NOTE: rows ingested before this requirement — which stored the filename id as the
title — are reconciled by a one-off maintenance backfill (`backfill-titles`) that
re-reads each stored `.mxl`, re-derives the title, and rewrites
`title`/`title_norm`/`work_key` together. Re-running the crawler cannot repair
them: SHA-256 dedup skips already-ingested content.

#### Scenario: Embedded title preferred over the adapter title
- **WHEN** a retained score embeds a `<work-title>` and its source adapter also
  supplies a title (e.g. an opaque filename id)
- **THEN** the `catalog_scores` row's `title` is the embedded work-title, and
  `title_norm`/`work_key` are derived from that same title

#### Scenario: Adapter title used only when the score has none
- **WHEN** a retained score carries no embedded title
- **THEN** the row's `title` falls back to the adapter-supplied title

#### Scenario: Title stays searchable
- **WHEN** a caller searches for a term contained in a score's embedded title
- **THEN** the score matches, because its `title_norm` was derived from that same
  title with the normalisation the query also uses

### Requirement: Confidence separation in the store

The system SHALL keep `verified` (high-confidence) and `unverified`
(low-confidence) material separate in both the object store (distinct prefixes)
and the catalog (`confidence` column), and SHALL NEVER place an `unverified`
entry in the high-confidence corpus.

#### Scenario: Low-confidence routed to its own prefix
- **WHEN** an `unverified` item is ingested
- **THEN** its bytes land under the low-confidence prefix and its `catalog_scores`
  row has `confidence = 'unverified'`, never the high-confidence prefix

### Requirement: Idempotent ingestion

The system SHALL make ingestion idempotent: re-running the crawler SHALL NOT
create duplicate object-store objects or duplicate `catalog_scores` rows for the
same content. Dedup SHALL use the SHA-256 content hash, checked against both the
in-run set and existing `catalog_scores` rows.

#### Scenario: Re-ingesting existing content is a no-op
- **WHEN** the crawler encounters content whose SHA-256 already exists in
  `catalog_scores`
- **THEN** no new object is written and no new row is inserted for that content

### Requirement: CLI and Docker fan-out drive crawl and ingestion

The system SHALL be operated as a headless CLI (`score-crawler --sources … [--all]
[--limit N]`) and as a Docker Compose fan-out (one one-shot container per source).
A single unresponsive or failing source SHALL NOT block the others: per-source
prepare/discover failures are logged and skipped. Both entry points run the same
license-first orchestrator + ingestion path, so results are identical.

NOTE: an interactive `ratatui` TUI was originally proposed but was **dropped** —
the tool is operated headless (locally via the CLI, in production via the Docker
fan-out), where a TUI serves no purpose.

#### Scenario: Operator drives ingestion from the CLI
- **WHEN** the operator selects sources and starts a run (`--sources …` / `--all`)
- **THEN** each accepted item is converted, deduplicated, and ingested into the
  shared store, and a per-source summary (accepted / rejected / low-confidence)
  is printed

#### Scenario: Docker fan-out ingests per source
- **WHEN** the operator runs the Docker Compose fan-out
- **THEN** one container per source crawls and ingests into the same store/catalog
  (via `CYMBRA_SCORE_DATABASE_URL`), each exiting when its crawl finishes

#### Scenario: One failing source does not stop the run
- **WHEN** a source fails to prepare (e.g. a network/clone error)
- **THEN** it is logged and skipped and the remaining sources still run

### Requirement: Ingested scores are unvalidated by default

Ingestion SHALL persist every newly ingested `catalog_scores` row with moderation
status `pending` (unvalidated). The crawler MUST NOT auto-validate any score, and the
licensing `confidence` value (`verified` / `unverified`) MUST NOT be used to grant
validation — a high-confidence licence still yields a `pending` score. Consequently a
freshly crawled score SHALL NOT appear in the public hub until a reviewer validates
it. This is independent of and additional to the existing confidence separation.

#### Scenario: Newly ingested score is pending

- **WHEN** the crawler ingests a score that passes the licence gate and conversion
- **THEN** its `catalog_scores` row has moderation status `pending` and it is not
  publicly visible

#### Scenario: High-confidence licence does not auto-validate

- **WHEN** an ingested score has `confidence = 'verified'`
- **THEN** its moderation status is still `pending`, not `accepted`

#### Scenario: Freshly crawled score is absent from the hub

- **WHEN** a normal caller browses the hub right after a crawl run
- **THEN** the newly ingested scores do not appear, because they are `pending`

