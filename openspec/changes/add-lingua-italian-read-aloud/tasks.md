# Tasks

## 1. The voices (apps/lingua-extension)

- [ ] 1.1 `src/reading/speech.ts`: `PREFERRED_REGIONS.it = ["it"]` (D1); `THREE_LETTER_REGIONS.ita = "it"` (D2); their doc comments name this change.
- [ ] 1.2 `test/speech.spec.ts`: Italian's ranking over the seven captures and a synthetic `it-CH`-first list; `ita-ITA-default` placed in Italy and named in the three interface languages; the voice-ranking snapshot gains Italian's block and every other block passes unchanged (D5).

## 2. The catalogue

- [ ] 2.1 `src/i18n/{fr,en,es}/languages.ts`: `italian` with its name, forms, `windowsVoice` and Italian `preview` (D3); the voice block and the level titles read it through the speaker's language; `StudiedLanguage` and `ownNames` untouched; the i18n checks pass.

## 3. The card

- [ ] 3.1 `src/reading/wordpopup.ts`: `heardWord` reads Italian by D4 — the glued word (its leaning word's accented text where change 10 read it), a split preposition's piece as written, a truncation as written, a word read for an accent as its text, a single accented vowel followed by a full stop, else the form seen — a truncation and an accent spelling told apart before `ELIDED` is read; the doc comment names this change.
- [ ] 3.2 Tests: each scenario of *An Italian piece is heard as the page writes it*; the French, English and Spanish cases unchanged; `yarn test` passes.

## 4. Gates

- [ ] 4.1 The on-device checklist of D7 written into change 20's tasks (its dogfood), not run here.
- [ ] 4.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; no `-u` on any snapshot but the voice-ranking's Italian block.
- [ ] 4.3 `openspec validate add-lingua-italian-read-aloud --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-read-aloud` names its prerequisites; `docs/lingua/italian-programme.md` row 15's status names this change's pull requests.
