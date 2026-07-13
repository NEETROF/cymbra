## Scope revision (post-bring-up)

After implementing and test-running every proposed source, four web-crawl
sources were **removed** because none serves free MusicXML that can be harvested
lawfully and automatically:

- **cpdl** (ChoralWiki) — automated access returns HTTP 403; scores are mostly
  PDF/MuseScore, MusicXML is rare.
- **imslp** — paywalled/wait-gated PDF scans; almost no MusicXML.
- **gutenberg** — the "sheet music" holdings are text ebooks (GUTINDEX); no
  MusicXML is served.
- **hymnary** — scores are gated, paid "FlexScores"; no free bulk MusicXML and
  no work index to crawl.

Because these were the **only** consumers of the generic web-crawl/HTTP layer,
that layer was removed with them: `sources::web`, `sources::web_index`, `http`
(`HttpFetcher`/`Fetcher`), `robots` (`texting_robots`), and `politeness`
(back-off + concurrency), plus the `scraper`/`texting_robots` deps and the
per-host `delay`/`concurrency`/`contact` config knobs. The robots.txt /
User-Agent / rate-limit discussion below therefore describes the *original*
design; it no longer applies now that the surviving sources are all git clones
or the PDMX bulk dataset (no polite HTTP crawling). The exclusion list and
reasons live in code as `sources::EXCLUDED_SOURCES`; restore from git history if
a source becomes viable. **Delivered sources:** openscore, mutopia, musetrainer,
eduardomourar, pdmx.

The proposed **`ratatui` TUI was also dropped.** The crawler is operated
headless — locally via the CLI, in production via the Docker Compose fan-out —
where an interactive terminal UI serves no purpose. `tui.rs`, the
`ratatui`/`crossterm` deps, and the `run::ProgressEvent`/mpsc progress channel
were removed; the "TUI" sections below describe the original design only. The
operator interface is now the CLI + `docker-compose.yml`.

## Context

Cymbra is a Cargo + Melos monorepo. The Rust workspace (`Cargo.toml`) already
reserves a `crates/*` member glob (commented out, pending the "F1" engine
extraction). The only MusicXML parser today lives inside the Flutter FFI crate:
`apps/music/rust/src/api/musicxml_core.rs` (pure, streaming `quick-xml`
state machine, ~1.3k LOC) plus `musicxml.rs` (the `#[frb]`-decorated data model
+ FFI wrappers, excluded from the coverage gate). The core module imports its
data-model types *from* the FFI file and carries `flutter_rust_bridge::frb`
attributes, so it is not reusable as-is outside the app.

We want an operator/dev tool — a crawler + TUI — that harvests scores from many
online libraries, keeps only provably redistributable material, converts to
compact `.mxl`, and **ingests the result into the same score store the app
serves from**. The sibling change `add-user-score-upload` (manual per-user
upload) has already decided that store: a backend `score` module owning a
`user_scores` table in its own Postgres schema plus an object store
(S3-compatible; MinIO/local in dev), with the app reading scores through that
module. It also lifts the MusicXML parser into `crates/musicxml-core`. This
crawler is the bulk counterpart and MUST reuse both, not build a parallel S3
dump. The project's real risk is legal (licence correctness), not technical.

Constraints: Rust 2024 edition workspace, rustls TLS stack (no OpenSSL/native-tls
— matches sqlx/reqwest elsewhere), CLAUDE.md conventions (no `unwrap()` in
production paths, clippy `-D warnings`, `cargo fmt`), and the shared parser must
not regress the app.

## Goals / Non-Goals

**Goals:**
- Reuse the shared `musicxml-core` crate (parser/validator, zero FFI coupling)
  — the SAME extraction `add-user-score-upload` relies on, delivered once.
- A polite, resumable, deduplicated crawler with a per-source `SourceAdapter`
  trait and a license-first pipeline that rejects anything off the whitelist
  before heavy download.
- Convert to validated, spec-compliant `.mxl` via native compression or external
  binaries (MuseScore CLI / Verovio / python-ly), degrading gracefully.
