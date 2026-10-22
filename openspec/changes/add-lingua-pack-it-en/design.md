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
  (the reference of French; its tables read for expressions only). Both reduce with the English
  edition's rules, `reduce_edition_en.py`, whose bytes are in both pins (`pin.json` `reducer.files`),
  and fr-en's requirement says no other committed pair reads them (D11).
- **The sources**: change 9 pins the English Wiktionary's Italian section as `pack_sources.derive`
  writes it from the English dump (`kaikki-Italian.jsonl`); change 16 creates `EDITIONS["it"]` for the
  Italian Wiktionary.
- **Change 11**'s acronym rows (`NATO`, `USA`) are glossed per pair through their expansions or
  synonyms, and its `reduce_italian_acronyms.py` is imported by every pair glossing Italian; **change
  12** keys an Italian pack's expressions at build; **change 18**'s D8 hands this change the English
  card snapshot (`test/word-card-it-en.spec.ts`).

## Goals / Non-Goals

**Goals:**
- it-en's native side committed at or above a floor fixed before measuring, every gloss English
  written by a person, read against Italian's committed studied tables.
- The English edition's shared rules untouched; es-en's and fr-en's tables untouched.
- An it-en baseline over the Italian corpus, the studied side identical to it-fr's.

**Non-Goals:**
- Shipping it-en (change 25), its listings (change 26), its marks (change 24).
- The defects under *Known data defects*: change 23 (`refine-lingua-it-en-glosses`), after the
  committed tables, as `refine-lingua-fr-en-glosses` followed fr-en.
- Italian's dictionary words and levels: it-fr's (change 16).

## Measured

On the English dump of 2026-10-03 (2,981,058,381 B, the sha256 fr-en's pin records), derived by the
repository's own `pack_sources.derive` on `main` at `9c5c36b38`, with the shared rules and the English
edition's; reproduced by the review from the raw files:

- **The section** (`kaikki-Italian.jsonl` as derived: 656,480,462 B, 623,702 entries): 18,284
  proper-noun entries; 462,111 entries of pointer senses only; 143,179 entries with a meaning,
  135,532 distinct glossed headwords (noun 62,180; verb 46,550; adjective 25,022; adverb 4,909…).
  Changes 9, 11 and 13 recorded kaikki.org's own extract of the section (770,859,636 B, the same
  623,702 entries laid out differently); #997 corrects them.
- **Expressions**: 8,693 entries with a space, 8,584 distinct, 8,523 after the shared filters (302 of
  them species, « airone cenerino »). No headword writes `’` (9 words nest it inside entries; the
  French section's headwords write it 436 times). 108 words without a space that change 6's
  pre-pass splits — by the measurement's own reading of change 6's elision list —, 77 glossed with a
  meaning (`all'improvviso`, `d'accordo`, `senz'altro`, and `anch'io` « me too », whose pointer
  carries it), the rest pointers only (`c'è`, `c'era`).
- **The Italian Wiktionary's English translations** (`kaikki-it-traductions-en.jsonl`, derived from
  the Italian dump change 16 pins): 7,216,558 B, 42,969 entries, 109,653 translations, 32,836
  Italian words. **The English Wiktionary's Italian translations read backwards**
  (`kaikki-en-traductions-it.jsonl`): 9,643,709 B, 55,885 Italian words.
- **Coverage** of the 5,000 / 10,000 / 20,000 commonest lemmas, on the ranking change 16's review used
  (wordfreq's Italian order, change 9's drops reproduced; « strict » gives every form of another lemma
  to that lemma):

  | | loose | strict |
  |---|---|---|
  | definitions, the English edition's pre-passes | 91.5 / 84.7 / 76.2 | 84.6 / 76.9 / 68.1 |
  | + the Italian Wiktionary's translations | 91.8 / 85.1 / 76.7 | 85.1 / 77.5 / 68.7 |
  | + the English tables read backwards | 92.3 / 85.7 / 77.4 | 85.9 / 78.3 / 69.5 |
  | definitions, + a carried-meaning reader of every wording | 91.7 / 84.8 / 76.3 | 84.8 / 77.1 / 68.2 |

  fr-en committed 93.9 / 87.2 / 76.8 (floor 91.9 / 85.1 / 74.4); es-en 93.0 / 86.5 / 76.5; it-fr's
  study 87.5 / 79.6 / 70.5 from definitions. The ranking is read from this same section, which
  favours it-en: of 4,756 unglossed words in the loose 20,000, 4,617 have no entry at all
  (`facebook`, `of`, `piu`). Change 9's committed ranks will differ (the article `i`, which the dump
  holds only as the letter, is a form of `il` there).
