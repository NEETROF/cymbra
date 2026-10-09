# Design — add-lingua-pack-fr-en

## Context

See proposal.md (Why). Where fr-en stands, and what this design builds on:

| What | Where, today |
|---|---|
| French's studied side | change 43 (`add-lingua-french-forms-tables`, proposed, implementation in progress): `reduce-fr-en.py` is French's reference reducer and writes `tables/fr/` (`forms.tsv`, `freq.tsv`, `studied.json` naming fr-en, an empty `tags.tsv` and an empty `lexical.tsv`) and `tables/fr-en/` with an empty `gloss.tsv`; its rule digest is `reduce-fr-en.py` and `reduce_common.py`; its pin names `kaikki-French.jsonl`, derived from the English edition's dump, GSD's two sections and wordfreq. Its implementation ranks a lemma only when its own form reads as itself (`donnée`, read as *donner*, leaves the ranks: the builder finds a lemma's rank, gloss and level by its own form), and is still fixing lemmas such as `ma`, `mes`, `ils`, `cet`, ranked as their own. Changes 45 and 46 add `grammar.tsv`, the tag pool and `level.tsv` to the same reducer |
| The English edition's rules | `reduce_edition_en.py`: `EN` (pointer wordings and fields, letters, `capitalised=False`, `LONG_PARENTHESIS` 0), `without_letter_headwords`, `merge_same_pos_etymologies` (off), `read_as_meanings` (23b: D2 nested senses, D3 shortened and case forms, D4 a function word spelled like a place, D5 one typography). Loaded by es-en alone today. M20's two settings are the owner's (change 21, tasks 2.2 and 5.1), committed at 0 and off |
| The shared native side | `reduce_common.native_tables(entries, lemmas, studied, edition, fallbacks)`: the edition's entries gloss the lemmas (eight whole senses, grouped by part of speech) and the multi-word headwords; each fallback glosses what they leave out, words and expressions alike |
| The catalogue | change 38's `EDITIONS`: `kaikki-French.jsonl` (« French's studied side and fr-en's glosses (changes 43, 45, 48) »), `kaikki-en-traductions-fr.jsonl` (« fr-en's inverted fallback (change 48) »), `kaikki-fr-traductions-en.jsonl` (« fr-en's direct fallback (change 48) ») |
| Dictionary words | *A pack's dictionary words do not depend on its glosses*: « the lemmas its studied language's reference pack glosses — … for a language studied later the first pack built for it »; `split` writes them to `tables/<studied>/lexical.tsv` from the reference's `gloss.tsv`. They feed the vocabulary estimate (`Pack::dictionary_words`) and a names rule (`Pack::is_dictionary_word`, Spanish's `document_names`) |
| The French baseline | change 39: `french_baseline.rs` over `pages-fr.txt` (13 pages), `support/french.rs` with `pack: PackSource::Testdata` (the fixture: 296 forms, 114 ranks, 70 glosses, 9 expressions, 53 levels, no grammar, at analyser `0.2.0` since change 40), es-en beside it; 141 probes; *Hand-over* (D5): « when change 48 commits `tables/fr/` and `tables/fr-en/`, the scenario's pack source switches to the committed tables and the golden is re-blessed once, in that pull request » |
| Coverage | `gloss_coverage.py`: `FLOORS` es-en (es-fr's published figures), en-es (the study's figures less two points); the reduce job runs `--pair` for each; `lingua-coverage.json` lists the shipped pairs alone |

## Goals / Non-Goals

**Goals:**
- fr-en's English glosses, expressions and sense runs, from a source a person wrote in English for
  French words, read as meanings, committed and pinned beside French's studied tables.
- French's dictionary words: words of French, not whatever a table happens to translate.
- The French baseline on the real pack, re-blessed once and reviewed so the diff is read.
- en-fr, es-fr, es-en and en-es byte for byte.

