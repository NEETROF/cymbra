# Design — add-lingua-pack-es-it

## Context

**The data**, read on 2026-10-22, each dump checked against its earlier record:

| Source | Served | Gzipped | Decompressed |
|---|---|---|---|
| Italian Wiktionary | 2026-10-21 | 43,849,715 B (`1a8283c7…`) | 566,646,259 B (`09f22a16…`), as changes 35 and 47 |
| Spanish Wiktionary | 2026-10-20 | 103,226,087 B (`4f253676…`) | 1,233,016,232 B (`b332eaae…`); re-derives fr-es's pinned files byte for byte |
| French Wiktionary | 2026-10-20 | 736,596,809 B (`a1e04095…`) | `6d4cfc2c…` |
| German, Portuguese, Russian, Polish, Dutch Wiktionaries | 2026-10-20 | as change 27a's research recorded | — |
| Wikidata Lexemes | `wikidata-20261021` | 655,273,999 B (`85ebe293…`) | — |

- **Derived** with `main`'s `pack_sources.derive`, each identical to change 27's and 35's research:
  - `kaikki-it-Spagnolo.jsonl` (2,488,861 B), the section;
  - `kaikki-es-traductions-it.jsonl` (966,789 B), the direct table;
  - `kaikki-it-traductions-es.jsonl` (1,096,876 B), the inverted table;
  - `kaikki-es-Italiano.jsonl` (5,362,930 B), the Spanish Wiktionary's Italian section.
