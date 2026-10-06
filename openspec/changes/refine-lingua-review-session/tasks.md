# Tasks

## 1. The core session

- [x] 1.1 `lingua-core` (`decks/review.rs`): `start_for` orders every queue — single-language included — by retrievability at `now`, lowest first (ties: due date, language, lemma), never-reviewed cards in capture order (design D1). Verify with unit tests: a single-language queue no longer alphabetical; a mixed queue by retrievability; the language filter unchanged.
- [x] 1.2 Session options: a cap of distinct cards, the day's allowance of new cards and the day start; new cards at the 4th and 8th places when both kinds are due; « introduced today » = `reps == 1 && last_review >= day_start` (D2). Verify with unit tests for the spec scenarios « Forty cards due », « Continuing », « New words among reviews », « A heavy reading day » and « The next day », and that no option means no cap and no allowance.
- [x] 1.3 In-session return of a missed card (D3): the first « Pas su » grades `again` and re-queues three places later; later answers do not touch FSRS; « Su » takes it out as recovered; three asks at most. `grade` reports whether FSRS was updated. Verify with unit tests for « A word recovered », « Missed three times » and « One review counted » (one FSRS update per card per session).
- [x] 1.4 `ReviewSession::ignore` (D5): `Ignored` stamped in milliseconds, card retired as `Deck::retire` does, session advances. Verify with unit tests: status and `updated_at` set, card kept, no longer due; an ignored status exported as a status op.
- [x] 1.5 Session summary (D8): graded, recovered, scheduled ≥ 30 days away, marked known, hidden. Verify with a unit test on « End of a session »; then `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass.

## 2. The WASM engine

- [x] 2.1 `lingua-wasm`: `startReview(now, languages?, limit?, newPerDay?, dayStart?)`, `reviewGrade` returning whether FSRS was updated, `reviewIgnore(now)` and `reviewSummary()` (D10). Verify with native tests in `crates/lingua-wasm/tests/deck.rs`: options absent keep today's walk length, the cap, the allowance, an ignored card, a summary.

## 3. The port

- [x] 3.1 `analyzer/port.ts`, `analyzer/engine.ts` (WASM interface + adapter), `analyzer/messaging-port.ts` and the background's dispatch, and the fake in `test/helpers.ts`: the new options and methods, `reviewGrade`'s result. Verify with `yarn typecheck` and `test/messaging.spec.ts` round-tripping `reviewIgnore`, `reviewSummary` and the options.

## 4. The review widget and page

- [x] 4.1 Preference: the « Nouveaux mots par jour » key in `state/storage.ts` (steps 5, 10, 20; absent = 10) with its accessors, kept out of the engine backup. Verify with a storage spec: default, each step, an unknown value read as 10.
- [x] 4.2 `review/session.ts`: `ReviewController` starts sessions with a cap of 10, the allowance and the local day start; records a daily review only when the answer updated FSRS; gains `ignore()` and the summary. Verify with specs on the controller: options passed, one review recorded for a recovered card, ignore advancing.
- [x] 4.3 `review/view.ts` (D6): source label, sentence with the encountered form marked (text nodes only), headword when the form is absent or there is no sentence, an answer space present before the reveal, one action zone (« Afficher la réponse », then « Pas su » · « Su »), « Je connais · Ne plus me le montrer » links, the end view with « Encore 10 » only when cards remain due. Verify with view specs for the scenarios « A word met in a book », « A card without a sentence », « Remembered », « Forgotten », « End of a session » and « Nothing left to review », including that no markup from a sentence is ever parsed.
- [x] 4.4 Keyboard (D7): Space/Enter reveal, ←/→ answer, handled on the review root only; the focus moves into the card at the start. Verify with specs: keys answer while the review has the focus; keys dispatched on the page's document do nothing to the review.
- [x] 4.5 `styles/review.css` (D6, D7): the review root as a container; bottom bar below ~600 px with `env(safe-area-inset-bottom)` and a minimum padding; side bands when landscape with little height; a centred column from ~600 px; font sizes as `calc(<px> * var(--cymbra-lingua-ui-scale, 1))`; colours from `tokens.css` only; no transition. Verify with `yarn lint` (tokens and scale lints) and `yarn build`, then a look in the side panel and the drawer at three widths.
- [x] 4.6 `review/review-page.ts`: drop backup, restore and the sources; keep the language filter, the due count and the card; a restore made in this page ends its session. Verify with `test/review-page.spec.ts`: « The review page » scenario, and a restore during a session showing the idle state.
- [x] 4.7 Update the « Review (side panel + drawer) » section of `apps/lingua-extension/README.md` (two answers, the hide link, sessions of 10, the daily allowance, backup now in Réglages › Données). Verify by reading the section against the specs.

## 5. Réglages

- [x] 5.1 `reading/settings-view.ts`: a « Rythme de révision » block with « Nouveaux mots par jour » (5 / 10 / 20) in the « Langue » tab; « Fichier de sauvegarde » (Sauvegarder, Restaurer) and « Sources et confidentialité » in the « Données » tab. Verify with `test/settings-view.spec.ts` (the setting writes its key; backup downloads a versioned file; restore re-imports it; the sources show the pack's licences) and `test/lint-settings-hosts.spec.ts` passing with every host.

## 6. Integration and dogfood

- [x] 6.1 Gates: in `apps/lingua-extension`, `yarn typecheck && yarn lint && yarn test && yarn format:check && yarn build` for every target, then grep `dist-*/` for the new widget; at the root, `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`. Verify all pass.
- [ ] 6.2 Dogfood on the e-ink reader (Firefox for Android, e-ink colours): a session of 10 opened from a book, a missed word coming back, « Ne plus me le montrer », the end view; revealing repaints only the answer space. Verify by noting the result in the pull request.
- [ ] 6.3 Dogfood on an iPhone (Safari) upright and sideways, and in the Chrome side panel with the keyboard. Verify by noting the result in the pull request, including the bottom bar against the home indicator.
