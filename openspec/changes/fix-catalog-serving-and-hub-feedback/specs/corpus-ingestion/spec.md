## MODIFIED Requirements

### Requirement: Ingest into the shared score store

The system SHALL ingest each retained score into the same store the app reads:
its bytes into an object store abstracted by `object_store` (a local filesystem
folder in dev, S3/MinIO in prod, selected by config) and a provenance row into
the `catalog_scores` table in the backend `score` module's Postgres schema.
Ingestion SHALL write **directly** through the `score` module's repo/storage
with an admin/ingestion role (batch), NOT via per-file gRPC. The DB row SHALL be
the source of truth for a corpus entry.

A `catalog_scores` row SHALL become discoverable (returned by search / servable
by `get_catalog_bytes`) ONLY once its bytes are resolvable by the app's read
path — that is, present at the served local root (`SCORES_DIR` /
`CYMBRA_SCORE_LOCAL_ROOT`) OR in the S3 origin used as the read fallback.
Ingestion SHALL write the object before inserting the row, and SHALL place the
object where the serving read path resolves it, so that a row visible to the app
NEVER points at bytes the app cannot fetch. A deferred, out-of-band step (e.g. a
nightly mirror) SHALL NOT be required for a freshly ingested row to be servable.

#### Scenario: Retained score persisted to store and catalog
- **WHEN** a score passes the licence gate and conversion
- **THEN** its `.mxl` bytes are written to the object store under the public-corpus
  prefix and a `catalog_scores` row is inserted with its provenance fields

#### Scenario: Object store backend swapped by config
- **WHEN** the tool runs in dev with the local-filesystem backend and later in
  prod with the S3/MinIO backend
- **THEN** the same ingestion code writes to a local folder in dev and to the
  bucket in prod, with no code change

#### Scenario: A visible catalog row is immediately servable
- **WHEN** a crawl finishes and a `catalog_scores` row is visible in the hub,
  before any deferred mirror/sync step has run
- **THEN** opening that score returns its bytes from the app read path (served
  local root or S3 fallback), rather than a not-found error

### Requirement: CLI and Docker fan-out drive crawl and ingestion

The system SHALL be operated as a headless CLI (`score-crawler --sources … [--all]
[--limit N]`) and as a Docker Compose fan-out (one one-shot container per source).
A single unresponsive or failing source SHALL NOT block the others: per-source
prepare/discover failures are logged and skipped. Both entry points run the same
license-first orchestrator + ingestion path, so results are identical.

The Docker fan-out deployment SHALL write each source's corpus output directly
into the directory the server serves from (`SCORES_DIR`, mounted at the server's
`CYMBRA_SCORE_LOCAL_ROOT`), using the same `<prefix>/<shard>/<uuid>.mxl` object
keys, so that ingested bytes are on the server's read path without a separate
merge step. The off-box S3 mirror (`sync-scores.sh`) SHALL be scoped to
durability and the S3 read-fallback, NOT to making freshly crawled bytes
servable.

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

#### Scenario: Fan-out output lands on the server read path
- **WHEN** the Docker fan-out writes a source's corpus output
- **THEN** the `.mxl` bytes appear under the served root
  (`SCORES_DIR/<prefix>/<shard>/<uuid>.mxl`) resolvable by the server without a
  deferred merge, and the S3 mirror step only replicates them off-box

#### Scenario: One failing source does not stop the run
- **WHEN** a source fails to prepare (e.g. a network/clone error)
- **THEN** it is logged and skipped and the remaining sources still run
