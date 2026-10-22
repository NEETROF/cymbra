# Design — add-lingua-pack-it-en

## Context

Change 22 of the [Italian programme](../../../docs/lingua/italian-programme.md), the first of stage
2: English speakers studying Italian. Where things stand (proposals on `main`; each a precondition
of the implementation, listed in the proposal's Impact):

- **Italian's studied side** is written once by the reference pair it-fr (`tables/it/`, change 9),
  with its dictionary words and levels from it-fr's glosses (change 16). A reader pair reads it as
  committed, as es-en reads `tables/es/` (matrix change 21, `add-lingua-pack-es-en`).
- **Reader pairs glossed in English** exist twice: es-en (the English Wiktionary's Spanish section,
  then the Spanish Wiktionary's English translations; no inverted table — `reduce-es-en.py`) and fr-en
  (the reference of French, its tables read for expressions only). Both reduce with the English
  edition's rules, `reduce_edition_en.py`, whose bytes are in both pins (`pin.json` `reducer.files`).
- **The source**: change 9 pins the English Wiktionary's Italian section, derived from the English
  dump by `pack_sources.derive` (`kaikki-Italian.jsonl`); change 16 creates `EDITIONS["it"]` for the
  Italian Wiktionary.
- **Change 11**'s acronym rows (`NATO`, `USA`) are glossed per pair through their expansions or
  synonyms; **change 12** keys an Italian pack's expressions at build.

## Goals / Non-Goals

**Goals:**
- it-en's native side committed at or above a floor fixed before measuring, every gloss English
  written by a person, read against Italian's committed studied tables.
- The English edition's shared rules, and es-en's and fr-en's pins, untouched.
- An it-en baseline over the Italian corpus, the studied side identical to it-fr's.

**Non-Goals:**
- Shipping it-en (change 25), its listings (change 26), its marks (change 24).
- The defects listed under *Known data defects*: change 23 (`refine-lingua-it-en-glosses`), after
  the committed tables, as `refine-lingua-fr-en-glosses` followed fr-en.
- Italian's dictionary words and levels: it-fr's (change 16).

## Measured

On the English dump of 2026-10-03 (2,981,058,381 B), derived by the repository's own
`pack_sources.derive` on `main` at `9c5c36b38`, with the shared rules and the English edition's:

- **The section** (`kaikki-Italian.jsonl` as derived: 656,480,462 B, 623,702 entries): 18,284
  proper-noun entries; 462,111 entries of pointer senses only; 143,179 entries with a meaning,
  135,532 distinct glossed headwords (noun 62,180; verb 46,550; adjective 25,022; adverb 4,909…).
  Change 9's design records 770,859,636 B for the same 623,702 entries — kaikki.org's own extract,
  not the file the pipeline derives; corrected there by a separate amendment.
- **Expressions**: 8,693 entries with a space, 8,584 distinct, 8,523 after the shared filters (302
  of them species, « airone cenerino »); 9 headwords write `’` (the French section 436). 108 words
  without a space that change 6's pre-pass splits, 77 with a meaning (`all'improvviso`,
  `d'accordo`, `senz'altro`), 31 pointers only (`c'è`, `anch'io`).
- **The Italian Wiktionary's English translations** (`kaikki-it-traductions-en.jsonl`, derived from
  the Italian dump change 16 pins): 7,216,558 B, 42,969 entries, 109,653 translations, 32,836
  Italian words. **The English Wiktionary's Italian translations read backwards**
  (`kaikki-en-traductions-it.jsonl`): 9,643,709 B, 55,885 Italian words.
- **Coverage** of the 5,000 / 10,000 / 20,000 commonest lemmas, on the ranking change 16's review
  used (wordfreq's Italian order, change 9's drops reproduced; « strict » gives every form of
  another lemma to that lemma):

  | | loose | strict |
  |---|---|---|
  | definitions, the English edition's pre-passes | 91.5 / 84.7 / 76.2 | 84.6 / 76.9 / 68.1 |
  | + the Italian Wiktionary's translations | 91.8 / 85.1 / 76.7 | — |
  | + the English tables read backwards | 92.3 / 85.7 / 77.4 | 85.9 / 78.3 / 69.5 |
  | definitions, + D4's names rule and a carried-meaning reader | 91.6 / 84.7 / 76.2 | 84.6 / 76.9 / 68.1 |

  fr-en committed 93.9 / 87.2 / 76.8 (floor 91.9 / 85.1 / 74.4); es-en 93.0 / 86.5 / 76.5; it-fr's
  study 87.5 / 79.6 / 70.5 from definitions. The ranking is read from this same section, which
  favours it-en: of 4,756 unglossed words in the loose 20,000, 4,617 have no entry at all
  (`facebook`, `of`, `piu`). Change 9's committed ranks will differ (the article `i`, which the dump
  holds only as the letter, is a form of `il` there).
- **The fallbacks' rows** (loose 20,000): the Italian Wiktionary adds 88 (`perfino` « even »,
  `malapena` « hardly, scarcely », `ex` « ex, former »); the English tables read backwards add 149
  more, with noise (`etc` « cub », `cv` « hp », `es` « id », `ad` « to; CEO »).
- **Defects at the top** (5,000 / 10,000 / 20,000):
  - *A pointer carrying its meaning*: 13 / 20 / 28 unglossed lemmas — `del`, `della`, `delle`,
    `degli`, `dello` « of the, from the », `esse`; and shapes fr-en's reader refuses: `ed` and `ad`
    « alternative form of e for euphony…; and », `gli` « form of the article i (“the”)…; the » (read
    as a pronoun only), `mi` « clitic accusative of io. me » (opening on « used as ethical dative »).
  - *Names*: rows glossed by capitalised entries alone 250 / 581 / 1,369, mostly real names
    (`italia`, `milano`); 9 / 19 / 40 of them have a lowercase entry of their own (it-fr's D5 case:
    `matteo` « mad, crazy », `torno` « lathe », `rota` « wheel »); a name's run opens 147 rows and
    follows another run in 1,009, as fr-en's did; 10 function words carry one (`io` « Io », `ora`,
    `meno` « Main », `voi`, `durante`, `anzi`).
  - *Letters*: the English edition's pre-passes take 29 letters' glosses (`b`, `h`, `esse` as a
    letter's name).
  - *Notes in glosses*: « (all senses) » in 85 / 137 / 220 rows; « See Category:Italian transitive
    verbs… » in `avere`; bracketed constructions and auxiliaries in 36 / 52 / 86 rows; « etc »
    without its period in 32.
  - *Pointer wordings the edition does not know*: 192 untagged senses — « Old Italian form of »
    (`quei`, `quelli`), « proscribed spelling of » (`che`, rank 3, holds « proscribed spelling of
    ché »), « censored spelling of », « truncation of ».
  - *Apocopes misread*: `quel` reads a usage note (« except impure s, gn, pn, ps, x, z »), `or`, `mar`,
    `san` (« san (Greek letter) »); `gran` unglossed (kaikki's target « grande m or f by sense »);
    `alcun` unglossed (a determiner whose base is an adjective).
  - *Cognates* glossed by themselves: 281 rows (`idea`, `online`, `password`) — right as they are.

## Decisions

### D1 — `reduce-it-en.py` builds the native side alone, from the committed studied tables

As `reduce-es-en.py` (es-en's D1): the reducer reads `tables/it/forms.tsv` and `freq.tsv` as
committed and builds `gloss.tsv`, `senses.tsv` and `mwe.tsv` with `common.native_tables(entries,
ranks, studied=IT, edition=english.EN, fallbacks=[(direct, list)])`; nothing of the studied side is
computed (no forms, readings, levels or dictionary words — it-fr's). Its pin records the studied
tables it read (es-en's D3), so a re-reduced it-fr that moves one fails it-en's build until it-en is
re-reduced (`pairs --after it-fr`).

### D2 — Definitions, then the Italian Wiktionary's translations; no table read backwards

es-en's order: the section's definitions through the English edition's rules, then the Italian
Wiktionary's English translations, at most three per part of speech. Not the English tables read
backwards — es-en reads none, and *Measured* shows what they would add at the top: 149 rows at the
20,000, with `etc` « cub », `cv` « hp », `es` « id ». it-fr reads both its tables (es-fr's order)
because the French section misses more; the English section misses less. Expressions follow the
same sources (D5).

### D3 — The sources: no new download for definitions

it-en reads change 9's `kaikki-Italian.jsonl` as pinned, and `kaikki-it-traductions-en.jsonl`, a
new `("translations", "it", "en")` file of `EDITIONS["it"]` (change 16's Italian Wiktionary),
7,216,558 B. `DUMPS["it-en"]` names both; each pair pins its own extract and publishes its own
release (es-en's D2). The English dump is not fetched again for it-en.

### D4 — it-en's own pre-pass, beside the English edition's, never in it

`reduce_edition_en.py`'s bytes are in es-en's and fr-en's pins: editing it re-pins both. So what
Italian needs from the English edition goes into a module of its own, `reduce_italian_en.py`,
imported by `reduce-it-en.py` alone and in its rule digest (as `reduce_french_treebank.py` is
fr-en's and fr-es's). fr-en's `carried_meaning` (`reduce-fr-en.py`) is its model, not imported — a
dashed file cannot be, and moving it would re-pin fr-en. Before the shared rules, the pre-pass:

1. **Reads a meaning a pointer carries**: a pointer sense that also gives the meaning — after `;` or
   `:`, or, for a case form, after the pointer's closing full stop — glosses its headword with it — the `di` family (`del` « of the, from the »),
   `ed` / `ad` « and », `gli` « the » beside its pronoun senses, `mi` « me » (the edition's case-form
   shape needs a case word and a colon that Italian's « clitic accusative of io. me » does not have).
2. **Keeps names off common words**: it-fr's D5 — a proper noun's or an acronym's sense glosses no
   lemma the section gives a lowercase entry of its own (`torno` « lathe » alone); and fr-en's
   D6 — a name's run under a function word goes (`io`, `ora`, `meno`, `voi`, `durante`, `anzi`).
3. **Glosses no word with a letter**: the English edition's own pre-passes already do it, called as
   es-en calls them (29 letters' glosses at the 20,000: `b`, `h`, `esse`, `kappa`).
4. **Takes out notes that are not glosses**: « (all senses) », « See Category:… ».

The rest of *Measured*'s defects — pointer wordings the edition does not know, apocopes misread,
bracketed construction notes, « etc » — are change 23's, read on the committed tables and the
sample, as fr-en's were 48b's (*Known data defects*). Measured with items 1 and 2:
91.6 / 84.7 / 76.2 from definitions; their worth is at the top (`del`, `gli`, `mi`, `io`), not in
the figure.

### D5 — Expressions

`mwe.tsv` holds the section's glossed headwords with a space (8,523 after the shared filters), then
those the Italian Wiktionary's English translations gloss (1,192: « rosso vino », « meccanica
quantica »), then the 77 words without a space that Italian's pre-pass splits and the section glosses
with a meaning (`d'accordo` « agreed », `senz'altro`), read as fr-en reads a word the tokenisation
splits alone. Each is keyed at build by change 12's rules. The 31 that are pointers only (`c'è`)
gloss nothing here: their pieces do.

### D6 — The acronym rows

`tables/it/lexical.tsv` carries change 11's rows (`NATO`, `USA`), written by it-fr; it-en glosses
them by change 11's rule from its own glosses — a synonym's gloss read from its entry whatever its
part of speech, as change 16's D5 reads `stati uniti`'s.

### D7 — M6's rule: the floor fixed before the measurement

The figures above are on a ranking read from the same section, which favours it-en. So, as change 16's
D7: task 0.1 re-runs the study on change 9's committed ranks with D2's fallback and D4's pre-pass and
fixes `FLOORS["it-en"]` in `gloss_coverage.py` at those figures less two points, up or down, the owner
settling it before the first committed measurement; written into the requirement; never lowered after
it. Under it at the first measurement, the tables are not committed; a later reduction under it fails
the floor step of `lingua-extension-check.yml`'s `reduce` job, which runs `--pair it-en`.

### D8 — Coverage committed and shown

`tables/it-en/README.md`: the coverage against the floor, the glossed lemmas by source, a sample of 100
glosses marked by source, the pack's size; `SOURCES.md` the sources and licences (both Wiktionaries,
CC BY-SA); the programme's *Measured figures* the committed figures. Stored in no pack; the site
publishes nothing until change 25 lists it-en.

### D9 — An it-en baseline beside it-fr's

`crates/lingua-wasm/tests/it_en_baseline.rs` and `baseline/it-en.golden`: change 5's Italian corpus
(`pages-it.txt`) read with the committed it-en pack, blessed in this change's pull request, as
`es_en_baseline.rs` reads Spanish with es-en. `cross_native.rs` gains Italian: the same corpus read
with it-fr's and it-en's packs gives the same tokens, lemmas, readings and levels — the studied side
is one — and only the glosses differ.

### D10 — Not shipped

`packs.json` does not move; no reader's package, engine or translation route moves (change 2 routes
it-en already, inert). Change 25 lists it-en, its `testdata/it-en/` with it, as `enable-lingua-french`
wrote fr-es's.

