# Tasks

## 1. The count and the note
- [x] 1.1 `ReviewController`: count each card's showings in the session, keyed by language and headword, in `next()` only, reset by `start`; the view model's `showing` (D1). Verify with `test/session.spec.ts` cases: a missed card's second and third showing, a reveal not counted, a new session starting afresh, the same headword in two languages.
- [x] 1.2 `review.repeat(showing)` in `src/i18n/{fr,en,es}/review.ts` (D3), and the card head showing it in place of the remaining count from the second showing on (D2). Verify with `test/view.spec.ts` (first showing unchanged, second and third say it) and `test/i18n.spec.ts` (the drafts take the slot).
- [x] 1.3 The core: a missed card comes back only when another card remains after it (D4); the repeat note wins over practice's (D5). Verify with `cargo test -p lingua-core` (`spec_the_last_card_missed_is_not_shown_again_at_once`), the fake port aligned, `test/session.spec.ts` and `test/view.spec.ts`.

## 2. Integration and dogfood
- [x] 2.1 Gates: `cargo fmt --all --check`, clippy, `cargo llvm-cov` ≥ 80 %; `yarn gen:wasm && yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants` in `apps/lingua-extension`. Verify all pass.
- [x] 2.2 Dogfood: miss a card in the side panel (Chrome), in the drawer (Safari) and in the Cymbra Lingua app on an iPhone; each says « À revoir · 2ᵉ passage » when it comes back. Verify by noting it in the pull request.
