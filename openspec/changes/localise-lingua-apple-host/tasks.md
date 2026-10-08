# Tasks

## 1. The languages (apps/lingua-apple)

- [ ] 1.1 A Run Script phase on both app targets writes `CFBundleLocalizations` and `CFBundleDevelopmentRegion` from the copied extension's `_locales` (D1); its script test in `lingua-apple-build` (D4).

## 2. The activation page

- [ ] 2.1 `Resources/copy.js` (fr byte for byte, en and es drafts), `Main.html`'s `data-copy` keys and `data-copy-pending`, `Script.js`'s `show(…, language)` filling and setting `lang` (D2).
- [ ] 2.2 `ViewController.swift` passes the bundle's preferred localisation mapped to fr, en or es (D1, D2). Test: jsdom over `copy.js` and `show()` (D4).

## 3. The sign-in sheet

- [ ] 3.1 `apps/lingua-extension`: `hostAppSignInUrl(provider, language)`, the background passes the interface language (D3). Test.
- [ ] 3.2 `LinguaSignIn`: `SignInLink.language(from:)`, `SignInCopy(language:)` with fr/en/es tables; `SignInView`, `SceneDelegate`, `AppDelegate` pass the language (D3). `swift test` (D4).

## 4. Gates, review and docs

- [ ] 4.1 `swift test` in `LinguaSignIn`; the `lingua-apple-build` workflow green (unsigned iOS simulator and macOS builds); in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 4.2 `apps/lingua-apple/README.md`: the languages rule and the sheet's `lang`.
- [ ] 4.3 [manual] The owner reviews the English and Spanish copy of the page and the sheet (M9).
- [ ] 4.4 `openspec validate localise-lingua-apple-host --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-apple-host` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 28 is marked done in `docs/lingua/language-matrix-programme.md`.
