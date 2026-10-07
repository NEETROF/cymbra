# Tasks

## 1. The review page and its card

- [ ] 1.1 `review/review-page.ts` (D2, D3, D4): an optional `translator` (`TranslatorSource`) in `ReviewPageOptions`; when a card without a gloss becomes the current card, ask its translation once, bounded by `TRANSLATION_WAIT_MS`, tied to that card so an answer landing after it was answered is dropped; an expression its sentence does not hold asked alone, a single word in that case not asked; the route of the review's language warmed when a session starts. Verify with `test/review-page.spec.ts`, the translator a test double injected through the options: asked on arrival with the card's language, sentence and span; not asked for a card with a gloss, nor without a translator, nor for a single word its sentence does not hold; an expression asked alone; a late answer for an answered card shown nowhere; warmed at the start.
- [ ] 1.2 `review/view.ts` (D5): once revealed, the translation in the answer space — « Dans votre phrase — traduction automatique », or « Traduction automatique » for an expression translated alone, then the sentence with each mark as a `<mark>`, from text nodes, a span out of range skipped; « Traduction en cours… » while it is on its way after the reveal; the no-translation note only when none came; nothing of it before the reveal. Verify with `test/view.spec.ts` for the scenarios « An expression in review », « Ready at the reveal », « Still on its way », « A sentence that does not hold the expression », and a translation holding markup shown as text.
- [ ] 1.3 Hosts (D4): `reading/drawer.ts` takes the reading session's translator and hands it to `mountReview`; `sidepanel/sidepanel.ts` passes `createTranslatorPort()`. Verify with `test/lint-translator-placement.spec.ts` passing unchanged and the drawer and side panel specs.
- [ ] 1.4 `styles/review.css` (D5): the translation block from `tokens.css`, sizes from `--cymbra-lingua-ui-scale`, no transition. Verify with `yarn lint` (tokens and scale lints).

## 2. Documentation

- [ ] 2.1 The review section of `apps/lingua-extension/README.md`: a card without a gloss is translated at review where a model is ready, never stored. Verify by reading it against the spec.

## 3. Integration and dogfood

- [ ] 3.1 Gates in `apps/lingua-extension`: `yarn typecheck && yarn lint && yarn format:check && yarn test && yarn build && yarn check:variants`. Verify all pass.
- [ ] 3.2 Dogfood in Chrome's side panel with extended translation on: an expression without a gloss, a Spanish card (through English), a card with a gloss (no translation), then extended translation off (no line). Verify by noting the results in the pull request.
- [ ] 3.3 Dogfood on the e-ink reader (Firefox for Android): after the first card, the translation is part of the answer at the reveal, in one repaint. Verify by noting the result in the pull request.
