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

Its bundle declares French alone (`CFBundleLocalizations` `[fr]`, `CFBundleDevelopmentRegion`
`fr`), which is also what the App Store's "Languages" line shows. The extension's interface
follows the reader's native language (changes 13–20); the host app's two screens do not.

The activation page runs before the extension has ever run, so no interface language exists
yet: it can only follow the device's language. The sign-in sheet is a step of the extension's
own flow, opened from a page in the reader's language: it follows the extension. And neither may
speak a language no shipped pair is glossed in — an English page offering a product an English
reader cannot use yet — so the app offers exactly the languages the extension ships (change 27).

## What Changes

- **The app's languages are the extension's shipped natives**: a build phase reads the
  `_locales` folders of the copied Safari extension (none: French alone) and writes
  `CFBundleLocalizations` and `CFBundleDevelopmentRegion` (English when English ships, French
  otherwise) into the built app. While every shipped pair is French-native, the app is what it is
  today.
- **The activation page's copy in three languages**: `Main.html`'s and `Script.js`'s texts move to
  one copy table (`copy.js`, fr/en/es; the French byte for byte); Swift passes the bundle's
  preferred localisation to `show()`, which fills the page before it is shown; the French page is
  unchanged.
- **The sign-in sheet in the extension's language**: the extension adds its interface language to
  the link (`cymbra-lingua://signin?provider=apple&lang=es`); `SignInLink` reads it; `SignInCopy`
  becomes a table per language (fr byte for byte, en and es drafts); a link without `lang`, or with
  a language the app does not offer, uses the bundle's preferred localisation. Apple's button is
  labelled by the system in that localisation.
- **The drafts** for the owner's review (M9; M10).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-apple-app`: MODIFIED *Guided activation* (held by the open `add-lingua-apple`, so this
  change is archived after it): "the shipping copy is French" becomes the languages of the shipped
  natives, the device's chosen among them; both scenarios kept. ADDED *The sign-in sheet speaks
  the extension's language*.

## Impact

- **Products.** Cymbra Lingua: `apps/lingua-apple` (the activation page, `ViewController.swift`,
  `LinguaSignIn`, the project's build phase, the Info.plists) and `apps/lingua-extension`
  (`src/state/native-signin.ts`, the background's call). ID, Music, Live, the back office and the
  site are untouched.
- **No byte moves for a French reader**: while only French-native pairs ship, the app offers
  French alone, its page and sheet byte for byte; the link's added parameter is read only by a
  host app that understands it, and an older host app ignores it.
- **The first build offering English** is the one that ships es-en (change 34); the App Store's
  "Languages" line then names English, and the English listing goes with it (change 36, M16).
- **Order.** After change 27 (the extension's `_locales` the build phase reads) and change 13 (the
  interface language the extension passes). Archived after `add-lingua-apple`.
- **Not here.** The App Store listing per locale and the review notes (36, 37; M16); the macOS
  menu bar's template titles, English today for every reader (a visible change for French readers,
  left to change 33's review).
