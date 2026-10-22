# Tasks

## 1. The messages and the check

- [ ] 1.1 `apps/lingua-extension/_locales/it/messages.json` (D1): `extensionDescription`, `commandCaptureSelection`, `commandToggleDrawer`, `commandSidePanel`, the drafts or the reviewer agent's words, the commands' nouns those of `src/i18n/it/settings.ts`'s shortcut lines; each with an English `description` for translators, the description's naming en-it as the pair it follows and changes 50 and 54 as those that rewrite it.
- [ ] 1.2 `tool/check_version.mjs` `LANGUAGE_NAMES.it` as `src/i18n/it/languages.ts` names the four languages; `test/check-version.spec.ts`'s `catalogue()` gains `it` (D2).

## 2. Tests

- [ ] 2.1 `test/manifests.spec.ts` (D3): for each target, en-it listed beside today's pairs — `_locales/it` carried with the other shipped natives' folders, `default_locale` `en`, every `__MSG_` reference resolving; today's list — no `_locales/it`.
- [ ] 2.2 `test/check-version.spec.ts` (D3): en-it listed — the committed Italian description passes `summaryProblems`; a description naming « francese » fails, naming the Italian description and French; the Italian description within 112 characters.

## 3. Review and gates

- [ ] 3.1 [agent] The Italian reviewer agent's pass over the four messages, its findings applied or answered in the pull request (D4, I2).
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; the three packages byte for byte what they were (D5).
- [ ] 3.3 `openspec validate localise-lingua-manifest-italian --strict`; the archive order; `docs/lingua/italian-programme.md` row 39's status.
