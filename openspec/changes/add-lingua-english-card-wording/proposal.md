# add-lingua-english-card-wording — the word card of English-native readers of Spanish, judged on the real es-en pack

## Why

Change 23 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, risk 4. Change 18 (`generalise-lingua-card-wording`) split the word card's grammar
wording into a description that names no language and a renderer per interface language, and
drafted the English renderer — on the French spec's made-up inputs only. Change 21
(`add-lingua-pack-es-en`) commits the es-en tables: the glosses as the English Wiktionary's Spanish section, reduced by `reduce_edition_en.py`. What a reader of
es-en will actually read has not been looked at: the renderer's lines over real Spanish
forms, the glosses as that edition writes them — its senses start in lower case (98.3 %, `capitalised=False`), its long explanatory parentheses are change 21's setting (M20), its translation-table glosses are words — their headings, pages and
row cuts.

Risk 4 asks for "goldens per pair that bound it", and no change owns them: the WASM goldens pin
en-fr's and es-fr's JSON, the extension's word-grammar spec pins French wording on synthetic inputs.
An engine holds one native's packs, so es-en needs a baseline of its own.

## What Changes

- **A es-en baseline** in `crates/lingua-wasm/tests/` (`es_en_baseline.rs`, `baseline/es-en.golden`):
  the real es-en pack built from the committed tables, over `baseline/pages-es.txt` (the Spanish corpus es-fr's golden reads) — the same pages, the
  same lemmas, phrases and es-fr's 28 Spanish word-grammar probes (`tests/support/spanish.rs`), as the reference golden of its studied language — frozen
  and re-blessed only on purpose, as the other two.
- **The wording pinned per pair**: `test/word-card-es-en.spec.ts` renders the golden's
  word-grammar JSON, and a sample of real glosses with their sense runs, through `src/i18n/en/grammar.ts` and
  the card's layout, and pins the lines — the grammar lines, the headings, the pages, the rows.
- **The wording judged on real entries**: the renderer's Spanish-studied tables (tense names,
  their order, the moods named — the preterite, the imperfect, the present subjunctive, the gerund) corrected where the real forms read wrong in
  the English Wiktionary's form-of wording (change 18 D4); the card's French-only text heuristics generalised where an English gloss meets
  them (the row cut's trailing quotes “ ” and ‘ ’, the empty-sense pattern).
- **Dogfood on a development build** listing es-en (a local `packs.json`, never committed):
  twenty pages read with the interface in English; what reads wrong is fixed here, or listed for
  change 33 when it is the owner's wording call.
- **No new data**: usage, register and regional labels are not in the packs (the reducers keep the
  definition text alone); adding them would be new pack data, out of scope.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *The card of English-native readers of Spanish is pinned on the real pack*. The
  requirements change 18 adds or modifies are read as written; archived after changes 18, 21
  and 14.

## Impact

- **Products.** Cymbra Lingua only: `crates/lingua-wasm/tests` (a baseline and its support),
  `apps/lingua-extension` (`src/i18n/en/grammar.ts`, `src/reading/selection-card.ts` and `wordpopup.ts` where a
  heuristic is French-only, a new spec). en-fr's and es-fr's goldens, packs and wording do not move.
  ID, Music, Live, the back office and the site are untouched.
- **Nothing shown to anyone**: es-en ships with change 34.
- **Order.** After changes 18 (the renderer), 21 (the tables) and 14 (the card's copy). Before change 34.
- **Not here.** The card's other copy (14); new pack data; the owner's corrections after dogfood on
  the five targets (33).
