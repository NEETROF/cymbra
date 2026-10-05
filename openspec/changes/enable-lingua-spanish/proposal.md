# enable-lingua-spanish — Spanish reading, published

## Why

The Spanish programme's internal build (G1) is complete:
- the es-fr pack's forms, grammar, French glosses and estimated levels;
- Spanish analysis and its Catalan and Galician guard;
- the word card's Spanish names;
- a voice of Spain for read-aloud.

The platform was generalised for it in R2 and R3: a pack per pair, a language per document, the
language choice, statistics, review and sync per language. Every surface shows its language
controls once a package ships more than one pair. Yet every package still ships en-fr alone:
`packs.json` lists it, and `check_variants` refuses any other list « until enable-lingua-spanish ».

The server keys cards by language already (`add-lingua-card-language`, backend 0.35.0, deployed
2026-09-30), so a Spanish card never reaches an older client: the programme's « server first » rule
is met.

This is change 28, R4 in `docs/lingua/spanish-programme.md`: Spanish reading, published.

## What Changes

- **Both pairs ship.** `packs.json` lists `en-fr` and `es-fr`, en-fr first, so English stays the
  default studied language. `check_variants`'s `SHIPPED_PAIRS` follows, in the same pull request as
  its gate requires.
- **A testdata fixture for es-fr**, which `yarn gen:pack` builds for dogfooding and the checks, as
  en-fr's.
- **The manifest summary names both languages**, within 112 characters. The wording, settled by
  the owner on 2026-10-05: « Lisez l'anglais et l'espagnol sur le web : mots inconnus surlignés,
  pourcentage honnête. Hors ligne et privé. »
- **What readers see** (built in R3, shown once two pairs ship): « Langues étudiées » in Réglages
  and in onboarding. Once a reader accepts Spanish, they also get a level per language, the
  statistics' language selector and the review's language filter.
- **A dogfood pass on five targets** before merge: Chrome and Firefox on macOS, Firefox for
  Android, Safari on macOS and on iOS.
- **A beta ring first.** The owner gives the go-ahead before the merge and before each store
  submission.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED — *The shipped pairs are one list*: the list holds en-fr and es-fr,
  and the variant check refuses a list it does not name.

## Impact

- **Products.** Cymbra Lingua's extension and its Safari host app:
  - `apps/lingua-extension/packs.json`, `tool/check_variants.mjs` and `manifest.json`;
  - `scripts/lingua-data/testdata/es-fr/`.

  Each package grows by the es-fr pack (2.19 MB). Memory does not grow for a reader of English
  alone: an engine loads a pair's pack the first time its language is needed
  (`package-lingua-packs-per-pair`).
- **The owner**:
  - the go-ahead before merge;
  - the manifest summary's wording (settled 2026-10-05);
  - the dogfood pass on devices;
  - the beta ring and every store submission;
  - the kaikki snapshot release named in `tables/es-fr/pin.json` (published 2026-10-05).
- **Release.** R4: the first Spanish-capable packages. Nothing is submitted by this pull request.
