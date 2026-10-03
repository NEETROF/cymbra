## 1. Naming the languages

- [ ] 1.1 `src/analyzer/language-labels.ts` (design D1), used by:
  - Réglages: the level titles and the « Aucune voix … » note;
  - the statistics' level heading;
  - the popup: its « Pas de texte … détecté » note and its level prompt and line;
  - onboarding.

  Static page copy becomes neutral or filled at load.
- [ ] 1.2 `test/lint-language-labels.spec.ts`: no « anglais » in any source or page outside the module.

## 2. Choosing the languages

- [ ] 2.1 `src/reading/studied-languages-view.ts` (design D2), mounted by `mountSettings`. Spec, with en-fr and es-fr shipped:
  - ticking and unticking;
  - the last box disabled;
  - the profile saved and persisted;
  - a profile language the package does not ship kept;
  - hidden with en-fr alone.
- [ ] 2.2 Onboarding: the boxes when several languages ship, then a level row per chosen language (design D5).

## 3. A level and a voice per language

- [ ] 3.1 `mountSettings`: a level block per accepted language, rebuilt when the list changes (design D3); `needsLevelChoice` filters by language. Specs:
  - two blocks with en and es;
  - a Spanish decision leaves the English choice open;
  - one block, « Niveau d'anglais », for every reader today.
- [ ] 3.2 Voice per language (design D4):
  - `loadVoice` / `saveVoice(area, lang, uri)`, the old string read as English;
  - `SpeechSettings.voices`, and the speaker picks its language's voice;
  - the settings save for `speaker.lang`.

  Specs: two languages, two voices; an old value as English.

## 4. Gates

- [ ] 4.1 In `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn test` (coverage gate), `yarn format:check`, `yarn build` and `yarn check:variants`.
- [ ] 4.2 `openspec validate add-lingua-language-choice --strict` passes. In `docs/lingua/spanish-programme.md`, change 13 is marked done.
