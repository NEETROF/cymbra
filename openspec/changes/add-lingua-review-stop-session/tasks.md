# Tasks

## 1. Ending a session

- [ ] 1.1 `ReviewController.end()`: drop the current card and run the end of the session — the summary so far, then « Encore 10 » or practice measured as after the last answer — and mark the view `ended` (D2, D3); a no-op outside a session. Verify with `test/session.spec.ts` cases: ending with answered cards (summary counts), unseen cards left due and unanswered, a missed card waiting to come back not shown again, a revealed card left unanswered, practice, and ending before any answer.
- [ ] 1.2 The link: `review.end` and `review.endLabel` in `src/i18n/{fr,en,es}/review.ts` (« Terminer » / "End" / « Terminar », « Terminer la séance » / "End the session" / « Terminar la sesión »); the card head's link on every card of a session, before and after the reveal, in practice and on a card that came back; none at the start or the end; the end of a session ended before any answer titled without counts (D1, D3, D5); its style in `src/styles/review.css` (no padding, pushed right, no transition). Verify with `test/view.spec.ts` (link only during a session, fr/en/es copy and accessible name, the end without counts) and `test/i18n.spec.ts`.
- [ ] 1.3 The review page: the `end` action runs `end()` as an action that changed the deck — the backup saved and the counts refreshed with no session under way (D4). Verify with `test/review-page.spec.ts`: ending shows the end, « Encore 10 » offered while cards remain due and starting a new session, practice ended the same way.

## 2. Integration and dogfood

- [ ] 2.1 Gates in `apps/lingua-extension`: `yarn typecheck && yarn lint && yarn format:check && yarn test` (coverage ≥ 80 %) `&& yarn build && yarn check:variants`. Verify all pass.
- [ ] 2.2 Dogfood: end a session after a few cards in the side panel (Chrome), in the drawer (Safari) and in the Cymbra Lingua app on an iPhone; each shows the end with « Encore 10 », and the cards not shown come up in the next session. Verify by noting it in the pull request.
