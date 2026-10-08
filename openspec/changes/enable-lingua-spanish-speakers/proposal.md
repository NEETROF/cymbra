# enable-lingua-spanish-speakers — Spanish speakers learning English: en-es ships

## Why

Change 35 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the stage-2 release for Spanish speakers learning English (after change 9 (merged) and change 34). Everything it needs was built silently:
the native-language platform and the interface catalogue (changes 4–20), the en-es pack
(change 22), its card wording (change 24), its route and its marks (changes 25, 26), the
manifest's `_locales` (27), the host app's languages (28), the site's pages (29–31) and the
e-mails' links (32). Every one of them waits for the same edit to show itself: `packs.json` listing
en-es. The native-language choice (change 20) appears when two native languages have a shipped
pair; `_locales`, the host app's languages and the site's pages follow the shipped natives.

## What Changes

- **en-es ships.** `packs.json` becomes `["en-fr", "es-fr", "es-en", "en-es"]`; `check_variants`'s `SHIPPED_PAIRS` follows
  in the same pull request, as its gate requires. en-fr stays first, so a French-native reader's
  default is unchanged.
- **What a new reader whose browser is in Spanish sees**: the onboarding asks for the native
  language, preset to Spanish (change 20, M3); the interface, the card, review and statistics in
  Spanish; the extension's description in `_locales/es` beside `fr` and `en` (change 27); the host app declares Spanish (change 28); the Lingua pages of change 30 (`/es/lingua/` appears with this pair).
- **What an installed reader sees**: nothing new unless they choose another native language in
  Réglages (M22).
- **Translation**: the en-es route (change 25) is offered as the owner settles under M15 from change
  26's marks — marked if the pair reached the first tier, unmarked or not offered otherwise.
- **A dogfood pass on five targets** with the interface in Spanish, and the beta ring first; the
  owner gives the go-ahead before the merge and before each submission (M18).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *The shipped pairs are one list* (held by
  `enable-lingua-english-speakers`, so this change is archived after it): the list holds en-fr, es-fr, es-en then en-es.

## Impact

- **Products.** Cymbra Lingua's extension and Safari host app: `apps/lingua-extension/packs.json`,
  `tool/check_variants.mjs`. Each package grows by the en-es pack; an engine loads a pair's pack
  only when its language is needed, and a French-native reader never loads it.
- **Prerequisites, all the owner's** (M18):
  - Cymbra ID's and Lingua's servers store the gloss's language and the day's native language,
    deployed and checked from outside (change 10's tasks 5.x, change 12's 6.1) — server first (M4).
  - The privacy annex (change 31) and the App Store privacy answers published before the release
    (M14).
  - The Spanish copy reviewed (M9): changes 13–20's drafts, changes 24 and 33's corrections.
- The en-es model host deployed (change 25's task 4.1) and its soak read by the owner (change 25 recorded which selections trap; risk 3, change 9).
- The Spanish site pages live (change 29, M11) and Cymbra ID's e-mails linking them deployed (change 32).
  - The Spanish listings (change 37) in the same submission (M13), and the site deployed and `/es/lingua/` appears with it.
- **Release.** The first packages for Spanish speakers learning English. Nothing is submitted by this pull request.
