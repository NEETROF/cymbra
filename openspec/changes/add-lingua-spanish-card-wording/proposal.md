# add-lingua-spanish-card-wording — the word card of Spanish-native readers of English, judged on the real en-es pack

## Why

Change 24 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, risk 4. Change 18 (`generalise-lingua-card-wording`) split the word card's grammar
wording into a description that names no language and a renderer per interface language, and
drafted the Spanish renderer — on the French spec's made-up inputs only. Change 22
(`add-lingua-pack-en-es`) commits the en-es tables: the glosses as the Spanish Wiktionary's English section, with the English Wiktionary's Spanish translation tables and the Spanish Wiktionary's English ones read backwards, reduced by `reduce_edition_es.py`. What a reader of
en-es will actually read has not been looked at: the renderer's lines over real English
forms, the glosses as that edition writes them — its senses are capitalised (99.0 %), its pointers « Forma … de » are dropped, and many glosses are translation-table words rather than definitions (change 22 measures the share) — their headings, pages and
row cuts.

Risk 4 asks for "goldens per pair that bound it", and no change owns them: the WASM goldens pin
en-fr's and es-fr's JSON, the extension's word-grammar spec pins French wording on synthetic inputs.
An engine holds one native's packs, so en-es needs a baseline of its own.

## What Changes

- **A en-es baseline** in `crates/lingua-wasm/tests/` (`en_es_baseline.rs`, `baseline/en-es.golden`):
  the real en-es pack built from the committed tables, over `baseline/pages.txt` (the English corpus en-fr's golden reads) — the same pages, the
  same lemmas, phrases and en-fr's 32 English word-grammar probes (`tests/support/english.rs`), as the reference golden of its studied language — frozen
  and re-blessed only on purpose, as the other two.
- **The wording pinned per pair**: `test/word-card-en-es.spec.ts` renders the golden's
  word-grammar JSON, and a sample of real glosses with their sense runs, through `src/i18n/es/grammar.ts` and
  the card's layout, and pins the lines — the grammar lines, the headings, the pages, the rows.
- **The wording judged on real entries**: the renderer's English-studied tables (tense names,
  their order, the moods named — « pasado simple », « forma en -ing » (M10), « participio ») corrected where the real forms read wrong in
  RAE/ASALE terms (change 18 D4; M10); the card's French-only text heuristics generalised where an Spanish gloss meets
  them (the row cut's trailing quotes « » and “ ”, the empty-sense pattern).
- **Dogfood on a development build** listing en-es (a local `packs.json`, never committed):
  twenty pages read with the interface in Spanish; what reads wrong is fixed here, or listed for
  change 33 when it is the owner's wording call.
- **No new data**: usage, register and regional labels are not in the packs (the reducers keep the
  definition text alone); adding them would be new pack data, out of scope.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *The card of Spanish-native readers of English is pinned on the real pack*. The
  requirements change 18 adds or modifies are read as written; archived after changes 18, 22
  and 14.

## Impact

- **Products.** Cymbra Lingua only: `crates/lingua-wasm/tests` (a baseline and its support),
  `apps/lingua-extension` (`src/i18n/es/grammar.ts`, `src/reading/selection-card.ts` and `wordpopup.ts` where a
  heuristic is French-only, a new spec). en-fr's and es-fr's goldens, packs and wording do not move.
  ID, Music, Live, the back office and the site are untouched.
- **Nothing shown to anyone**: en-es ships with change 35.
- **Order.** After changes 18 (the renderer), 22 (the tables) and 14 (the card's copy). Before change 35.
- **Not here.** The card's other copy (14); new pack data; the owner's corrections after dogfood on
  the five targets (33).
