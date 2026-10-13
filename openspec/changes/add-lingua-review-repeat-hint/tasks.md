# Tasks

## 1. The count and the note
- [ ] 1.1 `ReviewController`: count each card's showings in the session, keyed by language and headword, in `next()` only, reset by `start`; the view model's `showing` (D1). Verify with `test/session.spec.ts` cases: a missed card's second and third showing, a reveal not counted, a new session starting afresh, the same headword in two languages.
- [ ] 1.2 `review.repeat(showing)` in `src/i18n/{fr,en,es}/review.ts` (D3), and the card head showing it in place of the remaining count from the second showing on (D2). Verify with `test/view.spec.ts` (first showing unchanged, second and third say it) and `test/i18n.spec.ts` (the drafts take the slot).

## 2. Integration and dogfood
- [ ] 2.1 Gates: `yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants` in `apps/lingua-extension`. Verify all pass.
- [ ] 2.2 Dogfood: miss a card in the side panel (Chrome), in the drawer (Safari) and in the Cymbra Lingua app on an iPhone; each says « À revoir · 2ᵉ passage » when it comes back. Verify by noting it in the pull request.
