# add-lingua-pack-es-en — Spanish glossed in English: the first pair of a studied language's second native

## Why

Change 21 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the first of stage 2, decision M1: es-en is the first new audience, English speakers studying
Spanish, and it can ship first — it needs no Spanish site pages and loads no new translation
model (es-en is pinned already). Stage 1 made it a matter of one reducer: the studied side of
Spanish is kept once (`tables/es/`, change 7) and reused byte for byte; a pack's dictionary
words no longer depend on its glosses (change 5); the English Wiktionary's rules exist and are
tested (`reduce_edition_en.py`, change 6); an engine holds one native's packs (change 4); the
translation routes are keyed by pair (change 8).

The glosses come from the English Wiktionary's Spanish section — es-fr's pinned `kaikki` source
already, 811,049 entries, reduced with the English edition's rules — with the Spanish
Wiktionary's English translations as the direct fallback (M5: words people wrote into a
translation table qualify). The study measured 93.6 / 87.0 / 77.1 % of the 5,000 / 10,000 /
20,000 commonest lemmas glossed: above es-fr's published 87.6 / 77.2 / 63.7 %.

This change commits the pair's tables, its reducer and its pin, and settles what change 7 left
to it: what a non-reference pair's pin records of the studied tables it reads, how its
`pack_version` is formed, and what its credits say. It does not ship the pack: `packs.json` is
unchanged, so no French reader's package or engine moves, and no English-native reader exists
until the interface speaks English (changes 13–20) and change 34 enables them.

Two measurements are decided in this pull request, by the owner (M9, M20): the long explanatory
parentheses of English glosses (≥ 40 characters: 5,724 of 147,653 meaning senses), and the
merging of same-part-of-speech etymologies before the round-robin (532 of the top-10,000 es-en
lemmas), each shown on a sample.

## What Changes

- **`tables/es-en/`**: `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json`,
  `README.md` — the native side only; `tables/es/` is read, not written.
- **`reduce-es-en.py`**: the Spanish studied side computed by `reduce-es-fr.py`'s code, loaded
  by import so that es-fr's own rule digest does not move; the native side bound to the English
  edition (`reduce_edition_en.EN`): the English Wiktionary's Spanish section as entries, the
  Spanish Wiktionary's English translations (derived from es-fr's `kaikki-es` dump, a new
  derived file) as the direct fallback; no inverted table.
- **A non-reference pair's pin**: `studied` — the reference pair, its snapshot and the sha256 of
  each studied table read — so a diff shows when the studied side moved; `pack_version` is the
  pair's own snapshot and rule digest, which includes the reference's rules it loads, so a change
  to es-fr's rules re-pins es-en as the reduce job's `pairs --after` re-reduces it; the credits
  name both sides' sources.
- **The pipeline knows it**: `KAIKKI` and `DUMPS` entries for es-en (the entries file shared with
  es-fr's snapshot; the new derived translations file), `lingua-pack-update`'s dispatch options
  and monthly matrix, `max_lemmas` 60,000 for a Spanish pair, the reduce job (which lists it
  after es-fr), `pack_report`, the committed-tables check (its exact file set).
- **Fixtures**: `testdata/es-en/` so `gen:pack` can build it; `cross_native.rs` compares the real
  es-en pack with the reference's studied side instead of a synthetic one.
- **Measured and shown**: gloss coverage (`gloss_coverage.py --pair es-en`, not published on the
  site until the pair ships), the two samples for the owner, the pack's size (bound 5 MiB).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *A pair of a studied language's second native language*, *A
  non-reference pair's pin records what it read*, *Spanish is glossed in English from the English
  Wiktionary's Spanish section*. *The shipped pairs are one list* (held by `enable-lingua-spanish`)
  is untouched: the pair is not shipped here.

## Impact

- **Products.** Cymbra Lingua only: `scripts/lingua-data` (the reducer, `pack_sources.py`,
  `build.sh`, the tables, SOURCES.md, the READMEs), `.github/workflows/lingua-pack-update.yml`,
  `crates/lingua-pack` (the committed-tables check's file set), `crates/lingua-wasm/tests`
  (cross-native), `apps/lingua-extension/scripts`… no: `apps/lingua-extension` only for
  `testdata` through `gen_pack.sh`. ID, Music, Live, the back office and the site are untouched.
- **No byte moves**: es-fr's and en-fr's tables, pins, packs and goldens are untouched (es-en
  loads es-fr's rules without editing them); `packs.json` is unchanged; `lingua-coverage.json` is
  unchanged until the pair ships.
- **The reduce job** grows by one pair reduced after es-fr (≈ 30 s, the sources shared).
- **Not here.** Shipping es-en (change 34: `packs.json`, the listings); the English card wording
  (23); the es-en marks (26); the English interface (13–20).