- Ingest the vetted corpus **directly into the shared score store**: bytes to an
  `object_store`-abstracted object store (local folder in dev ⇄ S3/MinIO in
  prod), provenance to a `catalog_scores` table in the `score` module schema.
- A `ratatui` TUI that drives crawling + ingestion and lets the operator review
  accepted / rejected / quarantined items and the catalog.

**Non-Goals:**
- No changes to the shipping Flutter app UI. Surfacing the public corpus in the
  app library is a follow-up, not this change.
- No MIDI-as-score-source support (lossy).
- No attempt to *judge* borderline licences: ambiguity is rejected, not resolved.
- Not a long-running service; it is a CLI/TUI operator ingestion tool run on
  demand (writing directly to the store, not over per-file gRPC).

## Decisions

### One new crate; reuse the shared parser crate
This change adds `crates/score-crawler` (library + binary + TUI) under the
`crates/*` glob. The parser lives in `crates/musicxml-core`, **shared with
`add-user-score-upload`**: whichever change lands first performs the extraction
(enabling the `crates/*` glob), the other depends on the existing crate. The
crawler consumes `musicxml-core` for validation; it does **not** depend on
`rust_lib_music` (that crate is `cdylib/staticlib` with `flutter_rust_bridge`,
midir, cpal, rustysynth — wrong shape and heavy for a CLI).

### Parser extraction mechanics (coordinated prerequisite)
Move the pure data-model structs/enums (`ScoreDocument`, `ScoreMeta`,
`Attributes`, `Clef`, `NotationMeasure`, `NoteEvent`, `Pitch`, …) and the
streaming parser + geometry into `musicxml-core`, stripped of
`flutter_rust_bridge::frb`. The app's `musicxml.rs` keeps the `#[frb]` surface as
thin newtype/wrapper or `pub use` re-exports that delegate to the crate; the
`#[frb]`-facing types remain in the app so the generated Dart API is byte-for-byte
unchanged. *Decision:* keep the crate's public types plain Rust; if the frb
generator needs the structs annotated, wrap rather than annotate the shared crate
(the shared crate must not depend on frb). Validate the refactor by running the
existing `score-notation` tests + `flutter_rust_bridge_codegen generate` and
diffing the generated Dart. Because `add-user-score-upload` needs the same crate
(and adds `.mxl` decode to it), the two changes must not both extract — this
change treats the crate as a prerequisite that may already exist.

### `SourceAdapter` trait + orchestrator
`#[async_trait] trait SourceAdapter { async fn discover(&self) -> Result<Vec<Item>>; async fn extract_license(&self, item: &Item) -> Result<RawLicense>; async fn fetch(&self, item: &Item) -> Result<RawScore>; async fn to_musicxml(&self, raw: RawScore) -> Result<Converted>; }`.
The orchestrator owns the enabled-adapter registry and the fixed pipeline order:
`discover → extract_license → [gate] → fetch → convert → validate → dedup →
ingest (object store + catalog_scores)`; the manifest is a derived export. The
gate sits between `extract_license` and `fetch` so heavy content is never
downloaded for rejected licences. Adapters fall into two
families reusing shared helpers: **git-clone sources** (OpenScore, Mutopia,
musetrainer, eduardomourar) via `git2`/`git` subprocess + on-disk
walk; **web-crawl sources** (CPDL MediaWiki API, IMSLP, Gutenberg, Hymnary,
PDMX/Zenodo) via `reqwest` + `scraper`/`quick-xml`.

### License engine is pure and fixture-tested
`license.rs` exposes `normalize(raw: &RawLicense) -> LicenseOutcome` as a pure
function (no I/O) returning a canonical code + URL + confidence, and
`is_redistributable(outcome) -> Decision`. This is the most-tested unit: table
tests over CC URLs, SPDX ids, version-less labels, NC/ND clauses, ambiguous /
empty inputs, and per-source status strings. Whitelist = { CC0, PublicDomain,
CC-BY-*, CC-BY-SA-* }; everything else → reject. Self-declared PD → `unverified`.

