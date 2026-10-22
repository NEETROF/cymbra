# add-lingua-pack-it-es — Italian glossed in Spanish, direct sources first, then a pivot

## Why

Change 27 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2: Spanish speakers
studying Italian. Its planned sources — the Spanish Wiktionary's Italian section, the Italian Wiktionary's
Spanish translations, the Spanish Wiktionary's tables read backwards — gloss only 57.8 / 46.8 / 35.0 % of
the 5,000 / 10,000 / 20,000 commonest Italian lemmas, 25 / 24 / 22 points under fr-es. The owner had new
sources researched and chose, on 2026-10-22 (I4, I11, #1004), the full pivot: after every direct source,
Wikidata Lexemes, the Italian Wiktionary's Spanish section, and seven other Wiktionaries' translation
tables pairing an Italian and a Spanish word under one sense — 84.7 / 76.1 / 65.5 together, as designed. Change 27a
derives and pins those files; this change reads them into it-es's glosses.

At the top of the ranking the Spanish section describes rather than glosses (`tu` « Pronombre personal
nominativo… », `è` « Verbo auxiliar… »), names gloss common words (`secondo` « Apellido; Segundo;
Según »), and the tables repeat a word across parts of speech (`accaduto` « Hecho; Hecho; Hecho »): a
pre-pass of it-es's own fixes these, and fr-es's table rules, which it-es needs too, are copied into a
module of their own, so that no shipped pack moves.

## What Changes

- **A reader pair** (design D1): `reduce-it-es.py`, against `tables/it/` as committed.
- **The sources in order** (D2): definitions, the two Wiktionaries' tables, Wikidata's links, glosses and
  items, the Italian
  Wiktionary's Spanish section, then the pivot — at most three words per part of speech from each.
- **The pivot** (D3): groups named by a sense (or one-sense entries), names and capitals left
  out, Spanish words ordered by how many tables give them; no expressions.
- **it-es's pre-pass** (D4), `reduce_italian_es.py`: names and acronyms off common words, pointers lending
  their base to a headword with no meaning and no tagged pointer, sense links, superscripts and subscripts
  out, function words and Italian words that are Spanish too from reviewed lists; **fr-es's table rules
  copied** into `reduce_spanish_tables.py`, parameterised and applied to every table-shaped source — fr-es
  and en-es untouched.
- **The Spanish edition's third reader** (D5): fr-es's requirement names it-es.
- **Expressions** (D6), **acronym rows** (D7).
- **The floor** (D8): fixed at task 0.1 on the committed ranks less two points, owner-settled before the
  first measurement.
- **Provenance and credits** (D9): coverage by source; a sample marked by source, a pivot row by edition,
  entry and sense; Wikidata's CC0 credited.
- **The no-pivot requirement** (D10), MODIFIED for it-es alone.
- **A baseline** (D11); **not shipped** before change 30.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *Italian is glossed in Spanish from the Spanish Wiktionary's Italian section,
  the two Wiktionaries' tables, Wikidata and a pivot* and *it-es is committed beside Italian's studied
  tables, held to a coverage floor, and not shipped*. MODIFIED *A gloss is in the reader's language,
  written by a person* (held by no open change): never pivoted but as it-es's exception; *A pivot for
  it-es* added. MODIFIED *French is glossed in Spanish from the Spanish Wiktionary's French section and
  the French Wiktionary's translation tables* (held by `enable-lingua-french`): the Spanish edition's
  rules read by en-es, fr-es and it-es.

## Impact

- **Products.** Lingua's data only: `scripts/lingua-data/` — *new*: `reduce-it-es.py`,
  `reduce_italian_es.py`, `reduce_spanish_tables.py`, `test_reduce_it_es.py`, `tables/it-es/`; *changed*:
  `gloss_coverage.py` (`FLOORS["it-es"]`), `test_gloss_coverage.py`, `test_reduce_editions.py`,
  `SOURCES.md`; `.github/workflows/{lingua-pack-update.yml, lingua-extension-check.yml}` (it-es reduced
  after it-fr, its floor, its baseline and its card snapshot); `crates/lingua-wasm/tests/{it_es_baseline.rs,
  cross_native.rs, baseline/it-es.golden}`; `crates/lingua-pack/src/licence.rs` (`Cc0`) and
  `tests/committed_tables.rs`; `apps/lingua-extension/test/word-card-it-es.spec.ts` (change 18's Spanish
  snapshot). `reduce-fr-es.py`, `reduce-en-es.py`, their tables, pins and goldens are untouched. No
  extension code, site or backend change.
- **Prerequisites**: changes 5, 9, 11, 12, 16, 22 and 27a implemented.
- **The owner**: the floor (task 0.1), the sources release, the dispatch, the reduction's pull request.
- **Order**: after change 27a; before changes 28–30. Archived after the changes of `.openspec.yaml`.
- **Effort, against 3–5 ideal days before I4**: 4–7 (design, *Effort*).
