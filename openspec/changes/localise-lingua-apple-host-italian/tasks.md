# Tasks

## 1. Swift

- [ ] 1.1 `LinguaSignIn/Sources/LinguaSignIn/SignInLanguage.swift`: `it` appended; the doc comments (D1).
- [ ] 1.2 `SignInFlow.swift`: `SignInCopy`'s `.it` case and its 11-entry table (`SignInLink.language(from:)` reads `lang=it` once 1.1 lands; comments only), the drafts or the reviewer agent's words; the comments naming the languages and the drafts (D2).
- [ ] 1.3 `NativeMessage.swift`: accepts and stores `it` once 1.1 lands; its comments and tests only (D2).
- [ ] 1.4 `SignInLanguageTests` (Italian offered, preferred, stored; `NativeMessage`'s case; a `[fr, en, es]` bundle never showing `.it`), `SignInFlowTests` (`lang=it`; the Italian draft not the French; « tu »), `AppSessionStoreTests` (Italian's `appLede` and `emailButton`); `swift test` (D5).

## 2. The activation page and the build

- [ ] 2.1 `Shared (App)/Resources/copy.js`: the `it` table of 15 keys, naming English, no longer than the Spanish page; the header naming each native's pairs (D3).
- [ ] 2.2 `tool/app_localizations.sh`: Italian in the warning and order lists and the comment naming fr, en, es; `tool/test_app_localizations.sh`: en-it → `["fr","en","es","it"]`, region `en`, no warning (D4).
- [ ] 2.3 `apps/lingua-extension/test/apple-activation-page.spec.ts` (D5): `LANGUAGES` with `it`; the no-French-copy check, the fill loop and a « tu » check for Italian; the naming checks against the shipped pairs plus en-it; the second-language step-3 check only for natives with two listed pairs or more, a one-pair native held to its language's level and no Settings tab. `test/native-signin.spec.ts`: the teller sends no `it` while `INTERFACE_LANGUAGES` lacks it.
- [ ] 2.4 `README.md`: `lang=fr|en|es|it` and the languages the app holds.

## 3. Review, release, gates

- [ ] 3.1 [agent] The Italian reviewer agent's pass over `copy.js`'s and `SignInCopy`'s Italian (D6), its findings applied or answered in the pull request.
- [ ] 3.2 [owner] The host-app release carrying this change and change 32, live before change 45 (D7); the build declares `[fr, en, es]`.
- [ ] 3.3 `swift test`; `bash tool/test_app_localizations.sh`; in `apps/lingua-extension`: `yarn gen:pack`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`; `openspec validate localise-lingua-apple-host-italian --strict`; the archive order; `docs/lingua/italian-programme.md` row 40's status.