### Politeness stack
`texting_robots` for robots.txt (fetched + cached per host), a descriptive
`User-Agent` with a contact string from config, a per-host delay (default 2 s), a
Tokio `Semaphore` for the concurrency cap (default 2), and `backoff` for
exponential retry on 429/transport errors. Resume state = a JSON/sled on-disk
cache keyed by a stable item id; completed ids are skipped. Dedup = a set of
seen SHA-256 (`sha2`) content hashes.

### Conversion pipeline
`convert.rs` dispatches on origin format. Native `.musicxml` → validate via
`musicxml-core` → build `.mxl` with the `zip` crate writing a proper
`META-INF/container.xml`. `.mscx/.mscz` → MuseScore CLI (`QT_QPA_PLATFORM=offscreen`)
emitting `.mxl` directly. MEI → Verovio `-t musicxml`. `.ly` → `python-ly`, and
on failure keep source + mark `failed_kept_source`. All externals via
`std::process::Command` with exit-code checks and timeouts. Every produced `.mxl`
is re-opened and re-parsed before being counted as converted.

### Storage & ingestion — the shared `score` store, not a standalone S3 dump
The crux of the realignment. Retained bytes go to an object store abstracted by
the **`object_store`** crate: `LocalFileSystem` (a folder) in dev, `AmazonS3`
(S3/MinIO) in prod, selected by config — this subsumes "local files in a folder"
*and* S3 behind one interface, and matches the rustls/no-OpenSSL stack.
Provenance goes to a **`catalog_scores`** table in the backend `score` module's
Postgres schema (public corpus: licence-bearing, no owner), **alongside**
`user_scores` from `add-user-score-upload`. The catalog row is the source of
truth; the manifest (CSV/JSON) is a derived export.

Ingestion is **direct, in batch**, through the `score` module's repo/storage
under an admin/ingestion DB role — the same "write directly, don't go over gRPC"
posture the `worker` uses — because per-file gRPC upload (the user-upload
transport) is wrong for a bulk import. *Alternatives rejected:* (a) a private S3
bucket + CSV manifest the app can't query — bytes without a catalog isn't a
library; (b) `aws-sdk-s3` directly — narrower than `object_store`'s local/S3 swap
and it wouldn't match the sibling change; (c) reusing the per-file gRPC upload —
too slow/costly at corpus scale. Bucket/prefix/credentials come from the shared
`CYMBRA_SCORE_S3_*` config; the public corpus uses a distinct prefix with a
low-confidence sub-prefix.

### Catalog schema & search-readiness
`catalog_scores` is designed so a later `SearchScores` gRPC method needs only a
handler + indexes, never a re-parse backfill. Guiding rule: **anything the parser
or source yields for free at ingest is persisted now**; adding a column later is
a cheap `ALTER`, but backfilling parser-derived musical metadata means re-reading
every `.mxl` (source bytes may be gone). Columns:
- provenance/attribution: `id` (UUID v7), `title`, `composer`, `arranger`,
  `source`, `source_url`, `source_item_id`, `license` (CHECK ∈ whitelist),
  `license_url`, `confidence` (CHECK verified|unverified), `origin_format`,
  `conversion_status` (CHECK), `sha256` (UNIQUE), `object_key`, `size_bytes`,
  `created_at`.
- search/facet + musical metadata (captured at ingest from the parsed
  `ScoreDocument`): `work_key` (normalised composer+title, dedup/grouping),
  `title_norm` (unaccented/lowercased for text search), `is_piano` /
  `instrumentation`, `key_fifths`, `time_sig`, `measure_count`, `language`,
  `voicing`, `level`, `level_source`.
- `metadata JSONB NOT NULL DEFAULT '{}'` for source-specific extras without a
  per-source migration (precedent: `users.preferences`).