- **The fallbacks' rows** (loose 20,000): the Italian Wiktionary adds 88 (`perfino` « even »,
  `malapena` « hardly, scarcely »; 11 of them repeat a sense across parts of speech, `ex` « ex, former;
  ex, former »); the English tables read backwards 149 more — in the top 5,000, 24 rows of which about
  6 are wrong (`es` « id », `etc` « cub », `cv` « hp », `del` « some; by », `xi`, `sur` « atop »).
- **Defects at the top** (5,000 / 10,000 / 20,000):
  - *A pointer carrying its meaning*: 13 / 20 / 28 lemmas unglossed (`apr`, `bot`, `degli`, `del`,
    `della`, `delle`, `dello`, `dic`, `ed`, `esse`, `ex`, `feb`, `giu`, `lug`, `mag`, `nov`, `ott`,
    `terapista`…), among them the `di` family (`del` « contraction of di + il; some, of the, from the
    »), `ed` « …for euphony; and » and the months' abbreviations; and misread: `ad` reads the acronym
    AD's « chief executive officer, CEO » where its own sense is « alternative form of a for euphony…;
    to, at, in », `gli` reads as a pronoun only (« form of the article i (“the”)…; the »), `mi` opens on
    « used as ethical dative » (« clitic accusative of io. me »). A reader of every carried wording
    regains these and mis-reads others: `quanti` « those who », `colla` opening on « with the »,
    `contenente` « containing” », `giu` « abbreviation of giugno (“June”) ».
  - *Names*: rows glossed by capitalised entries alone 250 / 581 / 1,369, mostly real names
    (`italia`, `milano`); a name's run opens 147 rows and follows a common word's in 1,009, as fr-en's
    did; 10 function words carry one (`io` « Io », `ora`, `meno` « Main », `voi`, `durante`, `anzi`).
    it-fr's D5, applied to this section as worded, strips the name from 1,145 of the 1,156 rows that
    mix a name with a common word — `roma` « a type of rice », `natale` without « Christmas », `terra`
    without « Earth », `luna` without « the Moon », `matteo` « mad, crazy » — and leaves eight
    (`giacomo`, `diego`…) with no gloss. Its acronym half alone clears `ad`.
  - *Letters*: the English edition's pre-passes take 29 letters' glosses (`b`, `h`, `esse`, `kappa`);
    Greek letters' names survive them: `mi` « mu (Greek letter) », `san` « san (Greek letter) »,
    `coppa` « koppa », `ni` « nu », `eta`, `ro`, `psi`, `omega`.
  - *Notes in glosses*: « (all senses) » in 85 / 137 / 220 rows; « See Category:Italian transitive
    verbs… » in `avere`; bracketed constructions and auxiliaries in 36 / 52 / 86 rows; « etc » without
    its period in 32.
  - *Pointer wordings the edition does not know*: about 40 untagged senses — « Old Italian form of »
    23 (`quei`, `quelli`), « censored spelling of » 8, « proscribed spelling of » 3 (`che`, rank 3, holds
    « proscribed spelling of ché »), « truncation of » — among 192 whose text opens like a pointer,
    most of them meanings (« a form of solidarity… »).
  - *Apocopes misread*: `quel` reads a usage note (« except impure s, gn, pn, ps, x, z »), `or`, `mar`,
    `san`; `gran` unglossed (kaikki's target « grande m or f by sense »); `alcun` unglossed (a
    determiner whose base is an adjective).
  - *Cognates* glossed by themselves: 281 rows (`idea`, `online`, `password`) — right as they are.
- **Size**: es-en's pack is 2,612,075 B, 49.8 % of the 5 MiB budget; it-en's is measured at its
  reduction.

## Decisions

### D1 — `reduce-it-en.py` builds the native side alone, from the committed studied tables

As `reduce-es-en.py` (es-en's D1): the reducer reads `tables/it/forms.tsv` and `freq.tsv` as committed
and builds `gloss.tsv`, `senses.tsv` and `mwe.tsv` with `common.native_tables(entries, ranks,
studied=IT, edition=english.EN, fallbacks=[(direct, list)])`, `IT` the Italian token pattern
redefined here to match it-fr's; nothing of the studied side is computed (no forms, readings, levels
or dictionary words — it-fr's). Its pin records the studied tables it read (es-en's D3), so a
re-reduced it-fr that moves one fails it-en's build until it-en is re-reduced (`pairs --after it-fr`).
Its rule digest is its own file, `reduce_italian_en.py` (D4), `reduce_italian_acronyms.py` (change
11), `reduce_common.py` and `reduce_edition_en.py`.

### D2 — Definitions, then the Italian Wiktionary's translations; no table read backwards