## Known data defects

For change 23 (`refine-lingua-it-en-glosses`), after the committed tables: pointer wordings the English
edition does not know (« Old Italian form of », « proscribed spelling of » — `che` at rank 3 —,
« censored spelling of », « truncation of »); apocopes misread (`quel`'s usage note, `or`, `mar`,
`san`) or left unglossed (`gran` by kaikki's target, `alcun` across the determiner and the adjective);
bracketed construction and auxiliary notes (86 rows of the 20,000); « etc » without its period; the
species among the expressions (302); names' runs after a common word's (1,009 rows, fr-en's class);
the `del` family's runs read as `X` (kaikki's `contraction`).

## Risks / Trade-offs

- [The study's ranking favours it-en] → the floor is fixed on change 9's committed ranks (D7).
- [Defects at the top left to change 23] → D4 takes the ones that reach the commonest words (`del`,
  `gli`, `mi`, `io`); the rest are listed, and change 25 lists it-en only after change 23, as fr-en
  shipped after 48b.
- [A pre-pass module beside the English edition's] → one more file in it-en's digest; the English
  edition stays shared and untouched.

## Migration Plan

Nothing deploys: the tables are committed, the pack is not listed. Rollback is a revert.

## Effort

Against the programme's 3–5 ideal days: the reducer and its pre-pass module 1–1.5; the sources and
the pipeline (`pack_sources.py`, the update workflow, the floor step) 0.5; the reduction, the floor and
the README 0.5–1 (the owner's dispatch); the baseline, `cross_native.rs` and the checks 0.5–1; the
tests 0.5. 3–4.5.