- **The pivot and lexeme files**: change 27a's, re-extracted byte for byte. The one exception is the English
  pivot, which comes from the English dump of 2026-10-03 (fr-en's pin); kaikki serves a newer one since
  2026-10-21, so the research used change 27a's file (50,180 rows).
- **The method**: change 47's prototype of change 35's rules, every rule on; Spanish's committed ranks
  (`tables/es/`, 60,000 lemmas) read as `reduce-es-en.py`'s `read_studied` reads them; the table readers
  copied from es-fr and fr-es. Without the letters rule it reproduces change 35's es-it figures exactly. Task 0.1 re-measures on
  the real `reduce_edition_it.py`.
- **Loose** coverage is any gloss; **strict** leaves out a lemma glossed only through a pointer.

**Coverage after each source** (5,000 / 10,000 / 20,000):

| After | Loose | Strict |
|---|---|---|
| the section's definitions, change 35's rules | 33.6 / 23.4 / 14.9 | 33.6 / 23.3 / 14.9 |
| + the direct table | 55.4 / 42.1 / 29.6 | 55.3 / 42.0 / 29.6 |
| + the inverted table | 64.0 / 49.9 / 36.5 | 63.9 / 49.8 / 36.5 |
| + Wikidata's translation links | 64.1 / 50.0 / 36.6 | |
| + Wikidata's lexemes of one item | 66.0 / 51.7 / 38.1 | |
| + the Spanish Wiktionary's Italian section read backwards | 68.5 / 54.1 / 40.2 | 68.4 / 54.1 / 40.2 |
| + the pivot, change 27's groups | **86.0 / 75.8 / 64.0** | 85.9 / 75.7 / 63.9 |

- With Wikidata's letters, abbreviations and affixes left out: 85.7 / 75.6 / 63.9.
- For comparison: es-fr 87.6 / 77.2 / 63.7 and es-en 93.0 / 86.5 / 76.5 (floor 87.6 / 77.2 / 63.7); it-es
  as designed 84.7 / 76.1 / 65.5; fr-it 81.3 / 70.1 / 57.8.
- The programme's 64.2 / 50.0 / 36.6 for the direct sources becomes 64.0 / 49.9 / 36.5. The difference is
  11 one-letter lemmas the letters rule removes; one is a real loss, the conjunction `o` « o » (D4).
- **The pivot, by edition alone** after the three direct sources: English 83.4 / 72.3 / 59.5, German
  78.1 / 64.9 / 50.4, French 74.4 / 61.2 / 47.6, then Polish, Russian, Portuguese and Dutch.
- **Its precision.** The pivot's first word agrees with the direct tables' on 75.7 % of the 20,000
  commonest; the two direct tables agree with each other on 75.0–75.5 %. A sample of 60 random pivot rows
  in the 10,000: 50 right, 8 partly right, 2 wrong (`pp` « bastardo », `ucrania` « ucraina »).
- **Spanish's dictionary words** (`tables/es/lexical.tsv`): the direct sources gloss 74.4 / 66.4 / 58.5 %;
  with the pivot 98.0 / 97.0 / 95.5 %.

**What is particular to Spanish in the Italian Wiktionary:**
- **The section**: 7,181 entries, 8,660 senses, 6,861 headwords, 5,473 with a sense that is not a form.
  1,611 senses are tagged forms; 81 carry a pointer wording without a tag (`esta` « forma femminile di
  "este" »); 41 are English bot pointers.
- **Heraldry**: 606 headwords whose every sense is heraldic (34 / 63 / 99 at the tops: `en`, `partido`,
  `campo`, `jefe`, `red`); the tables gloss 97 of the 99 again. The inverted table carries 389
  translations tagged « (araldica) », which feed 6 / 17 / 32 rows (`en` « posto », `curva` « piegato »).
- **Reflexive verbs**: 35 `-se` headwords (`acordarse` « ricordarsi »). Spanish's analyser maps them to
  the base verb, so they gloss nothing; unlike the French and English Wiktionaries, the Italian one files
  the pronominal meaning under the `-se` headword, so `acordar` reads only « mettere d'accordo » (change
  52's).
- **Names**: 241 entries tagged name; months and days capitalised (« Febrero »); `tierra` headed as a
  proper noun.
- **Empty senses**: 101 headwords have only senses kaikki extracted with no gloss (`decir`, `agua`,
  `cabeza`): the pages write the definition as a bullet, which wiktextract skips. The tables gloss 65 of
  the 72 at the 20,000.
- **Unglossed at the very top** by the direct sources: `a` (rank 6), `se`, `para`, `o`, `le`, `bien`,
  `ni`, `e`, `les`, `durante`, `sólo`, `luego`, `mientras`, `según`. `del` and `al` are split by the
  analyser before lookup.
- **Where the risk sits.** In the 1,000 commonest, 145 rows come from Wikidata, the backwards section or
  the pivot; 16 are letters and about 18 are wrong: `a` « a, ampere, amperaggio; a », `real` « soldi,
  denaro, moneta », `durante` « …; autunno », `china` « arancia… », `cuyo` « cavia… », `mundial`
  « mondo ». D4's rules and the reviewed list are aimed there.

## Goals / Non-Goals

**Goals:**
- es-it committed at or above a floor fixed before the first committed measurement, every gloss Italian
  written by a person, the pivot's pairing marked by its source.
- it-es, es-fr, es-en, en-it, fr-it and every other pair untouched.

**Non-Goals:**
- Shipping es-it (change 54).
- The refinements of D11 (change 52).
- New derived sources: change 27a's are read as they are.

## Decisions

### D1 — A reader pair, with the Italian edition's rules alone

`reduce-es-it.py` reads `tables/es/` as committed (es-en's `read_studied`) and writes `tables/es-it/{gloss.tsv,
senses.tsv, mwe.tsv, NOTICE, manifest.json}`. Its rule digest is its own file, `reduce_common.py` and
`reduce_edition_it.py`. It does not import `reduce_edition_es.py`, even to read the Spanish Wiktionary's
Italian section backwards, so that change 27's « the Spanish edition's rules … read by en-es, fr-es and
it-es » holds. Read backwards, that section gives Italian headwords as glosses of the Spanish words its
definitions name; es-it takes no Spanish key from its pointer wordings (« Forma del plural de »,
« Variante de », « Grafía alternativa de »), most of which the two-word filter already cuts, as change 27
takes no Italian key from the Italian section's notes; the count is measured at task 2.1. Its pin
records the studied tables it read.

### D2 — The sources, in it-es's order mirrored

1. The Italian Wiktionary's Spanish section's definitions, with change 35's rules.
2. The Spanish Wiktionary's Italian translations (the direct table).
3. The Italian Wiktionary's Spanish translations read backwards (the inverted table), ordered by
   wordfreq's Italian list.
4. Wikidata's lexeme translation links, then its lexemes of one item. Wikidata's Spanish glosses of
   Italian senses are one-way, and their mirror adds one lemma in 60,000, so it is not read.
5. The Spanish Wiktionary's Italian section read backwards (`kaikki-es-Italiano.jsonl`, change 27a's).
6. The pivot of the English, French, German, Portuguese, Russian, Polish and Dutch Wiktionaries'
   translation tables: change 27a's files keep each group's Italian and Spanish words together, so they
   read the other way unchanged.

At most three words per part of speech from each table-shaped source; a pivot's Italian words are
ordered by the number of tables and editions giving them, then by frequency, then alphabetically.

### D3 — The pivot's filters, mirrored from change 27

A table's group named by a sense or a sense number, or the whole entry of a one-sense entry; never a « to
sort » table. Left out: entries tagged name, abbreviation, symbol, character, letter, affix or
punctuation; one-letter headwords; capitalised headwords but German nouns; capitalised Spanish or Italian
words but acronyms of at most six letters; Italian words holding a letter Italian does not write (ñ, á,
í, ú, ü, ¿, ¡), « ° » or more than sixty characters — the mirror of it-es's grave accent. The pivot
supplies no expression. Measured at the research's filter stage, after the groups are read: 13,062
whole-entry groups on entries of several senses, 105 « to sort » tables and 5,280 capitalised entries
left out; 201 Italian words with a Spanish letter or « ° » and 174 over sixty characters (the raw files
give 14,044 / 200 / 124 / 57).

The prototype applied Wikidata, the backwards section and the pivot up to rank 20,000; the reducer applies
them to all 60,000 lemmas, and task 0.1 measures the layer's size and the pivot's share there.

### D4 — es-it's own rules

| Rule | Why | Effect |
|---|---|---|
| A reviewed list of the commonest function words, read first (the owner, 2026-10-22) | the sources gloss `a`, `ni` (« né; ni, nu »), `durante` (« …; autunno »), `sólo` and `le` badly or not at all; `para` « per » and `bien` « bene » are already glossed by the backwards section | each entry names a Spanish lemma, Italian words one of es-it's sources pairs with it, and that source: the list only chooses and orders them, and composes none (M5); a test fails on an entry no source supports; the owner approves each entry and its source (task 0.2) |
| Letters out as en-it reads them — a letter's entry, a one-letter word's noun entry, an entry whose every translation is its own letter, and, read backwards, every one-letter Spanish word —, but a one-letter conjunction or preposition whose translation is the same letter | the copied rule drops `o` « o », a real loss; it keeps `y` « e, ed, et » and `u` « oppure » from the direct table | `o` kept |
| Nothing from Wikidata or the pivot for a one-letter lemma; Wikidata's lexemes of category letter, abbreviation or affix out | `s` « secondo », `k` « potassio, kelvin », `d` « non disponibile » | 85.7 / 75.6 / 63.9 |
| Translations under a sense the Italian Wiktionary marks « araldica » out of the inverted table (389 translations; « termine araldico », 446 more, is not matched) | `en` (rank 4) « posto », `curva` « piegato » | 6 / 17 / 32 rows; D5's counts measured with the rule on |
| An Italian word listed once per row, read backwards | as fr-es, en-it, fr-it | |
| The Italian edition's typography for every table word | as en-it, fr-it | |
| **Not** fr-es's « the studied word is no definition » | Spanish and Italian share the word: it moves 10 rows, 7 worse (`alcaloide` → « alcaloidi », `consonante` → « consonànte ») | definitions kept |

fr-it's borrowed-gloss rule fixes 2 rows here (`canto` « cantare… », `peces` « pesce »): change 52's, so
the main spec's form-of requirement is not modified again.

### D5 — Expressions

The section's glossed headwords with a space, then those the two Wiktionaries' tables gloss: 1,175 (294
from the definitions, 445 direct, 436 inverted), keyed as `add-lingua-spanish-expression-keys`' *A
Spanish pack keys its expressions as Spanish is read* requires. es-fr keys 15,133 and es-en 15,515: the
gap is change 52's to measure, the pivot's 31,163 multi-word Spanish keys being left out as it-es leaves
its own (I11: the pivot supplies no expression). Change 35's heraldry-only rule takes 149 expressions out
of the definitions; the inverted table puts back 147.

### D6 — The floor (M6, I4)

Task 0.1 re-runs the measurement with the real `reduce_edition_it.py` on the committed ranks, D2–D4's
sources and rules applied, and fixes `FLOORS["es-it"]` at its figures less two points, the owner settling
it before the first committed measurement; never lowered after it. On this measurement, with the letters
left out, the floor is about 83.7 / 73.6 / 61.9. The reduce job's floor step runs `--pair es-it`.

### D7 — The sources registered

`EDITIONS` gains nothing: change 27a registers the section, both tables, the Spanish Wiktionary's Italian
section, the seven pivot files and the lexeme file. `DUMPS["es-it"]` names them under their editions, in
fr-es's shape, the native edition first:
`{"it": ("kaikki-it-Spagnolo.jsonl", "kaikki-it-traductions-es.jsonl"), "es": ("kaikki-es-traductions-it.jsonl",
"kaikki-es-Italiano.jsonl"), …}` with the seven pivot editions and Wikidata. No `PINNED` entry. Its
sources releases are `lingua-pack-sources-es-it-…`. Registering moves no committed pin.

Each es-it update fetches about 5.5 GB of dumps, as it-es does (changes 27 and 27a recorded about 5.3; the
English dump is now 3.08 GB);
`keep=…` reuses the last release's files on a routine update. The release assets are about 5 MB at zstd
19.

### D8 — The pipeline and the checks

- **`lingua-pack-update.yml`**: the pair option; the bless steps run `es_it_baseline` and the es-it card
  snapshot. Its `es` concurrency group already covers `es-*`.
- **`lingua-extension-check.yml`**: the baselines step and the floor step gain es-it.
- **`test_reduce_loops.py`**: es-it reduced after es-fr, and brought along by its updates.
- **`test_reduce_editions.py`**: the Italian edition's digest test names en-it, fr-it and es-it.
- **`crates/lingua-wasm/tests/es_it_baseline.rs`** and `baseline/es-it.golden`: the Spanish scenario
  (`support/spanish.rs`, `pages-es.txt`), es-en's 40 extra lemmas reused, an engine holding es-it alone
  as es-en's does, and a test that the studied side is es-fr's. With es-it alone, en-it's *Nothing else
  moves* needs no new exception.
- **`cross_native.rs`**: Spanish through Italian.
- **`crates/lingua-pack/tests/committed_tables.rs`**: es-it in every hard-coded list of committed and
  Spanish pairs; built to its pin and under 5 MiB; its credits; its sources releases; listed in no
  package while the list does not name it.
- **`scripts/lingua-data/testdata/es-it/`**, run by `pipeline_testdata.rs`.
- **`tables/es-fr/README.md`**, which says one other pair reads `../es/`, names es-it.
- **`apps/lingua-extension/test/row-gloss-tables.spec.ts`**: es-it's rows and expressions; none of the
  prototype's 10,619 direct-source rows has an empty first sense or ends on an opening mark, re-checked
  on the pivot's at implementation.
- **The card snapshot** change 34 hands over: `test/word-card-es-it.spec.ts` and
  `test/baseline/word-card-es-it.txt`, Spanish forms named in Italian (« passato remoto », « congiuntivo
  futuro (futuro de subjuntivo) », « condizionale presente », « gerundio »), re-blessed with the golden.

### D9 — Coverage, provenance and credits

`tables/es-it/README.md` publishes the coverage against the floor by source, the table share, a sample of
100 glosses — each row naming its source, a pivot row its edition, entry and sense —, and the layer's size
(about 0.21 MB at the 20,000 with the pivot; es-fr's is about 0.54 MB).

The notice and the manifest credit the Italian Wiktionary (its Spanish section, and the Spanish
translations its entries list), the Spanish Wiktionary (its Italian translations and its Italian section),
the seven Wiktionaries of the pivot (CC BY-SA 4.0 + GFDL), Wikidata under CC0 (`Licence::Cc0`, change 27),
wordfreq's Spanish and Italian lists, and Spanish's studied sources, with « levels are estimated ».

### D10 — Three requirements widened

Copied whole from their holders, as they then stand:
- **Change 47's *English is glossed in Italian…***: the Italian edition's rules are read by en-it, fr-it and
  es-it; *A rule of the Italian edition* moves three digests. Change 37 (`refine-lingua-en-it-glosses`) and
  this change: whichever is implemented second re-copies the other's block.
- **Change 27's *A gloss is in the reader's language, written by a person***: the pivot exception holds
  for it-es and es-it, after each pair's direct sources, and names Wikidata's lexeme translation links
  beside its same-item senses (change 27a's « linked to the same item or by a translation link »); *A pivot for it-es* reads « no pack but it-es and
  es-it glosses through that table »; *A pivot for es-it* is added (`comprador` « acquirente », from the
  English Wiktionary's `buyer`).
- **Change 27a's *it-es's further sources are derived and pinned like every source***: the pivot and lexeme
  files are pinned by it-es's and es-it's pins, and read by no other pack; its re-reduction and *Nothing
  else moves* scenarios name both.

True as written: change 27's ADDED it-es requirements; change 35's edition requirements; the main spec's
form-of requirement as change 47 leaves it (D4).

### D11 — Known defects, for change 52

| Defect | Count | Examples |
|---|---|---|
| A word repeated across parts of speech in the direct table | 147 / 222 / 332 rows | `más` « più; più; più; più »; fr-es's listed-once rule for the direct table fixes them, coverage unchanged |
| A `-se` meaning not lent to the base verb | 18 rows | `acordar` « mettere d'accordo » without « ricordarsi » |
| A pointer-borrowed gloss a table would fix | 2 rows | `canto` « cantare… », `peces` « pesce » (fr-it's rule) |
| Card headings | | `ser` and `estar` headed as nouns, the section's noun entry first; `tierra` as a proper noun |
| Capital openings | 113 / 149 / 196 rows | |
| Words described, not glossed | 15 | `dos` « il numero naturale due, 2 » |
| English text in a gloss | | `solo` (rank 51) « sole only, unique, single; lonely, lonesome; alone, by oneself », `freno` « brake, of a vehicle etc »: against M5's « never in a third language » until change 52 takes it out, as en-it's English text is until change 37; the ADDED requirement therefore claims « written by a person », not « Italian » |
| Name notes on common words | | `domingo` « domenica; nome proprio di persona maschile… » |
| Expressions from the pivot | 31,163 multi-word keys, not measured | es-it keys 1,175 against es-fr's 15,133 |
| kaikki's empty senses | 101 headwords | reported upstream; the tables gloss most of them |

### D12 — Not shipped

No package lists es-it before change 54, and no site figure moves (`gloss_coverage --check` reads shipped
pairs).

## Risks / Trade-offs

- **Pivot errors at the very top** → the reviewed function-word list, the letters rules, the heraldic
  translations out (D4), and the sample read by source (D9).
- **A pivot through other languages** → the owner's exception (I11), each word written by a person, the
  pairing inferred from a shared sense, the source marked.
- **A 5.4 GB download per update** → `keep=…`, as it-es.
- **A prototype, not the real edition reader** → task 0.1 re-measures before the floor is fixed.

## Migration Plan

Nothing deploys. The tables are committed by the owner-dispatched update's pull request, and the pack is
not listed. Rollback is a revert.

## Effort

4.75–8.75 ideal days, against the row's 3.5–6, raised to 4.75–8.75 with the pivot:

| Part | Ideal days |
|---|---|
| Registration and its tests | 0.25–0.5 |
| `reduce-es-it.py` on the direct sources, its tests | 1–1.75 |
| The pivot, Wikidata and backwards readers, mirrored from change 27 | 1–1.5 |
| The letters rules and the reviewed function-word list | 0.25–0.75 |
| Workflows and `test_reduce_loops` | 0.25–0.5 |
| The dispatch, the committed tables, the README | 0.5–1 |
| The golden, cross-native, committed-tables tests, testdata, the row cut | 0.75–1.25 |
| The card snapshot | 0.25–0.5 |
| The floor, the three MODIFIED blocks, OpenSpec | 0.5–1 |

## Open Questions

For the owner:
1. **The floor** (task 0.1): the re-measured figures less two points, about 83.7 / 73.6 / 61.9.
2. **The reviewed function-word list** (task 0.2): the words, their Italian and each one's source, approved
   by the owner before the dispatch; change 33's Italian reviewer agent may propose, never decide.
3. **The golden**: es-it alone, as es-en's (recommended), so that en-it's *Nothing else moves* keeps its
   one exception.
