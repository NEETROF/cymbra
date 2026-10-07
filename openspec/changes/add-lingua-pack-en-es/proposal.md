# add-lingua-pack-en-es — English glossed in Spanish: the first pair of a native language the shipped packs do not speak

## Why

Change 22 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the second of stage 2, decision M1: en-es is the second new audience, Spanish speakers studying
English. Its studied side exists (`tables/en/`, English's reference en-fr); its glosses do not:
no shipped pack is glossed in Spanish, and the Spanish Wiktionary's English section is small —
22,965 entries, 35,163 senses, 9.8 % pointers (the census of change 6, whose
`reduce_edition_es.py` already holds its rules). The study measured 93.4 / 85.2 / 71.9 % of the
commonest 5,000 / 10,000 / 20,000 English lemmas glossed when the English Wiktionary's own
translation tables — the Spanish translations its English entries list, written by people (M5)
— fill the gaps: a source no pair has read, which needs the English Wiktionary's English entries,
an extract no step has fetched (risk 6).

Change 21 settled the shape of a reader pair: the native side alone, from the committed studied
tables and the pair's own sources, a digest of its own, a pin recording the studied tables it
read, sources shared by release. This change reuses it and adds what en-es needs alone: the
Spanish edition bound to an English studied side, three tables of glosses (entries, direct and
inverted — es-fr's own shape), three derived files from two large dumps, the English extract
read as served, and a coverage floor the owner sets on the pull request, since en-es is the thin pair
risk 5 names.

## What Changes

- **`tables/en-es/`**: the native side only; `tables/en/` is read as committed.
- **`reduce-en-es.py`**: the native side alone, `EDITION = spanish.ES`: entries = the Spanish
  Wiktionary's English section (a derived file, `("entries", "en")`, from es-fr's `kaikki-es`
  whole-edition dump); direct = the English Wiktionary's Spanish translations (a derived file,
  `("translations", "en", "es")`, from the English Wiktionary's English extract); inverted = the
  Spanish Wiktionary's English translations read backwards (`kaikki-es-traductions-en.jsonl`,
  change 21's derived file) — M5 admits both directions, as es-fr uses them. No pivot, no
  machine translation.
- **The English extract as a dump source**: `kaikki.org/dictionary/English/kaikki.org-dictionary-
  English.jsonl`, served uncompressed; `derive` reads a plain or a gzipped file; fetched at an
  update only, derived in one pass, kept nowhere; its derived files published with the snapshot.
  The programme's risk 6 names the raw English dump; this change reads the extract and says why,
  and change 38 switches the address.
- **The pin and the credits** as change 21 settled them, the English studied side's sources
  credited as en-fr's notice credits them (ESDB with its WordNet notice, the French Wiktionary's
  form links, CEFR-J, Octanove, wordfreq); `max_lemmas` 40,000 as en-fr.
- **The pipeline knows it**: a pair whose sources are dumps alone (no studied-side extract), the
  dispatch options and monthly matrix, the reduce job (after en-fr), `pack_report`,
  `testdata/en-es/`.
- **Measured and shown**: gloss coverage against a floor the owner sets on the pull request
  (proposed at the study's figures less two points: 91.4 / 83.2 / 69.9 %), the share of glosses
  that came from a translation table rather than an entry among the glossed lemmas of the top
  10,000, a sample of 100 glosses, the pack's size.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *English is glossed in Spanish from the Spanish Wiktionary's English
  section and the English Wiktionary's translation tables*; MODIFIED *Sources derived from whole
  Wiktionary dumps are pinned* — one sentence: a language's extract is read the same way, plain
  or gzipped; both scenarios kept. Held by no open change. The requirements of change 21 (*A pair
  of a studied language's second native language*, *A reader pair's pin records the studied
  tables it read*) apply as written; this change is archived after it.

## Impact

- **Products.** Cymbra Lingua only: `scripts/lingua-data` (the reducer, `pack_sources.py`, the
  tables, `testdata/en-es/`, SOURCES.md, the READMEs), `.github/workflows/lingua-pack-update.yml`.
  ID, Music, Live, the back office and the site are untouched.
- **No byte moves**: en-fr's, es-fr's and es-en's tables, pins, packs and goldens are untouched;
  `packs.json` is unchanged; the site's figures are unchanged until the pair ships.
- **The reduce job** grows by one pair reduced after en-fr, from small derived assets.
- **Not here.** Shipping en-es (change 35, after change 9); the Spanish card wording (24); the
  en-es model (25) and marks (26); the Spanish interface (13–20); the raw dumps (38).
