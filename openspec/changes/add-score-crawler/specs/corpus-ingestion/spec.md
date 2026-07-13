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

### Requirement: TUI drives crawl and ingestion

The system SHALL provide a `ratatui` TUI to select sources, show live crawl +
ingestion progress (per-source accepted / rejected / low-confidence counts), and
browse the resulting catalog (title, author, source, licence, confidence). The
TUI SHALL be keyboard-operable and SHALL NOT block on a single unresponsive
source. A headless CLI mode SHALL run the same orchestrator + ingestion without
the UI for scripting/CI.

#### Scenario: Operator drives ingestion from the TUI
- **WHEN** the operator enables sources and starts a run in the TUI
- **THEN** per-source progress updates as items are accepted, rejected, or
  quarantined, and accepted items are ingested into the shared store

#### Scenario: Catalog reviewable in the TUI
- **WHEN** a run has ingested entries
- **THEN** the operator can browse `catalog_scores` entries within the TUI

#### Scenario: Headless run matches TUI ingestion
- **WHEN** the same configuration is run in headless CLI mode
- **THEN** it ingests into the same store/catalog with identical results, without
  rendering a UI
