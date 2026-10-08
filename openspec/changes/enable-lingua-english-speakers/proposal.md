# enable-lingua-english-speakers — English speakers learning Spanish: es-en ships

## Why

Change 34 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the stage-2 release for English speakers learning Spanish. (M1: es-en may ship first and loads no
new model, es-en being pinned; it still follows change 29, which the privacy annex of change 31
waits for.) What it needs is built silently before it — a precondition, listed under Impact's
prerequisites, not a fact today: the native-language platform and the interface catalogue
(changes 4–20), the es-en pack (change 21), its card wording (change 23), its route and its marks
(changes 25, 26), the manifest's `_locales` (27), the host app's languages (28), the Lingua pages
and the privacy annex (30, 31). Every one of them waits for the same edit to show itself:
`packs.json` listing es-en. The native-language choice (change 20) appears when two native
languages have a shipped pair; `_locales`, the host app's languages and the site's pages follow
the shipped natives.

## What Changes

- **es-en ships.** `packs.json` becomes `["en-fr", "es-fr", "es-en"]`; `check_variants`'s
  `SHIPPED_PAIRS` follows in the same pull request, as its gate requires. en-fr stays first, so a
  French-native reader's default is unchanged. `gloss_coverage.py --write` publishes es-en's
  figures in `apps/site/src/data/lingua-coverage.json` (change 21 D6), from which change 30's
  pages read their pairs. The extension's tests that read the default list as French-only take
  their list explicitly.
- **What a new reader whose browser is in English sees**: the onboarding asks for the native
  language, preset to English (change 20, M3); the interface, the card, review and statistics in
  English; the extension's description in `_locales/fr` and `_locales/en`, `default_locale`
  English (change 27, M13); the host app declares English beside French (change 28); the Lingua
  pages of change 30, the English page leading with Spanish for English speakers, its card's three
  buttons quoted from the English catalogue instead of today's French labels.
- **What an installed reader sees**: nothing asked and nothing moved (M22); Réglages › Langue
  gains the native-language choice (change 20), and a browser in English shows the English
  description in its own pages (change 27's accepted trade-off).
- **Translation**: the es-en route (change 25) is offered as the owner settles under M15 from
  change 26's marks — marked if the pair reached the first tier, unmarked or not offered
  otherwise. If the owner settles not to offer it, the `es-en` route leaves `model-manifest.json`
  (the model stays, as es-fr's pivot), and the pair is unavailable as *A pair without a route*
  says.
- **A dogfood pass on five targets** with the interface in English, and TestFlight first; the
  owner gives the go-ahead before the merge and before each submission (M18).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *The shipped pairs are one list* (held by
  `enable-lingua-spanish`, so this change is archived after it): the list holds en-fr, es-fr then
  es-en, and its first pair glossed in each native language gives that native language's default.

## Impact

- **Products.** Cymbra Lingua's extension and Safari host app: `apps/lingua-extension/packs.json`,
  `tool/check_variants.mjs`, `test/pairs.spec.ts` and the tests that read the default list, and
  `model-manifest.json` only if M15 withholds es-en's translation. The site:
  `apps/site/src/data/lingua-coverage.json` and the English text of `/en/lingua/` (change 30's
  table). Each package grows by the es-en pack; an engine loads a pair's pack only when its
  language is needed, and a French-native reader never loads it.
- **Prerequisites, all the owner's** (M18):
  - Cymbra ID's and Lingua's servers store the gloss's language and the day's native language,
    deployed and checked from outside (change 10's tasks 5.x, change 12's 6.1) — server first (M4).
  - Changes 21 (with M20 settled), 23, 23b (`refine-lingua-es-en-glosses`: es-en's glosses read as
    meanings, es-en re-pinned), 25, 26, 27, 28, 30, 31 and 33 merged — implementations, not
    proposals.
  - M15, M16 and M25 settled (open, before stage 2).
  - Change 27's task 2.5 (three browsers with a local `packs.json` holding es-en, and the owner's
    validation of a localised Safari archive) done before this change merges, and change 28's task
    4.5 on a simulator.
  - The model host serving every catalogue model (change 25's task 4.1).
  - Test accounts for the dogfood (risk 2).
  - The privacy annex in fr and en deployed (change 31; the App Store answers' categories
    unchanged, their notes updated) before the release (M14).
  - The English copy reviewed (M9): the drafts of changes 13–20, 23, 27, 28, 30 and 31, and change
    33's corrections merged.
  - The English listings (change 36) in the same submission (M13).
- **Release.** The first packages for English speakers learning Spanish, TestFlight first (D4);
  the site deployed after the merge, with es-en's figures, before the listings are pasted.
  Nothing is submitted by this pull request.
