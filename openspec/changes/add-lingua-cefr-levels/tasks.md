## 1. Engine — knowledge model (lingua-core, host-testable, no WASM)

- [x] 1.1 Add `KnownSource::Exposure` to `knowledge/status.rs`; extend `wire_provenance`/`from_wire` with `"exposure"` (round-trip test; unknown value still degrades to `manual`)
- [x] 1.2 Add a level trait (e.g. `CefrLevels { fn level(&self, lemma) -> Option<Level> }`) + a `Level` enum (A1..C2, ordered) with a host test helper, mirroring `FrequencyRanks`/`MapFrequencyRanks`
- [x] 1.3 Make `resolve_lemma` level-aware: below the declared level → implicit `Known(Calibration)`; fall back to the rank threshold when no level source is present; explicit status still wins (extend the existing tests)
- [x] 1.4 Add a declared-level field to `KnowledgeState` (set/get), alongside the existing per-language calibration threshold
- [x] 1.5 Extend `Exposure` with bounded distinct-UTC-day tracking; update `record(...)` to advance it from the caller-supplied timestamp; keep the D5 invariant (record never changes status) and its tests
- [x] 1.6 Add caller-driven `promote_by_exposure(...)`: promotes below-level, statusless lemmas with ≥ N distinct read-days (default 4) to `Known(Exposure)`; no-op on any explicit status; unit-test the four guardrails + the recording-alone-never-promotes case
- [x] 1.7 Add `band_stats(...)` folding `resolve_lemma` over a band → { confirmed, presumed, to_learn } with the total equal to band size; host tests

## 2. Engine — pack enumeration + deck seeding (lingua-core)

- [x] 2.1 Add a rank-band enumerator to `Pack` over the private `freq`/`glosses`/`lexicon` (lemmas + glosses of a rank band); host tests against a fixture pack. (The per-CEFR-level enumerator lands with the level table in 3.1.)
- [x] 2.2 Add `Card::seeded(...)` using `EncounterSource::Import` (no fabricated URL/session)
- [x] 2.3 Add `Deck::seed_lemmas(...)`: capped, skips already-carded/explicit-status lemmas, caller-chosen order; returns the count added; host tests (cap, idempotency, ordering)
- [x] 2.4 `cargo fmt --all` + `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo llvm-cov` ≥ 80% on the new core modules

## 3. Pack format + data pipeline (crates/lingua-pack, scripts/lingua-data)

- [ ] 3.1 Add the optional per-lemma CEFR level section to the pack container format + `PackMeta`; reader in `lingua-core::packs`; bump the pack format version and `ANALYZER_VERSION`
- [ ] 3.2 Add the CEFR-J v1.6 + Octanove C1/C2 join step to the build pipeline (lowest-level collapse rule); record sources in `scripts/lingua-data/SOURCES.md`
- [ ] 3.3 Extend the licence guard to admit CEFR-J (citation) + Octanove (CC BY-SA 4.0) and write both attributions into NOTICE
- [ ] 3.4 Regenerate the 3 fixture packs and the real gitignored pack (`gen:pack:real`) at the new version; confirm the `build.mjs` guard passes
- [ ] 3.5 Update the en-fr testdata pack so host tests in §1–§2 can assert real level enumeration

## 4. WASM bindings (crates/lingua-wasm — thin glue, coverage-excluded)

- [ ] 4.1 Expose `bandStats`, `seedBand`, `promoteByExposure`, and declared-level get/set as JSON methods mirroring `analyse`/`reviewCurrent`
- [ ] 4.2 Regenerate the extension's analyzer bindings and verify the port/types compile against the new methods

## 5. Extension surfaces (apps/lingua-extension)

- [ ] 5.1 Widen the analyzer port/engine/types seam with the new sync methods (band stats, seed, promote, level)
- [ ] 5.2 Popup: replace the frequency slider with a CEFR level picker for English (keep the slider as the no-CEFR fallback); persist the declared level in versioned local state (and sync it if D-open-question resolves to sync)
- [ ] 5.3 Highlighting: gate at the declared level and above (below-level presumed-known not highlighted); a clicked word still takes an explicit status
- [ ] 5.4 Call `promoteByExposure` on read (after page analysis), passing the day; ensure it is a no-op without a declared level
- [ ] 5.5 Stats screen: CEFR ladder A1→C2 (confirmed/presumed/to-learn per level + estimated position); degrade to frequency bands labelled "estimé" where no CEFR data
- [ ] 5.6 "Renforcer un niveau" control: level chips, count (bounded by the cap), order toggle, seeds via `seedBand` and reports the number added
- [ ] 5.7 Extension tests (vitest) for the new model/chart/logic; `yarn lint` + `yarn format:check` clean; keep coverage ≥ 80%

## 6. Validation + delivery

- [ ] 6.1 `openspec validate add-lingua-cefr-levels --strict` passes
- [ ] 6.2 Verify no proto/wire break (`buf breaking` unaffected — provenance stays a string)
- [ ] 6.3 Stack the implementation PRs core-first (feat, lingua-scoped); green CI; squash-merge each on CLEAN
- [ ] 6.4 After merge, `/opsx:archive add-lingua-cefr-levels`
