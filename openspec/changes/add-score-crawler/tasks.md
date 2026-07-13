## 1. Shared MusicXML crate (`musicxml-core`) — coordinated prerequisite

<!-- Shared with add-user-score-upload: if that change already extracted the
     crate, skip 1.1–1.7 and just depend on it. Otherwise perform the extraction
     here. Do NOT extract twice. -->

- [ ] 1.1 Confirm whether `crates/musicxml-core` already exists (delivered by `add-user-score-upload`); if so, depend on it and skip to §2
- [ ] 1.2 Enable the `crates/*` member glob in the workspace `Cargo.toml` and scaffold `crates/musicxml-core` (lib, edition 2024, Apache-2.0 headers)
- [ ] 1.3 Move the pure data model + streaming parser + geometry from `apps/music/rust/src/api/musicxml.rs` + `musicxml_core.rs` into the crate, stripped of `flutter_rust_bridge`; expose `parse(bytes) -> Result<ScoreDocument>`
- [ ] 1.4 Add a `validate_musicxml(bytes) -> bool` helper for native-input validation and `.mxl` re-parse verification
- [ ] 1.5 Rewire `apps/music/rust` to depend on `musicxml-core`; keep the `#[frb]` types/wrappers in the app crate as re-exports/wrappers
- [ ] 1.6 Run `flutter_rust_bridge_codegen generate`; confirm the generated Dart MusicXML API is unchanged (diff) and `score-notation` tests stay green
- [ ] 1.7 `cargo fmt` + `cargo clippy -D warnings` clean; `cargo llvm-cov` on `musicxml-core` ≥ 80%

## 2. Backend `score` catalog + ingestion path

<!-- The score module + object-store config are created by add-user-score-upload.
     This change ADDS the public-corpus catalog table + admin ingestion path. -->

- [ ] 2.1 Add a Postgres migration creating `catalog_scores` in the `score` module schema — provenance (id, title, composer, arranger, source, source_url, source_item_id, license CHECK, license_url, confidence CHECK, origin_format, conversion_status CHECK, sha256 UNIQUE, object_key, size_bytes, created_at), search/musical metadata (work_key, title_norm, is_piano/instrumentation, key_fifths, time_sig, measure_count, language, voicing, level CHECK, level_source CHECK), and `metadata JSONB NOT NULL DEFAULT '{}'`
- [ ] 2.2 Add btree indexes on `source`, `license`, `confidence`, `level`, `work_key`; defer fuzzy/full-text indexes to the search-API change — the `pg_trgm` GIN on `title_norm`/`composer` (typo tolerance) plus optional `tsvector`/`fuzzystrmatch`, and note that `CREATE EXTENSION pg_trgm`/`unaccent`/`fuzzystrmatch` must live in an admin/ops migration (not the least-privilege module role)
- [ ] 2.3 Extend the `score` module repo with an admin/ingestion write API (insert catalog row + dedup-by-sha256 check), scoped to the ingestion role
- [ ] 2.4 Reuse the `object_store` abstraction + `CYMBRA_SCORE_S3_*` config (LocalFileSystem in dev ⇄ S3/MinIO in prod); define the public-corpus prefix + low-confidence sub-prefix
- [ ] 2.5 Tests: catalog insert + dedup, confidence-prefix routing, backend-swap (local ⇄ s3) via config, CHECK-constraint rejection of bad enums

## 3. Crawler crate scaffold + config + CLI

