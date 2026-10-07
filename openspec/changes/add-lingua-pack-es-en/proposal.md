# add-lingua-pack-es-en — Spanish glossed in English: the first pair of a studied language's second native

## Why

Change 21 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the first of stage 2, decision M1: es-en is the first new audience, English speakers studying
Spanish, and it can ship first — it needs no Spanish site pages and loads no new translation
model (es-en is pinned already). Stage 1 made it a matter of one reducer for the native side:
the studied side of Spanish is kept once (`tables/es/`, change 7) and every other pair reads it
as committed; a pack's dictionary words no longer depend on its glosses (change 5); the English
Wiktionary's rules exist and are tested (`reduce_edition_en.py`, change 6); an engine holds one
native's packs (change 4); the translation routes are keyed by pair (change 8).

The glosses come from the English Wiktionary's Spanish section — es-fr's pinned `kaikki` source
already, 811,049 entries, reduced with the English edition's rules — with the Spanish
Wiktionary's English translations as the direct fallback (M5: words people wrote into a
translation table qualify). The study measured 93.6 / 87.0 / 77.1 % of the 5,000 / 10,000 /
20,000 commonest lemmas glossed: above es-fr's published 87.6 / 77.2 / 63.7 %, which this change
makes the pair's floor.

This change commits the pair's native side, its reducer and its pin, and settles what change 7
left to it: what a reader pair's pin records of the studied tables it reads, how its sources
are shared with the reference's, and what its credits say. It does not ship the pack:
`packs.json` is unchanged, so no French reader's package or engine moves, and no English-native
reader exists until the interface speaks English (changes 13–20) and change 34 enables them.

Two settings of the English edition are decided in this pull request, by the owner (M20, risk
4): the long explanatory parentheses of English glosses (≥ 40 characters: 5,724 of 147,653
meaning senses), and the merging of same-part-of-speech etymologies before the round-robin (532
of the top-10,000 es-en lemmas), each shown on a sample of the top 10,000.

## What Changes

- **`tables/es-en/`**: `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json`,
  `README.md` — the native side only; `tables/es/` is read as committed, not computed again.
- **`reduce-es-en.py`**: the native side alone — the English Wiktionary's Spanish section as
  entries, reduced by the English edition (`reduce_edition_en.EN`), the Spanish Wiktionary's
  English translations (a derived file from es-fr's `kaikki-es` dump) as the direct fallback, no
  inverted table — matched against the committed Spanish lemmas and ranks. Its rule digest is its
  own file, `reduce_common.py` and `reduce_edition_en.py`: es-fr's reducer and the French edition
  are not loaded, so nothing of es-fr moves and the rule that a reducer loads code by import
  statements alone stands.
- **Shared sources, one release per pair**: es-en's pin points at es-fr's release asset for the
  English extract (a pin may name another pair's release; the live fetch reuses the reference's
  pinned extract instead of downloading it again) and at its own release for its derived file;
  the update publishes a pair's own assets only; the reduce job keeps fetched assets in a cache
  across pairs, so the extract is fetched once per job.
- **A reader pair's pin records the studied tables it read**: the sha256 of each of the six
  studied tables, written by `record-build`, checked by `check-reducer` and named by
  `pack_report` — the "pair left behind" that the pack's sha256 already catches, now named by
  table. `pack_version` is the pair's own snapshot and digest.
- **The pipeline knows it**: `KAIKKI`/`DUMPS` entries, the dispatch options and the monthly matrix,
  the reduce job (after es-fr), `pack_report`, the committed-tables check's file set,
  `testdata/es-en/`, `cross_native.rs` on the real pack.
- **The two settings**: the long-parenthesis bound is `EN`'s `long_parenthesis`; the etymology
  merging is a pre-pass in `reduce_edition_en.py` that `reduce-es-en.py` calls before building the
  native tables — neither touches `reduce_common.py`, so no French-native pair is re-pinned.
- **Measured and shown**: `gloss_coverage.py --pair es-en --floor` against es-fr's published
  figures (the reduce job runs it; the site is unchanged until the pair ships), a sample of 100
  glosses, the pack's size (bound 5 MiB).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *A pair of a studied language's second native language*, *A reader
  pair's pin records the studied tables it read*, *Spanish is glossed in English from the English
  Wiktionary's Spanish section*. *Only the reference pair's reduction writes its studied language's
  tables* stands as written: a reader pair reads them as committed. *The shipped pairs are one
  list* (held by `enable-lingua-spanish`) is untouched: the pair is not shipped here.

## Impact

- **Products.** Cymbra Lingua only: `scripts/lingua-data` (the reducer, `pack_sources.py`,
  `build.sh`, `gloss_coverage.py`, `pack_report.py`, the tables, `testdata/es-en/`, SOURCES.md, the
  READMEs), `.github/workflows/{lingua-pack-update,lingua-extension-check}.yml`,
  `crates/lingua-pack/tests` (the file set), `crates/lingua-wasm/tests` (cross-native). ID, Music,
  Live, the back office and the site are untouched; `apps/lingua-extension` is untouched.
- **No byte moves**: es-fr's and en-fr's tables, pins, packs and goldens are untouched;
  `packs.json` is unchanged; `lingua-coverage.json` is unchanged until the pair ships.
- **The reduce job** grows by one pair reduced after es-fr, from the cached extract.
- **Not here.** Shipping es-en (change 34: `packs.json`, the listings); the English card wording
  (23); the es-en marks (26); the English interface (13–20).
