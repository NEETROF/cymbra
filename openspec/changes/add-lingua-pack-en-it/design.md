# Design — add-lingua-pack-en-it

## Context

**The data.**
- The Italian Wiktionary's dump of 2026-10-21 (43,849,715 B gzipped, 566,646,259 B decompressed) and
  the English Wiktionary's of 2026-10-03 (2,981,058,381 B gzipped, 25,614,284,530 B decompressed), both
  still served unchanged.
- English's committed ranks (`tables/en/`), read as `reduce-en-es.py`'s `read_studied` reads them:
  40,685 lemmas.

**The method.**
- The pair was prototyped step for step after `reduce-en-es.py`, with en-es's own functions and change
  35's rules (its prototype, with the review's abbreviation and heraldry-only rules).
- The same method reduces en-es from its pins and reproduces its committed `gloss.tsv` exactly (21,964
  rows).
- **Loose** coverage is any gloss, what `gloss_coverage.py` counts. **Strict** leaves out a lemma glossed
  only through a pointer.

**The sources:**

| Source | File | Size | As read |
|---|---|---|---|
| Italian Wiktionary, English section | `kaikki-it-Inglese.jsonl` | 13,670,867 B; 29,888 entries, 41,754 senses, 25,700 headwords | — |
| English Wiktionary, Italian translations (direct) | `kaikki-en-traductions-it.jsonl` | 9,643,709 B; 97,402 translations | 46,028 English keys (16,704 committed lemmas), 83,329 Italian word slots (56,463 distinct) |
| Italian Wiktionary, English translations (read backwards) | `kaikki-it-traductions-en.jsonl` | 7,216,558 B; 109,653 translations | 34,411 English keys (15,857 lemmas), 72,910 Italian word slots (32,836 distinct) |

en-es reads 35,167 senses, 56,339 direct keys and 21,876 inverted keys.

**Coverage after each source** (5,000 / 10,000 / 20,000):

| After | en-it loose | en-it strict | Lemmas glossed (of 40,685) | en-es loose (committed) |
|---|---|---|---|---|
| definitions, the Italian edition's rules | 84.8 / 72.2 / 53.0 | 84.6 / 72.0 / 52.8 | 13,673 | 80.4 / 62.9 / 42.5 |
| + the direct table | 90.6 / 81.1 / 67.0 | 90.4 / 80.9 / 66.8 | +6,117 | 92.6 / 84.2 / 70.8 |
| + the inverted table | **91.4 / 82.7 / 70.3** | 91.2 / 82.5 / 70.2 | 21,518 (52.9 %) | **93.0 / 85.0 / 71.7** (54.0 %) |

These confirm change 35's estimate for the definitions exactly, and its estimate with both tables to
0.1. The 0.1 lost at the 5,000 and 10,000 is the words change 35's heraldry-only rule empties and no table
glosses again (`s`, `p`, `v`, `g`, `er`, `jess`, `ulster`, and `gloucester` at the 10,000).

**Where the gap with en-es lies:**

| Top | English's dictionary words | en-it glosses | en-es glosses |
|---|---|---|---|
| 5,000 | 4,373 | 4,235 (96.8 %) | 4,232 |
| 10,000 | 8,022 | 7,611 | 7,625 |
| 20,000 | 13,930 | 12,752 | 12,724 |

The two are level on dictionary words. At the 5,000, en-it glosses 47 fewer contractions (24 of 81
against en-es's 71) and 37 fewer names and abbreviations.

**Where the top rows come from** (5,000 / 10,000 / 20,000):

| Source | Rows |
|---|---|
| Definitions | 4,229 / 7,199 / 10,564 |
| A pointer's word | 10 / 18 / 28 |
| The direct table | 292 / 893 / 2,804 |
| The inverted table | 39 / 163 / 665 |
| Unglossed | 430 / 1,727 / 5,939 |

The tables give 12.8 % of the 8,273 glossed among the 10,000 (893 + 163); en-es's give 26.0 %.

## Goals / Non-Goals

**Goals:**
- en-it committed at or above a floor fixed before the first committed measurement, every gloss Italian
  written by a person.
- en-es and every other pair untouched.

**Non-Goals:**
- Shipping en-it (change 45).
- The rules on the definitions that en-es took in its refinement (names on common words, determiners):
  change 37, with the defects of D10.

## Decisions

### D1 — A reader pair

`reduce-en-it.py` reads `tables/en/` as committed and writes `tables/en-it/{gloss.tsv, senses.tsv,
mwe.tsv, NOTICE, manifest.json}` with the Italian edition's rules (`reduce_edition_it.IT` and its
pre-pass), step by step as `reduce-en-es.py`; nothing of the studied side is computed. Its pin records
the studied tables it read. Its rule digest is its own file, `reduce_common.py` and
`reduce_edition_it.py`:
- not `reduce_edition_en.py`, since nothing of the English edition reads translation words, and
  importing it would re-pin en-it whenever es-en's or fr-en's rules change;
