# Design — enable-lingua-spanish-speakers

## Context

See proposal.md (Why). The list: `packs.json` gives the shipped pairs, its first the default studied
language of a French-native reader; `tool/gen_pack.sh` builds each listed pair from its committed
tables (`--real`) or its testdata (`yarn gen:pack`); `check_variants` holds the list equal to
`SHIPPED_PAIRS` and every package to exactly the listed packs. What follows the list: `pairs.ts`'s
`shippedNatives()` (change 20) and `readerPairs`; the build's `_locales` (change 27); the host app's
localisations (change 28); the site's Lingua pages (change 30); the translation controller's needs
(change 8). The requirement *The shipped pairs are one list* names en-fr, es-fr then es-en and is held by
`enable-lingua-english-speakers`.

## Goals / Non-Goals

**Goals:**
- en-es in every package, shown to Spanish speakers learning English, nothing moved for a French-native reader.

**Non-Goals:**
- Listings (change 37); new copy; a release.

## Decisions

### D1 — The list and the gate, together

`packs.json` becomes `["en-fr", "es-fr", "es-en", "en-es"]` and `SHIPPED_PAIRS` the same; the requirement's list follows.
`yarn gen:pack` builds the en-es testdata fixture (change 22), `yarn gen:pack:real` its committed
tables, checked against its pin.

### D2 — French-native readers unchanged

en-fr stays first; a French-native reader's pairs are en-fr and es-fr, their packages hold one more
pack they never load, and the English and Spanish baselines (S0, the es-fr golden) do not move.

### D3 — The dogfood pass, before merge

On Chrome and Firefox (macOS), Firefox for Android, Safari (macOS, iOS), with a package built from
this branch with the real packs:
1. A new install in a browser in Spanish: the onboarding asks for the native language preset to
   Spanish; the interface reads in Spanish; the extension's description in the browser's pages is
   in Spanish (Safari: in Safari's settings).
2. A English page: highlighted; the card in Spanish (grammar lines, headings, gloss); its level
   estimated where the pack says so.
3. Review and statistics in Spanish; a card syncs to a second device carrying its gloss's language.
4. Translation as the owner settled it (M15).
5. An installed French-native extension updated to this build: nothing asked, nothing moved.

### D4 — The release, a beta ring first

The owner submits to the beta channels first, with the Spanish listings (change 37).

## Risks / Trade-offs

- **A server that does not store the labels** → change 12 holds non-French labels until it does;
  the owner's prerequisite checks the server first anyway.
- **Package size** → one pack more, never loaded by a French-native reader.

## Migration Plan

Merged with the owner's go-ahead; released by the owner through the beta channels.
