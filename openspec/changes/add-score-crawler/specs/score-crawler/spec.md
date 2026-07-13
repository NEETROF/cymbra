## ADDED Requirements

### Requirement: Source adapter trait

The system SHALL define a `SourceAdapter` trait (async) that every source
implements with, at minimum: `discover()` to enumerate candidate items,
`extract_license(item)` to obtain the raw licence signal without heavy download,
`fetch(item)` to retrieve the raw score payload, and `to_musicxml(raw)` to yield
MusicXML (or delegate to the conversion pipeline). A central orchestrator SHALL
hold a registry of adapters and iterate only over the enabled ones.

#### Scenario: Enabled adapters are orchestrated
- **WHEN** the crawler runs with a set of enabled sources
- **THEN** the orchestrator invokes each enabled adapter's `discover`, evaluates
  licence via `extract_license` before `fetch`, and routes payloads through
  conversion, while disabled adapters are never invoked

#### Scenario: One adapter per implemented source
- **WHEN** the crawler is built
- **THEN** an adapter implementation exists for each in-scope source (OpenScore
  Lieder, Mutopia, CPDL, IMSLP, PDMX, musetrainer/library,
  eduardomourar/music-scores-musicxml, Project Gutenberg sheet music, Hymnary)
  and is registered in the orchestrator. NEUMA and the Josquin Research Project
  are out of scope — the NEUMA site no longer exists and the Josquin site could
  not be located, so their licences cannot be verified.

### Requirement: Politeness and legality

The system SHALL respect each host's `robots.txt`, send a descriptive
User-Agent that includes a contact, wait a configurable delay between requests
to the same host (default 2 s), apply exponential back-off on transport errors
and HTTP 429, and cap concurrency at a low configurable bound (default 2).

#### Scenario: Disallowed path is not fetched
- **WHEN** a target URL is disallowed by the host's `robots.txt`
- **THEN** the crawler does not request it and records it as skipped

#### Scenario: Rate limit triggers back-off
- **WHEN** a host responds with HTTP 429 or a transport error
- **THEN** the crawler retries with exponentially increasing delay up to a bound,
  rather than hammering the host

#### Scenario: Concurrency stays within the cap
- **WHEN** many items are queued
- **THEN** no more than the configured number of concurrent requests are in
  flight at once

### Requirement: Resumable state

The system SHALL persist crawl progress to an on-disk state cache and, when run
with resume, SHALL skip items already completed so previously downloaded or
converted material is not re-fetched.

#### Scenario: Resume skips completed work
- **WHEN** a crawl is interrupted and restarted in resume mode
- **THEN** items recorded as completed in the state cache are not downloaded or
  converted again, and the crawl continues from where it stopped

### Requirement: Content dedup

The system SHALL compute a SHA-256 of each retained score's content and SHALL
treat two items with identical hashes as duplicates, keeping a single copy even
when they originate from different sources, and skipping content whose hash
already exists in `catalog_scores`.

#### Scenario: Duplicate across sources kept once
- **WHEN** the same score content is discovered from two different sources
- **THEN** only one copy is ingested and the duplicate is recorded as
  deduplicated

#### Scenario: Content already in the catalog is skipped
- **WHEN** discovered content's SHA-256 already exists as a `catalog_scores` row
- **THEN** it is not re-ingested

### Requirement: Configuration

The system SHALL read a `config.yaml` deserialised into typed structures,
specifying at least: enabled sources, per-host delay, concurrency, per-source
item quotas for test runs, the object-store backend + prefixes, and the
Postgres/`score`-catalog connection for ingestion. Environment variables
(including the shared `CYMBRA_SCORE_S3_*` keys) MAY override config values.

#### Scenario: Quota bounds a test run
- **WHEN** a per-source quota (or `--limit`) is set
- **THEN** the crawler stops discovering/fetching that source after the quota is
  reached

### Requirement: Command-line interface

The system SHALL expose a `clap`-based CLI supporting at least: selecting
sources (`--sources openscore,mutopia`), a test limit (`--limit N`), crawling
everything (`--all`), resuming (`--resume`), and verbose logging (`--verbose`).
Logging SHALL be structured via `tracing` at INFO by default and DEBUG under
`--verbose`.

#### Scenario: Selective source run
- **WHEN** the operator runs with `--sources openscore,mutopia --limit 50`
- **THEN** only those two adapters run, each bounded to 50 items

### Requirement: Robust error handling

The system SHALL handle errors with `anyhow`/`thiserror` and SHALL contain no
`unwrap()`/`expect()` on fallible operations in production code paths; a failure
on one item SHALL NOT abort the whole crawl.

#### Scenario: Single-item failure is isolated
- **WHEN** fetching or converting one item fails
- **THEN** the error is logged, the item is recorded as failed, and the crawler
  proceeds to the next item without panicking
