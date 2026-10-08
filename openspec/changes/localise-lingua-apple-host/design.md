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
| Project | `developmentRegion = en`, `knownRegions (en, Base)`, string catalogues preferred; app Info.plists `CFBundleDevelopmentRegion` `fr`, `CFBundleLocalizations` `[fr]` — but Xcode writes the development language, `en`, as the built apps' `CFBundleDevelopmentRegion`, so the shipped app's region has always been `en`, and `Bundle.localizations` (`CFBundleLocalizations`, the `.lproj` folders and the development region) reads `[fr, Base, en]`; the "Copy Lingua extension" phase rsyncs `dist-safari` into each extension's bundle |
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
(the packs the Safari build carries — what the extension's copy phase reads; a native is 2–3
lowercase letters, or the build fails) and compares `CFBundleLocalizations` and
`CFBundleDevelopmentRegion` of the processed plist with what that list asks for; it writes only
when they differ. While every native is French, what it asks for is Xcode's own product, so a
French-only build never touches the processed plist (an iOS plist is binary, and a rewrite would
change its bytes): the source plist's `[fr]` on the apps, no `CFBundleLocalizations` on the
extensions, and as `CFBundleDevelopmentRegion` the project's development language, `en`, which
Xcode writes over the source's `fr` — left untouched. Once a non-French native ships it writes the
natives, with `CFBundleDevelopmentRegion` `en` when `en` is among them and `fr` otherwise, and a
French-only build after a localised one is reset to Xcode's product. It runs as a Run Script phase,
last on each app target and on both extension targets after their copy phase, with
`$(TARGET_BUILD_DIR)/$(INFOPLIST_PATH)` as its input (never an output, which would collide with
`ProcessInfoPlistFile`) and `alwaysOutOfDate = 1`, before CodeSign; it relies on
`ENABLE_USER_SCRIPT_SANDBOXING = NO`, which the project sets. The App Store's "Languages" line
reads `CFBundleLocalizations` (the precedent in `apps/music/ios/Runner/Info.plist`).

The languages a build offers are its `CFBundleLocalizations` alone: Swift reads them from the info
dictionary (`SignInLanguage.offered(in:)`), mapped to `fr`, `en` and `es`, and never counts
`Bundle.localizations`, which adds the `.lproj` folders and the development region — on a
French-only build `[fr, Base, en]`, so a device in English would get the English page and sheet
before English ships. The device's language is `Bundle.main.preferredLocalizations.first` mapped to
`fr`, `en` or `es` (anything else — `Base`, a regional id — maps to the development region), and
counts only when offered; when neither the language asked for nor the device's is offered, a screen
falls back to English once English is offered and to French otherwise. On a French-only build every
screen is therefore French, whatever the device and whatever a link names. A SwiftPM test builds a
bundle on disk as Xcode does (`CFBundleLocalizations`, `CFBundleDevelopmentRegion` `en`, a
`Base.lproj`), and `lingua-apple-build` checks each built app's `CFBundleLocalizations` against the
shipped natives. Whether `preferredLocalizations` honours `CFBundleLocalizations` without `.lproj`
folders, and whether Safari picks `_locales` from the system's languages or the extension bundle's,
is checked on a simulator (task 4.5).

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
background sends (Safari only) at start and on each change, only a language the key actually holds
and only when it differs from the last one sent — and the device's, among the offered (D1),
before (M22: a French
reader keeps a French page on a device in English). The lede's « en anglais », which predates
Spanish, is a wording point for the owner's review (change 33), not changed here (M23); the
English and Spanish drafts name the language their natives study — Spanish in `en`, English in
`es` — in the lede and step 3, not translations of « en anglais ».

### D3 — The sign-in sheet follows the extension

`hostAppSignInUrl(provider, language)` — the language required, so `tsc` holds every caller to
name one — appends `lang=<interface language>` (built with `URLSearchParams`); the background passes
the language it reads (change 13's key). `SignInLink.language(from:)` returns a `SignInLanguage`
(`fr`, `en`, `es`) or nil — the raw value is never shown or passed on; the sheet uses it when the
build offers it (D1), the device's language when offered otherwise, then D1's fallback. `SignInCopy` becomes `SignInCopy(language:)` with the same members, backed by a table
per language in the package (a SwiftPM package without resources: the table is Swift); the French
table's strings are today's, and `SignInFlowTests` keeps its assertions on them, plus one per
language that no message reads as a password error. Apple's `SignInWithAppleButton` keeps the label the
system gives it, in the bundle's preferred localisation.

### D4 — Tests

`swift test`: the link's `lang` parsed, absent, unknown; each language's copy. The extension:
`hostAppSignInUrl` in each language, its language required; the background's teller sends only a language the key holds, once until it changes. The build phase: a script test of
`tool/app_localizations.sh` (run by `lingua-apple-build`) that packs of French natives leave a
plist untouched, `fr` plus `en` natives yield `[fr, en]`/`en`, and a native that is not 2–3 lowercase letters fails. The page:
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
