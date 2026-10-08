# localise-lingua-manifest — the manifest's text in the languages a shipped pair is glossed in

## Why

Change 27 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, decision M13. The extension's own interface follows the reader's chosen language
(changes 13–20); what the browser shows about the extension — its description in the extensions
page and the store's summary, the descriptions of its keyboard commands in the shortcuts page —
comes from `manifest.json` and follows the **browser's** language through `_locales`, which
cannot follow a reader's choice. The programme keeps `_locales` for the manifest and the
per-locale listings, and ships it "with the first non-French pair": an English description shown
to an English-speaking browser before an English-glossed pair ships would offer a product that
reader cannot use yet.

Today every string is French and literal: the description (« Lisez l'anglais et l'espagnol sur le
web… », 109 characters, which both stores take as the listing's summary), the three commands'
descriptions, and the brand name and action title « Cymbra Lingua ».

## What Changes

- **`_locales/{fr,en,es}/messages.json` committed** in `apps/lingua-extension`: the description
  and the three commands' descriptions; the French byte for byte today's, the English and Spanish
  drafted for the owner's review (M9; M10: US English, Spanish tú), each naming what its own natives
  study (Spanish for English speakers, English for Spanish speakers), not a translation of the French. The name and the action's
  title stay the literal brand « Cymbra Lingua ».
- **The build ships a language where a shipped pair is glossed in it**: while every pair of
  `packs.json` is French-native, the built manifests are what they are today — literal French, no
  `_locales`, no `default_locale` — byte for byte. Once a pair glossed in English or Spanish ships
  (changes 34, 35), the build writes `__MSG_…__` references, copies the shipped natives' folders,
  and sets `default_locale` to `en` when an English-glossed pair ships (M13's fallback for any
  other browser language), `fr` otherwise.
- **The checks follow**: the 112-character bound Apple applies to the Safari extension's
  description holds for every committed language's description; `check_variants` asserts, per
  variant, that a manifest with `__MSG_` references carries `default_locale` and a folder per
  shipped native, every reference resolving in every folder, and that a French-only build carries
  none.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *The manifest speaks the browser's language among the shipped
  natives*. *The store listings name each studied language* (held by no open change) stands: the
  listings of English and Spanish speakers are changes 36 and 37.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (`_locales/`, `build.mjs`,
  `tool/check_version.mjs`, `tool/check_variants.mjs`, their specs, `STORE-LISTING.md`'s note on
  the summary). The Safari host app's copy phase removes `_locales/` from the extension before copying `dist-safari`. ID, Music, Live, the back
  office and the site are untouched.
- **No byte moves** in any package built while only French-native pairs ship.
- **The first package that carries `_locales`** is the one that ships es-en (change 34) — the
  English listing goes with it (M13, change 36): the owner checks on a device that Safari shows the
  localised description, and Apple's upload accepts it.
- **Not here.** The listings themselves (36, 37), the host app's own text (28).
