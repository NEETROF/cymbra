# Tasks

## 1. The messages and the check

- [ ] 1.1 `apps/lingua-extension/_locales/it/messages.json` (D1): `extensionDescription`, `commandCaptureSelection`, `commandToggleDrawer`, `commandSidePanel`, the drafts or the reviewer agent's words, each command's description containing the words of `src/i18n/it/settings.ts`'s matching shortcut line; each with an English `description` for translators naming the pairs it follows (en-it; fr-it and es-it to come), not programme change numbers.
- [ ] 1.2 `tool/check_version.mjs`: `LANGUAGE_NAMES.it` as `src/i18n/it/languages.ts` names the four languages, and the comment naming `src/i18n/{fr,en,es}` gains `it`; `test/check-version.spec.ts`: a fourth expected row `it: catalogue(itLanguages)`, importing `../src/i18n/it/languages` (D2).

## 2. Tests

- [ ] 2.1 `test/manifests.spec.ts` (D3): for each target, en-it listed beside today's pairs — `_locales/it` carried with the other shipped natives' folders, `default_locale` `en`, every `__MSG_` reference resolving; today's list — no `_locales/it`; the committed folders' expected set `["en", "es", "fr", "it"]`, its title with it.
- [ ] 2.2 `test/check-version.spec.ts` (D3): the committed set gains `it`; en-it listed — the committed Italian description passes `summaryProblems`; a description naming « francese » fails, naming the Italian description and « Francese »; the Italian line of the per-description test names « inglese », not « italiano »; the Italian description within 112 characters.
- [ ] 2.3 A test that, for fr, en, es and it, each Réglages shortcut line's text after the keys is contained, case-insensitively, in the matching `_locales/<language>` command's message, naming the command when it is not (D1, D3).

## 3. Review and gates

- [ ] 3.1 [agent] The Italian reviewer agent's pass over the four messages, its findings applied or answered in the pull request (D4, I2): « percentuale reale », « senza tracciamento », « riquadro laterale », « insieme alla frase ».
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn gen:pack`, `yarn build`, `yarn check:variants`, `yarn check:version`; the three packages byte for byte what they were before this change (D5).
- [ ] 3.3 `openspec validate localise-lingua-manifest-italian --strict`; `python3 scripts/openspec_archive_order.py localise-lingua-manifest-italian` (exit 10 until the five changes of `.openspec.yaml` archive); `docs/lingua/italian-programme.md` row 39's status.
