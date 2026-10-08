# Tasks

## 1. The baseline (crates/lingua-wasm, workflows)

- [ ] 1.1 `tests/en_es_baseline.rs` and `baseline/en-es.golden`: the en-es scenario over the reference's corpus and probes and the 40 lemma probes, blessed; both workflows name `en_es_baseline` (D1).
- [ ] 1.2 The golden-to-golden test through `studied_side`; `cross_native.rs`’s English check on the real en-es pack unless change 22 took it (D1).

## 2. The wording (apps/lingua-extension)

- [ ] 2.1 `test/word-card-en-es.spec.ts` and its snapshot: every probe of the committed golden rendered with the interface in Spanish; `lingua-pack-update` re-blesses the snapshot (D2).
- [ ] 2.2 `src/i18n/es/grammar.ts`: the English-studied tables corrected on the real readings; `word-grammar-es.spec.ts` moves with them (D3).
- [ ] 2.3 `selection-card.ts`: the row cut per D4.

## 3. Dogfood

- [ ] 3.1 A local build listing en-es beside the French pairs, the interface in Spanish, signed out or on a test account, twenty pages read; findings fixed or listed (D5).

## 4. Gates, review and docs

- [ ] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-wasm`; in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (`word-grammar.spec.ts` unchanged), `yarn build`, `yarn check:variants`.
- [ ] 4.2 [manual] The owner reviews the Spanish lines the snapshot pins (M9).
- [ ] 4.3 `openspec validate add-lingua-spanish-card-wording --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-spanish-card-wording` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 24 is marked done in `docs/lingua/language-matrix-programme.md`.
