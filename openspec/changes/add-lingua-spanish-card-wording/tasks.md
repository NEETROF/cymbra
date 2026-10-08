# Tasks

## 1. The baseline (crates/lingua-wasm, workflows)

- [x] 1.1 `tests/en_es_baseline.rs` and `baseline/en-es.golden`: the en-es scenario over the reference's corpus and probes and the 40 lemma probes, blessed; both workflows name `en_es_baseline` (D1).
- [x] 1.2 The golden-to-golden test through `studied_side`; `cross_native.rs`’s English check on the real en-es pack (Context). The synthetic second pack (`support/other_native.rs`) had no user left and is gone; en-es's NUM runs, which no en-fr run carries, are the spec's *sense part of speech the first pack never used* (« three »: NUM, NOUN, read as written), and its glosses of lemmas en-fr does not gloss are no dictionary words, as es-en's.

## 2. The wording (apps/lingua-extension)

- [x] 2.1 `test/word-card-en-es.spec.ts` and its snapshot: every probe of the committed golden rendered with the interface in Spanish; `lingua-pack-update` re-blesses the snapshot (D2).
- [x] 2.2 `src/i18n/es/grammar.ts`: the English-studied tables corrected on the real readings; `word-grammar-es.spec.ts` moves with them (D3). « pasado simple » and « forma en -ing » read right on every real reading; two names corrected, as the Spanish Wiktionary's English form-of lines give them — the past participle « participio pasado » (« participio pasado de go », was « participio de go ») and the present « presente simple » (« tercera persona del singular del presente simple de have », was « … del presente de have »), a Spanish participle staying the RAE's « participio »; the French unchanged; what reads wrong and is data, or wording the spec keeps, is listed in design.md *Known data defects*.
- [x] 2.3 `selection-card.ts`: the row cut per D4. No committed row ends on ¿ or ¡ (they stand against their words, so a cut lands on the space before them): the trailing set gains neither, `test/row-gloss-tables.spec.ts` measures them over every pair, a case pins the cut before them and a closing ? or ! kept.

## 3. Dogfood

- [ ] 3.1 A local build listing en-es beside the French pairs, the interface in Spanish, signed out or on a test account, twenty pages read; findings fixed or listed (D5).

## 4. Gates, review and docs

- [x] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-wasm`; in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (`word-grammar.spec.ts` unchanged), `yarn build`, `yarn check:variants`.
- [ ] 4.2 [manual] The owner reviews the Spanish lines the snapshot pins (M9).
- [x] 4.3 `openspec validate add-lingua-spanish-card-wording --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-spanish-card-wording` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 24 is marked done in `docs/lingua/language-matrix-programme.md`.
