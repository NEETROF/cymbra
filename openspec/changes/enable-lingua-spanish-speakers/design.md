# Design — enable-lingua-spanish-speakers

## Context

See proposal.md (Why). The list: `packs.json` gives the shipped pairs, its first the default studied
language of a French-native reader; `tool/gen_pack.sh` builds each listed pair from its committed
tables (`--real`) or its testdata (`yarn gen:pack`); `check_variants` holds the list equal to
`SHIPPED_PAIRS` and every package to exactly the listed packs. What follows the list: `pairs.ts`'s
`shippedNatives()` (change 20) and `readerPairs`; the build's `_locales` (change 27); the host app's
localisations (change 28); the site's coverage figures (`apps/site/src/data/lingua-coverage.json`,
written by `gloss_coverage.py --write` for the pairs of `packs.json` and held to them by
`test_gloss_coverage.py`) and the Lingua pages built from them — `/es/lingua/` only when a
Spanish-glossed pair is in that file (change 30 D3); the translation controller's needs (change 8).
The requirement *The shipped pairs are one list* names en-fr, es-fr then es-en and is held by
`enable-lingua-english-speakers`, itself archived after `enable-lingua-spanish`.

Why after change 34: its list holds es-en, and M13 makes the first `_locales` the English one —
en-es shipped first would set `default_locale` to `fr` (change 27 D2).

## Goals / Non-Goals

**Goals:**
- en-es in every package, shown to Spanish speakers learning English, nothing moved for a French-native reader.

**Non-Goals:**
- Listings (change 37); new copy; a release.

## Decisions

### D1 — The list, the gate and the published figures, together

`packs.json` becomes `["en-fr", "es-fr", "es-en", "en-es"]` and `SHIPPED_PAIRS` the same; the
requirement's list follows. `yarn gen:pack` builds the en-es testdata fixture (change 22),
`yarn gen:pack:real` its committed tables, checked against its pin. `gloss_coverage.py --write`
rewrites `lingua-coverage.json` with en-es's figures in the same pull request, since
`test_gloss_coverage.py` fails while the file and `packs.json` disagree; with a Spanish-glossed
pair in it, the site builds `/es/lingua/` (change 30 D3).

### D2 — French-native readers unchanged

en-fr stays first; a French-native reader's pairs are en-fr and es-fr, their packages hold one more
pack they never load, and the English and Spanish baselines (S0, the es-fr golden) do not move.

### D3 — The dogfood pass, before merge

On Chrome and Firefox (macOS), Firefox for Android, Safari (macOS, iOS), with a package built from
this branch with the real packs, on test accounts (risk 2):
1. A new install in a browser in Spanish: the onboarding asks for the native language preset to
   Spanish; the interface reads in Spanish; the extension's description in the browser's pages is
   in Spanish (Safari: in Safari's settings).
2. An English page: highlighted; the card in Spanish (grammar lines, headings, gloss); its level
   estimated where the pack says so.
3. Review and statistics in Spanish; a card syncs to a second device carrying its gloss's language.
4. Translation as the owner settled it (M15).
5. An installed French-native extension updated to this build: nothing asked, nothing moved.
6. A new install with the browser in German presets English, not Spanish (M13, change 20).
7. An English-native install of change 34's release, updated to this build: nothing asked; three
   native languages offered in Réglages › Langue.
8. A test account created from the Spanish interface: the deletion link opens
   `/es/eliminar-cuenta/`; the Spanish verification e-mail links `/es/terminos/` and
   `/es/privacidad/` (changes 29, 32).
9. A selection change 25's soak recorded as trapping (if any): one respawn, the card says
   translation is unavailable, and the next selection translates (risk 3, change 9).

### D4 — The release, TestFlight first

TestFlight first. As M13 binds the English listing to the first `_locales`, the Spanish listing
goes with the first `_locales/es`: the submission that carries it carries change 37's listings.

## Risks / Trade-offs

- **A server that does not store the labels** → change 12 holds non-French labels until it does;
  the owner's prerequisite checks the server first anyway.
- **Package size** → one pack more, never loaded by a French-native reader.
- **en-es traps the engine on some inputs** (risk 3) → change 9's respawn, change 25's soak read
  by the owner before the merge, and D3's step 9.

## Migration Plan

Merged with the owner's go-ahead; released by the owner, TestFlight first, with the Spanish
listings.
