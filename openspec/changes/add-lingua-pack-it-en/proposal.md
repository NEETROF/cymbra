# add-lingua-pack-it-en — Italian glossed in English: the first reader pair of Italian

## Why

Change 22 of the [Italian programme](../../../docs/lingua/italian-programme.md), the first of stage
2: English speakers studying Italian (I1, I5). Italian's studied side is written once by the
reference pair it-fr (changes 9 and 16); like es-en for Spanish, it-en is a reader pair that adds
the native side alone. The glosses come from the English Wiktionary's Italian section — the file
change 9 already pins for Italian's forms, so no new download for definitions — with the Italian
Wiktionary's English translations as the fallback (M5: words people wrote).

Measured on the derived section (656,480,462 B, 623,702 entries, 135,532 glossed headwords) with the
shared rules and the English edition's: **91.5 / 84.7 / 76.2 %** of the 5,000 / 10,000 / 20,000
commonest Italian lemmas from the definitions, **91.8 / 85.1 / 76.7** with the Italian Wiktionary's
translations — on a ranking read from the same section, which favours it-en; 84.6 / 76.9 / 68.1 if
every form of another lemma goes to that lemma. es-en committed 93.0 / 86.5 / 76.5, fr-en
93.9 / 87.2 / 76.8.

At the top of the ranking, the English edition misreads Italian where its rules never met it: the
articulated prepositions are pointers carrying their meaning (`del` « contraction of di + il; of the,
from the ») that the shared rules leave unglossed; `gli` reads as a pronoun only and `mi` opens on
« used as ethical dative »; names gloss common words (`torno` « lathe » beside a place) and function words
(`io` « Io », `meno` « Main »). Those are fixed here, in a module of it-en's own: the English
edition's rules are in es-en's and fr-en's pins and do not move.

## What Changes

- **`reduce-it-en.py`** (design D1): the native side alone, against `tables/it/` as committed — its
  pin records the studied tables it read.
- **Definitions, then the Italian Wiktionary's translations** (D2): es-en's order, at most three
  words per part of speech; never the English tables read backwards, which add noise at the top
  (`etc` « cub », `cv` « hp »).
- **Sources** (D3): change 9's section as pinned; `kaikki-it-traductions-en.jsonl` (7,216,558 B) in
  `EDITIONS["it"]`; `DUMPS["it-en"]`; a release of its own.
- **it-en's own pre-pass** (D4), `reduce_italian_en.py`: meanings carried by a pointer (the `di`
  family, `ed`/`ad`, `gli`, `mi`); names kept off common and function words (it-fr's D5, fr-en's
  D6); letters through the English edition's own pre-passes; notes such as « (all senses) » out.
- **Expressions** (D5): the section's (8,523), the Italian Wiktionary's (1,192), and the 77 split
  single words with a meaning (`d'accordo`), keyed by change 12's rules.
- **The acronym rows** (D6) glossed by change 11's rule from it-en's glosses.
- **The floor** (D7): fixed at task 0.1 on change 9's committed ranks, less two points, up or down,
  owner-settled before the first committed measurement, never lowered after; `FLOORS["it-en"]` in
  `gloss_coverage.py`, the extension check's floor step running it.
- **Coverage shown** (D8) in the README, `SOURCES.md` and the programme.
- **An it-en baseline** (D9) over the Italian corpus; `cross_native.rs` holds the studied side one.
- **Not shipped** (D10): no package lists it-en before change 25.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *Italian is glossed in English from the English Wiktionary's Italian
  section and the Italian Wiktionary's English translations* and *it-en is committed beside
  Italian's studied tables, held to a coverage floor, and not shipped*. es-en's *A pair of a studied
  language's second native language* and *A reader pair's pin records the studied tables it read*
  hold as written.

## Impact

- **Products.** Lingua only:
  - `scripts/lingua-data/` — *new*: `reduce-it-en.py`, `reduce_italian_en.py`,
    `test_reduce_it_en.py`, `tables/it-en/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json,
    pin.json, README.md}`; *changed*: `pack_sources.py` (`EDITIONS["it"]`, `DUMPS["it-en"]`),
    `test_pack_sources.py`, `gloss_coverage.py` (`FLOORS["it-en"]`), `test_gloss_coverage.py`,
    `SOURCES.md`; *consumed*: `tables/it/`, `reduce_common.py`, `reduce_edition_en.py`, unchanged.
  - `.github/workflows/lingua-pack-update.yml` (it-en among the pairs, reduced after it-fr) and
    `lingua-extension-check.yml` (the floor step runs it-en).
  - `crates/lingua-wasm/tests/{it_en_baseline.rs, cross_native.rs, baseline/it-en.golden}`;
    `crates/lingua-pack/tests/committed_tables.rs` (it-en built to its pin).
  - `docs/lingua/italian-programme.md` (the measured coverage).
  Nothing of the extension, the site, the back office or the backend; no other pair's tables, pin or
  golden.
- **Prerequisites**: changes 5, 9, 11, 12 and 16 implemented — Italian's studied tables and their
  dictionary words committed, the Italian corpus and baseline in place, `EDITIONS["it"]` created.
- **The owner**: the floor (task 0.1), the sources release, the dispatch and the reduction's pull
  request (I12).
- **Order**: after change 16's committed tables; before changes 23 (refinement), 24 (marks), 25
  (enable). Archived after the changes of `.openspec.yaml`.
- **Effort, against 3–5 ideal days**: 3–4.5 (design, *Effort*).
