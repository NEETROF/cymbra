# Tasks

## 1. The languages (apps/lingua-apple)

- [x] 1.1 `tool/app_localizations.sh` and its Run Script phase, last on both app targets and after the copy on both extension targets, writing `CFBundleLocalizations` and `CFBundleDevelopmentRegion` from the natives of `dist-safari`'s packs only when they differ (D1); its script test in `lingua-apple-build` (D4).

## 2. The activation page

- [x] 2.1 `Resources/copy.js` (fr byte for byte, en and es drafts naming what their natives study), `Main.html`'s `data-copy` keys, `copy.js` filling on `DOMContentLoaded` and setting `lang`; `copy.js` in the project and both app targets' Copy Bundle Resources (D2).
- [x] 2.2 `ViewController.swift` injects the language at document start: the extension's interface language from the App Group once the extension has run, else the bundle's preferred localisation mapped to fr, en or es; a `NativeMessage` case taking a `SignInLanguage`, sent by the background (Safari only) at start and on each change, and the native handler writing it to the App Group (D1, D2). Test: `apps/lingua-extension/test/apple-activation-page.spec.ts` and the `ext` filter (D4).

## 3. The sign-in sheet

- [x] 3.1 `apps/lingua-extension`: `hostAppSignInUrl(provider, language)`, the background passes the interface language (D3). Test.
- [x] 3.2 `LinguaSignIn`: `SignInLink.language(from:)`, `SignInCopy(language:)` with fr/en/es tables; `SignInView`, `SceneDelegate`, `AppDelegate` pass the language (D3). `swift test` (D4).

## 4. Gates, review and docs

- [x] 4.1 `swift test` in `LinguaSignIn`; local unsigned iOS simulator and macOS builds pass (the `lingua-apple-build` workflow runs on the pull request); in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [x] 4.2 `apps/lingua-apple/README.md`: the languages rule and the sheet's `lang`.
- [ ] 4.5 [manual] A simulator with a local build carrying es-en, before the extension has run: device in English → English page, German → English, French → French; `preferredLocalizations` and Safari's `_locales` choice observed. And a French-only build (today's packs) on a device in English: the page French, and the sheet French for `cymbra-lingua://signin?provider=apple&lang=en` and for a link naming no language.
  - For the owner: the live App Store listing (id 6813053825) may already show English, because the shipped app's `CFBundleDevelopmentRegion` has always been `en` (Xcode writes the project's development language over the plist's `fr`). If it does, setting the project's `developmentRegion` / `DEVELOPMENT_LANGUAGE` to `fr` cures it — the owner's call, not made here.
- [ ] 4.3 [manual] The owner reviews the English and Spanish copy of the page and the sheet (M9).
  - Points raised in review, left as drafted: Spanish « Ajustes » is Spain's label for Settings (Latin America reads « Configuración »); "address bar menu" where Apple says "Page Menu"; "Safari's settings" where Apple says "Safari Settings"; the sheet's "find your words" for *retrouver tes mots*.
  - The live App Store listing (id 6813053825) may already show English, because the shipped app's development region has always been `en`; if so, setting the project's `developmentRegion` / `DEVELOPMENT_LANGUAGE` to `fr` cures it — the owner's call (see 4.5).
- [x] 4.4 `openspec validate localise-lingua-apple-host --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-apple-host` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 28 is marked done in `docs/lingua/language-matrix-programme.md`.
