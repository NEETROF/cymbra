## Why

Cymbra's score library ships a handful of hand-picked MusicXML files. To grow a
large, **legally re-distributable** corpus we need to harvest public-domain and
permissively-licensed scores from many online libraries — but the hard part is
legal, not technical: shipping a single wrongly-licensed file is a liability.
We need a tool that treats the license as a gate, keeps only provably free
material, converts everything to compact compressed MusicXML (`.mxl`), and — so
the app can actually serve it — **ingests the vetted corpus into the same score
store the app already reads**: Postgres for the queryable catalog plus an object
store for the bytes. This change is the bulk/automated counterpart of the manual
`add-user-score-upload` flow; the two SHARE the score storage and the extracted
MusicXML parser.

## What Changes

- Add a new Rust workspace crate `crates/score-crawler` (library + binary +
  TUI): a polite, resumable crawler that collects scores from multiple online
  score libraries through a per-source `SourceAdapter` trait, orchestrated
  centrally.
- **License-first pipeline**: the licence of each item is determined and
  normalised **before** any heavy content is downloaded; anything not on an
  explicit whitelist (CC0 / confirmed Public Domain / CC-BY any version /
  CC-BY-SA any version) is rejected and journalled, never ingested.
- **Low-confidence quarantine**: user-declared / unverified public-domain
  material (e.g. musetrainer, non-vetted PDMX subset) is marked `unverified` and
  kept out of the high-confidence corpus (separate prefix + `confidence` flag in
  the catalog), never mixed with the safe corpus.
- **Conversion to `.mxl`**: validate native MusicXML, convert MuseScore
  (`.mscx/.mscz`), LilyPond (`.ly`), and MEI via external binaries
  (MuseScore CLI, `python-ly`, Verovio) invoked as subprocesses with timeouts;
  produce spec-compliant `.mxl` and verify each output re-parses. MIDI is never
  treated as a score source.
- **Ingest into the shared score store** (aligned with `add-user-score-upload`):
  write each retained file's bytes to an **object store** abstracted by the
  `object_store` crate — a **local filesystem folder in dev**, S3/MinIO in prod,
  swapped by config — and write its provenance as a row in a new
  **`catalog_scores`** table in the backend `score` module's Postgres schema.
  The crawler writes **directly** through the `score` module's repo/storage with
  an admin/ingestion role (like the `worker`), not over per-file gRPC. Postgres
  is the source of truth; `manifest.csv` + `manifest.json` become a **derived
  export**, and a `rejected.log` records exclusions.
- **Politeness & robustness**: honour `robots.txt`, descriptive contact
  User-Agent, configurable inter-request delay (default 2 s), exponential
  back-off on errors/429, low concurrency cap (default 2), SHA-256 content
  dedup across sources (and against rows already in `catalog_scores`), resumable
  on-disk state cache, no `unwrap()` in production paths.
- **TUI (ratatui)**: an interactive terminal app to pick sources, watch crawl +
  ingestion progress, review what was accepted / rejected / quarantined, and
  browse the resulting catalog before/after ingestion.
- **Reuse the extracted MusicXML parser**: consume the shared `musicxml-core`
  crate (the parser lifted out of `apps/music/rust` — the SAME extraction
  `add-user-score-upload` relies on) for native-input validation and `.mxl`
  re-parse verification. The extraction is delivered once and coordinated
  between the two changes.

## Capabilities

### New Capabilities

- `score-crawler`: the crawl engine — the `SourceAdapter` trait, the per-source
  adapters, central orchestration, politeness (robots/delay/back-off/concurrency),
  resumable state, content dedup, config (`config.yaml`), and the CLI.
- `license-filtering`: detection + normalisation of licences into a whitelist of
  redistributable codes, the license-first gate, rejection journalling, and the
  low-confidence classification. The legally-critical, most heavily tested unit.
- `score-conversion`: the conversion pipeline to validated, spec-compliant `.mxl`
  (native validation, MuseScore/LilyPond/MEI converters, `.mxl` container build
  + re-parse verification, conversion-status reporting).
- `corpus-manifest`: the provenance model (the `catalog_scores` columns) that is
  the source of truth for lawful re-distribution, plus the derived
  `manifest.csv` / `manifest.json` export and `rejected.log`.
- `corpus-ingestion`: ingestion of the vetted corpus into the shared `score`
  store — bytes to the `object_store`-abstracted object store (local folder in
  dev ⇄ S3/MinIO in prod), provenance rows into `catalog_scores`, confidence
  separation, dedup/idempotency — plus the ratatui TUI that drives it.

### Modified Capabilities

<!-- The MusicXML parser extraction into crates/musicxml-core is shared with
     add-user-score-upload and does not change score-notation's app-observable
     requirements, so it is not listed as a modified capability here. It is
     tracked in this change's tasks as a coordinated prerequisite. -->

## Impact

- **New crate**: `crates/score-crawler` (crawler lib + `score-crawler` binary +
  TUI), added under the `crates/*` member glob.
- **Shared parser crate** `crates/musicxml-core`: the pure data model + parser
  lifted out of `apps/music/rust/src/api/musicxml.rs` +
  `musicxml_core.rs`, decoupled from `flutter_rust_bridge`, consumed by the app
  engine, the backend `score` module, and this crawler. **Coordinated with
  `add-user-score-upload`** — extracted once, whichever change lands first; the
  other depends on it. No change to the generated Dart API or app behaviour;
  `score-notation` tests must stay green.
- **Backend `score` module**: this change ADDS a `catalog_scores` table (public
  corpus: licence-bearing, no owner) in the `score` module's schema, alongside
  the `user_scores` table created by `add-user-score-upload`, and an
  admin/ingestion write path the crawler uses. The app lists bundled catalog +
  user uploads + public corpus through this module.
- **Object store**: reuse the `object_store` abstraction (LocalFileSystem in dev,
  S3/MinIO in prod) and the `CYMBRA_SCORE_S3_*` config introduced by
  `add-user-score-upload`, with a distinct prefix for the public corpus (and a
  low-confidence sub-prefix).
- **New dependencies** (crawler crate only): `tokio`, `reqwest` (rustls, stream,
  gzip), `scraper`, `quick-xml`/`roxmltree`, `texting_robots`, `backoff`,
  `governor`, `sha2`, `zip`, `git2`, `serde`/`serde_json`/`serde_yaml`/`csv`,
  `clap`, `tracing`/`tracing-subscriber`, `ratatui`/`crossterm`, `sqlx` +
  `object_store` (via the `score` module's storage), `anyhow`/`thiserror`.
- **External binaries** (runtime, invoked as subprocesses): MuseScore CLI,
  Verovio, `python-ly`. Documented as optional prerequisites; conversion degrades
  gracefully when absent.
- **Legal**: the tool filters on licence but the README must state clearly that
  final verification responsibility remains with the user.
- Out of scope: no changes to the shipping Flutter app UI; app-side surfacing of
  the public corpus in the library is a follow-up. The crawler is an
  operator/dev ingestion tool.
