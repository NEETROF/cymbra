## 1. Shared MusicXML crate (`musicxml-core`) — coordinated prerequisite

<!-- Shared with add-user-score-upload: if that change already extracted the
     crate, skip 1.1–1.7 and just depend on it. Otherwise perform the extraction
     here. Do NOT extract twice. -->

<!-- §1 delivered as `cymbra-musicxml-core` via PR #79 (merged to main); the
     crawler depends on it. The extraction is complete — these boxes reflect that
     inherited work. -->

- [x] 1.1 Confirm whether `crates/musicxml-core` already exists (delivered by `add-user-score-upload`); if so, depend on it and skip to §2 — exists as `cymbra-musicxml-core` (PR #79); the crawler depends on it.
- [x] 1.2 Enable the `crates/*` member glob in the workspace `Cargo.toml` and scaffold `crates/musicxml-core` (lib, edition 2024, Apache-2.0 headers) — done in PR #79.
- [x] 1.3 Move the pure data model + streaming parser + geometry from `apps/music/rust/src/api/musicxml.rs` + `musicxml_core.rs` into the crate, stripped of `flutter_rust_bridge`; expose `parse(bytes) -> Result<ScoreDocument>` — done in PR #79.
- [x] 1.4 Add a `validate_musicxml(bytes) -> bool` helper for native-input validation and `.mxl` re-parse verification — `validate()` + `mxl::decode` in the crate (PR #79).
- [x] 1.5 Rewire `apps/music/rust` to depend on `musicxml-core`; keep the `#[frb]` types/wrappers in the app crate as re-exports/wrappers — done via `#[frb(mirror(...))]` (PR #79).
- [x] 1.6 Run `flutter_rust_bridge_codegen generate`; confirm the generated Dart MusicXML API is unchanged (diff) and `score-notation` tests stay green — byte-identical Dart; 65 app tests green (PR #79).
- [x] 1.7 `cargo fmt` + `cargo clippy -D warnings` clean; `cargo llvm-cov` on `musicxml-core` ≥ 80% — 95% coverage (PR #79).

## 2. Backend `score` catalog + ingestion path

<!-- The score module + object-store config are created by add-user-score-upload.
     This change ADDS the public-corpus catalog table + admin ingestion path. -->

- [x] 2.1 Add a Postgres migration creating `catalog_scores` in the `score` module schema — provenance (id, title, composer, arranger, source, source_url, source_item_id, license CHECK, license_url, confidence CHECK, origin_format, conversion_status CHECK, sha256 UNIQUE, object_key, size_bytes, created_at), search/musical metadata (work_key, title_norm, is_piano/instrumentation, key_fifths, time_sig, measure_count, language, voicing, level CHECK, level_source CHECK), and `metadata JSONB NOT NULL DEFAULT '{}'` — new `backend/score` module (`cymbra-score`): `migrations/0001_catalog.sql` + `score` schema/role in `roles.sql.tpl`/`00-roles.sh`.
- [x] 2.2 Add btree indexes on `source`, `license`, `confidence`, `level`, `work_key`; defer fuzzy/full-text indexes to the search-API change — the `pg_trgm` GIN on `title_norm`/`composer` (typo tolerance) plus optional `tsvector`/`fuzzystrmatch`, and note that `CREATE EXTENSION pg_trgm`/`unaccent`/`fuzzystrmatch` must live in an admin/ops migration (not the least-privilege module role) — btree indexes created; the deferral is noted in the migration.
- [x] 2.3 Extend the `score` module repo with an admin/ingestion write API (insert catalog row + dedup-by-sha256 check), scoped to the ingestion role — `CatalogRepo` (`sha_exists` + idempotent `insert` via `ON CONFLICT (sha256) DO NOTHING`); `FakeCatalogRepo` unit-tested + `PgCatalogRepo`.
- [~] 2.4 Reuse the `object_store` abstraction + `CYMBRA_SCORE_S3_*` config (LocalFileSystem in dev ⇄ S3/MinIO in prod); define the public-corpus prefix + low-confidence sub-prefix — LocalFs backend + confidence prefixes done (`output.rs`); the S3 backend is config-recognised but not yet wired (`main` bails on S3).
- [~] 2.5 Tests: catalog insert + dedup, confidence-prefix routing, backend-swap (local ⇄ s3) via config, CHECK-constraint rejection of bad enums — insert+dedup + confidence-prefix + env-override tested with fakes; **a real Postgres integration test** (`tests/pg_ingest_it.rs`, gated on `CYMBRA_SCORE_DATABASE_URL`) crawls the fixture, writes the corpus, and ingests into live `score.catalog_scores` (idempotent re-ingest verified); the `roles.sql.tpl`/`00-roles.sh` bootstrap was validated on a fresh volume. Bad-enum CHECK rejection + the S3 backend remain untested.

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

- [x] 5.1 HTTP client wrapper (descriptive User-Agent+contact, per-host delay, `backoff` retry on 429/errors) — built as `politeness.rs` + `http.rs` (`HttpFetcher` behind an injectable `Fetcher`), then **REMOVED** with the web sources: the surviving git/dataset sources do no polite HTTP crawling. See design "Scope revision".
- [x] 5.2 robots.txt fetch/cache/enforcement via `texting_robots` — built as `robots.rs`, then **REMOVED** with the web layer (no web crawling remains).
- [x] 5.3 Concurrency cap via a Tokio `Semaphore` (default 2) — built in `politeness`, then **REMOVED** with the web layer.
- [x] 5.4 Resumable on-disk state cache (skip completed ids) + SHA-256 content dedup across sources and against `catalog_scores` — `state.rs` (completed ids + seen hashes, JSON, seeds `Orchestrator::with_seen`). **Two-layer dedup**: exact SHA-256 (in-run + state + `catalog_scores.sha256 UNIQUE`) **and** a musical **content fingerprint** (`fingerprint.rs`: hash of the notes, encoding-independent) so re-encodings / cross-site duplicates are caught; `catalog::ingest` skips on either (`sha_exists`/`fingerprint_exists`).
- [x] 5.5 Define the `SourceAdapter` async trait + central orchestrator enforcing pipeline order with the license-first gate before `fetch` — `sources.rs` + `crawl.rs`; test asserts `fetch` is not called for a rejected licence.
- [x] 5.6 Isolate single-item failures (logged + recorded, never abort); no `unwrap()`/`expect()` in production paths — per-item failures journalled, crawl continues; verified by test.

## 6. Conversion pipeline → `.mxl`

- [x] 6.1 Native `.musicxml`/`.xml` → validate via `musicxml-core`, reject non-MusicXML XML
- [x] 6.2 Spec-compliant `.mxl` builder (`zip` + `META-INF/container.xml`) + re-parse verification step
- [x] 6.3 MuseScore `.mscx/.mscz` → MuseScore CLI headless with exit-code check + timeout — `convert::musescore_to_mxl` via `run_external` (subprocess + timeout + exit-code; missing binary/image = clean per-item failure). The temp-file extension is sniffed from the bytes (`PK` zip magic → `.mscz`, else `.mscx`) so MuseScore reads the format correctly (tested). A configurable **`local` vs `docker`** backend (also settable via `CYMBRA_SCORE_CONVERTER_BACKEND` / `…_MUSESCORE_IMAGE`, tested) runs the tool from PATH or inside an image. A real headless MuseScore 4 image is provided (`musescore.Dockerfile`): its `mscore` shim rewrites `-o OUT IN` into MuseScore's offscreen **batch-job** mode, the one combination that converts and exits (env `offscreen` is ignored by 4.7; `xvfb` hangs; `-platform offscreen` on argv corrupts file parsing). `run_external` tested (success/exit/not-found/timeout); the Docker round-trip tested via `alpine` (opt-in `SCORE_CRAWLER_DOCKER_TEST=1`); MuseScore conversion verified end-to-end on real OpenScore scores.
- [~] 6.4 MEI → Verovio (`-t musicxml`); LilyPond `.ly` → python-ly with `failed_kept_source` fallback (keep .ly + PDF); never convert MIDI — Verovio (`verovio_to_mxl`) + python-ly (`lilypond_to_mxl`) wired; MIDI has no `OriginFormat`/dispatch branch so it can't be converted. The `failed_kept_source` refinement (persisting the `.ly`/PDF) awaits a writer that stores non-`.mxl` artefacts.
- [x] 6.5 Record `conversion_status`; tests for `.mxl` build + re-parse verification (fixtures)

## 7. Metadata extraction + difficulty + ingestion into the shared store

- [x] 7.1 `metadata.rs`: derive search/musical fields from the parsed `ScoreDocument` (work_key, title_norm, is_piano/instrumentation, key_fifths, time_sig, measure_count, language, voicing) at ingest — pure; language/voicing left to adapters (not in the parsed model).
- [x] 7.2 `difficulty.rs`: pure heuristic `estimate_level(&ScoreDocument) -> Level` (note density, smallest rhythmic value/tuplets, polyphony, pitch range + max leap, key_fifths, staff count, tempo); set `level_source=source` when the adapter supplies a grade, else `heuristic`, else null — never mark a heuristic as source; unit tests over crafted scores
- [x] 7.3 `ingest.rs`: write `.mxl` bytes to the object store under the confidence-appropriate prefix, then insert the `catalog_scores` row with all metadata (dedup-by-sha256, idempotent) — LocalFs write (`output.rs`) + `catalog.rs` maps each entry to a `CatalogEntry` and inserts via `CatalogRepo`; `main` connects Postgres + runs migrations + ingests when `CYMBRA_SCORE_DATABASE_URL` is set. (Exercising the real insert needs a running Postgres.)
- [x] 7.4 Enforce confidence separation (verified prefix vs low-confidence prefix + `confidence` column); never mix — `output.rs` routes by confidence; test asserts no cross-contamination.
- [x] 7.5 Derived manifest export: `manifest.csv` + `manifest.json` from `catalog_scores` (consistent), and append `rejected.log` (source, url, raw licence, reason) — `manifest.rs` + `output.rs`; JSON/CSV consistency tested.
- [x] 7.6 Tests: idempotent re-ingest is a no-op, confidence separation, CSV/JSON consistency, rejection journalling, level_source provenance — all tested with fakes: `catalog::ingest`/`FakeCatalogRepo` dedup by sha256 (idempotent), confidence separation, CSV/JSON consistency, rejection journalling, level_source provenance.

## 8. Source adapters

- [x] 8.1 Shared helpers: git-clone family (git2 or `git` subprocess + on-disk walk) and web-crawl family (reqwest + scraper/quick-xml) — git-clone (`sources/git.rs`) + web-crawl (`http.rs` `Fetcher` + `sources/web.rs` scraper link/licence helpers, fixture-tested).
- [x] 8.2 OpenScore Lieder (git, CC0) — `GitRepoSource::openscore` clones the corpus, walks it, and gates CC0. `.mscx`/`.mscz` convert through the docker MuseScore backend (see §6.3); each song ships both formats, so the redundant `.mscz` is dropped in `discover` (tested) to run MuseScore once. Verified end-to-end both ways: host binary + docker backend (13 CC0 `.mxl`, 0 failures) and the compose fan-out service spawning a `cymbra/musescore` sibling container.
- [~] 8.3 Mutopia (git/site, per-file PD/CC-BY/CC-BY-SA from `.ly` header) with fixture — `sources/mutopia.rs`: clones MutopiaProject, reads the **per-file** licence from the `.ly` header (`license`/`copyright`) and gates it (the licence engine now also parses the spelled-out "Creative Commons Attribution-ShareAlike 4.0" form). Fixtures prove PD/CC-BY-SA accepted and **CC-BY-NC rejected per file**. Registered. LilyPond→MusicXML conversion needs `python-ly` (else the item is a per-item failure).
- [x] 8.4 CPDL (MediaWiki API, per-page licence) — **REMOVED after evaluation**: automated access returns HTTP 403 and MusicXML is rare (mostly PDF/MuseScore). Adapter + web-crawl layer deleted; recorded in `sources::EXCLUDED_SOURCES`.
- [x] 8.5 IMSLP (web crawl, per-file legal status) — **REMOVED after evaluation**: paywalled PDF scans, almost no MusicXML. Adapter deleted; recorded in `sources::EXCLUDED_SOURCES`.
- [~] 8.6 PDMX (Zenodo dataset, `no_license_conflict` subset only; rest rejected) with fixture — reworked to the **real** shape: PDMX is a bulk Zenodo dataset (`PDMX.csv` index + `mxl.tar.gz`), not a per-item API. `sources/pdmx.rs` `PdmxDatasetSource` downloads + extracts both (streaming, `flate2`/`tar`), discovers only rows with `subset:no_license_conflict`, and reads the extracted `.mxl` (no conversion). CSV subset-filtering + extracted-file read are fixture-tested; the exact CSV column names + the 1.9 GB download/extract need confirming on a real run.
- [x] 8.7 musetrainer/library (git, self-declared PD → low-confidence) with fixture — `GitRepoSource::musetrainer`; offline fixture repo flows end-to-end through the orchestrator to low-confidence (test).
- [~] 8.8 eduardomourar/music-scores-musicxml (git, verify README/LICENSE) with fixture — `GitRepoSource::eduardomourar` constructor done (MusicXML, same tested path); dedicated fixture + LICENSE verification pending.
- [x] 8.9 Project Gutenberg sheet music (web crawl, Gutenberg PD licence) — **REMOVED after evaluation**: the holdings are text ebooks (GUTINDEX); no MusicXML is served. Adapter deleted; recorded in `sources::EXCLUDED_SOURCES`.
- [x] 8.10 NEUMA (REST, MEI/MusicXML per collection) with fixture — DROPPED from scope: the NEUMA site (`neuma.huma-num.fr`) no longer exists, so it cannot be crawled or its licences verified. Removed from `ALL_SOURCES`.
- [x] 8.11 Josquin Research Project (site/repo, open access — verify terms) with fixture — DROPPED from scope: the site could not be located, so its terms cannot be verified. Removed from `ALL_SOURCES`.
- [x] 8.12 Hymnary.org (web crawl, per-item) — **REMOVED after evaluation**: scores are gated paid "FlexScores"; no free bulk MusicXML and no work index to crawl. Adapter deleted; recorded in `sources::EXCLUDED_SOURCES`.
- [x] 8.13 Register all adapters in the orchestrator registry — `registry.rs` `build_adapters` wires the **delivered** sources (openscore, mutopia, musetrainer, eduardomourar, pdmx) + the CLI/`main` run loop; the `unsupported` list catches unknown names and the excluded web sources (cpdl/imslp/gutenberg/hymnary/neuma/josquin — see `sources::EXCLUDED_SOURCES`).

## 9. TUI

- [~] 9.1 `ratatui`/`crossterm` TUI: source selection, live per-source progress (accepted/rejected/low-confidence) via mpsc, catalog browser — `tui.rs`: pure `App` state (source selection + per-source/total progress) unit-tested; ratatui render + crossterm loop stream `run::ProgressEvent`s over an mpsc channel. The manifest/catalog browser pane is still to add.
- [x] 9.2 Wire TUI actions and headless CLI mode to the same orchestrator + ingestion path — both go through `run::run_all` (shared license-first loop, dedup across sources) then `OutputWriter`; `--tui` vs headless is the only difference.

## 10. Docs, quality gates, validation

- [x] 10.1 `crates/score-crawler/README.md` (Rust toolchain, external binaries MuseScore CLI/Verovio/python-ly, object-store + catalog config, usage examples, and the clear "final licence verification is the user's responsibility" disclaimer)
- [x] 10.2 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` clean — `score-crawler` + `cymbra-score` clippy-clean; `cargo fmt --all --check` and `cargo build --workspace` pass with the two new members.
- [x] 10.3 `cargo test` green; license + `.mxl`-validation + ingestion modules covered with offline fixtures — 85 crawler tests + score-module tests green; ~83% line coverage; licence engine 99.6%.
- [x] 10.4 Run `openspec validate add-score-crawler --strict` and fix any issues — passes.