es-en's order: the section's definitions through the English edition's rules, then the Italian
Wiktionary's English translations, at most three per part of speech. Not the English tables read
backwards — es-en reads none, and in the top 5,000 they would add 24 rows of which about 6 are wrong
(*Measured*). it-fr reads both its tables (es-fr's order) because the French section misses more; the
English section misses less. Expressions follow the same sources (D5).

### D3 — The sources

An it-en update fetches the English dump (2,981,058,381 B) and derives the English catalogue, as
fr-en's does, and pins its own fetch of the section (`read_edition("en")`); a run where it-fr's update
brings it-en along reads no dump. The Italian Wiktionary's English translations are a new
`("translations", "it", "en")` file of `EDITIONS["it"]` (change 16), 7,216,558 B. `DUMPS["it-en"]`
names both; it-en publishes its own release (es-en's D2). The update workflow offers it-en
(`test_reduce_loops.py` holds the options) and reduces it after it-fr.

### D4 — it-en's own pre-pass, beside the English edition's, never in it

`reduce_edition_en.py`'s bytes are in es-en's and fr-en's pins: editing it re-pins both. What Italian
needs from the English edition goes into a module of its own, `reduce_italian_en.py`, imported by
`reduce-it-en.py` and by its tests (a dashed reducer cannot be imported, which is why fr-en keeps its
rules inside its reducer and it-en does not). It runs after the English edition's pre-passes
(`read_as_meanings`, as es-en calls them) and before the shared rules:

1. **Reads a meaning a pointer carries, by a reviewed list of wordings**, as `refine-lingua-fr-en-glosses`'
   D4 does with fr-en's `carried_meaning` and `_past_pointer`, plus Italian's shapes: « … for euphony;
   meaning », « form of the article … (“the”); meaning », and the case forms « clitic accusative of io.
   me », whose meaning follows the pointer's closing full stop (the edition's `_CASE_FORM` needs the
   case word to open the gloss and a colon). It reads `del` « some, of the, from the », `della` « of
   the, from the », `ed` « and », `ad` « to, at, in », `gli` « the » beside its pronoun senses, `mi` « me
   », `anch'io` « me too »; not every wording, which would read `quanti` « those who » and `giu`
   « abbreviation of giugno (“June”) » (*Measured*).
2. **Keeps acronyms off common words and names off function words**: the acronym half of it-fr's D5 —
   an acronym's sense glosses no lemma the section gives a lowercase entry of its own (`ad` without
   « CEO ») — and `refine-lingua-fr-en-glosses`' D6 — a name's run under a function word goes (`io`,
   `ora`, `meno`, `voi`, `durante`, `anzi`). Not it-fr's rule for names: on this section it strips
   « Rome », « Christmas » and « Earth » from the commonest rows (*Measured*). A name beside a common
   word's senses stays; the 1,009 rows where it follows them are change 23's to order.
3. **Glosses no word with a letter's name**: the English edition's own pre-passes for the Latin
   letters, as es-en calls them, and a sense naming a Greek letter (« mu (Greek letter) ») here.
