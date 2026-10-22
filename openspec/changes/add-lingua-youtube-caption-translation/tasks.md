Estimate: 3–4 ideal days — agent ~3 days (groups 1, 2, 4), owner ~0.5 day (group 3). Starts once
`add-lingua-youtube-captions` is implemented, and only if its probe notes (task 1.2 step 2, `tlang`)
show YouTube's translation still answers in at least one browser.

## 1. The translation line (~2.5 d)

- [ ] 1.1 The line under the caption line (beside it in `apps/lingua-extension/src/youtube/`): follows the default, theatre and full-screen layouts; hidden during advertisements; a control beside the caption line and « Afficher la traduction des sous-titres » in Réglages (the one builder `src/reading/settings-view.ts`, copy in `src/reading/settings-copy.ts`); per-device preference `cymbra-lingua-caption-translation`, hidden by default; tests
- [ ] 1.2 No engine (`createTranslatorPort(...)("en") === null`, `src/translate/create-port.ts`): when the line is first shown on a video whose track YouTube translates, replay the caption request with `tlang=<gloss language>` once; parse it with the caption mode's json3 parser; show it per cue under « Traduction automatique YouTube »; never mark a word; never on another site; never in degraded mode; « Pas de traduction pour ces sous-titres » for a track YouTube does not translate; tests; a track YouTube does not translate, or a `tlang` request it does not answer, shows « Pas de traduction pour ces sous-titres », with a test
- [ ] 1.3 Engine ready: keep the next three sentences ahead of the playhead translated, one request at a time, waiting while a card's request is in flight; keep answers for the session; restart the window after a forward seek; switch source when the setting or the model state changes; tests with a mocked `TranslatorPort`
- [ ] 1.4 Degraded mode: the engine translates the line on screen when it appears; no request to YouTube for a translation; tests
- [ ] 1.5 YouTube's translation is never stored, synchronised or sent: a card captured while it is shown keeps what a page card keeps; test on card creation

## 2. Copy, disclosure and review (~0.5 d)

- [ ] 2.1 Copy in French, English and Spanish, and in Italian once change 33's `src/i18n/it/` exists — whichever of the two lands second writes it, read by change 33's Italian reviewer agent (`src/i18n/`): the control, the preference, « Traduction automatique YouTube », « Pas de traduction pour ces sous-titres »
- [ ] 2.2 Lingua annex of the privacy policy in French, English and Spanish (`apps/site/src/pages/confidentialite.md`, `en/privacy.md`, `es/privacidad.md`), and in Italian on change 41's page (`it/privacy.md`, its `/it/privacy/`) where it exists, by its mirror rule, read by change 33's Italian reviewer agent: without a translation engine, the translation shown under the captions is YouTube's, fetched from YouTube only when the reader shows it
- [ ] 2.3 `apps/lingua-extension/REVIEWERS.md`: the `tlang` replay — same request, same video, one extra parameter, only while the line is shown with no engine

## 3. Manual pass (owner, ~0.5 d)

- [ ] 3.1 In Chrome, Firefox and Safari on macOS: line hidden by default and the choice remembered; no engine → YouTube's translation, labelled, in Spanish for a Spanish-native reader, and in Italian for an Italian-native one once en-it ships (if it has not shipped when this is implemented, that check is change 45's dogfood's); engine ready → each line translated as it appears, and a card opened mid-video answers without delay; model downloading → YouTube's until ready; degraded mode → no YouTube translation. Record whether three sentences ahead is enough (design open question 2)

## 4. Gates

- [ ] 4.1 `openspec validate add-lingua-youtube-caption-translation --strict`, and `scripts/openspec_archive_order.py add-lingua-youtube-caption-translation` names only the changes in `archiveAfter` still open
- [ ] 4.2 Extension lint, typecheck and tests green; coverage ≥ 80 %
