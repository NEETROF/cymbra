# add-site-lingua-matrix-pages — the Lingua page describes every shipped pair, in each reader's language

## Why

Change 30 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, with M11's `/es/lingua`. The site's Lingua page (`/lingua/`, `/en/lingua/`) was
written for one audience: French speakers learning English and Spanish. Its languages are a
hard-coded map (`{'en-fr': 'Anglais', 'es-fr': 'Espagnol'}`), its coverage table is headed "a
French gloss", the English page says the product is "made for French speakers… the interface and
the translations are in French", and its translation note says extended translation serves English, and
Spanish "through English". Its figures already come from the committed tables, per pair
(`src/data/lingua-coverage.json`, written by `gloss_coverage.py` for the pairs `packs.json`
ships), so the data is ready for more pairs; the words around it are not.

When es-en ships (change 34), an English speaker learning Spanish reads `/en/lingua/`; when en-es
ships (change 35), a Spanish speaker reads `/es/lingua/`. Neither should read a page written for
French speakers.

## What Changes

- **The page is built from the shipped pairs**: for each pair of `packs.json`, its studied
  language, its native language, its coverage figures ("a gloss in <native>") and whether extended
  translation serves it, directly or through English (from the catalogue's routes). The language
  names come from one table per site language.
- **Each language's page leads with its readers' pairs**: `/en/lingua/` first presents the pairs
  glossed in English, `/lingua/` those glossed in French, `/es/lingua/` those glossed in Spanish;
  the other pairs follow, named for what they are.
- **`/es/lingua/` is built once a pair glossed in Spanish ships**, as the manifest's `_locales`
  (change 27): deploying the site never publishes a Spanish Lingua page for a product no Spanish
  speaker can use yet. Likewise the English page's audience sentence names the English-glossed
  pairs only once one ships.
- **No byte moves while only French-native pairs ship** for `/lingua/` and `/en/lingua/`; the
  English page's quoted French buttons (« Je connais », « + Deck ») are change 34's to replace when it
  ships the English interface.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `site-lingua-page`: MODIFIED *The Lingua page names its languages and publishes their coverage*
  (held by no open change): per pair, in each site language, from the shipped pairs; every scenario
  kept (*Two languages* and *Translation for English only* reworded per pair), three added. Archived after change 29 (the Spanish locale).

## Impact

- **Products.** The site only: `apps/site/src/pages/{lingua,en/lingua,[locale]/lingua}.astro`, a
  shared component and its data reader (`lingua-coverage.json`, the extension's
  `model-manifest.json` routes), `stores.ts` and `Community.astro` in Spanish, and `site-check.yml`'s
  filter watching the catalogue. The extension, ID and Music are untouched.
- **Order.** After change 29. Its pages change what they say when changes 34 and 35 list their
  pairs, with no edit here; the owner deploys the site with those releases (M18).
- **Not here.** The listings (36, 37); the figures themselves (`gloss_coverage.py`, unchanged).
