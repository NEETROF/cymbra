# Tasks

## 1. The baseline (crates/lingua-wasm, workflows)

- [x] 1.1 `tests/es_en_baseline.rs` and `baseline/es-en.golden`: the es-en scenario over the reference's corpus and probes and the 40 lemma probes, blessed; both workflows name `es_en_baseline` (D1).
- [x] 1.2 The golden-to-golden test through `studied_side` (D1).

## 2. The wording (apps/lingua-extension)

- [x] 2.1 `test/word-card-es-en.spec.ts` and its snapshot: every probe of the committed golden rendered with the interface in English; `lingua-pack-update` re-blesses the snapshot (D2).
- [x] 2.2 `src/i18n/en/grammar.ts`: the Spanish-studied tables corrected on the real readings; `word-grammar-en.spec.ts` moves with them (D3).
- [x] 2.3 `selection-card.ts`: the row cut per D4, with the snapshot of every en-fr and es-fr row committed first.

## 3. Dogfood

- [ ] 3.1 A local build listing es-en beside the French pairs, the interface in English, signed out or on a test account, twenty pages read; findings fixed or listed (D5).

## 4. Gates, review and docs

- [x] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-wasm`; in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (`word-grammar.spec.ts` unchanged), `yarn build`, `yarn check:variants`.
- [ ] 4.2 [manual] The owner reviews the English lines the snapshot pins (M9).
- [x] 4.3 `openspec validate add-lingua-english-card-wording --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-english-card-wording` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 23 is marked done in `docs/lingua/language-matrix-programme.md`.
