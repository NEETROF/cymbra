# add-lingua-pack-fr-en — French glossed in English: fr-en's native side, and the French baseline on it

## Why

Change 48 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
stage 3 (French studied: fr-en, fr-es). Change 43 (`add-lingua-french-forms-tables`) makes fr-en
French's reference pair: `reduce-fr-en.py` writes `tables/fr/` from the English Wiktionary's French
section, and commits `tables/fr-en/` with an empty `gloss.tsv` and an empty `tables/fr/lexical.tsv`
— « the glosses, expressions and dictionary words (48) ». Until they exist, no French word has an
English gloss, French has no dictionary word, and change 39's French invariance baseline runs over
a hand-written fixture of 286 forms and 70 glosses (`scripts/lingua-data/testdata/fr-en/`), « until
change 48 commits `tables/fr/` and `tables/fr-en/` ».

The programme names the source: « fr-en | 93.9 / 87.1 / 76.4 % (prototype ranks, ± 1–2) | English
Wiktionary, French section ». It is the file French's forms already come from
(`kaikki-French.jsonl`, 403,269 entries), and its glosses are the English Wiktionary's, which es-en
reads already: the English edition's rules (`reduce_edition_en.py`, change 6, change 21) and change
23b's pre-pass, `read_as_meanings`, which reads es-en's senses as meanings and not as the page's
layout. Change 38's catalogue also registered two translation tables « for change 48 »: the French
Wiktionary's English translations (direct) and the English Wiktionary's French translations, read
backwards (inverted).

This change prototyped fr-en's native side on the real data — change 43's implementation tables
(124,040 forms, 60,000 lemmas), the French section of 2026-10-03, both translation tables, the
committed rules — and measured what each source and each rule does (design, *Measured*). The French
section alone glosses 4,678 / 8,686 / 15,256 of the 5,000 / 10,000 / 20,000 commonest lemmas
(93.6 / 86.9 / 76.3 %, inside the programme's ± 1–2). The translation tables add 1,988 lemmas,
1,895 of them words the section has no entry for — English words, names, initialisms, unaccented
misspellings — and 1,193 listing the word itself among their translations (« in » « in », « jack »
« jack »); and fr-en is French's reference, so every lemma it glosses becomes a French dictionary
word (*A pack's dictionary words do not depend on its glosses*), counted by the vocabulary estimate
and kept as a word by a names rule.
23b's rules, run on fr-en, move 269 rows (125 of the top 10,000), gain 2 glosses and lose none.

## What Changes

- **fr-en's native side, from the English Wiktionary's French section alone** (design D1, D3):
  `reduce-fr-en.py` — French's reference reducer, change 43's — gains `gloss.tsv`, `senses.tsv` and
  `mwe.tsv`, read from the same derived file as French's forms, through the English edition's rules
  as es-en's are (its letters, the single-capital headwords, `read_as_meanings`, the etymology
  merging and the long-parenthesis bound at their committed settings) before the shared ones
  (`reduce_common.native_tables`), a typographic apostrophe in a headword read as `'` as French's
  forms are. No shared module is edited: es-en, en-fr, es-fr and en-es do not move.
- **23b's rules, measured on fr-en and kept** (D2): sense-group labels (« chambre » no longer « a
  chamber in its various senses, including »), a pronoun's senses nested under its description
  (« nous » « we; us, to us »), the edition's descriptions in lower case (« du » « forms the
  partitive article »), one ellipsis, curly quotes. No new rule of the English edition (it would
  re-pin es-en) and no fr-en-only rule: what still reads wrong is listed (D7) for a refinement
  before change 52, as 23b was for es-en before 34.
- **No translation table** (D3): measured and declined for words and for expressions — the
  inverted table makes French's commonest bigrams expressions (« il est » "he's", 782 times in
  425,111 words of UD French) — and the two entries change 38's catalogue holds for them are
  removed; adopting the French Wiktionary's table for expressions is left to the owner (Open
  Question 1).
- **French's dictionary words** (D5): `pack_sources.py split` writes `tables/fr/lexical.tsv`, fr-en's
  30,059 glossed lemmas; the fr-en pack carries no lexical table (its dictionary words are its
  glosses), and fr-es (49) reads them as committed.
- **Re-reduced from change 43's pin, at its snapshot** (D5): no source fetched, the pin's snapshot and
  sources byte for byte, every other studied table byte for byte; its rule digest now names
  `reduce_edition_en.py`, which es-en and fr-en read from here on.