**Non-Goals:**
- Shipping fr-en (52: `packs.json`, the site's figures, the listings); the card's English wording for
  French (51); marks (50); fr-es (49).
- New rules of the English edition (they would re-pin es-en) or a fr-en refinement: the defects
  left are listed (D7) for one, as change 23 listed es-en's for 23b.
- French's studied side (43, 45, 46): the forms, ranks, readings and levels are read as the reducer
  computes them, and move nowhere here.
- M20: the owner's two settings stay where change 21 left them (D5).

## Measured

A prototype in the scratchpad (never committed): change 43's implementation tables in progress
(branch `claude/add-lingua-french-forms-tables-impl`, reduced on 2026-10-09 before its fixes:
124,040 forms, 60,000 lemmas, six of them lemmas whose own form reads as another, `donnée` among
them), the French section change 38 derived on 2026-10-08 from the English dump regenerated on
2026-10-03 (`kaikki-French.jsonl`, 403,269 entries, the bytes change 43's pin records), the French
Wiktionary's English translations derived from the French dump regenerated on 2026-10-02
(`kaikki-fr-traductions-en.jsonl`, 108,947 entries that are not names), the English Wiktionary's
French translations (`kaikki-en-traductions-fr.jsonl`, 128,303 French translations listed, 74,233
French words and expressions read backwards), the rules of `origin/main` (`reduce_common.py`,
`reduce_edition_en.py` with M20 at 0 and off, the same at `1cfa4d99` and `35faf774`), wordfreq
3.1.1. Packs built by `lingua-pack-build` from `origin/main`'s builder; the golden by
`french_baseline.rs` with its pack source switched, in a scratch copy of `origin/main` at
`35faf774` (change 40 landed: French's tokenisation at `0.2.0`). Re-run by the review: every table
byte for byte; on change 43's tables after its first fix (`donnée` and five others out of the
ranks), 30,056 glossed lemmas, 93.6 / 86.8 / 76.3 %.

**The sources.** Coverage is `gloss_coverage.py`'s: the share of the 5,000 / 10,000 / 20,000
commonest lemmas of `tables/fr/freq.tsv` with a gloss.

| Native side | Glossed lemmas | 5k / 10k / 20k | Expressions | Pack |
|---|---|---|---|---|
| The French section, without `read_as_meanings` | 30,057 | 93.6 / 86.9 / 76.3 % | 15,524 | — |
| **The French section, with `read_as_meanings` (this design)** | **30,059** (4,678 / 8,686 / 15,256) | **93.6 / 86.9 / 76.3 %** | **15,526** | **2,201,349 B** |
| + the French Wiktionary's English translations (direct), words and expressions — es-en's shape | 31,735 (+1,676; 134 of the top 10,000) | 94.3 / 88.2 / 78.1 % | 37,542 (+22,016; +24,138 with its headwords' ’ read as `'`) | 2,582,698 B |
| + the English Wiktionary's French translations, backwards (inverted) — en-es's shape | 32,047 (+312; 38) | 94.6 / 88.6 / 78.7 % | 50,411 (+12,869) | 2,773,699 B |
| The section for words, the direct table for expressions only | 30,059 | 93.6 / 86.9 / 76.3 % | 39,664 | 2,564,304 B |

The programme's « 93.9 / 87.1 / 76.4 % (prototype ranks, ± 1–2) » names the section alone as its
source; on change 43's tables it reads 93.6 / 86.9 / 76.3. The pack of change 43's two tables alone is 1,239,838 B;
with changes 45's and 46's prototype tables (`grammar.tsv`, the pinned pool, `level.tsv`) the
proposed pack is 2,421,321 B. es-en's is 2,567,804 B.

**What the tables gloss.** Of the 1,676 words the direct table glosses, 1,603 (112 of the top 10,000)
have no entry in the French section at all, 18 only a name's; 1,041 list the word itself among their
translations. Of the inverted table's 312, 292 have no entry and 152 list the word itself. They are
English words in French text (« in » « in », « and » « AND », « end » « NDE, NDI, NDT », « at »,
« out », « king » « sounding stone, bianqing, king »), names (« kim », « sam », « jack »),
initialisms (« sh » « CW », « cde » « end of operation »), unaccented misspellings (« tres » « too »,
« secretaire » « amanuensis »). Kept to the 64 words the section lists but does not gloss, and that
are not glossed by themselves (16 of the top 10,000), about half still read wrong: « stp » « pipeline-end
termination », « french » « French kiss », « v » « G », « ep » « cockpit voice recorder, CVR, EP ».

**What the tables' expressions are met as.** Keyed by the forms table and counted in UD French's
425,111 syntactic words (GSD's three sections and PUD; UD writes `du` as `de le`, so the count is an
approximation): the section's expressions occur 32,998 times (2,223 distinct), the direct table's
3,248 (828 distinct), the inverted table's 6,443 (1,062 distinct). The inverted table's commonest
are French's commonest bigrams, which it makes expressions because an English entry lists them as
its French translation: « il est » "he's" (782), « la plus » "most" (421), « tous les » "all"
(346), « plus de » "above, upwards of" (227), « la mort » "death" (62), « dans laquelle »
"whereinto" (52). The direct table's are mostly the French Wiktionary's own locutions, some right
(« plus tard » "later" 124, « autour de » "around", « il existe » "there exists", « première guerre
mondiale »), some compositional and wrong in most sentences (« le grand » "the Great" 158, « les
autres » "other people" 150). A selection's phrase gloss shows every expression it holds
(`match_expressions`), so each is read where a reader selects it.

**23b's rules on fr-en.** Each rule of `read_as_meanings` alone, then together, against the section
without them:

| Rule | Rows / top 10k | First sense | Expressions | Examples |
|---|---|---|---|---|
| D2, nested senses | 21 / 13 | 12 / 7 | 1 changed, 1 gained | « chambre » (324) « a chamber in its various senses, including » → « a room; a hotel room; a bedroom; a house of a parliament »; « champ », « scène », « nous » (34) « the plural personal pronoun in the first person; … » → « we; us, to us; … », « concevoir », « parrain », « ours » |
| D3, shortened and case forms | 8 / 4; 2 lemmas gained (« eine », « mam'zelle ») | 3 / 0 | 1 gained | « elle » (28) gains « her, it, à elle = hers, its », « elles » « them (female) », « pus » « more, no more, no longer » |
| D4, a function word spelled like a place | 0 | 0 | 0 | — |
| D5, one typography | 243 / 110 | 93 / 30 | 141 | « du » (10) « Forms the partitive article » → « forms the partitive article », « se », « que », « ou » « either...or » → « either … or », « dont »'s « translated as "including" » → « … as “including” » |
| **Together** | **269 / 125** | **107 / 37** | **142 changed, 2 gained** | no gloss lost |

Before them, the section reads as es-en read before 23b: 8 / 6 rows with `...`, 46 / 28 with
straight quotes, 188 / 77 opening a sense on one of the edition's descriptions in a capital
(110 expressions). After them, no word's gloss holds any of those.

**M20 on fr-en.** The long-parenthesis bound at 40 changes 2,342 rows (902 of the top 10,000), the
etymology merging 304 (168); neither loses a gloss.

**The golden.** `fr-en.golden` re-blessed over the prototype pack, against the fixture's on `main`
at `35faf774` (French at `0.2.0`, change 40's re-bless: 141 probes, 162,405 B):

| | Probes moved, of 141 | Unchanged | Bytes |
|---|---|---|---|
| Changes 43 and 48 only (no readings, no levels) | 132 | 9 (`about`, `beside es-en`, four glosses, `calibration`, `declared-level`, `export-declared-levels`) | 162,405 → 364,717 |
| **With changes 45's and 46's prototype tables** (46's `levels_estimated` set) | **125** | **16** (those, and `has-levels`, both `seed-level`, `start-review`, `review-remaining`, `deck-count`, `due-count`) | **162,405 → 332,811** |

(Before change 40, on `1cfa4d99`: 127 and 120 of 136.) Without levels, `has-levels` turns false, the
ladder empties, `seed-level` adds nothing and the deck holds 3 cards instead of 11: the fixture's 53
levels are what kept those probes answering. On the thirteen analysed pages, 1,002 tokens become
1,004 (`M` on `noms` and `e` on `mixte`, single letters the lexicon now lists); 113 tokens' lemmas
move (`présenté` → *présenter*, `travaux` → *travail*, `porte` → *porter*, M8; nine read as
`ma`, `mes`, `ils`, `cet`, `amie`, change 43's lemmas, which its fixes in progress may take back) and 42
tokens' classes, the proper nouns out of the lexicon going from 44 to 2 (the real lexicon holds the
names); glossed tokens go from 387 to 975; the vocabulary estimate's universe from 55 to 30,094
dictionary words (30,054 without levels: a levelled lemma counts, D10). Of the 82 JSON probes that
move with 45's and 46's tables, 67 move in what is not a gloss (lemmas, classes, readings, counts)
and 15 in their glosses alone.

The prototype's pack also glossed *venir* « coming, arrival »: `venue`, ranked in change 43's tables
before its fix while its own form reads as *venir*, had its row attached to *venir* by the builder,
which keys a gloss by looking its lemma up as a form. On change 43's tables after that fix the same
132 probes move and `gloss venir` reads « to come … ». D8's review and task 3.1 check for it.

**The sample.** 100 rows drawn from the top 10,000 (seed 48) read right but for the page's own
notes: « heure » « hour, time; o'clock », « planète » « planet », « casser » « to break; to break up
(with somebody); to get lost, go away, skidaddle, leave, see se casser », « blond » « …; blond (all
senses) », « téléphonie » « telephony (2); … », « serre » « …; Press or pinces allowing… ». The pull
request draws it again from its own tables for the owner (task 6.1).

## Decisions

### D1 — fr-en's native side, in French's reference reducer, through the English edition's rules

`reduce-fr-en.py` gains the native side, after the studied side it computes (change 43), as
`reduce-es-en.py` writes es-en's:

1. **The section, cut** (`native_fields`): each entry's `word`, `pos` and senses (`glosses`, `tags`,
   `form_of`, `alt_of`), a typographic apostrophe in the headword read as `'` — the way French's forms
   are read (change 43 D3) and the core reads a page (`push_word`): 3 headwords (« nombre d’oxydation »,
   « c’est carré comme en Corée », « jaune d’oeuf »). The case is kept: the shared rules read an
   acronym by its capitals and D4 a place by its initial.
2. **The English edition's pre-passes**, in es-en's order: `common.without_letter_senses` (a
   `character` entry, a sense naming a letter), `english.without_letter_headwords` (a single capital
   letter), `english.read_as_meanings` (D2), `english.merge_same_pos_etymologies` (M20's setting).
3. **The shared rules**: `common.native_tables(entries, ranks, studied=FR, edition=english.EN,
   fallbacks=[])` over the 60,000 ranked lemmas the reducer has just computed — `gloss.tsv`,
   `senses.tsv`, `mwe.tsv`. `FR` is French's `common.Studied`: change 43's token pattern (letters,
   hyphens, an inner apostrophe, an elided piece's final one), and French's seven coordinating
   conjunctions — *et*, *ou*, *mais*, *ni*, *or*, *car*, *donc* — which make kaikki's `conj` a CCONJ in
   a sense run (change 45 writes the same `FR` for the readings; whichever lands second reuses it).
4. **The credits**: `NOTICE` and the manifest name the English Wiktionary's French section for the
   English glosses of French words and expressions too; no source is added.

Nothing of the studied side changes: the same lemmas, ranks and forms. The glosses are keyed by the
ranked lemmas, and the pack builder attaches a gloss to the lemma its key reads as in the forms table
(`FstLexicon::id_of`): a ranked lemma whose own form reads as another would lend its gloss to that
word. Change 43's implementation ranks no such lemma (its fix of 2026-10-09: `donnée`, read as
*donner*, leaves the ranks); the prototype, on its tables before that fix, glossed *venir* with
*venue*'s « coming, arrival » (*Measured*). This change relies on no such lemma and checks that none
is glossed (task 3.1). fr-en's rule digest becomes `reduce-fr-en.py`, `reduce_common.py` and
`reduce_edition_en.py` (`rule_files`, by its imports).

*Rejected — a module of French native-side rules.* fr-es (49) glosses in Spanish from other editions
and would load nothing of it; fr-en has no rule of its own to put there (D2, D3).

### D2 — 23b's pre-pass runs on fr-en, as it is

The English Wiktionary writes its French section as it writes its Spanish one, and the measurement
shows the same layouts read as meanings: a sense-group label or a list's introduction (« chambre »,
« champ »: « … in its various senses, including »), a pronoun's senses under its description
(« nous »), the edition's descriptions in a capital (188 rows, 77 of the top 10,000), ellipses
written `...` (8 rows), straight quotes (46). `read_as_meanings` reads them right on fr-en — 269 rows, 125 of the top 10,000, the first
sense of 107 / 37, 2 glosses gained (« eine », « mam'zelle »), none lost — and moves nothing it should
not: D4 changes no French row, D3's lending stays bounded by `_MIN_BASE`.

Nothing is added to the English edition here. A rule there re-pins es-en and moves its rows, a
refinement of its own; a rule only fr-en needs belongs in `reduce-fr-en.py`, and none is written
before the owner has read the sample and the defects (D7). The pre-pass's rules are the English
edition's, so from this change on they are es-en's and fr-en's alike: an edit to
`reduce_edition_en.py` re-pins both (D5).

*Rejected — fr-en without the pre-pass.* It would ship, to English speakers, the layouts 23b removed
for them in es-en: « chambre » read as « a chamber in its various senses, including ».

### D3 — No translation table: the French section alone glosses words and expressions

M5 lets a word a person wrote into a translation table gloss, when the table pairs the two languages
directly; es-en reads the Spanish Wiktionary's English translations, en-es both of its tables. For
fr-en, measured, neither table is taken:

- **Words.** The section already holds French's words: what a table adds is a word the section has
  no entry for (1,895 of the 1,988), and 1,193 of them list the word itself as its translation
  (*Measured*). fr-en is French's reference pair: every lemma it glosses is a dictionary word of
  French (*A pack's dictionary words do not depend on its glosses*), so « in », « end », « kim » and
  « sam » would be counted by every French reader's vocabulary estimate, in fr-en and in fr-es alike,
  and kept as words by a names rule that reads dictionary words. Kept only for words the section
  lists, the table glosses 64, about half of them wrong.
- **Expressions.** The inverted table makes the commonest French bigrams expressions (« il est »
  "he's", « la plus » "most", « tous les » "all"): 6,443 occurrences in 425,111 words, against the
  section's 32,998, and wrong wherever they are not a translation of an English word. The direct
  table's are mostly the French Wiktionary's locutions, 3,248 occurrences (10 % of the section's), some
  compositional and wrong (« le grand » "the Great", « les autres » "other people"), for 24,138 more
  rows and 363 KB of pack.

A lemma or an expression the section does not gloss has no gloss. The two catalogue entries change
38 registered for this change (`kaikki-en-traductions-fr.jsonl`, `kaikki-fr-traductions-en.jsonl`)
are removed from `EDITIONS`: no pair reads them, and an entry nobody reads is derived on every update
of its edition. No pin names them, no pair's `DUMPS` lists them (change 43 registers
`kaikki-French.jsonl` alone), and `pack_sources.py` is in no rule digest, so nothing is re-pinned and
no other pair's tables can move. What moves is tests and words: `test_pack_sources.py`'s test of
change 38's scenario *A pair of stage 3 registers what it reads* (fr-en registering the three files,
`check_registered`) and of *The catalogue derived whole* (the French edition's derived
`kaikki-fr-traductions-en.jsonl`), its note of the English catalogue's « 4 files », change 43's test
of the English catalogue's four names and its `DUMPS` comment (« fr-en's glosses add the translation
tables »), `SOURCES.md`'s catalogue table. Change 38's scenario reads « WHEN fr-en is added with … as
its sources »: a condition this change does not meet, its wording left to Open Question 4.

*Rejected — es-en's shape (the direct table, words and expressions).* +1.3 points at 10,000, for the
noise above, made French's dictionary words.
*Rejected — the direct table for expressions only.* The best of the tables, and the owner's call
(Open Question 1): it needs a source fr-en's pin does not hold, so a fetch and a new snapshot, which
would move `tables/fr/` in this pull request (D5). A later update of fr-en can add it, as a change of
its own.

### D4 — A floor fixed before the committed measurement, nothing published

`gloss_coverage.py` gains `FLOORS["fr-en"] = (91.9, 85.1, 74.4)`: the study's 93.9 / 87.1 / 76.4 less
two points, en-es's rule. No French pair is published to hold fr-en to (es-en was held to es-fr's
published figures); M6 is fr-es's. The prototype measures 93.6 / 86.9 / 76.3, 1.7 / 1.8 / 1.9 points
above. The reduce job runs `gloss_coverage.py --pair fr-en`, which fails under the floor; the site's
figures (`--write`) list the shipped pairs alone, so fr-en's appear with change 52. The owner settles
the floor (task 6.1), here and nowhere else.

### D5 — Re-reduced from change 43's pin; French's dictionary words

fr-en is reduced again from the pin change 43 recorded — `build.sh --reduce fr-en`, nothing fetched
but what the pin records (its release's `kaikki-French.jsonl` asset, GSD's two files by URL and
sha256, wordfreq 3.1.1) — with the native side added: the pin's `snapshot` and `sources` byte for
byte, its `reducer` digest and `pack` moved, `pack_version` `<snapshot>+<digest[:7]>` (fr-en is a
reference: no studied digest). `tables/fr/`'s forms, ranks — and readings, tag pool and levels, if
changes 45 and 46 have landed — come out byte for byte; `split` writes `tables/fr/lexical.tsv`, fr-en's
glossed lemmas (30,059 in the prototype), and the fr-en pack carries no lexical table, its dictionary
words being its glosses. fr-es (49) reads them as committed: its pin's studied record will name this
`lexical.tsv`, so fr-es follows this change.

From now on `reduce_edition_en.py` is in es-en's and fr-en's digests. A change to the English
edition re-pins both, and a gloss it adds or removes moves French's dictionary words, and fr-es's
pack with them (*A change to a studied language's tables reaches every pair of that language*). M20's
two settings are read as committed; whichever of the owner's answer and this change lands second
re-pins both pairs (measured on fr-en: 2,342 / 902 and 304 / 168 rows, no gloss lost). A rule meant
for one of the two pairs goes in its own reducer.

*Rejected — an update for the first native side.* It would read today's French section: a new
snapshot, so `tables/fr/` moving in the same pull request as the glosses and the golden's hand-over,
three diffs in one. The source is pinned already.

### D6 — The pack, measured and not shipped

2,201,349 B with change 43's two tables (gloss 1,617,741 B, senses 512,132 B, expressions 754,014 B
raw); 2,421,321 B with changes 45's and 46's prototype tables; under the 5 MiB budget, below es-en's
2,567,804 B. The extension check builds it from the committed tables against its pin (change 43's
loop). `packs.json`, `check_variants`' list, the site and the listings are untouched: no package
carries it before change 52. `row-gloss-tables.spec.ts` checks every committed pair's rows against
the row cut — no row empty, none ending on an opening mark — from a literal list: fr-en joins it;
measured, none of its 30,059 rows fails.

### D7 — Known data defects, for a refinement before change 52

What still reads wrong in fr-en, measured on the prototype, and where its fix belongs. None is fixed
here (D2): together they are the input of `refine-lingua-fr-en-glosses`, outside the 57, before
change 52 ships fr-en — as 23b was before 34 — with the owner's reading of the sample. Its programme
row, and change 52's row naming it among its prerequisites, are written by its own proposal, as 23b's
and 24b's were; until then change 48's row names it (task 5.4), where change 52's proposal meets
it.

| Class | Rows / top 10k | Examples | Where |
|---|---|---|---|
| A borrowed gloss that is wrong for the word | of 540 / 116 rows glossed from a pointer's target | « des » (6) « of the; some, the feminine partitive article » — its pointers carry « some » (« plural of un (“some”, …) ») and lend a target's senses; « ca » (144) « board of directors » (wordfreq's `ca` is mostly an unaccented `ça`) | a rule of fr-en (a pointer's carried meaning); `ca` as a lemma is change 43's |
| A pointer's own meaning left out | « il y a » and about 50 words (15 of the top 10,000) | « il y a » « ago »: its « there is, there are » is written on a pointer to *y avoir*, which the shared rules leave out | with the row above |
| An expression that only points | 18 expressions | « crème fraiche », « s'il vous plait » « post-1990 spelling of … » (17), « y a-t-il » « subject-inverted form of il y a »; « à la » « in the style of », met on every « à la » — change 44's proposal ([#823](https://github.com/NEETROF/cymbra/pull/823), D6, D8) hands these to 48 | the English edition's pointer wordings for expressions, or fr-en (Open Question 5) |
| A proper noun's run in a common word's row | first 218 / 108 (es-en 242 / 123); after another run 484 / 260 (es-en 729 / 430) | « marche » « Marche (a department of France); march… », « midi », « somme », « réunion », « bordeaux »; « le » ending on « a surname from Vietnamese » | 23b's Q3, a case-aware card |
| The page's own notes | « see usage notes » 6 / 4; « (all senses) » 97 / 30; « in its various senses » 2 / 2; « (Folk etymology: …) » 2 / 2 (es-en: 1 / 1, 15 / 7, 0, 0) | « en », « dans », « ne » (« …, see usage notes »), « contrôle » « control (all senses) », « mon », « stand », « consul » | the English edition, or fr-en alone |
| A description in a capital outside 23b's list | 64 / 30 (es-en 45 / 17) | « que » « Substitutes for… », « il » « Impersonal subject, it », « mon » « Followed by rank… » | the English edition |
| A citation inside a sense | 2 / 2 | « liberté » « liberty, freedom. 1688, Guy Miège, … » | fr-en, or upstream |
| A source's numbered sense in another shape | 1 / 1 | « téléphonie » « telephony (2) » | the English edition |
| « etc » without its period | 51 / 23 | « le », « pas », « possible » | shared, every pair (23b D6) |
| Labels left out | 443 / 163 rows hold an obsolete or archaic sense | « or », « monde » | 23b's Q1 |
| The part of speech a row opens on | 2,896 / 1,264 rows hold two or more | — | 23b's Q4 |

The hand-over also shows the studied side as change 43's tables read on 2026-10-09: `ma`, `mes`,
`ils`, `cet`, `amie` as lemmas of their own (some of which 43's fixes in progress take back),
`vînmes` unknown (an unattested form). Those are tables/fr's, for changes 43 and 41, not glosses, and
no task here depends on them.

### D8 — The hand-over of change 39's baseline

`support/french.rs`'s `FRENCH` says `pack: PackSource::Tables`: the pack is built from `tables/fr/` and
`tables/fr-en/`, as es-en's beside it is, and the golden's pack line reads fr-en's manifest. Its doc
comment and `french_baseline.rs`'s say so. The fixture (`scripts/lingua-data/testdata/fr-en/`) stays:
`crates/lingua-pack/tests/pipeline_testdata.rs` builds it, and
`a_fixture_left_behind_its_analyser_names_its_manifest` still checks that a manifest left behind
French's analyser is named — its manifest keeps following the version, as change 39 D5 says — and
`the_fixture_lists_every_word_the_pre_pass_writes` (change 40) still reads the fixture's forms.
`the_nfd_block_s_memoire_is_glossed_once_french_composes_it` asserted the fixture's « memory »: the
pack's gloss of the composed `mémoire` now opens on « memory » (« memory; memo; dissertation, … »),
and the decomposed token still has none. Measured, it is the one test of `french_baseline.rs` that
fails on the committed tables; `french_has_its_pre_pass_and_the_baseline_s_lemmas` passes on them.

From the hand-over on, the golden reads the committed French tables: a pull request that moves
`tables/fr/` or `tables/fr-en/` — a dictionary update, change 45's readings or 46's levels landing
after this change, an English-edition rule — can move it too, and then re-blesses it and says so, as
a dictionary update does the other baselines. `french_baseline.rs`'s doc and its failure message,
which name three reasons today, name that one too.

Change 39's scenario *The committed tables replace the fixture* says « WHEN the French tables and the
fr-en pair's tables are committed »: read, as change 43 D11 and the programme read it, as « when
fr-en's glosses are committed » — this change. The golden is re-blessed once, here, and the pull
request changes no French rule, so the analyser version in its pack line is the one on `main` and
every probe moves for the pack alone.

**How the re-bless is reviewed.** 125 of 141 probes move once changes 45 and 46 have landed (132
before), the golden doubles (162 KB to 333–365 KB): its diff is not read line by line. A comparison script, run in a
scratch folder over `git show origin/main:crates/lingua-wasm/tests/baseline/fr-en.golden` and the
re-blessed file and pasted into the pull request with its output so that a reviewer runs it again,
splits both goldens at their `### ` probe lines and gives:
- **mechanical checks**, each a yes or no: the same probe names in the same order; `about`,
  `beside es-en`, `calibration`, `declared-level` and `export-declared-levels` byte for byte; every
  analysed page's `analyzer_version` unchanged and every token's span unchanged but the tokens the
  lexicon now lists (`M`, `e` in the prototype); every `gloss <word>` probe equal to the row
  `tables/fr-en/gloss.tsv` holds for the lemma `tables/fr/forms.tsv` reads the word as, or none —
  the check that catches a gloss lent to another lemma (*venir* « coming, arrival » in the
  prototype);
- **the probes by kind**, moved and unchanged, and the JSON probes that move in their glosses alone
  (their other fields compared with the glosses taken out);
- **per analysed page, every token whose lemma or class moved**, fixture → tables, with its line of
  the forms table and its cause (an inflection now resolved, M8's homographs, change 43's lemma
  choices, a name the lexicon holds) — 113 lemmas and 42 classes in the prototype;
- **the 45 gloss and 23 phrase probes**, the fixture's gloss beside the tables' first sense, and the
  expressions each phrase finds;
- **the provenance and the counts**: the `pack`, `notice` and `licences` lines, the vocabulary
  estimate, tracked, deck, due, promote-by-exposure and the ladder, each with its cause.
A reviewer checks the mechanical lines first, then that every other move has a cause in the tables.
en-fr's, es-fr's, es-en's and en-es's goldens, the extension's snapshots and the French golden's
`beside es-en` line pass without re-blessing.

**Order.** The hand-over wants change 46 first: measured, before it the level probes go blank
(`has-levels` false, an empty ladder, a deck of 3 instead of 11) until 46 re-blesses them. 45 is
not needed by the golden (the fixture has no readings) but precedes in the programme. If 46 slips,
this change may still merge, its pull request saying the level probes are blank until 46.

### D9 — What cannot move

- **en-fr, es-fr, es-en, en-es.** No file of their rule digests changes: `reduce_common.py` and the
  three `reduce_edition_*.py` are not edited, their reducers neither; `pack_sources.py`,
  `gloss_coverage.py` and `build.sh` are in no digest (change 38). No table of `tables/en/`,
  `tables/es/`, `tables/en-fr/`, `tables/es-fr/`, `tables/es-en/`, `tables/en-es/` moves. Their packs
  rebuild to their pins' sha256, their goldens and the extension's snapshots (`word-card-es-en.txt`,
  `word-card-en-es.txt`, `selection-rows-fr.txt`) pass as committed.
- **French's studied side.** Every table of `tables/fr/` but `lexical.tsv`, byte for byte (D5).
- **The reduce job** reproduces every committed byte, fr-en's included; `check-reducer` passes for
  the five pairs.

OpenSpec: two ADDED requirements in `lingua-data-packs`, nothing MODIFIED. `archiveAfter`: change 38
(the catalogue this change prunes), 21 (the English edition and its settings), 23b (the pre-pass),
39 (the hand-over), 43 (French's tables and reference pair); `openspec_archive_order.py` exits 10
naming those still open. 45 and 46 have no proposal on `main` yet (46's is open, #822): their order
is a merge order (D8, D10), not an archive one.

### D10 — Order, and what later changes take from here

| Change | Relation |
|---|---|
| 39 French baseline | before (required): the golden this change hands over |
| 40 tokenisation | before, landed (`35faf774`, French at `0.2.0`): the golden's figures are measured on it |
| 43 forms tables | before (required), its implementation with its fixes: the reducer, `tables/fr/`, the pin this change re-reduces from. No task here relies on a lemma its fixes take back (`donnée` ranked, `ma`, `mes`, `ils`, `cet` lemmas of their own); task 3.1 checks the one that would lend a gloss |
| 45 grammar, 46 levels | before (planned): the hand-over keeps the golden's level probes (D8). 46's proposal ([#822](https://github.com/NEETROF/cymbra/pull/822), its D3 and D11) hands this change a check that every levelled lemma is a dictionary word: measured on its prototype table, 40 of its 8,302 levelled lemmas have no fr-en gloss — words met only in an expression (`parce`, `quant`, `instar`, `for`), initialisms written in capitals (`pme`, `tom`), pointers (`expliquez`, `ès`, `french`), a letter (`x`). The pack's vocabulary estimate counts a levelled lemma whether glossed or not (`Pack::dictionary_words`), so they count; a card seeded from a level would carry no gloss. This change measures and lists them (task 3.1) and asserts nothing: the rule is 46's (Open Question 5) |
| 41 analysis, 42 detection guard, 44 expression keys | either side. Each bumps or keys through the analyser, not the glosses. Landed before, the hand-over shows the real analysis on the real pack; after, each re-blesses over the real pack. 41's names rule, if it reads dictionary words as Spanish's does, reads the fixture's 70 before this change and fr-en's 30,059 after; 44 keys whatever `mwe.tsv` holds. 44's proposal ([#823](https://github.com/NEETROF/cymbra/pull/823), D9) hands 48 more: the words the pre-pass splits offered as expressions (`d'abord`, `c'est`: 101 headwords `reduce_expressions` does not read), and its pointer-only and function-word expressions left out (D7); this design does neither — no fr-en rule here (D2) — and lists them for the refinement (Open Question 5) |
| 49 fr-es | after: French's dictionary words (`tables/fr/lexical.tsv`), the cross-native test for French (two natives) |
| 50 marks | after or beside: marks are the translation engine's, not the glosses' |
| 51 word card | after: the English card renders fr-en's glosses and runs |
| 52 enable | after: lists fr-en, publishes its coverage, the owner's dogfood |
| `refine-lingua-fr-en-glosses` | after, before 52: D7, outside the 57 |

## Risks / Trade-offs

- **[Fewer glosses than a table would give]** → 1.3 points at 10,000, measured to be mostly noise
  that would become French's dictionary words (D3); the floor is set on the section alone (D4).
- **[An English-edition rule now moves two pairs]** → said in D5 and tested
  (`test_reduce_editions.py`: an edit of `reduce_edition_en.py` moves es-en's and fr-en's digests and
  no other); a one-pair rule goes in its reducer.
- **[A defect ships to English speakers]** → nothing ships before 52; D7 lists them, the refinement
  before 52 takes them, the owner reads the sample.
- **[The golden moves whole]** → re-blessed once, with no French rule in the same pull request, and
  reviewed by probe kind (D8).
- **[Change 46 slips]** → the level probes go blank until it lands, said in the pull request (D8).
- **[Change 43's tables move before this lands]** → the figures are re-measured on the committed
  tables (task 2.2); the floor holds them, not the prototype's figures. On 43's tables after its
  first fix, measured: 30,056 glossed lemmas, 93.6 / 86.8 / 76.3 %, the same 132 probes moving.
- **[A gloss lent to another lemma]** → a ranked lemma whose own form reads as another would give
  its gloss to that word (*venir* « coming, arrival » in the prototype): change 43 ranks none, and
  task 3.1 and D8's mechanical check hold it.
- **[Change 38's catalogue loses two files a later change wants]** → re-added by that change, in its
  edition, as the catalogue's rule says.

## Migration Plan

Nothing to migrate: no reader holds a French pack. Rollback is a revert of the tables, the pin and
the harness's source; `lexical.tsv` returns to empty, the golden to the fixture's.

## Effort

2–3.5 ideal days, the programme's: the reducer's native side and its tests 1–1.5; the catalogue,
the floor and its job, the re-reduction, pin, README and `SOURCES.md` 0.25–0.5; the hand-over — the
harness, the `mémoire` test, the re-bless, the comparison script and the review tables of D8 —
0.5–1; the sample, the
defects' list, `committed_tables.rs`, the row check, spec and programme 0.25–0.5.

## Open Questions

For the owner, none blocking:
1. **Translation tables for expressions** (D3): declined here; the French Wiktionary's English
   translations would add 24,138 expressions (3,248 occurrences in UD's 425,111 words beside the
   section's 32,998, some compositional and wrong), 363 KB, and a source — a later update of fr-en,
   as a change of its own, if wanted.
2. **The floor** (D4): 91.9 / 85.1 / 74.4, the study's figures less two points, as en-es's.
3. **The sample and 23b's questions**: the owner reads 100 glosses of the top 10,000 (task 6.1); 23b's
   Q1–Q4 (labels, the part of speech a row opens on, proper nouns, single letters) now answer for
   fr-en too, through the English edition, and M20's settings re-pin both pairs.
4. **Sentences true of their time** — 23b's « which only es-en reads: they re-pin es-en alone », 43's
   scenario *The reference pair writes French's folder* (an empty `lexical.tsv` and `gloss.tsv`), 38's
   scenario *A pair of stage 3 registers what it reads* (fr-en with three sources), 39's *The committed
   tables replace the fixture* (« the French tables and the fr-en pair's tables »): read as before this
   change, and their words best made « es-en and fr-en », « while fr-en glosses nothing », « when a
   pair of stage 3 reads them » and « when fr-en's glosses are committed » when each is archived, so
   that no two specs say opposite things of this change. 39's requirement names three reasons the
   golden moves; this change's spec adds the fourth, a change to the committed French tables.
5. **What the refinement and changes 44 and 46 expect of fr-en** (D7, D10): change 44's proposal
   hands this change French's split words as expressions (`d'abord`, `c'est`) and the expressions to
   leave out (17 « post-1990 spelling of », `y a-t-il`, `à la`); change 46's, a check that every
   levelled lemma is a dictionary word, which 40 of 8,302 are not. Measured and listed here, not
   decided: each is a rule of fr-en's reducer, of the English edition or of 46's levels, and the
   owner says which of the refinement before 52, change 44 and change 46 takes it.