4. **Takes out notes that are not glosses**: « (all senses) », « See Category:… ».
5. **Reads the words without a space that Italian's pre-pass splits**, which the shared
   `reduce_expressions` drops: the section's glossed with a meaning (`d'accordo`, `all'improvviso`),
   and the Italian Wiktionary's (6: `all'inizio`, `all'antica`, `sott'acqua`), as it-fr's D6 reads its
   tables' — a reader in this module after fr-en's `split_word`, which cannot be imported.

Task 0.1 measures items 1–5 together on change 9's committed ranks; their worth is at the top (`del`,
`gli`, `mi`, `ad`, `io`), not in the figure (a reader of every wording moved it 0.1 point).

### D5 — Expressions

`mwe.tsv` holds the section's glossed headwords with a space (8,523 after the shared filters), then
those the Italian Wiktionary's English translations gloss (1,192: « rosso vino », « meccanica
quantica »), then the split words of D4.5. Each is keyed at build by change 12's rules, which keep a
headword of two to seven tokens. The pointers-only ones (`c'è`, `c'era`) gloss nothing here: their
pieces do.

### D6 — The acronym rows

`tables/it/lexical.tsv` carries change 11's rows (`NATO`, `USA`), written by it-fr; it-en glosses them
by change 11's rule from its own glosses — a synonym's gloss read from its entry whatever its part of
speech, as change 16's D5 reads `stati uniti`'s.

### D7 — M6's rule: the floor fixed before the measurement

The figures above are on a ranking read from the same section, which favours it-en. So, as change 16's
D7: task 0.1 re-runs the study on change 9's committed ranks with D2's fallback and D4's pre-pass and
fixes `FLOORS["it-en"]` in `gloss_coverage.py` at those figures less two points, up or down, the owner
settling it before the first committed measurement; written into the requirement; never lowered after
it. Under it at the first measurement, the tables are not committed; a later reduction under it fails
the gloss-coverage floor step of `lingua-extension-check.yml`'s `reduce` job, which runs
`--pair it-en` and is renamed for the pairs it holds.

### D8 — Coverage and credits

`tables/it-en/README.md`: the coverage against the floor, the glossed lemmas by source, a sample of 100
glosses marked by source, the pack's size. The notice and the manifest credit both sides, as es-en's
requirement asks: the English Wiktionary's Italian section and the Italian Wiktionary for the glosses;
for the studied side, everything it-fr's notice credits — the English Wiktionary's Italian section,
wordfreq, UD Italian-MarkIT (CC BY 4.0), Morph-it! if change 9 keeps it — and the French Wiktionary's
Italian section, whose glosses decide Italian's dictionary words and levels (change 16); not UD
Italian-PUD, a gate only (change 9). The manifest says the levels are estimated and names Italian's
analyser version. `SOURCES.md` and the programme's *Measured figures* record the committed figures.

### D9 — An it-en baseline beside it-fr's, and the English card

`crates/lingua-wasm/tests/it_en_baseline.rs` and `baseline/it-en.golden`: change 5's Italian corpus
(`pages-it.txt`) read with the committed it-en pack, as `es_en_baseline.rs` reads Spanish with es-en;
a test that the golden is it-fr's on the studied side (es-en's
`the_golden_is_the_spanish_one_on_the_studied_side`); `cross_native.rs` gains Italian. The two
workflows that run the baselines (`lingua-extension-check.yml`, `lingua-pack-update.yml`) gain `--test
it_en_baseline`. Change 18's D8 hands this change the English card snapshot over the real pack
(`apps/lingua-extension/test/word-card-it-en.spec.ts`): taken here. The golden and the snapshot are
blessed in the pull request that commits it-en's tables (task 3.1's).

### D10 — Not shipped

`packs.json` does not move; no reader's package, engine or translation route moves (change 2 routes it-en
already, inert). `committed_tables.rs` asserts no package lists it-en while the list does not name it;
change 25 lists it, its `testdata/it-en/` with it.

### D11 — A third reader of the English edition's rules

fr-en's requirement *French is glossed in English from the English Wiktionary's French section* says
the English edition's rules « SHALL be read by es-en and fr-en and by no other committed pair », and its
scenario *A rule of the English edition* that only those two digests move. Committing it-en makes both
false; this change MODIFIES the requirement to name it-en, every other word and scenario kept, as
fr-es's requirement names the Spanish edition's readers. Archived after `add-lingua-pack-fr-en`, which
holds it.

## Known data defects

For change 23 (`refine-lingua-it-en-glosses`), after the committed tables: the pointer wordings the
English edition does not know (« Old Italian form of », « proscribed spelling of » — `che` at rank 3 —,
« censored spelling of », « truncation of »); apocopes misread (`quel`'s usage note, `or`, `mar`, `san`)
or left unglossed (`gran` by kaikki's target, `alcun` across the determiner and the adjective);
bracketed construction and auxiliary notes (86 rows of the 20,000); « etc » without its period; names'
runs after a common word's (1,009 rows, fr-en's class); the Italian Wiktionary's rows repeating a sense
across parts of speech (11: `ex`, `beige`, `wow`, `horror`); `negra` « black, nigger », unlabelled; the
species among the expressions (302); the `del` family's runs read as `X` (kaikki's `contraction`).

## Risks / Trade-offs

- [The study's ranking favours it-en] → the floor is fixed on change 9's committed ranks (D7).
- [Defects at the top left to change 23] → D4 takes the ones that reach the commonest words (`del`,
  `gli`, `mi`, `ad`, `io`); the rest are listed, and change 25 lists it-en only after change 23, as
  fr-en shipped after 48b.
- [A reviewed list of carried wordings misses some] → those left unglossed stay so (change 23); a
  reading of every wording would gloss more and mis-read the commonest (`quanti`).
- [A pre-pass module beside the English edition's] → one more file in it-en's digest; the English
  edition stays shared and untouched.

## Migration Plan

Nothing deploys: the tables are committed, the pack is not listed. Rollback is a revert.

## Effort

Against the programme's 3–5 ideal days: the reducer and its pre-pass module 1.25–1.75; the sources and
the pipeline (`pack_sources.py`, the update workflow, the floor step) 0.5; the reduction, the floor and
the README 0.5–1 (the owner's dispatch); the baseline, the English card snapshot, `cross_native.rs`
and the checks 0.5–1; the tests 0.5. 3.25–4.75.
