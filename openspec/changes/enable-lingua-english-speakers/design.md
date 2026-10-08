# Design — enable-lingua-english-speakers

## Context

See proposal.md (Why). The list: `packs.json` gives the shipped pairs, its first the default studied
language of a French-native reader; `tool/gen_pack.sh` builds each listed pair from its committed
tables (`--real`) or its testdata (`yarn gen:pack`); `check_variants` holds the list equal to
`SHIPPED_PAIRS` and every package to exactly the listed packs. What follows the list: `pairs.ts`'s
`shippedNatives()` (change 20) and `readerPairs`; the build's `_locales` (change 27); the host app's
localisations (change 28); the site's coverage figures (`apps/site/src/data/lingua-coverage.json`,
written by `gloss_coverage.py --write` for the pairs of `packs.json` and held to them by
`test_gloss_coverage.py`) and the Lingua pages built from them (change 30); the translation
controller's needs (change 8). `test/pairs.spec.ts` *for every reader today* asserts that the
default list is French-native only. The requirement *The shipped pairs are one list* names en-fr
then es-fr and is held by `enable-lingua-spanish`.

## Goals / Non-Goals

**Goals:**
- es-en in every package, shown to English speakers learning Spanish, nothing moved for a French-native reader.

**Non-Goals:**
- Listings (change 36); new copy; a release.

## Decisions

### D1 — The list, the gate and the published figures, together

`packs.json` becomes `["en-fr", "es-fr", "es-en"]` and `SHIPPED_PAIRS` the same; the requirement's
list follows. `yarn gen:pack` builds the es-en testdata fixture (change 21), `yarn gen:pack:real`
its committed tables, checked against its pin. `gloss_coverage.py --write` rewrites
`lingua-coverage.json` with es-en's figures in the same pull request (change 21 D6:
the figures are published when the pair ships), since `test_gloss_coverage.py` fails while the
file and `packs.json` disagree. Change 30's English page quotes today's French card buttons;
they become the English catalogue's (`apps/lingua-extension/src/i18n/en/card.ts`), handed over by
change 30. `test/pairs.spec.ts` *for every reader today* is rewritten for a list with a second
native, and every test that asserts a French-only build from the default list (change 20's
*Every reader today*) passes its list explicitly.

### D2 — French-native readers unchanged

en-fr stays first; a French-native reader's pairs are en-fr and es-fr, their packages hold one more
pack they never load, and the English and Spanish baselines (S0, the es-fr golden) do not move.

### D3 — The dogfood pass, before merge

On Chrome and Firefox (macOS), Firefox for Android, Safari (macOS, iOS), with a package built from
this branch with the real packs, on test accounts (risk 2):
1. A new install in a browser in English: the onboarding asks for the native language preset to
   English; the interface reads in English; the extension's description in the browser's pages is
   in English (Safari: in Safari's settings).
2. A Spanish page: highlighted; the card in English (grammar lines, headings, gloss); its level
   estimated where the pack says so.
3. Review and statistics in English; a card syncs to a second device carrying its gloss's language.
4. « Traduction étendue » on in English; es-en downloaded once at its catalogue size; a Spanish
   selection translated into English, marked if es-en is in `MARKED_PAIRS` (change 26), or as M15
   settled; absent on Firefox for Android.
5. An extension of the last store release, used in French (a level, cards), replaced in place by
   this build (reason `update`), with the browser in English: no native-language prompt in the
   popup, interface French, studied languages unchanged (M22).
6. Réglages › Langue: choose English — the consequence stated, the pages reload in English,
   Spanish studied, French cards kept; then French again.
7. A new install with the browser in German presets English and shows the English description
   (M13, changes 20 and 27).
8. Safari: the activation page in English on an English device and in French for the installed
   reader (change 28); the sign-in sheet in English; the popup's first-run call to action when the
   onboarding tab did not open.
9. A test account created from the English interface: the verification e-mail in English with
   `/en/` links; the deletion link opens `/en/delete-account/`; a card captured on the
   English-native device shows on a French-native device of the same account with the French
   pack's gloss, and the reverse (M4); the day's statistic stored with `en` (M14).

### D4 — The release, TestFlight first

TestFlight first. The first store submission carrying `_locales` carries change 36's listings
(M13): on the Chrome Web Store that package is uploaded without publishing, the French listing
re-entered under `fr` and the English filled under `en`, then submitted; the owner checks both
dashboards after it and records the result in `STORE-LISTING.md` (change 27's risk).

### D5 — Translation as M15 settles it

The `es-en` route (change 25) stays in `model-manifest.json` when the owner offers the pair's
translation, marked or not as change 26 measured. If the owner settles under M15 not to offer it,
the route is removed (the model stays, as es-fr's pivot), so the pair is unavailable as *A pair
without a route* says; nothing else moves.

## Risks / Trade-offs

- **A server that does not store the labels** → change 12 holds non-French labels until it does;
  the owner's prerequisite checks the server first anyway.
- **Package size** → one pack more — es-en's, 2,568,024 B as change 21 measured it, re-measured
  after its correction — never loaded by a French-native reader.
- **The dashboards re-keyed by `default_locale: en`** → D4's upload without publishing, and the
  owner's check recorded in `STORE-LISTING.md`.

## Migration Plan

Merged with the owner's go-ahead; released by the owner, TestFlight first, with the English
listings.