Indexes: `UNIQUE(sha256)`; btree on `source`, `license`, `confidence`, `level`,
`work_key`. **Typo-tolerant (fuzzy) search** comes from **`pg_trgm`** — a GIN
trigram index on `title_norm`/`composer` giving `%`/`similarity()`/
`word_similarity()` (e.g. "beethvn" ≈ "Beethoven"), with a tunable threshold.
Plain FTS (`tsvector`) is stemming/ranking but **not** typo-tolerant on its own,
so trigram is the primary fuzzy layer; a `tsvector` column may be added for
multi-word ranked relevance and `fuzzystrmatch` (`levenshtein`/`dmetaphone`) for
phonetic composer-name matching. `title_norm` is written unaccented+lowercased at
ingest so accents/case never block a match. All of these indexes are deferred to
the search-API change — one-time `CREATE INDEX CONCURRENTLY`, cheap to add late,
no re-parse. **Ops caveat:** `CREATE EXTENSION pg_trgm` (and `unaccent`,
`fuzzystrmatch`) is a DB-global privileged op — NOT runnable by the `score`
module's least-privilege role; it belongs in an admin/ops migration per
`ops-db-access`, not the module migration.

Per repo convention: runtime `sqlx::query().bind()` (not `query!`), per-module
schema with pinned `search_path`, no cross-schema FK (public corpus has no
owner).

### Search-API pagination (token/keyset, not offset)
gRPC has no built-in pagination — it is a message convention (Google AIP-158).
`SearchScores` is a **unary** RPC with request fields
`query`, `filters{source,license,confidence,level,key,…}`, `page_size`, and an
opaque `page_token` (empty = first page); the response carries
`repeated ScoreSummary results` + `next_page_token` (empty = no more pages). The
client re-invokes the same RPC feeding `next_page_token` back until it is empty.
Server-streaming is reserved for bulk export, not the UI search path (it can't
model "jump to page N"). Paging also keeps each response under the gRPC max
message size.

The server MUST translate the token to a **keyset (cursor)** query, never
`OFFSET`: `OFFSET` scans+discards and is unstable under concurrent inserts
(duplicates/skips), while keyset is O(page_size) via index and stable.
- **Browse / non-ranked list**: order by UUID-v7 `id` (monotonic in time), so the
  cursor is just the last `id`: `WHERE id < :last ORDER BY id DESC LIMIT :n+1`
  (fetch n+1 to know whether a next page exists).
- **Fuzzy-ranked search** (`ORDER BY similarity(title_norm,$q) DESC`): the sort
  key is non-unique, so the cursor MUST be the tuple `(similarity, id)` with `id`
  as tie-break — `WHERE (similarity,id) < (:s,:last) ORDER BY similarity DESC,
  id DESC LIMIT :n+1` — otherwise rows with equal similarity can be skipped or
  duplicated across pages.
The `page_token` opaquely encodes that cursor tuple (e.g. base64); it is server
-internal and never parsed by the client. `total_size` is omitted (an exact count
over a fuzzy predicate is expensive); the UI relies on `next_page_token` presence.

### Difficulty (`level`) — source-first, heuristic fallback, provenance-tracked
Difficulty is neither given reliably by sources nor trivially computable
(symbolic difficulty estimation is an open MIR problem). Policy: use a
source-declared grade when present (`level_source='source'`); otherwise compute a
cheap heuristic estimate at ingest from parser features — note density, smallest
rhythmic value/tuplets, polyphony (`is_chord`/simultaneity), pitch range + max
leap, `key_fifths`, active staff count, tempo when a metronome direction exists —
weighted and bucketed into Beginner/Intermediate/Advanced, stored with
`level_source='heuristic'`; `NULL` only if it can't run. `level_source ∈
source|heuristic|manual|null` keeps the estimate distinguishable from a real
grade, so a later curation pass or ML model overwrites only `heuristic`/`null`
rows, never a human/source grade. The heuristic is a pure function
(`difficulty.rs`) over `ScoreDocument`, unit-testable, and explicitly non-authoritative.

### TUI
`ratatui` + `crossterm`. The crawl + ingestion runs on Tokio tasks and streams
progress events over an `mpsc` channel into the TUI render loop; a headless/CLI
mode (clap) runs the same orchestrator + ingestion without the UI for
CI/scripting. This keeps the engine (library) independent of the presentation
layer, and both paths write to the same store/catalog.

## Risks / Trade-offs

- **Wrong-licence leak (the core risk)** → License-first gate + pure,
  exhaustively fixture-tested normaliser; ambiguity/unknown always rejects;
  self-declared PD quarantined to a low-confidence prefix + `confidence` flag;
  every rejection journalled for audit; README disclaimer that final
  verification is the user's responsibility.
- **Extraction done twice / conflicts with `add-user-score-upload`** →
  `musicxml-core` is a single coordinated crate; whichever change lands first
  creates it, the other depends on it (this change treats it as a prerequisite).
  Align on the crate name and public API before either merges.
- **frb codegen breaks on extraction** → Keep `#[frb]` types in the app crate
  (wrap the shared crate); gate the refactor on unchanged generated Dart + green
  `score-notation` tests before touching anything else.
