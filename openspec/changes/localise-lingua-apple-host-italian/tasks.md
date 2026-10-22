# Tasks

## 1. Swift

- [ ] 1.1 `LinguaSignIn/Sources/LinguaSignIn/SignInLanguage.swift`: `it` appended; the doc comments (D1).
- [ ] 1.2 `SignInFlow.swift`: `SignInLink.language(from:)` reads `lang=it`; `SignInCopy`'s `.it` case and its 11-entry table, the drafts or the reviewer agent's words; the comments naming the languages and the drafts (D2).
- [ ] 1.3 `NativeMessage.swift`: `interface.language` = `it` accepted and stored (D2).
- [ ] 1.4 `SignInLanguageTests`, `SignInFlowTests` and `NativeMessage`'s tests: Italian offered, preferred and stored; `lang=it`; the Italian draft not the French; « tu »; `swift test` (D5).

## 2. The activation page and the build

- [ ] 2.1 `Shared (App)/Resources/copy.js`: the `it` table of 15 keys, naming English; the header naming each native's pairs (D3).
- [ ] 2.2 `tool/app_localizations.sh`: Italian in the warning and order lists; `tool/test_app_localizations.sh`: en-it → `["fr","en","es","it"]`, region `en`, no warning (D4).
- [ ] 2.3 `apps/lingua-extension/test/apple-activation-page.spec.ts`: Italian among the languages; the checks on studied languages and step 3 apply to the natives the listed pairs have; the expected names from change 33's catalogue. `test/native-signin.spec.ts`: the `it` case (D5).
- [ ] 2.4 `README.md`: `lang=fr|en|es|it` and the languages the app holds.

## 3. Review, release, gates

- [ ] 3.1 [agent] The Italian reviewer agent's pass over `copy.js`'s and `SignInCopy`'s Italian (D6), its findings applied or answered in the pull request.
- [ ] 3.2 [owner] The host-app release carrying this change and change 32, live before change 45 (D7); the build declares `[fr, en, es]`.
- [ ] 3.3 `swift test`; `bash tool/test_app_localizations.sh`; in `apps/lingua-extension`: `yarn gen:pack`, `yarn lint`, `yarn typecheck`, `yarn test`; `openspec validate localise-lingua-apple-host-italian --strict`; the archive order; `docs/lingua/italian-programme.md` row 40's status.
