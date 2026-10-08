# add-lingua-english-card-wording — the word card of English-native readers of Spanish, judged on the real es-en pack

## Why

Change 23 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, risk 4. Change 18 (`generalise-lingua-card-wording`) split the word card's grammar
wording into a description that names no language and a renderer per interface language, and
drafted the English renderer — on the French spec's made-up inputs only. Change 21
(`add-lingua-pack-es-en`) commits the es-en tables: the glosses as the English Wiktionary's Spanish section, reduced by `reduce_edition_en.py`. What a reader of
es-en will read has not been looked at: the renderer's lines over real Spanish forms, the
glosses as that edition writes them — its senses start in lower case (98.3 %), its long explanatory parentheses are change 21's setting (M20), its translation-table glosses are words — their headings, pages and row cuts.

Risk 4 asks for "goldens per pair that bound it", and no change owns them: the WASM goldens pin
en-fr's and es-fr's JSON, the extension's word-grammar spec pins French wording on made-up inputs.
An engine holds one native's packs, so es-en needs a baseline of its own — run where the others
are run, re-blessed where they are re-blessed.

## What Changes

- **An es-en baseline** in `crates/lingua-wasm/tests/` (`es_en_baseline.rs`, `baseline/es-en.golden`):
  the real es-en pack built from the committed tables, over es-fr's corpus and probes
  (`baseline/pages-es.txt`, the 28 Spanish grammar probes) and 40 more lemmas asked as grammar probes, so
  that their glosses and sense runs are the engine's; run by `lingua-extension-check`'s invariance
  step and re-blessed by `lingua-pack-update`, as the other two.
- **The wording pinned per pair**: `test/word-card-es-en.spec.ts` reads every probe from the
  committed golden and renders it through `src/i18n/en/grammar.ts` and the card's layout functions with the
  interface in English, pinning the lines in a committed snapshot re-blessed with the golden.
- **The wording judged on real entries**: the renderer's Spanish-studied tables (tense names and their order, the gerund's name, which moods are named)
  corrected where the real forms read wrong.
- **The row cut**: its rules owned here — the opening marks “ ‘ added only if es-en rows end on one, a closing mark never stripped, every French row held by a snapshot.
- **Dogfood on a development build** listing es-en beside the French pairs (a local `packs.json`,
  never committed), signed out or on a test account: what reads wrong is fixed here — wording, or a
  rule of the edition or the pair's reducer that re-pins es-en alone — or listed for change 33 or a
  named follow-up.
- **No new data**: usage, register and regional labels are not in the packs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *The card of English-native readers of Spanish is pinned on the real pack*. The
  requirements change 18 adds or modifies are read as written.

## Impact

- **Products.** Cymbra Lingua only: `crates/lingua-wasm/tests` (a baseline, its support, the
  cross-native check), the two workflows' baseline lines, `apps/lingua-extension` (`src/i18n/en/grammar.ts`,
  `src/reading/selection-card.ts` (the row cut), a new spec and its snapshot, change 18's
  `word-grammar-en.spec.ts` moving with each correction). en-fr's and es-fr's goldens, packs and wording do not
  move. ID, Music, Live, the back office and the site are untouched.
- **Nothing shown to anyone**: es-en ships with change 34.
- **Order.** After changes 18 (the renderer), 20 (the choice D5 needs), 21 (the tables) and 14 (the card's copy). Before change 34.
- **Not here.** The card's other copy and the studied words' `lang` (14, 18); new pack data; the
  owner's corrections after dogfood on the five targets (33).
