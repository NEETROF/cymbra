# add-lingua-pack-en-it — English glossed in Italian, for Italian speakers

## Why

Change 36 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3: Italian
speakers. en-it is the first pair glossed in Italian: an Italian speaker reading English, shipped with
the Italian interface in change 45. Its sources are the Italian Wiktionary's English section, then the
English Wiktionary's Italian translation tables, then the Italian Wiktionary's English translations
read backwards (I11), as en-es reads its own (matrix change 22). Change 35 wrote the Italian
Wiktionary's rules (`reduce_edition_it.py`); en-it is their first reader.

Measured on the committed English ranks, the three sources gloss 91.4 / 82.7 / 70.3 % of the 5,000 /
10,000 / 20,000 commonest English lemmas, against en-es's committed 93.0 / 85.0 / 71.7. On English's
dictionary words the two are level (96.8 % of the 4,373 at the 5,000). The gap is in contractions and
names, which the Italian Wiktionary writes fewer of.

## What Changes

- **A reader pair** (design D1): `reduce-en-it.py` reads English's committed studied tables and writes
  `tables/en-it/`. Its rule digest is its own file, `reduce_common.py` and `reduce_edition_it.py`.
- **The sources in en-es's order** (D2): the section's definitions with the Italian edition's rules, the
  English Wiktionary's Italian tables, then the Italian Wiktionary's English tables read backwards. No
  pivot and no machine translation.
- **en-es's table rules, copied** (D3), not shared, so that en-es's pin does not move:
  - letters out;
  - a word listed once across a row's parts of speech (19 / 69 / 285 inverted rows at the 5,000 / 10,000
    / 20,000), ordered by Italian frequency;
  - the translators' typography.
- **Expressions** (D5): 13,374 keyed as en-es keys them.
- **The floor** (D6): fixed before the first committed measurement at the measured figures less two
  points, about 89.4 / 80.7 / 68.3, the owner settling it.
- **The sources registered** (D7): the English Wiktionary's Italian translations and the Italian
  Wiktionary's English section, `DUMPS["en-it"]`. No committed pin moves.
- **The checks** (D8):
  - a baseline and golden over the English corpus;
  - the cross-native and committed-tables tests;
  - the selection card's row cut;
  - both workflows;
  - the full card snapshot in Italian that change 34 hands over (`word-card-en-it.spec.ts`).
- **Coverage, share and sample published** (D9): the README shows the coverage by source, the 12.8 %
  of the 8,273 glossed among the 10,000 that a table glosses, and a sample of 100 marked by source.
- **Change 35 amended** (D5): its heraldry-only rule holds for an expression as for a word. **I11's
  text** names the Italian Wiktionary's own tables read backwards (D2).
- **Known defects listed for change 37** (D10): names on common words, contractions, function words
  described rather than glossed, Italian plurals in the inverted table, and the rest.
- **Not shipped** (D11): no package lists en-it before change 45.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *English is glossed in Italian from the Italian Wiktionary's English
  section and the two Wiktionaries' translation tables* and *en-it is committed beside English's
  studied tables, held to a coverage floor, and not shipped*.

## Impact

- **Products.** Lingua's data and tests:
  - `scripts/lingua-data/`:
    - new: `reduce-en-it.py`, `test_reduce_en_it.py`, `tables/en-it/`;
    - changed: `pack_sources.py` (`EDITIONS`, `DUMPS`), `test_pack_sources.py`, `gloss_coverage.py`
      (`FLOORS["en-it"]`), `test_gloss_coverage.py`, `test_reduce_editions.py`, `test_reduce_loops.py`,
      `SOURCES.md`, `tables/en-fr/README.md`, `testdata/en-it/`.
  - `openspec/changes/add-lingua-italian-edition-rules/` (the heraldry amendment) and
    `docs/lingua/italian-programme.md` (I11, the *Measured figures*).
  - `.github/workflows/{lingua-pack-update.yml, lingua-extension-check.yml}`.
  - `crates/lingua-wasm/tests/{en_it_baseline.rs, baseline/en-it.golden, cross_native.rs}`.
  - `crates/lingua-pack/tests/committed_tables.rs`.
  - `apps/lingua-extension/test/{row-gloss-tables.spec.ts, word-card-en-it.spec.ts,
    baseline/word-card-en-it.txt}`.

  No extension code, site, app or backend change.
- **The owner**: the floor (task 0.1), the sources release, the dispatch, the reduction's pull request.
- **Prerequisites**: changes 3 and 4 (base and layer), 16 (`EDITIONS["it"]`), 32 (a native `it`), 34
  (the Italian renderer) and 35 (the Italian edition's rules) implemented. Change 22 registers
  `kaikki-it-traductions-en.jsonl`; if stage 3 lands first (I5), this change registers it, under the
  same name and kind.
- **Order**: before changes 37, 38 and 45.
- **Effort, against 3.5–6 ideal days**: 3.5–6 (design, *Effort*).
