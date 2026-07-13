## ADDED Requirements

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
