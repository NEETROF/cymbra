# enable-lingua-english-speakers — English speakers learning Spanish: es-en ships

## Why

Change 34 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the stage-2 release for English speakers learning Spanish (M1: es-en may ship first; it needs no Spanish site pages and loads no new model (es-en is pinned)). Everything it needs was built silently:
the native-language platform and the interface catalogue (changes 4–20), the es-en pack
(change 21), its card wording (change 23), its route and its marks (changes 25, 26), the
manifest's `_locales` (27), the host app's languages (28), the site's pages (29–31) and the
e-mails' links (32). Every one of them waits for the same edit to show itself: `packs.json` listing
es-en. The native-language choice (change 20) appears when two native languages have a shipped
pair; `_locales`, the host app's languages and the site's pages follow the shipped natives.

## What Changes

- **es-en ships.** `packs.json` becomes `["en-fr", "es-fr", "es-en"]`; `check_variants`'s `SHIPPED_PAIRS` follows
  in the same pull request, as its gate requires. en-fr stays first, so a French-native reader's
  default is unchanged.
- **What a new reader whose browser is in English sees**: the onboarding asks for the native
  language, preset to English (change 20, M3); the interface, the card, review and statistics in
  English; the extension's description in `_locales/fr` and `_locales/en`, `default_locale` English (change 27, M13); the host app declares English beside French (change 28); the Lingua pages of change 30 (the English page leads with Spanish for English speakers).
- **What an installed reader sees**: nothing new unless they choose another native language in
  Réglages (M22).
- **Translation**: the es-en route (change 25) is offered as the owner settles under M15 from change
  26's marks — marked if the pair reached the first tier, unmarked or not offered otherwise.
- **A dogfood pass on five targets** with the interface in English, and the beta ring first; the
  owner gives the go-ahead before the merge and before each submission (M18).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *The shipped pairs are one list* (held by
  `enable-lingua-spanish`, so this change is archived after it): the list holds en-fr, es-fr then es-en.

## Impact

- **Products.** Cymbra Lingua's extension and Safari host app: `apps/lingua-extension/packs.json`,
  `tool/check_variants.mjs`. Each package grows by the es-en pack; an engine loads a pair's pack
  only when its language is needed, and a French-native reader never loads it.
- **Prerequisites, all the owner's** (M18):
  - Cymbra ID's and Lingua's servers store the gloss's language and the day's native language,
    deployed and checked from outside (change 10's tasks 5.x, change 12's 6.1) — server first (M4).
  - The privacy annex (change 31) and the App Store privacy answers published before the release
    (M14).
  - The English copy reviewed (M9): changes 13–20's drafts, changes 23 and 33's corrections.
  - The English listings (change 36) in the same submission (M13), and the site deployed.
- **Release.** The first packages for English speakers learning Spanish. Nothing is submitted by this pull request.
