# enable-lingua-spanish-speakers — Spanish speakers learning English: en-es ships

## Why

Change 35 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the stage-2 release for Spanish speakers learning English, after change 9 (merged as #765) and
change 34: its list holds es-en, and M13 makes the first `_locales` the English one — en-es
shipped first would set `default_locale` to `fr` (change 27 D2). What it needs is built silently
before it — a precondition, listed under Impact's prerequisites, not a fact today: the
native-language platform and the interface catalogue (changes 4–20), the en-es pack (change 22),
its card wording (change 24), its route and its marks (changes 25, 26), the manifest's `_locales`
(27), the host app's languages (28), the site's Spanish pages, the Lingua pages and the privacy
annex (29–31) and the e-mails' links (32). Every one of them waits for the same edit to show
itself: `packs.json` listing en-es. The native-language choice (change 20) appears when two native
languages have a shipped pair; `_locales`, the host app's languages and the site's pages follow
the shipped natives.

## What Changes

- **en-es ships.** `packs.json` becomes `["en-fr", "es-fr", "es-en", "en-es"]`; `check_variants`'s
  `SHIPPED_PAIRS` follows in the same pull request, as its gate requires. en-fr stays first, so a
  French-native reader's default is unchanged. `gloss_coverage.py --write` publishes en-es's
  figures in `apps/site/src/data/lingua-coverage.json`, from which change 30's pages read their
  pairs; `/es/lingua/` is built only when a Spanish-glossed pair is in that file (change 30 D3).
- **What a new reader whose browser is in Spanish sees**: the onboarding asks for the native
  language, preset to Spanish (change 20, M3); the interface, the card, review and statistics in
  Spanish; the extension's description in `_locales/es` beside `fr` and `en` (change 27); the host
  app declares Spanish (change 28); the Lingua pages of change 30, `/es/lingua/` appearing with
  this pair.
- **What an installed reader sees**: nothing asked and nothing moved (M22); Réglages › Langue
  offers Spanish beside French and English (change 20), and a browser in Spanish shows the Spanish
  description in its own pages (change 27's accepted trade-off).
- **Translation**: the en-es route (change 25) is offered as the owner settles under M15 from
  change 26's marks — marked if the pair reached the first tier, unmarked or not offered
  otherwise.
- **A dogfood pass on five targets** with the interface in Spanish, and TestFlight first; the
  owner gives the go-ahead before the merge and before each submission (M18).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *The shipped pairs are one list* (held by
  `enable-lingua-english-speakers`, itself archived after `enable-lingua-spanish`, so this change
  is archived after both): the list holds en-fr, es-fr, es-en then en-es, and its first pair glossed
  in each native language gives that native language's default.

## Impact

- **Products.** Cymbra Lingua's extension and Safari host app: `apps/lingua-extension/packs.json`,
  `tool/check_variants.mjs`. The site: `apps/site/src/data/lingua-coverage.json`, from which
  `/es/lingua/` is built. Each package grows by the en-es pack; an engine loads a pair's pack only
  when its language is needed, and a French-native reader never loads it.
- **Prerequisites, all the owner's** (M18):
  - Cymbra ID's and Lingua's servers store the gloss's language and the day's native language,
    deployed and checked from outside (change 10's tasks 5.x, change 12's 6.1) — server first (M4).
  - Changes 22 (its floor set), 24, 24b (`refine-lingua-en-es-glosses`: en-es's glosses read as
    meanings, en-es re-pinned), 26 and 33 merged — implementations, not proposals.
  - M15 and M25 settled (open, before stage 2).
  - Change 27's D4 check and change 28's task 4.5 repeated with en-es (`_locales/es`,
    `CFBundleLocalizations` `[fr, en, es]`).
  - The en-es model host deployed (change 25's task 4.1) and its soak read by the owner (change 25
    recorded which selections trap; risk 3, change 9).
  - The Spanish site pages live (change 29, and change 29b's home and Music page,
    `extend-site-spanish-locale`, M11), `https://cymbra.app/eliminar-cuenta` registered on the
    Services ID (change 29's task 4.3), and Cymbra ID's e-mails linking them deployed (change 32).
  - The privacy annex in fr, en and es deployed (change 31; the App Store answers' categories
    unchanged, their notes updated) before the release (M14).
  - The Spanish copy reviewed (M9): the drafts of changes 13–20, 24, 27, 28, 29, 29b, 30 and 31, and
    change 33's corrections.
  - The Spanish listings (change 37) in the same submission: as M13 binds the English listing to
    the first `_locales`, the Spanish listing goes with the first `_locales/es`.
- **Release.** The first packages for Spanish speakers learning English, TestFlight first (D4); the
  site deployed after the merge, with en-es's figures, so that `/es/lingua/` appears, before the
  listings are pasted. Nothing is submitted by this pull request.
