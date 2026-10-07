# add-lingua-pack-en-es — English glossed in Spanish: the first pair of a native language the shipped packs do not speak

## Why

Change 22 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the second of stage 2, decision M1: en-es is the second new audience, Spanish speakers studying
English. Its studied side exists (`tables/en/`, English's reference en-fr); its glosses do not:
no shipped pack is glossed in Spanish, and the Spanish Wiktionary's English section is small —
22,965 entries, 35,163 senses, 9.8 % pointers (the census of change 6, whose `reduce_edition_es.
py` already holds its rules). The study measured 93.4 / 85.2 / 71.9 % of the commonest 5,000 /
10,000 / 20,000 English lemmas glossed when the English Wiktionary's own translation tables —
the Spanish translations its English entries list, written by people (M5) — fill the gaps: a
source no pair has used, which needs the English Wiktionary's English entries, a 2 GB extract no
pipeline step has fetched (risk 6: the per-language extracts are deprecated but served; the raw
dump is the fallback).

Change 21 settled the shape of a reader pair: the native side committed, the studied side loaded
from the reference's reducer by import, the pin recording what was read, the digest covering the
reference's rules, the credits naming both sides. This change reuses it and adds what en-es needs
alone: the Spanish edition bound to an English studied side, two new derived files, and a gloss
coverage published with the pair when it ships (change 35), with its floor fixed before the
measurement is committed (M6's precedent for a thin pair).

## What Changes

- **`tables/en-es/`**: the native side only; `tables/en/` is read, not written.
- **`reduce-en-es.py`**: en-fr's English studied side loaded by import; the native side bound to
  the Spanish edition (`reduce_edition_es.ES`): the Spanish Wiktionary's English section as
  entries — a new derived file, `("entries", "en")`, from es-fr's `kaikki-es` whole-edition dump —
  with the English Wiktionary's Spanish translations as the direct fallback — a new derived file,
  `("translations", "en", "es")`, from the English Wiktionary's English extract (`kaikki.org/
  dictionary/English`), derived once in `fetch_live` and published as a release asset, so a pinned
  reduction fetches ≈ 10 MB, not 2 GB. No inverted table, no pivot, no machine translation.
- **The pin, digest and credits** as change 21 settled them; `max_lemmas` 40,000 as en-fr.
- **The pipeline knows it**: `KAIKKI`/`DUMPS` entries (a new dump source, the English extract, with
  its licence line), the dispatch options and monthly matrix, the reduce job (after en-fr),
  `pack_report`, `testdata/en-es/`.
- **Measured and shown**: gloss coverage (`--pair en-es`, not published until the pair ships), the
  share of glosses that are translation-table words rather than definitions, a sample of 100
  glosses from the top 10,000 for the owner (M9), the pack's size.
- **A floor for a thin pair** (M6's rule, applied to en-es as it will be to fr-es): the three
  coverage figures the pair must reach to ship, fixed in this change's spec at the study's
  measurement less two points, so that a regression in the sources is caught before change 35.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *English is glossed in Spanish from the Spanish Wiktionary's English
  section and the English Wiktionary's translation tables*, *A source derived from a large dump
  is fetched as its derived files*. The requirements of change 21 (*A pair of a studied language's
  second native language*, *A non-reference pair's pin records what it read*) apply as written;
  this change is archived after it.

## Impact

- **Products.** Cymbra Lingua only: `scripts/lingua-data` (the reducer, `pack_sources.py`, the
  tables, SOURCES.md, the READMEs), `.github/workflows/lingua-pack-update.yml`, `testdata`. ID,
  Music, Live, the back office and the site are untouched.
- **No byte moves**: en-fr's, es-fr's and es-en's tables, pins, packs and goldens are untouched;
  `packs.json` is unchanged; the site's figures are unchanged until the pair ships.
- **The reduce job** grows by one pair reduced after en-fr (the derived files are small; the
  English extract is fetched only by `fetch_live`, at an update).
- **Not here.** Shipping en-es (change 35, after change 9); the Spanish card wording (24); the
  en-es model (25) and marks (26); the Spanish interface (13–20).