- not `reduce-en-es.py` (D3).

### D2 — The sources, in en-es's order

1. The section's definitions, with change 35's rules.
2. The English Wiktionary's Italian translations (the direct table).
3. The Italian Wiktionary's English translations read backwards (the inverted table), ordered by Italian
   frequency.

I11 names, for Italian natives, the Italian Wiktionary's sections and the other Wiktionaries' Italian
tables; the Italian Wiktionary's own tables read backwards (up to 3.3 points at the 20,000) are words
people wrote in Italian, which M5 allows and en-es reads for Spanish. This pull request adds them to
I11's text.

At most three words per part of speech from each table, as en-es reads them. Reading the inverted table
before the direct one ends at the same coverage but moves 205 / 570 / 1,489 rows; en-es's order is kept.

### D3 — en-es's table rules, copied

The readers of `reduce-en-es.py` (`read_studied`, `without_letter_translations`, `read_translated`,
`translators_words`, `listed_once`) are copied into `reduce-en-it.py`, as en-es copied en-fr's English
description and change 27 copied fr-es's table rules. Moving them to a shared module would add a rule
file to en-es and move its pin and `pack_version`.

They come with what they read: `translators_word`, the disused-label, note, parenthesis, number and word
patterns, `_names_a_letter`, `read_readings`, `read_table`, and `EN`. Two things change: the Spanish
edition's typography becomes `reduce_edition_it.typography` (in `translators_words` and `listed_once`),
and the label test's word list becomes wordfreq's Italian (`SPANISH_WORD_ZIPF` → Italian), since a label
is a note when no Italian word it holds rates at 1.0 Zipf or more. These are the substitutions the
measurement made.

| Rule | Effect |
|---|---|
| A word listed once across a row's parts of speech, in the inverted table | 19 / 69 / 285 rows; `left-hand` « sinistro; sinistro; sinistro », `la` « la; la; la » |
| Letters out | 1 row |
| The translators' typography | 4 direct rows in all: « ... » → « … », `enrichment` « arricchimento (1,2) » → « arricchimento » |

The direct table's repeats (21 / 47 / 107 rows, `national` « nazionale; nazionale ») are kept, as en-es's
D7 keeps its own.

### D4 — Case

The Italian edition's `capitalised=False`: a table gloss keeps its words' case. Measured against
`capitalised=True`, 324 / 1,037 / 3,429 rows differ. With it, 6 / 19 / 40 rows open on a capital (brands,
abbreviations: `xbox` « XBox »), 3 / 12 / 25 capitalise a word the reader may not expect (at the 5,000 a
label and two brands: `del` « Canc », `xbox`, `nintendo`), and 1 / 3 / 4 hold the same word
twice by case alone (`satan` « Satana, satana »): change 37's.

### D5 — Expressions

| Source | Expressions |
|---|---|
| The section's definitions | 1,380 |
| The direct table | 9,855 (of 11,686 multi-word English words) |
| The inverted table | 2,139 (of 3,698) |
| **Total** | **13,374** |

en-es keys 17,094; 8,222 keys are shared. Keys follow en-es's exactly (`reduce_common.reduce_expressions`,
`fallback_expressions`): lower case, NFC, single spaces, every part an English token, names and pointer
senses left out. The builder keeps only expressions whose words are in the lexicon. No new key rule is
needed: the studied language is English.