- [x] 3.1 Scaffold `crates/score-crawler` (lib + `score-crawler` bin) with modules `sources/`, `license.rs`, `convert.rs`, `catalog.rs`, `ingest.rs`, `crawl.rs`, `config.rs`, `tui.rs`, `main.rs` — engine modules landed (license, convert, metadata, difficulty, manifest, sources, crawl, config, cli, main); `catalog.rs`/`ingest.rs`/`tui.rs` deferred with their backend/TUI slices.
- [~] 3.2 Add dependencies — engine set added (cymbra-musicxml-core, anyhow, thiserror, serde/json/yaml/csv, sha2, zip, unicode-normalization, clap, tracing, tokio, async-trait); network/backend/TUI deps (reqwest, scraper, texting_robots, backoff, governor, git2, object_store, sqlx, ratatui/crossterm) attach with their modules.
- [x] 3.3 Define `config.yaml` schema + serde structs (enabled sources, per-host delay, concurrency, per-source quotas, User-Agent contact, object-store backend + prefixes, Postgres/catalog connection); env override support incl. `CYMBRA_SCORE_S3_*` — Postgres/catalog connection deferred to ingestion.
- [x] 3.4 Implement the `clap` CLI (`--sources`, `--limit`, `--all`, `--resume`, `--verbose`) and `tracing` subscriber (INFO default, DEBUG on `--verbose`)

## 4. License engine (most critical)

- [x] 4.1 Define `RawLicense`, `LicenseOutcome` (canonical code, licence URL, confidence), and `Decision` types
- [x] 4.2 Implement pure `normalize(raw) -> LicenseOutcome` (CC URLs, SPDX ids, version-less labels, NC/ND detection, per-source status strings, ambiguity/unknown handling)
- [x] 4.3 Implement `is_redistributable(outcome) -> Decision` enforcing the whitelist (CC0 / PublicDomain / CC-BY-* / CC-BY-SA-*) and `unverified` classification for self-declared PD
- [x] 4.4 Exhaustive table tests: accepted codes, NC/ND/ARR rejection, ambiguous/empty rejection, version-less "any version", self-declared PD → unverified — `license.rs` 99.6% covered.

## 5. Politeness, orchestration, dedup, resume

- [~] 5.1 HTTP client wrapper (descriptive User-Agent+contact, per-host delay, `backoff` retry on 429/errors) — back-off + retry primitives done (`politeness.rs`, pure schedule + `retry_async`); the `reqwest` GET wrapper ships with the web adapters.
- [x] 5.2 robots.txt fetch/cache/enforcement via `texting_robots` — `robots.rs` (parse/allow tested offline); per-host fetch/cache wires with the HTTP client.
- [x] 5.3 Concurrency cap via a Tokio `Semaphore` (default 2) — `politeness::ConcurrencyLimiter`.
- [x] 5.4 Resumable on-disk state cache (skip completed ids) + SHA-256 content dedup across sources and against `catalog_scores` — `state.rs` (completed ids + seen hashes, JSON, seeds `Orchestrator::with_seen`); catalog-seeding wires with ingestion.
- [x] 5.5 Define the `SourceAdapter` async trait + central orchestrator enforcing pipeline order with the license-first gate before `fetch` — `sources.rs` + `crawl.rs`; test asserts `fetch` is not called for a rejected licence.
- [x] 5.6 Isolate single-item failures (logged + recorded, never abort); no `unwrap()`/`expect()` in production paths — per-item failures journalled, crawl continues; verified by test.

## 6. Conversion pipeline → `.mxl`

- [x] 6.1 Native `.musicxml`/`.xml` → validate via `musicxml-core`, reject non-MusicXML XML
- [x] 6.2 Spec-compliant `.mxl` builder (`zip` + `META-INF/container.xml`) + re-parse verification step
- [ ] 6.3 MuseScore `.mscx/.mscz` → MuseScore CLI headless (`QT_QPA_PLATFORM=offscreen`) with exit-code check + timeout — external converter, deferred (dispatch stub errors cleanly, isolated per-item).
- [ ] 6.4 MEI → Verovio (`-t musicxml`); LilyPond `.ly` → python-ly with `failed_kept_source` fallback (keep .ly + PDF); never convert MIDI — external converters, deferred.
- [x] 6.5 Record `conversion_status`; tests for `.mxl` build + re-parse verification (fixtures)

## 7. Metadata extraction + difficulty + ingestion into the shared store