- **Catalog vs upload schema drift** → `catalog_scores` (public, no owner) and
  `user_scores` (owned) share the `score` module schema but are distinct tables;
  the app's read path lists both plus the bundled catalog through the module.
- **External converters absent/version-drift** → Treated as optional; missing
  binary or non-zero exit → recorded failure (LilyPond → `failed_kept_source`),
  never a panic; documented as prerequisites.
- **Site TOS / robots changes / brittle scrapers** → Per-source adapters
  isolated; offline HTML/XML fixtures test parsing; robots respected; a broken
  adapter fails only its own items.
- **`git2` build friction (libgit2/openssl)** → Prefer the `git` binary via
  `Command` for clone/pull to avoid native-TLS pull-in; fall back to `git2` only
  if needed.
- **Large corpus disk/time** → `--limit`/quotas for test runs, resume cache, and
  low concurrency keep runs bounded and restartable.

## Migration Plan

0. **Prerequisite (coordinated with `add-user-score-upload`):** `crates/musicxml-core`
   exists — the pure parser lifted out of the app, generated Dart +
   `score-notation` tests unchanged. If that change hasn't landed it, this change
   performs the extraction; otherwise it depends on the existing crate.
1. Land the `catalog_scores` table (new migration in the `score` module schema)
   and the admin/ingestion write path in the `score` module.
2. Add `crates/score-crawler` (engine library) with the license engine,
   orchestrator, politeness, conversion — behind the CLI, no TUI yet — writing to
   the shared store via the `object_store` + `score` repo.
3. Add adapters incrementally (start with the CC0 git sources: OpenScore,
   Mutopia), each with offline fixtures.
4. Add the `ratatui` TUI and the derived manifest export on top of the engine.
- **Rollback:** the crawler crate is additive (new member); removing it reverts
  cleanly. The `catalog_scores` table is additive (a forward-only migration; drop
  it if unused). The parser extraction is shared and rolls back with its owning
  change.

## Open Questions

- ~~`catalog_scores` columns/indexes and difficulty policy~~ — RESOLVED: schema +
  search-readiness and the source-first/heuristic/`level_source` difficulty policy
  are decided above.
- Heuristic difficulty thresholds/weights need calibration against a few graded
  reference pieces; the buckets will drift until tuned (it is explicitly
  non-authoritative in the meantime).
- Object-store prefix layout for the public corpus (and low-confidence
  sub-prefix) and credential source (env vs shared profile) — confirm against the
  `CYMBRA_SCORE_S3_*` config `add-user-score-upload` introduces.
- Whether ingestion should also enqueue a `job-infrastructure` job (retryable)
  vs writing inline in the crawler process.
- IMSLP/CPDL crawl scope and rate limits acceptable to those hosts for the first
  real run (start conservative: low quota, 2 s delay, concurrency 2).
- ~~Whether NEUMA/Josquin need MEI→MusicXML~~ — RESOLVED (dropped): the NEUMA
  site no longer exists and the Josquin site could not be located, so both are
  out of scope.
- Resume-cache backend: flat JSON vs `sled` — decide when state volume is known.
