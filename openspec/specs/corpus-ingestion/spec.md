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

Object-level idempotence SHALL hold **at write time and without consulting the catalog**,
because the crawler writes its objects before — and independently of — any catalog
connection. The object key of a retained score SHALL therefore be derived from its content
hash, so that re-crawling unchanged content resolves to the same key and rewrites the same
object instead of creating a second one. A crawl that retains only already-known content
SHALL leave the number of corpus objects unchanged.

Deriving the object key from content SHALL NOT change how a catalog row is identified: the
row's own identifier stays independent of the key, and readers SHALL resolve an object only
through the `object_key` recorded on the row, never by rebuilding it from the identifier.

#### Scenario: Re-ingesting existing content is a no-op
- **WHEN** the crawler encounters content whose SHA-256 already exists in
  `catalog_scores`
- **THEN** no new object is written and no new row is inserted for that content

#### Scenario: Re-crawling unchanged content does not grow the corpus
- **WHEN** a crawl retains only content that was already ingested by an earlier run
- **THEN** the corpus contains no object it did not contain before the run, the earlier
  objects remaining referenced by their existing rows

#### Scenario: Identical content resolves to one row and one object
- **WHEN** the same content is retained from two different sources, or by two successive runs
- **THEN** a single `catalog_scores` row and a single corpus object exist for it, dedup having
  collapsed the duplicate before a second row or object could be created

#### Scenario: Row identity is unaffected by the content-derived key
- **WHEN** a score is ingested with a content-derived `object_key`
- **THEN** its catalog row keeps its own identifier, and the object is resolved from the
  stored `object_key` rather than from that identifier

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

### Requirement: Crawler working files live outside the served corpus

The crawler SHALL write its working files — source checkouts and any other material kept for
its own operation rather than for serving — to a work location configured independently of
the corpus root, never derived from it. The corpus root SHALL contain only servable objects
under the corpus prefixes.

The crawler SHALL refuse to run when the resolved work location is inside the corpus root,
failing at startup rather than writing there, so a misconfigured deployment is reported
instead of silently polluting the served corpus.

#### Scenario: Source checkouts stay out of the corpus

- **WHEN** the crawler prepares its sources during a run
- **THEN** the checkouts are created under the configured work location, and the corpus root
  gains no entry other than servable objects under the corpus prefixes

#### Scenario: Work location inside the corpus root is refused

- **WHEN** the crawler is configured with a work location that resolves inside the corpus root
- **THEN** it exits with an error before crawling, and writes nothing to the corpus root

#### Scenario: Work location is configurable independently

- **WHEN** an operator points the corpus root and the work location at two different
  directories
- **THEN** servable objects are written to the former and working files to the latter, with no
  path of one nested in the other

### Requirement: The off-box mirror carries only servable objects

The off-box mirror of the corpus SHALL transfer only objects under the servable corpus
prefixes, selected by an allow-list rather than by excluding known-unwanted paths, so that any
non-servable content present at the corpus root is not mirrored. The mirrored key SHALL remain
equal to the catalog `object_key`, so the mirror stays readable as the storage origin.

#### Scenario: Non-servable content at the corpus root is not mirrored

- **WHEN** the corpus root contains an entry that is not under a servable corpus prefix and the
  mirror runs
- **THEN** that entry is not transferred, while the servable objects are

#### Scenario: Mirrored key equals the object key

- **WHEN** a servable object is mirrored off-box
- **THEN** its key in the mirror is exactly the `object_key` recorded on its catalog row

### Requirement: Unreferenced corpus objects are reconcilable

The system SHALL provide a maintenance operation that reconciles the corpus against the
catalog: it reports every corpus object, local and off-box, that no `catalog_scores` row
references. The operation SHALL report without writing by default, and remove the unreferenced
objects only when explicitly asked to apply.

The operation SHALL reason over the *set* of referenced `object_key` values, never per row, so
an object referenced by any remaining row is never removed. That set SHALL cover **every**
store of references to corpus objects, not only the catalog: an object whose only reference
lives in another table is a referenced object, and treating it otherwise destroys live data.

It SHALL abort without writing when the referenced set is empty, when the proportion of
objects it would remove exceeds a configured safety threshold, or when **any single corpus
prefix holds objects of which none at all are referenced** — a wholly unreferenced prefix
indicates a reference source missing from the query far more often than it indicates a prefix
of pure garbage. Removal SHALL be reversible until an explicit, separate purge step.

#### Scenario: Dry run reports without writing

- **WHEN** the reconciliation runs without being asked to apply
- **THEN** it reports the unreferenced objects it found and neither removes nor moves anything

#### Scenario: Referenced objects are never removed

- **WHEN** the reconciliation applies its removals and an object is referenced by at least one
  catalog row
- **THEN** that object remains in place and servable, including when another row that also
  referenced it has been deleted

#### Scenario: Implausible reference set aborts the run

- **WHEN** the referenced-key set is empty, or the share of objects to remove exceeds the
  safety threshold
- **THEN** the operation aborts without removing anything and reports why

#### Scenario: A wholly unreferenced prefix aborts the run

- **WHEN** one corpus prefix holds objects and not one of them is referenced
- **THEN** the operation aborts without removing anything and reports that a reference source
  is likely missing, rather than treating the entire prefix as garbage

#### Scenario: Objects referenced outside the catalog are kept

- **WHEN** an object's only reference is held by a store other than the score catalog, such as
  a user's own uploaded scores
- **THEN** it counts as referenced and is never removed

#### Scenario: Removal is reversible until purged

- **WHEN** the reconciliation has applied its removals and no purge has been run
- **THEN** the removed objects can still be restored

