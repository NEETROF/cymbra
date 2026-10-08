# localise-lingua-apple-host — the Safari host app speaks its reader's language

## Why

Change 28 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2. On iPhone, iPad and Mac, Cymbra Lingua is a Safari extension inside a small host app
(`apps/lingua-apple`). The app shows two things of its own, both in French only:

- **the activation page** — a web view (`Main.html`, `Script.js`): how to enable the extension in
  Safari on iOS, and on macOS a button that opens Safari's settings and the extension's state
  (12 texts, plus 3 for macOS before version 13);
- **the sign-in sheet** — SwiftUI, opened by the extension through
  `cymbra-lingua://signin?provider=…` to run Apple's or Google's native sign-in: its copy is
  `SignInCopy` in the `LinguaSignIn` package (9 messages).

Its Info.plists declare French alone (`CFBundleLocalizations` `[fr]`, `CFBundleDevelopmentRegion`
`fr`), and `CFBundleLocalizations` is what the App Store's "Languages" line shows. The built app's
`CFBundleDevelopmentRegion` is `en`, though: Xcode writes the project's development language over
the plist's, as it always has, and this change leaves that `en` untouched (a French-only build is
never touched). So `Bundle.localizations`, which adds the development region, names English on a
French-only build, and the Swift side counts only `CFBundleLocalizations` as the languages the app
offers. The extension's interface follows the reader's native language (changes 13–20); the host
app's two screens do not.

The activation page runs before the extension has ever run, so no interface language exists
yet: it first follows the device's language, and the extension's interface language once the
extension has run (M22). The sign-in sheet is a step of the extension's
own flow, opened from a page in the reader's language: it follows the extension. And neither may
speak a language no shipped pair is glossed in — an English page offering a product an English
reader cannot use yet — so the app offers exactly the languages the extension ships (change 27).

## What Changes

- **The app's languages are the extension's shipped natives**: a build phase reads the natives of
  the packs the Safari build carries (`dist-safari/assets/packs/<studied>-<native>.lingua`; French
  alone today) and writes `CFBundleLocalizations` and `CFBundleDevelopmentRegion` (English when
  English ships, French when only other natives join French) into the built app and its extension
  — only when they differ from what the plist holds. A French-only build is never touched: the
  apps keep `[fr]` and Xcode's `en` region, the extensions their generated plist. While every
  shipped pair is French-native, the app is what it is today.
- **The activation page's copy in three languages**: `Main.html`'s and `Script.js`'s texts move to
  one copy table (`copy.js`, fr/en/es; the French byte for byte); Swift injects the language before
  the page loads, and `copy.js` fills it; a French page is left untouched. Once the extension has
  run, the page follows the extension's interface language (written to the App Group the sign-in
  handoff already uses), so a French reader on a device in English keeps a French page (M22); the
  device's language only before.
- **The sign-in sheet in the extension's language**: the extension adds its interface language to
  the link (`cymbra-lingua://signin?provider=apple&lang=es`); `SignInLink` reads it; `SignInCopy`
  becomes a table per language (fr byte for byte, en and es drafts); a link without `lang`, or with
  a language the app does not offer, uses the device's language when the app offers it, English
  once English ships and French otherwise. Apple's button is labelled by the system.
- **The drafts** for the owner's review (M9; M10).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-apple-app`: MODIFIED *Guided activation* (held by the open `add-lingua-apple`, so this
  change is archived after it): "the shipping copy is French" becomes the languages of the shipped
  natives, the extension's interface language once it has run, the device's before, among them; both scenarios kept. ADDED *The sign-in sheet speaks
  the extension's language*.

## Impact

- **Products.** Cymbra Lingua: `apps/lingua-apple` (the activation page, `ViewController.swift`,
  `LinguaSignIn`, the project's build phase, the Info.plists) and `apps/lingua-extension`
  (`src/state/native-signin.ts`, the background's call). ID, Music, Live, the back office and the
  site are untouched.
- **A French reader sees the same text**: while only French-native pairs ship, the app offers
  French alone, its page and sheet unchanged; the sign-in link gains `&lang=fr`.
- **The first build offering English** is the one that ships es-en (change 34); the App Store's
  "Languages" line then names English, and the English listing goes with it (change 36, M16).
- **Order.** After change 27 (the extension's `_locales`, whose languages the app's match) and change 13 (the
  interface language the extension passes). Archived after `add-lingua-apple`.
- **Not here.** The App Store listing per locale and the review notes (36, 37; M16); the macOS
  menu bar's template titles, English today for every reader (a visible change for French readers,
  left to change 33's review).
