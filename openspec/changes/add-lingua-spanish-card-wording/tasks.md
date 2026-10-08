# Tasks

## 1. The baseline (crates/lingua-wasm)

- [ ] 1.1 `tests/en_es_baseline.rs` and `baseline/en-es.golden`: the en-es scenario over the reference's corpus and probes, blessed (D1).

## 2. The wording (apps/lingua-extension)

- [ ] 2.1 `test/word-card-en-es.spec.ts`: the golden's word-grammar probes and 40 glossed lemmas rendered with the interface in Spanish, every line pinned (D2).
- [ ] 2.2 `src/i18n/es/grammar.ts`: the English-studied tables corrected on the real readings; each correction a line of the spec (D3).
- [ ] 2.3 `selection-card.ts`: the empty-sense pattern per edition, the cut's trailing set; en-fr's and es-fr's rows byte for byte over their first 500 lemmas (D4).

## 3. Dogfood

- [ ] 3.1 A local build listing en-es, the interface in Spanish, twenty pages read; findings fixed or listed for change 33 (D5).

## 4. Gates, review and docs

- [ ] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-wasm`; in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (`word-grammar.spec.ts` unchanged), `yarn build`, `yarn check:variants`.
- [ ] 4.2 [manual] The owner reviews the Spanish lines the spec pins (M9).
- [ ] 4.3 `openspec validate add-lingua-spanish-card-wording --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-spanish-card-wording` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 24 is marked done in `docs/lingua/language-matrix-programme.md`.