Change 35's heraldry-only rule applies to a multi-word headword as to a word. Change 35 stated it for a
word, and its rule belongs to the edition, which fr-it and es-it also read, so this pull request amends
change 35's requirement, design and tasks to « a headword — a word or an expression » with a scenario of
its own. It takes 424 heraldic-only expressions out of the definitions, and the tables gloss 312 of them
again (« arms of alliance » takes « arma d'alleanza »): a net 112 fewer. A phrase glossed only by its
heraldic sense is the heraldic term, which a reader of English text rarely means.

### D6 — The floor (M6)

Task 0.1 re-runs this measurement on the committed ranks at implementation, and fixes `FLOORS["en-it"]`
at its figures less two points, the owner settling it before the first committed measurement; never
lowered after it. On this measurement the floor is 89.4 / 80.7 / 68.3. en-es's floor (91.4 / 83.2 / 69.9)
would fail en-it at the 10,000 (82.7), and hold at the 5,000 only because the check compares rounded
figures (91.4 against 91.4). The gap is contractions and names, not
dictionary words (*Context*), so en-it's floor is its own, as every pair's is. The reduce job's floor
step runs `--pair en-it`.

### D7 — The sources registered

- `EDITIONS["en"]` gains `kaikki-en-traductions-it.jsonl` (`("translations", "en", "it")`); no earlier
  change registers it.
- `EDITIONS["it"]` (change 16's) gains `kaikki-it-Inglese.jsonl` (`("entries", "en")`), and
  `kaikki-it-traductions-en.jsonl` if change 22 has not registered it yet (I5 lets stage 3 land first),
  with the same name and kind.
- `DUMPS["en-it"]` names the three. No `PINNED` entry: the English studied side is read as committed.

Registering moves no pin: with the entries in place, the six committed pairs' rule files, digests,
`pack_version`, `check_reducer` and `check_registered` are unchanged. Two tests pin the English
catalogue and gain the new file: `test_fr_en_registers_the_catalogue_s_file_and_no_derivation` and
`test_the_dump_s_identity`. The editions' addresses test moves with change 16, which creates
`EDITIONS["it"]`; en-it adds no address.

`migrate-lingua-pack-sources-to-raw-dumps`' *Every kaikki source is derived from one dump per Wiktionary
edition* names the English, French and Spanish editions only. Change 16, which creates the Italian
edition, is the one to amend it; this change records the gap for it.

Each en-it update fetches the English dump, 3 GB gzipped, for one 9.6 MB file, as en-es does (1–2.5
minutes to fetch and 3–4 to derive on a runner); `keep=en` reuses the last release's files on a routine
update. The Italian dump takes seconds. The release assets add about 3.3 MB at zstd 19.

### D8 — The pipeline and the checks

- **`lingua-pack-update.yml`**: the pair option; the `keep` input's text names `it`; the bless steps run
  `en_it_baseline` and the en-it card snapshot. Its concurrency group already puts every `en-` pair in
  the `en` group.
- **`lingua-extension-check.yml`**: the baselines step and the floor step gain en-it.
- **`test_reduce_loops.py`**: the options and the reduction order.
- **`crates/lingua-wasm/tests/en_it_baseline.rs`** and `baseline/en-it.golden`: the English scenario with
  an engine holding en-it alone, en-es's 40 extra lemmas reused for comparability (14 of them have a
  single en-it sense run, though en-es chose them for having two or more), and a test that the studied
  side is en-fr's.
- **`cross_native.rs`**: English through Italian.
- **`crates/lingua-pack/tests/committed_tables.rs`**: built to its pin, its credits; and
  `scripts/lingua-data/testdata/en-it/`, run by `crates/lingua-pack/tests/pipeline_testdata.rs`.
- **`tables/en-fr/README.md`**, which says « one other pair reads `../en/` », names en-it too, as en-es's
  task 4.2 did.
- **`apps/lingua-extension/test/row-gloss-tables.spec.ts`**: en-it's rows through the selection card's
  row cut, measured to pass (none empty, none ending on an opening mark).
- **The card snapshot change 34 hands over**: `test/word-card-en-it.spec.ts` and
  `test/baseline/word-card-en-it.txt`, the full card in Italian over en-it's golden, as
  `word-card-en-es.spec.ts`, re-blessed with the golden.

English has no guard in `GUARDED`, so nothing there.

### D9 — Coverage, provenance and credits

`tables/en-it/README.md` publishes:
- the coverage against the floor, by source;
- the table share (12.8 % of the 8,273 glossed among the 10,000);
- a sample of 100 glosses marked by source;
- the pack's size (about 1.58 MB, the native tables about 513 KB against en-es's 624 KB).

The notice and the manifest credit the Italian Wiktionary, the English Wiktionary (CC BY-SA 4.0 + GFDL),
wordfreq's Italian list (which orders the inverted words and drives the label test) and English's
studied sources. en-it's sense runs use no tag outside `tables/en/tags.tsv`, so its pool is
en-fr's. `SOURCES.md` and the programme's *Measured figures* record the committed coverage.

**A sample of 50 rows at the 5,000**: 44 right, 5 partly right, 1 wrong.
- **Wrong**: `trash` (rank 2574), glossed as *thrash* in the source itself.
- **Partly right**: `must` « dovere; un «uso»; obbligo », `phase`, `compliance` « ubbidienza », `mt` (a
  postal code), `subtle` « astuto, perspicace » without « sottile ».

The tables are rare at the top, so two extra samples were drawn:
- 20 direct rows at the 5,000: 17 right, 3 partly right;
- 25 inverted rows at the 10,000: 15 right, 9 partly right (mostly plurals: `speedy` « veloce, spedito,
  rapidi »), 1 wrong (`po` « padano »).

