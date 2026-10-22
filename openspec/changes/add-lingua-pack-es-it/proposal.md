# add-lingua-pack-es-it — Spanish glossed in Italian, for Italian speakers, through the full pivot

## Why

Change 51 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 4: Italian
speakers learning Spanish. es-it is the third pair glossed in Italian, after en-it (change 36) and fr-it
(change 47).

Its direct sources are thin, as I4 expected:
- the Italian Wiktionary's Spanish section, with the Italian edition's rules (change 35);
- the Spanish Wiktionary's Italian translation tables;
- the Italian Wiktionary's Spanish translations read backwards.

Together they gloss 64.0 / 49.9 / 36.5 % of the 5,000 / 10,000 / 20,000 commonest Spanish lemmas, against
es-fr's 87.6 / 77.2 / 63.7 and es-en's 93.0 / 86.5 / 76.5. A quarter of the commonest dictionary words
are left unglossed, `a`, `para`, `bien` and `ni` among them.

The owner settled it on 2026-10-22 (I4, I11): es-it reads the full pivot it-es reads, the other way
round. That adds Wikidata's lexemes, the Spanish Wiktionary's Italian section read backwards, and the
translation tables of seven other Wiktionaries, where a Spanish and an Italian word stand under the same
sense. With them es-it reaches 86.0 / 75.8 / 64.0, level with es-fr, and a reviewed list gives the
commonest function words.

## What Changes

- **A reader pair** (design D1): `reduce-es-it.py` reads Spanish's committed studied tables and writes
  `tables/es-it/`, with the Italian edition's rules and never the Spanish edition's. Its rule digest is its
  own file, `reduce_common.py` and `reduce_edition_it.py`.
- **The sources in it-es's order, mirrored** (D2): the three direct sources, then Wikidata's lexemes,
  then the Spanish Wiktionary's Italian section read backwards, then the pivot. Change 27a's derived
  files are read as they are; nothing new is derived.
- **The pivot's filters, mirrored from change 27** (D3): a sense's group, never a « to sort » table; names,
  letters, affixes and capitalised words out; Italian words holding a Spanish letter or over sixty
  characters out; no expression from the pivot.
- **es-it's own rules** (D4):
  - a reviewed list of the commonest function words, read first, which only chooses among the Italian
    words es-it's sources pair with each word and composes none (M5): `a` « a », `ni` « né »,
    `durante` « durante » rather than the pivot's « autunno »;
  - letters out, but a one-letter conjunction or preposition whose translation is the same letter (`o`
    « o »);
  - Wikidata's letters, abbreviations and affixes out, and nothing from Wikidata or the pivot for a
    one-letter lemma;
  - translations under a sense the Italian Wiktionary marks heraldic out (`en` « posto »);
  - no Spanish key from the Spanish Wiktionary's pointer wordings read backwards;
  - an Italian word listed once per row read backwards;
  - the Italian typography.
  - fr-es's « the studied word is no definition » is **not** copied: Spanish and Italian share the word
    (7 of 10 rows worse).
- **Expressions** (D5): the section's and the two tables', keyed as Spanish keys them; about 1,175.
- **The floor** (D6): fixed before the first committed measurement at the measured figures less two
  points, about 83.7 / 73.6 / 61.9 with Wikidata's letters left out, the owner settling it.
- **The sources registered** (D7): `DUMPS["es-it"]`, change 27a's files named under their editions; no new
  `EDITIONS` entry.
- **The checks** (D8): a golden over the Spanish corpus, cross-native, committed tables, the row cut, both
  workflows, and the Italian card snapshot (`word-card-es-it.spec.ts`).
- **Coverage, provenance and sample** (D9): by source, every pivot row naming its edition, entry and
  sense.
- **Three requirements widened** (D10): the Italian edition's rules read by en-it, fr-it and es-it; the
  pivot exception for it-es and es-it; the pivot and lexeme files pinned by it-es and es-it.
- **Known defects listed for change 52** (D11).
- **Not shipped** (D12): no package lists es-it before change 54.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`:
  - ADDED *Spanish is glossed in Italian from the Italian Wiktionary's Spanish section, the two
    Wiktionaries' tables, Wikidata and a pivot*;
  - ADDED *es-it is committed beside Spanish's studied tables, held to a coverage floor, and not shipped*;
  - MODIFIED *English is glossed in Italian…* (held by `add-lingua-pack-fr-it`): read by en-it, fr-it
    and es-it;
  - MODIFIED *A gloss is in the reader's language, written by a person* (held by
    `add-lingua-pack-it-es`): the pivot exception for it-es and es-it, Wikidata's translation links
    named, with *A pivot for es-it*;
  - MODIFIED *it-es's further sources are derived and pinned like every source* (held by
    `add-lingua-it-es-pivot-sources`): pinned by it-es's and es-it's pins.

## Impact

- **Products.** Lingua's data and tests:
  - `scripts/lingua-data/`:
    - new: `reduce-es-it.py`, `test_reduce_es_it.py`, `tables/es-it/`, `testdata/es-it/`;
    - changed: `pack_sources.py` (`DUMPS["es-it"]`), `test_pack_sources.py`, `gloss_coverage.py`
      (`FLOORS["es-it"]`), `test_gloss_coverage.py`, `test_reduce_editions.py`, `test_reduce_loops.py`,
      `SOURCES.md`, `tables/es-fr/README.md`.
  - `.github/workflows/{lingua-pack-update.yml, lingua-extension-check.yml}`.
  - `crates/lingua-wasm/tests/{es_it_baseline.rs, baseline/es-it.golden, cross_native.rs}`.
  - `crates/lingua-pack/tests/committed_tables.rs`.
  - `apps/lingua-extension/test/{row-gloss-tables.spec.ts, word-card-es-it.spec.ts,
    baseline/word-card-es-it.txt}`.
  - `docs/lingua/italian-programme.md`: I4, I11, the *Measured figures* and row 51, amended in this pull
    request.

  No extension code, site, app or backend change.
- **The owner**: the floor (task 0.1), the reviewed function-word list, each entry with its source (task
  0.2, `scripts/lingua-data/es_it_function_words.tsv`, a rule file of es-it's reducer), the sources
  release, the dispatch, and the reduction's pull request.
- **Prerequisites**, implemented first:
  - changes 3 and 4 (base and layer);
  - change 16 (`EDITIONS["it"]`) and change 27a (the pivot and lexeme files);
  - change 27 (it-es's readers, which this change mirrors, and `Cc0`);
  - changes 32, 34 and 35;
  - change 36, then change 47 (the holder of the Italian edition's requirement).
- **Order**: before changes 52, 53 and 54.
- **Effort, against the row's 3.5–6 ideal days, raised to 4.75–8.75 for the pivot** (design, *Effort*).