- [x] 7.1 `metadata.rs`: derive search/musical fields from the parsed `ScoreDocument` (work_key, title_norm, is_piano/instrumentation, key_fifths, time_sig, measure_count, language, voicing) at ingest — pure; language/voicing left to adapters (not in the parsed model).
- [x] 7.2 `difficulty.rs`: pure heuristic `estimate_level(&ScoreDocument) -> Level` (note density, smallest rhythmic value/tuplets, polyphony, pitch range + max leap, key_fifths, staff count, tempo); set `level_source=source` when the adapter supplies a grade, else `heuristic`, else null — never mark a heuristic as source; unit tests over crafted scores
- [ ] 7.3 `ingest.rs`: write `.mxl` bytes to the object store under the confidence-appropriate prefix, then insert the `catalog_scores` row with all metadata (dedup-by-sha256, idempotent)
- [ ] 7.4 Enforce confidence separation (verified prefix vs low-confidence prefix + `confidence` column); never mix
- [ ] 7.5 Derived manifest export: `manifest.csv` + `manifest.json` from `catalog_scores` (consistent), and append `rejected.log` (source, url, raw licence, reason)
- [ ] 7.6 Tests: idempotent re-ingest is a no-op, confidence separation, CSV/JSON consistency, rejection journalling, level_source provenance

## 8. Source adapters

- [~] 8.1 Shared helpers: git-clone family (git2 or `git` subprocess + on-disk walk) and web-crawl family (reqwest + scraper/quick-xml) — git-clone family done (`sources/git.rs`: `git` subprocess `ensure_checkout` + recursive on-disk walk, `origin_from_ext`); web-crawl family (reqwest + scraper) pending.
- [~] 8.2 OpenScore Lieder (git, CC0 — verify repo LICENSE) with offline fixture — `GitRepoSource::openscore` constructor + walk done; `.mscx` items need the MuseScore CLI converter (deferred), so end-to-end awaits §6.3.
- [ ] 8.3 Mutopia (git/site, per-file PD/CC-BY/CC-BY-SA from `.ly` header) with fixture
- [ ] 8.4 CPDL (MediaWiki API, per-page licence) with fixture
- [ ] 8.5 IMSLP (web crawl, robots-respecting, per-file legal status) with fixture
- [ ] 8.6 PDMX (Zenodo dataset, `no_license_conflict` subset only; rest rejected) with fixture
- [x] 8.7 musetrainer/library (git, self-declared PD → low-confidence) with fixture — `GitRepoSource::musetrainer`; offline fixture repo flows end-to-end through the orchestrator to low-confidence (test).
- [~] 8.8 eduardomourar/music-scores-musicxml (git, verify README/LICENSE) with fixture — `GitRepoSource::eduardomourar` constructor done (MusicXML, same tested path); dedicated fixture + LICENSE verification pending.
- [ ] 8.9 Project Gutenberg sheet music (web crawl, Gutenberg PD licence) with fixture
- [ ] 8.10 NEUMA (REST, MEI/MusicXML per collection) with fixture
- [ ] 8.11 Josquin Research Project (site/repo, open access — verify terms) with fixture
- [ ] 8.12 Hymnary.org (web crawl, per-item) with fixture
- [ ] 8.13 Register all adapters in the orchestrator registry

## 9. TUI

- [ ] 9.1 `ratatui`/`crossterm` TUI: source selection, live per-source progress (accepted/rejected/low-confidence) via mpsc, catalog browser
- [ ] 9.2 Wire TUI actions and headless CLI mode to the same orchestrator + ingestion path

## 10. Docs, quality gates, validation

- [ ] 10.1 `crates/score-crawler/README.md` (Rust toolchain, external binaries MuseScore CLI/Verovio/python-ly, object-store + catalog config, usage examples, and the clear "final licence verification is the user's responsibility" disclaimer)
- [ ] 10.2 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] 10.3 `cargo test` green; license + `.mxl`-validation + ingestion modules covered with offline fixtures
- [ ] 10.4 Run `openspec validate add-score-crawler --strict` and fix any issues
