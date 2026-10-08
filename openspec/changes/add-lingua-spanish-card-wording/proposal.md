# add-lingua-spanish-card-wording — the word card of Spanish-native readers of English, judged on the real en-es pack

## Why

Change 24 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, risk 4. Change 18 (`generalise-lingua-card-wording`) split the word card's grammar
wording into a description that names no language and a renderer per interface language, and
drafted the Spanish renderer — on the French spec's made-up inputs only. Change 22
(`add-lingua-pack-en-es`) commits the en-es tables: the glosses as the Spanish Wiktionary's English section, with the English Wiktionary's Spanish translation tables and the Spanish Wiktionary's English ones read backwards, reduced by `reduce_edition_es.py`. What a reader of
en-es will read has not been looked at: the renderer's lines over real English forms, the
glosses as that edition writes them — its senses are capitalised (99.0 %), its pointers « Forma … de » are dropped, and many glosses are translation-table words rather than definitions (change 22 measures the share) — their headings, pages and row cuts.

Risk 4 asks for "goldens per pair that bound it", and no change owns them: the WASM goldens pin
en-fr's and es-fr's JSON, the extension's word-grammar spec pins French wording on made-up inputs.
An engine holds one native's packs, so en-es needs a baseline of its own — run where the others
are run, re-blessed where they are re-blessed.

## What Changes

- **An en-es baseline** in `crates/lingua-wasm/tests/` (`en_es_baseline.rs`, `baseline/en-es.golden`):
  the real en-es pack built from the committed tables, over en-fr's corpus and probes
  (`baseline/pages.txt`, the 32 English grammar probes) and 40 more lemmas asked as grammar probes, so
  that their glosses and sense runs are the engine's; run by `lingua-extension-check`'s invariance
  step and re-blessed by `lingua-pack-update`, as the other two.
- **The wording pinned per pair**: `test/word-card-en-es.spec.ts` reads every probe from the
  committed golden and renders it through `src/i18n/es/grammar.ts` and the card's layout functions with the
  interface in Spanish, pinning the lines in a committed snapshot re-blessed with the golden.
- **The wording judged on real entries**: the renderer's English-studied tables (tense names and their order, the -ing form's name, the past participle's)
  corrected where the real forms read wrong.
- **The row cut**: the Spanish opening marks ¿ ¡ added to change 23’s rule if en-es rows end on one, a closing mark never stripped.
- **Dogfood on a development build** listing en-es beside the French pairs (a local `packs.json`,
  never committed), signed out or on a test account: what reads wrong is fixed here — wording, or a
  rule of the edition or the pair's reducer that re-pins en-es alone — or listed for change 33 or a
  named follow-up.
- **No new data**: usage, register and regional labels are not in the packs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *The card of Spanish-native readers of English is pinned on the real pack*. The
  requirements change 18 adds or modifies are read as written.

## Impact

- **Products.** Cymbra Lingua only: `crates/lingua-wasm/tests` (a baseline, its support, the
  cross-native check), the two workflows' baseline lines, `apps/lingua-extension` (`src/i18n/es/grammar.ts`,
  `src/reading/selection-card.ts` (the row cut), a new spec and its snapshot, change 18's
  `word-grammar-es.spec.ts` moving with each correction). en-fr's and es-fr's goldens, packs and wording do not
  move. ID, Music, Live, the back office and the site are untouched.
- **Nothing shown to anyone**: en-es ships with change 35.
- **Order.** After changes 18 (the renderer), 20 (the choice D5 needs), 22 (the tables) and 14 (the card's copy), 23 (the row cut's rules). Before change 35.
- **Not here.** The card's other copy and the studied words' `lang` (14, 18); new pack data; the
  owner's corrections after dogfood on the five targets (33).
