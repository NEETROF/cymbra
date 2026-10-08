# Design — localise-lingua-apple-host

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `Shared (App)/Resources/Base.lproj/Main.html` | the activation page, `<html lang="fr">`: icon alt, title, lede (« Lis le web en anglais… »), the iOS steps, the macOS states and button « Activer dans Safari », notes — 12 texts |
| `Shared (App)/Resources/Script.js` | `show(platform, enabled, useSettingsInsteadOfPreferences)`; 3 French strings for macOS before 13 (« préférences ») |
| `Shared (App)/ViewController.swift` | loads `Main.html` from the bundle; on `didFinish` calls `show('ios')`, or `show('mac', enabled, …)` from `SFSafariExtensionManager` |
| `Shared (App)/SignInView.swift` | the sheet; shows `SignInCopy.*` at 9 sites through `Text(String)`/`Button(String, …)`; `SignInWithAppleButton(.continue)` labelled by the system |
| `LinguaSignIn/Sources/LinguaSignIn/SignInFlow.swift` | `SignInLink.provider(from:)` parses `cymbra-lingua://signin?provider=…`; `enum SignInCopy` — "The sheet's French copy", 6 constants and 3 functions of the provider; `swift test` runs `SignInFlowTests` (asserts the French never reads as a password error) |
| `iOS (App)/SceneDelegate.swift`, `macOS (App)/AppDelegate.swift` | open the sheet for a sign-in link |
| `apps/lingua-extension/src/state/native-signin.ts` | `hostAppSignInUrl(provider)`; the background opens it (`background.ts`) |
| Project | `developmentRegion = en`, `knownRegions (en, Base)`, string catalogues preferred; app Info.plists `CFBundleDevelopmentRegion` `fr`, `CFBundleLocalizations` `[fr]`; the "Copy Lingua extension" phase rsyncs `dist-safari` into each extension's bundle |
| Change 27 | the Safari variant carries `_locales/<native>` for each shipped native, none while only French ships |
| Change 13 | the extension's interface language, `interfaceLanguage(area)`, under `cymbra-lingua-interface-language` |

## Goals / Non-Goals

**Goals:**
- The activation page in the device's language, the sign-in sheet in the extension's, among the
  shipped natives.
- Nothing moves while only French-native pairs ship.

**Non-Goals:**
- Listings, review notes, screenshots per locale (36, 37; M16).
- The macOS menu bar (English template today).
- A language the extension does not ship.

## Decisions

### D1 — One list of languages: the extension's shipped natives

A Run Script phase on each app target, after the extension's copy, lists the `_locales` folders of
the copied `dist-safari` (none means `fr`) and writes them into the built app's Info.plist with
PlistBuddy: `CFBundleLocalizations` as that list, `CFBundleDevelopmentRegion` `en` when `en` is in
it, `fr` otherwise. The committed Info.plists keep `[fr]`/`fr`; a French-only build's plist is byte
for byte today's. Swift reads `Bundle.main.preferredLocalizations.first` — the system's choice
among the declared localisations for the device's preferred languages — and maps it to `fr`, `en`
or `es`.

Why not `.lproj` variants of `Main.html`: three copies of one page's structure, and the bundle's
`.lproj` folders, not the plist, would decide which languages exist — a French-only build would
still declare English and Spanish.

### D2 — The activation page fills itself from one table

`Resources/copy.js` holds the page's texts per language — `fr` byte for byte today's, `en` and `es`
drafts — including the macOS-before-13 variants. `Main.html` keeps its structure; each text node
carries `data-copy="key"`, its French text stays in place, and `<html>` carries
`data-copy-pending`, which hides the body until `show()` has filled the page in the language Swift
passes as `show(platform, enabled, useSettings, language)` and set `lang`. A French reader sees
the same bytes; the lede's « en anglais », which predates Spanish, is a wording point for the
owner's review (change 33), not changed here (M23).

### D3 — The sign-in sheet follows the extension

`hostAppSignInUrl(provider, language)` appends `&lang=<interface language>`; the background passes
the language it reads (change 13's key). `SignInLink.language(from:)` reads it; the sheet uses it
when it is one of the app's declared localisations, the bundle's preferred localisation
otherwise. `SignInCopy` becomes `SignInCopy(language:)` with the same members, backed by a table
per language in the package (a SwiftPM package without resources: the table is Swift); the French
table's strings are today's, and `SignInFlowTests` keeps its assertions on them, plus one per
language that no message reads as a password error. Apple's `SignInWithAppleButton` follows the
app's localisation on its own.

### D4 — Tests

`swift test`: the link's `lang` parsed, absent, unknown; each language's copy. The extension:
`hostAppSignInUrl` with and without a language. The build phase: a script test (shell, run by
`lingua-apple-build`) that a `dist-safari` without `_locales` yields `[fr]`/`fr` and one with
`fr` and `en` yields `[fr, en]`/`en`. The page: a jsdom test of `copy.js` and `show()` (every key
in every language, the French text equal to `Main.html`'s, `lang` set).

## Risks / Trade-offs

- **A flash of French before `show()`** → the body is hidden until filled.
- **An older host app opened with `lang`** → it ignores the parameter and shows French, as today.
- **The App Store's "Languages" line** → it names English and Spanish only in the build that ships
  their pairs (D1).

## Migration Plan

No visible change while only French-native pairs ship. The first build that declares English is
change 34's; the owner delivers it with the English listing.
