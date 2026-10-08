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
- The activation page in the extension's interface language once it has run, the device's before,
  and the sign-in sheet in the extension's, among the shipped natives.
- Nothing moves while only French-native pairs ship.

**Non-Goals:**
- Listings, review notes, screenshots per locale (36, 37; M16).
- The macOS menu bar (English template today).
- A language the extension does not ship.

## Decisions

### D1 — One list of languages: the extension's shipped natives

`tool/app_localizations.sh` lists the natives of `${SRCROOT}/../lingua-extension/dist-safari/assets/packs/*.lingua`
(the packs the Safari build carries — what the extension's copy phase reads) and compares
`CFBundleLocalizations` and `CFBundleDevelopmentRegion` of the processed plist with that list
(`CFBundleDevelopmentRegion` `en` when `en` is in it, `fr` otherwise); it writes only when they
differ, so a French-only build never touches the processed plist (an iOS plist is binary, and a
rewrite would change its bytes) and a French-only build after a localised one is reset. It runs as
a Run Script phase, last on each app target and on both extension targets after their copy phase,
with `$(TARGET_BUILD_DIR)/$(INFOPLIST_PATH)` as its input (never an output, which would collide
with `ProcessInfoPlistFile`) and `alwaysOutOfDate = 1`, before CodeSign; it relies on
`ENABLE_USER_SCRIPT_SANDBOXING = NO`, which the project sets. The committed app Info.plists keep
`[fr]`/`fr`; the extensions' generated plists hold `CFBundleDevelopmentRegion` `en` and no
`CFBundleLocalizations`, which is what a French-only list means on the extension targets, so the
script writes there only once a non-French native ships and restores those values after. The App Store's "Languages" line reads `CFBundleLocalizations` (the precedent in
`apps/music/ios/Runner/Info.plist`). Swift reads `Bundle.main.preferredLocalizations.first` and
maps it to `fr`, `en` or `es`; anything else (`Base`, a regional id) maps to the development
region. Whether `preferredLocalizations` honours `CFBundleLocalizations` without `.lproj` folders,
and whether Safari picks `_locales` from the system's languages or the extension bundle's, is
checked on a simulator (task 4.5).

Why not `.lproj` variants of `Main.html`: three copies of one page's structure, and the bundle's
`.lproj` folders, not the plist, would decide which languages exist — a French-only build would
still declare English and Spanish.

### D2 — The activation page fills itself from one table

`Resources/copy.js` holds the page's texts per language — `fr` byte for byte today's, `en` and `es`
drafts — including the macOS-before-13 variants. Swift injects the language before load (a
`WKUserScript` at `.atDocumentStart`: `window.linguaLanguage = "en"`); `copy.js` fills the page on
`DOMContentLoaded` and sets `lang`; nothing is hidden, a French page is left untouched, and
`show()` keeps its three parameters. `Main.html` keeps its structure, its French text in place;
each text carries `data-copy="key"`; entries whose copy holds `<strong>` are HTML fragments from
`copy.js` (bundled, never from Swift or a link), set with `innerHTML`; the icon's `alt` uses
`data-copy-alt`. `copy.js` is added to the project (a file reference in `Resources`, a build file in
both app targets' Copy Bundle Resources). The language Swift injects is the extension's interface
language once the extension has run — its native handler writes it to the App Group suite
`IdTokenHandoff` already uses, on a `NativeMessage` case taking a `SignInLanguage` that the
background sends (Safari only) at start and on each change — and the bundle's preferred localisation
before (M22: a French
reader keeps a French page on a device in English). The lede's « en anglais », which predates
Spanish, is a wording point for the owner's review (change 33), not changed here (M23); the
English and Spanish drafts name the language their natives study — Spanish in `en`, English in
`es` — in the lede and step 3, not translations of « en anglais ».

### D3 — The sign-in sheet follows the extension

`hostAppSignInUrl(provider, language)` appends `&lang=<interface language>`; the background passes
the language it reads (change 13's key). `SignInLink.language(from:)` returns a `SignInLanguage`
(`fr`, `en`, `es`) or nil — the raw value is never shown or passed on; the sheet uses it when it is
one of the app's declared localisations, the bundle's preferred localisation otherwise. `SignInCopy` becomes `SignInCopy(language:)` with the same members, backed by a table
per language in the package (a SwiftPM package without resources: the table is Swift); the French
table's strings are today's, and `SignInFlowTests` keeps its assertions on them, plus one per
language that no message reads as a password error. Apple's `SignInWithAppleButton` keeps the label the
system gives it, in the bundle's preferred localisation.

### D4 — Tests

`swift test`: the link's `lang` parsed, absent, unknown; each language's copy. The extension:
`hostAppSignInUrl` with and without a language. The build phase: a script test of
`tool/app_localizations.sh` (run by `lingua-apple-build`) that packs of French natives leave a
plist untouched, and `fr` plus `en` natives yield `[fr, en]`/`en`. The page:
`apps/lingua-extension/test/apple-activation-page.spec.ts`, a jsdom test of `copy.js` (every key in
every language, each French entry equal to `Main.html`'s markup, `lang` set), with
`lingua-extension-check`'s `ext` filter gaining `apps/lingua-apple/Shared (App)/Resources/**`;
`lingua-apple-build` asserts `copy.js` is in both built apps.

## Risks / Trade-offs

- **A link not built by the extension** (any page can open `cymbra-lingua://signin…`) →
  `SignInLink.language(from:)` accepts a closed list; the raw value is never shown or passed on.
- **Apple's button in another language than the sheet** → e.g. « Continue with Apple » in a French
  sheet on an English device once English ships: the system draws it; a custom button with
  `SignInCopy.button(.apple)` within Apple's guidelines is the alternative, the owner's call.
- **The App Store's "Languages" line** → it names English and Spanish only in the build that ships
  their pairs (D1).

## Migration Plan

No visible change while only French-native pairs ship. The first build that declares English is
change 34's; the owner delivers it with the English listing.