### D10 — Known defects, for change 37

Counted at the 5,000 / 10,000 / 20,000:

| Defect | Count | Examples | Note |
|---|---|---|---|
| Names on common words | 12 / 24 / 26 person-name notes | `summer` « estate; nome proprio di persona inglese femminile » | en-es's refinement rule, transposed, changes 10 / 20 / 22 rows and loses none |
| US state codes on two-letter tokens | 15 / 21 / 21 | `de` « Sigla postale per il Delaware », `co`, `al`, `mo` « sigla per lo Stato del Missouri » | Canadian codes (`bc`, `nb`) too |
| Function words described, not glossed | 19 / 24 / 25 rows every sense of which describes | `my` « forma possessiva di I… », `an`, `would` | |
| A label before a colon | 59 / 84 / 114 | `he` « in funzione di soggetto: egli, lui, esso » | |
| English text in a gloss | 4 / 8 / 15 | `media` | |
| Tagged pointers with text after their target | 2 / 2 / 2 | `upper` loses « più alto » | change 35's D10 |
| Untagged pointer wording before a meaning | 7 / 10 / 13 | `those` « plurale di that: quelli, quelle » | |
| Capital openings | 63 / 111 / 172 rows | `you` « tu; voi; Lei », `okay` « Indica un assenso » | |
| A definition repeating another | 222 / 294 / 344 | `well` « …, pozzo, …; pozzo » | |
| Heraldic and obsolete senses shown last | 96 / 134 / 174 and 39 / 61 / 86 | | change 35's D8 |
| A main sense missing where the direct table has it | | `can` « latta… » without « potere »; `do` opens on « festa » | definitions come first, as en-es reads them |
| Italian inflected forms in the tables | inverted: plurals 2 / 25 / 94 rows, any form 3 / 34 / 128; direct: 18 / 56 / 149 (plurals 3 / 7 / 20) | `polar` « polare; polari », `member` « membro, iscritto, membra » | the Italian dump writes a page's translation table on its « forma flessa » entries, and the derived file keeps no form-of tag; `tables/it/forms.tsv` could tell them apart, at the cost of a dependency. Listed-once also moves a plural to the front in 1 / 7 / 22 rows (`solitary` « isolate, appartato ») |
| Unglossed contractions | 57 / 147 / 462 | `you're`, `that's`, `i'll`; `doesn't` borrows `don't`'s « contrazione di do not » | no source writes them, and M5 forbids composing one: an owner's question for change 37 (a person-written list, or the gap accepted) |
| The determiners' heading | 5 rows | `my`, `your`, `his`, `its`, `thy` | Italian grammar says « aggettivo possessivo » where en-fr and en-es head them as determiners; put to the reviewer agent with change 34's « determinante » |
| A page copied from another word | | `some` (rank 60) « molti, tanti; la folla; molti, molte », the page copied from `many` with its pronunciation | wrong in the source, like `trash`; one of the baseline's 40 lemmas, so the golden and the card snapshot pin it until change 37 |
| Change 35's hand-offs | | words of other languages filed in the English section (`reclamo`, `modulate`); kaikki's no-gloss senses at the top; the English `zeta`'s letter name (rank 17,979, still glossed) | change 35's D7 and D10 |

### D11 — Not shipped

No package lists en-it before change 45, and no site figure moves (`gloss_coverage --check` reads shipped
pairs).

Change 34's hand-off of « participio presente » for an -ing form is closed: no en-it row says it.

## Risks / Trade-offs

- **A thinner pair than en-es** → its own floor, the gap published and explained; change 37 refines.
- **Copied table rules** → a third copy of en-es's readers until en-es re-pins; each copy's tests hold
  its behaviour.
- **A 3 GB download per update** → `keep=en`, as en-es.

## Migration Plan

Nothing deploys. The tables are committed by the owner-dispatched update's pull request, and the pack is
not listed. Rollback is a revert.

## Effort

3.5–6 ideal days, against the programme's 3.5–6:

| Part | Ideal days |
|---|---|
| Sources and registration, with the three tests and `SOURCES.md` | 0.5–0.75 |
| `reduce-en-it.py`, the copied readers, the share, the notice, its tests | 1–1.5 |
| Workflows and `test_reduce_loops` | 0.25–0.5 |
| The dispatch, the committed tables, the README | 0.5–1 |
| The golden, cross-native, committed-tables tests, testdata, the row cut | 0.75–1.25 |
| The card snapshot | 0.25–0.5 |
| The floor, OpenSpec, review | 0.25–0.5 |