- **Measured against a floor, not shipped** (D4, D6): `FLOORS["fr-en"]` = 91.9 / 85.1 / 74.4, the
  study's figures less two points, as en-es's; the reduce job runs `gloss_coverage.py --pair fr-en`;
  the pack is 2,201,349 B with change 43's two tables (2,421,321 B with changes 45's and 46's
  prototype tables), under the 5 MiB budget; `packs.json`, the site and the listings are unchanged
  until change 52; fr-en's rows join the row cut's check over every committed gloss.
- **The hand-over of change 39's baseline** (D8): `support/french.rs` builds fr-en's pack from the
  committed tables (`PackSource::Tables`); `fr-en.golden` is re-blessed once, in this pull request,
  which reviews it probe kind by probe kind; the fixture stays for the tests that build it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *French is glossed in English from the English Wiktionary's French
  section* and *fr-en is committed at its studied tables' snapshot, and the French baseline runs on
  it*. Nothing is modified. The requirements this change builds on are open changes': the English
  edition's rules and settings (`add-lingua-pack-es-en`, *Spanish is glossed in English from the
  English Wiktionary's Spanish section*, its scenario *A setting of the English edition* still true),
  the pre-pass (`refine-lingua-es-en-glosses`), French's tables and reference pair
  (`add-lingua-french-forms-tables`), the catalogue (`migrate-lingua-pack-sources-to-raw-dumps`),
  and the French baseline's hand-over, its scenario *The committed tables replace the fixture* read
  as « when fr-en's glosses are committed » (`add-lingua-french-baseline`): all five are in
  `archiveAfter`. Three sentences of those changes are true of their time and not after this one —
  23b's « which only es-en reads: they re-pin es-en alone », 43's scenario *The reference pair
  writes French's folder* (« `tables/fr/lexical.tsv` [is] empty, and `tables/fr-en/gloss.tsv` is
  empty »), 38's scenario *A pair of stage 3 registers what it reads* (fr-en with three sources) —
  and their words are best amended when those changes are archived (Open Question 4).
- `lingua-analysis`: none. Change 39's requirement *A French invariance baseline runs beside the
  English and Spanish ones* already says what this change does to the baseline.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *changed*: `reduce-fr-en.py` (the native side) and its tests,
    `tables/fr-en/` (`gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json`,
    `README.md`), `tables/fr/lexical.tsv`, `pack_sources.py` (two catalogue entries removed),
    `gloss_coverage.py` (`FLOORS["fr-en"]`), `test_reduce_editions.py` (the English edition's
    digest now fr-en's too), `SOURCES.md`; *consumed*: `reduce_common.py`, `reduce_edition_en.py`,
    unchanged.
  - `crates/lingua-wasm/tests` — *changed*: `support/french.rs` (the pack source), `french_baseline.rs`
    (its doc and the `mémoire` test), `baseline/fr-en.golden` re-blessed; *consumed*: the harness.
  - `crates/lingua-pack/tests/committed_tables.rs` — fr-en's file set and dictionary words.
  - `apps/lingua-extension/test/row-gloss-tables.spec.ts` — fr-en among the pairs whose rows are
    checked; no source, bundle or package changes.
  - `.github/workflows/lingua-extension-check.yml` — the reduce job holds fr-en to its floor.

  ID, Music, Live, the back office, the site, the backend, lingua-core, the engine, the Apple host
  app and the agent plugin are untouched.
- **Licences.** The English Wiktionary (CC BY-SA 4.0 + GFDL), already credited by fr-en for the forms,
  is credited for the glosses too; no source is added.
- **Release.** Silent: no package lists fr-en before change 52.
- **What does not move.** en-fr, es-fr, es-en and en-es — tables, pins, packs, goldens and the
  extension's snapshots — byte for byte; `tables/fr/`'s forms, ranks, readings and levels byte for
  byte; the French golden's `beside es-en` line.
- **What moves.** fr-en's pin (rule digest, pack), its manifest's `pack_version`, `tables/fr/lexical.tsv`
  (0 → 30,059 words), and `fr-en.golden`: measured, 120 of its 136 probes move when changes 45 and
  46 have landed, 127 when they have not, the level probes then going blank (design D8).
- **Order.** After 39 and 43 (required), after 45 and 46 (planned, the programme's order, so that the
  hand-over keeps the golden's level probes answering); before 49 (fr-es reads French's dictionary
  words), 51 and 52. 40, 41, 42 and 44 may land on either side.
- **Effort, against 2–3.5 ideal days**: 2–3.5 (design, *Effort*).
