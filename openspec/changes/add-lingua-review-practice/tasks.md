# Tasks

## 1. The core
- [x] 1.1 `SessionOptions.practice` and the practice queue (D1); `grade` changing no card in a practice session, returning true on a first answer (D2); the snapshot keeping it. Verify with `cargo test -p lingua-core` cases: queue (reviewed, non-retired, not due, lowest recall first, no new card, languages, limit), a recalled and a missed answer leaving the card's `ReviewState` and `updated_at` unchanged, the requeue, the summary, the snapshot round trip and an old snapshot resuming as a review.
- [x] 1.2 `crates/lingua-wasm` `startReview(…, practice)` (D3). Verify with `cargo test -p lingua-wasm` and `cargo clippy --workspace --all-targets -- -D warnings`.

## 2. The extension
- [x] 2.1 `ReviewOptions.practice` through `WasmAnalyzerPort` and the messaging port (D3); `ReviewController` `practiceAvailable` / `practice` and a practice start (D4). Verify with `test/session.spec.ts` and the fake port.
- [x] 2.2 The view: « S'entraîner quand même » under nothing due and at an end with nothing due, « Entraînement · N carte(s) » on a practice card (D4); copy in `src/i18n/{fr,en,es}/review.ts`. Verify with `test/view.spec.ts` and `test/i18n.spec.ts`.
- [ ] 2.3 Gates: `cargo fmt --all --check`, clippy, `cargo llvm-cov` ≥ 80 %; the extension's `yarn gen:wasm && yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants`. Verify all pass.

## 3. Dogfood
- [ ] 3.1 On the iPad (nothing due in English): practise in the Cymbra Lingua app and in Safari's drawer; the due dates are unchanged afterwards and the day's reviews grew. Verify by noting it in the pull request.
