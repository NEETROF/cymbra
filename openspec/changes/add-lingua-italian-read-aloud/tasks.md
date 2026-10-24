# Tasks

## 1. The voices (apps/lingua-extension)

- [x] 1.0 On `main`'s code, before `speech.ts` is edited: `fr` joins `LANGUAGES` in `test/voice-ranking.spec.ts` (84 rankings), re-blessed and committed on its own (D5).
- [x] 1.1 `src/reading/speech.ts`: `PREFERRED_REGIONS.it = ["it"]` (D1); `THREE_LETTER_REGIONS.ita = "it"` (D2); their doc comments name this change.
- [x] 1.2 `test/speech.spec.ts`: Italian's ranking over the seven captures and a synthetic `it-CH`-first list; `ita-ITA-default` placed in Italy and its region named in the three interface languages; the voice-ranking snapshot passes as committed after the edit, Italian added in a separate commit whose diff only adds `it` blocks (D5).

## 2. The catalogue

- [x] 2.1 `src/i18n/{fr,en,es}/languages.ts`: `italian` with its capitalised name, forms (Spanish `of` « de italiano »), `windowsVoice` and Italian `preview` (D3); `src/analyzer/language-labels.ts`: `NamedLanguage` gains `it`, `NAMED.it`, `words()` maps `it`; `StudiedLanguage` and `ownNames` untouched.
- [x] 2.2 `test/i18n.spec.ts` (`SAME_EVERYWHERE` gains the preview), `test/language-labels.spec.ts` (the voice messages for `it`), the labels lint's `NAMES` (the Italian names); the Italian strings read by the dedicated Italian reviewer (I2).

## 3. The card

- [x] 3.1 `src/reading/session.ts` `pageHit`: the text of the token starting where the hit ends, from `blockTokensAt`, added to `PageHit` and carried by the card content (`selection-card.ts`) to `heardWord` (D4.1).
- [x] 3.2 `src/reading/wordpopup.ts`: `heardWord` reads Italian by D4 — the glued word (its last word replaced by the next token's text where it differs only by an accent and an apostrophe; an elided piece before a number glued to it, « l'8 »), a split preposition's piece as written, a truncation of change 6's list as written before `ELIDED`, otherwise French's rule, then a one-vowel word (accented or a capital, its trailing apostrophe aside, not « é » nor its capital « É ») followed by a full stop; the label without the stop; the doc comment names this change.
- [x] 3.3 Tests: each scenario of *An Italian piece is heard as the page writes it*; the French, English and Spanish cases unchanged; `yarn test` passes.

## 4. Gates

- [ ] 4.1 [manual] The on-device checklist of D7 recorded in the programme's row 20, for change 20's dogfood; not run here.
- [x] 4.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; no `-u` on any snapshot but the commits of task 1.0 and of Italian's ranking blocks (1.2).
- [x] 4.3 `openspec validate add-lingua-italian-read-aloud --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-read-aloud` names its prerequisites; `docs/lingua/italian-programme.md` row 15's status names this change's pull requests.
