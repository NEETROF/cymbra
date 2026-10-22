# add-lingua-pack-it-en — Italian glossed in English: the first reader pair of Italian

## Why

Change 22 of the [Italian programme](../../../docs/lingua/italian-programme.md), the first of stage
2: English speakers studying Italian (I1, I5). Italian's studied side is written once by the
reference pair it-fr (changes 9 and 16); like es-en for Spanish, it-en is a reader pair that adds
the native side alone. The glosses come from the English Wiktionary's Italian section — the section
change 9 reads for Italian's forms — with the Italian Wiktionary's English translations as the
fallback (M5: words people wrote).

Measured on the derived section (656,480,462 B, 623,702 entries, 135,532 glossed headwords) with the
shared rules and the English edition's: **91.5 / 84.7 / 76.2 %** of the 5,000 / 10,000 / 20,000
commonest Italian lemmas from the definitions, **91.8 / 85.1 / 76.7** with the Italian Wiktionary's
translations — on a ranking read from the same section, which favours it-en; 85.1 / 77.5 / 68.7 if
every form of another lemma goes to that lemma. es-en committed 93.0 / 86.5 / 76.5, fr-en
93.9 / 87.2 / 76.8.

At the top of the ranking, the English edition misreads Italian where its rules never met it: the
articulated prepositions are pointers carrying their meaning (`del` « contraction of di + il; some, of
the, from the ») that the shared rules leave unglossed; `ad` reads the acronym AD's « CEO », `gli` a
pronoun only, `mi` opens on « used as ethical dative »; function words carry names (`io` « Io », `meno`
« Main »); Greek letters gloss common words (`mi` « mu »). Those are fixed here, in a module of it-en's
own: the English edition's rules are in es-en's and fr-en's pins and do not move. it-fr's rule keeping
names off common words is not taken: on this section it would strip « Rome », « Christmas » and
« Earth » from the commonest rows.

## What Changes

- **`reduce-it-en.py`** (design D1): the native side alone, against `tables/it/` as committed — its
  pin records the studied tables it read.
- **Definitions, then the Italian Wiktionary's translations** (D2): es-en's order, at most three
  words per part of speech; never the English tables read backwards (in the top 5,000, 24 rows of
  which about 6 wrong: `etc` « cub », `cv` « hp »).
- **Sources** (D3): its own fetch of the English Wiktionary's Italian section, as fr-en's;
  `kaikki-it-traductions-en.jsonl` (7,216,558 B) in `EDITIONS["it"]`; `DUMPS["it-en"]`; a release of
  its own; the update workflow offers it-en, after it-fr.
- **it-en's own pre-pass** (D4), `reduce_italian_en.py`: meanings carried by a pointer, by a reviewed
  list of wordings (the `di` family, `ed`, `ad`, `gli`, `mi`); acronyms off common words and names off
  function words; letters' names, Greek ones included, gloss nothing; notes such as « (all senses) »
  out; the split words without a space read. Measured: 91.7 / 84.7 / 76.2 from definitions,
  92.0 / 85.1 / 76.7 with the Italian Wiktionary.
- **Expressions** (D5): the section's (8,523), the Italian Wiktionary's (1,192), and the split single
  words with a meaning (`d'accordo`), keyed by change 12's rules.
- **The acronym rows** (D6) glossed by change 11's rule from it-en's glosses.
- **The floor** (D7): fixed at task 0.1 on change 9's committed ranks, less two points, up or down,
  owner-settled before the first committed measurement, never lowered after; `FLOORS["it-en"]` in
  `gloss_coverage.py`, the extension check's floor step running it.
- **Coverage and credits** (D8): the README, `SOURCES.md` and the programme; the notice and manifest
  credit both sides.
- **An it-en baseline and the English card** (D9): a golden over the Italian corpus, `cross_native.rs`
  holding the studied side one, the workflows running it; change 18's English card snapshot.
- **Not shipped** (D10): no package lists it-en before change 25.
- **A third reader of the English edition** (D11): fr-en's requirement names it-en.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *Italian is glossed in English from the English Wiktionary's Italian
  section and the Italian Wiktionary's English translations* and *it-en is committed beside
  Italian's studied tables, held to a coverage floor, and not shipped*. MODIFIED *French is glossed in
  English from the English Wiktionary's French section* (held by `add-lingua-pack-fr-en`): the English
  edition's rules read by es-en, fr-en and it-en, and its scenario *A rule of the English edition*
  naming it-en's digest; every other word and scenario kept. es-en's *A pair of a studied language's
  second native language* and *A reader pair's pin records the studied tables it read* hold as
  written.

## Impact

- **Products.** Lingua only:
  - `scripts/lingua-data/` — *new*: `reduce-it-en.py`, `reduce_italian_en.py`,
    `test_reduce_it_en.py`, `tables/it-en/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json,
    pin.json, README.md}`; *changed*: `pack_sources.py` (`EDITIONS["it"]`, `DUMPS["it-en"]`),
    `test_pack_sources.py`, `test_reduce_loops.py` (the workflow's pair options), `gloss_coverage.py`
    (`FLOORS["it-en"]`), `test_gloss_coverage.py`, `SOURCES.md`; *consumed*: `tables/it/`,
    `reduce_common.py`, `reduce_edition_en.py`, `reduce_italian_acronyms.py`, unchanged.
  - `.github/workflows/lingua-pack-update.yml` (it-en among the pairs, reduced after it-fr, its
    baseline run) and `lingua-extension-check.yml` (the floor step and the baseline run it-en).
  - `crates/lingua-wasm/tests/{it_en_baseline.rs, cross_native.rs, baseline/it-en.golden}`;
    `crates/lingua-pack/tests/committed_tables.rs` (it-en built to its pin, its credits, not listed).
  - `apps/lingua-extension/test/word-card-it-en.spec.ts` (change 18's English card snapshot).
  - `docs/lingua/italian-programme.md` (the measured coverage).
  Nothing of the extension's code, the site, the back office or the backend; no other pair's tables,
  pin or golden.
- **Prerequisites**: changes 5, 9, 11, 12, 16 and 18 implemented — Italian's studied tables and their
  dictionary words committed, the Italian corpus and baseline in place, `EDITIONS["it"]` created, the
  card's English renderer.
- **The owner**: the floor (task 0.1), the sources release, the dispatch and the reduction's pull
  request (I12).
- **Order**: after change 16's committed tables; before changes 23 (refinement), 24's run (its gloss
  column), 25 (enable). Archived after the changes of `.openspec.yaml`.
- **Effort, against 3–5 ideal days**: 3.25–4.75 (design, *Effort*).
