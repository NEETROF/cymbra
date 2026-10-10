# Design — refine-lingua-fr-en-expressions

## Context

See proposal.md (Why). What exists, on `main` at `2d6ec09f`:

| Where | What |
|---|---|
| `tables/fr-en/mwe.tsv` | 17,635 expressions, the English Wiktionary's French section alone (change 48 D3, D11; 48b D4, D7): its headwords with a space and the single words French's tokenisation splits (`d'abord`, `allez-y`), less the senses that only point and the 17 names of `LEFT_OUT`. The builder keys 14,494 of them |
| `reduce-fr-en.py` `expression_senses` | the pass over the section's expressions before the shared rules: `LEFT_OUT`'s names go, each with its reason; a sense that only points (`_POINTS`: « post-1990 spelling of », « subject-inverted form of », « que + elle ») goes, a meaning written after its target stays; an expression no sense is left to is none |
| Change 48c (`add-lingua-fr-en-translated-expressions`) | proposal merged (#888), implementation open (#893): the French Wiktionary's English translations complete the expressions; a headword the section glosses, a `LEFT_OUT` name, a headword `expression_senses` records as pointer-only, and one whose words read as a section expression's are not read from that table (its D2); `LEFT_OUT` gains 51 of the table's expressions (its D5). Its open question 3, settled by the owner on 2026-10-10: the section's own expressions get this change |
| The builder | keys a French headword through the core's reading (change 44), two to seven tokens; several headwords of a key are kept and the match named by the one written most like the page (change 44e, merged as #892) |
| The matcher | `match_expressions`: from each token, the longest run that is a key whose pieces meet the page as the headword writes them (changes 44c, 44d) |
| The goldens | `fr-en.golden` over the committed tables; its phrase probes are shared with `fr-es.golden` (`support/french.rs` `PHRASES`); the extension's `word-card-fr-en.txt` renders every phrase probe |

## Goals / Non-Goals

**Goals:**
- An expression the section glosses is shown where its gloss is what the words mean.
- Each choice measured where a reader meets it: matches on French text, read in context.
- `tables/fr/`, fr-en's word glosses and sense runs, fr-es, en-fr, es-fr, es-en and en-es byte for
  byte.

**Non-Goals:**
- The matcher. A run that crosses a number or a punctuation mark the tokeniser drops (« en 1799,
  est » meets « en être », « de 13 % par » « de par », « bien, qu'il » « bien que »: 547 matches,
  0.9 %), and a split word keyed by its verb's dictionary form met in another tense (« j'ai » on
  « j'avais », « j'aurais »), are the engine's, not the table's: the data cannot tell them apart, and
  an engine change moves fr-es too. Not proposed here (*Found while measuring*).
- fr-es's expressions (its own reducer; nothing of it moves).
- The French Wiktionary's table: 48c reads it, and its rules keep this change's names out of it (D6).
- A new snapshot: fr-en is re-reduced at its pin.

## Measured

A prototype in the scratchpad (never committed): change 48c's harness (a scratch example of
`lingua-pack`) builds a pack from tables with `build_pack` and runs the engine's `gloss_phrase` on
every line of 48c's five corpora — UD French-GSD's training and development sentences, four Gutenberg
novels, French Wikipedia, a Tatoeba sample, the baseline's 13 pages: 1,060,904 words in 51,790 lines,
each line a selection, its `expressions` the matches a reader's phrase card shows. On `main`'s tables
it builds the committed pack (2,536,702 B) and meets **60,361 expressions** (56.9 per 1,000 words,
52.2 % of the lines meet one). The prototype's variants edit `mwe.tsv` alone, as the rules below do.

**A judged sample.** 300 matches drawn at random (seed 4900), each read in its sentence by one judge
(the prototype's author): **221 right, 13 partly, 66 wrong** (48c's sample of 100, seed 4800: 73, 1,
26). The wrong and the partly right, by what makes them so:

| Class | Right | Partly | Wrong | Examples |
|---|---|---|---|---|
| « de l' » glossed by the partitive alone | 1 | 0 | 29 | « l'appui de l'Arabie Saoudite » "some" |
| Every sense labelled (informal, slang, Quebec, Internet, derogatory, dated, literary…) | 10 | 2 | 16 | « n'a pas de visage » « pas de » "without"; « J'aime ses yeux » "a like"; « prêt à tout » « à toute » "see you" |
| Met through another word's form (the page does not write the headword) | 51 | 5 | 5 | « Il en fait l'acquisition » « en fait » "actually"; « de conserver » « de conserve »; « Ce fait » « ce faisant » |
| Met as written | 159 | 6 | 13 | « au pied de quelque cyprès » "to heel!"; « un homme fort agréable » « homme fort »; « tous les deux ou trois ans » "both" |
| Across a number or a punctuation mark | 0 | 0 | 3 | « en 2007 est » « en être »; « en 2017 faisant de » « en fait de » |

**Where they are.** 50 of the 66 wrong belong to expressions met at least 100 times (171 of the 300
matches), 6 to those met 20–99 times, 5 to 10–19, 5 below ten. The 734 expressions met ten times or
more carry 51,107 of the 60,361 matches (85 %); the 376 met twenty times or more, 77 %.

**The review.** Each of the 734 was read in its contexts — the 376 met twenty times or more in four,
the 358 met 10–19 times in three, the ten read both ways in ten more (« de plus », « en fait »,
« d'autre », « nombre de », « de nouveau », « tenir de », « c'est moi », « comme pour », « je vois »,
« y avoir »). **111 read wrong in most of their contexts** (D1): 7,197 matches. Left in, read partly
right: « est-il » on « serait-il », « en passant » "in passing" on « en passant par », « deux ou
trois » "one or two", « et alors » "and then?", « sais-tu », « non pas » "no", « comme pour » "as if"
(six right in ten), « je vois » "I see, right" (literal « I see » reads right), « au fait » "au fait;
by the way" (English writes « au fait », informed). The expressions met fewer than ten times (3,860
distinct, 15 % of the matches) are not read one by one.

**Rules, measured on the reviewed expressions.** Each candidate rule, by the share of the listed
wrong matches it removes against the share of the right matches it removes (the 111 listed = wrong,
the 623 others = right; 11,937 and 39,170 matches):

| Rule (left out) | Expressions | Of the wrong | Of the right |
|---|---|---|---|
| Every sense labelled — a register or a region | 79 | 25 % | 2.4 % (953 matches beyond the list: « au début », « en ligne », « suite à », « s'il te plaît ») |
| An invariable headword (adverb, conjunction, preposition, phrase…) whose key reads one of its words as another (« de fait » `de faire`) | 97 | 16 % | 16.5 % (« peut être », « tout à fait », « y compris », « du fait de ») |
| « de », « d' », « du », « des » + one word | 62 | 54 % | 25.4 % (« de la », « d'abord », « d'origine », « de base ») |
| « à », « au », « aux », « en » + one word | 94 | 8 % | 6.7 % |
| A pronoun first (« je », « tu », « m' », « t' », « s' », « il », « vous ») | 28 | 8 % | 7.5 % |

The labelled rule is the one that looks good, but the list removes its wrong ones anyway; beyond it,
on the reviewed expressions it removes only right ones, and on the tail a sample of its matches
(60, seed 4901, among expressions met fewer than ten times) reads 43 right, 2 partly, 15 wrong — three
right ones lost for each wrong one. 48c's rule for the French Wiktionary's table, a translation that
repeats its headword (its D3), applied to the section: 252 headwords hold one, 161 hold nothing else
(51 matches) — English loanwords the English Wiktionary glosses as English (« coup d'état », « de
facto », « vice versa », « crème brûlée », 48c's probe).

**Senses that only note a form** (D2). Over the section's expressions, the senses — and the pieces of
a sense between semicolons — that open on a form's or a usage's note rather than a meaning: « compound
of je and ai » (« j'ai » "I've, I have; compound of je and ai"), « the interrogative form of avoir in
the third-person singular masculine » and « a form of avoir (when used as a past auxiliary verb) in
the third-person… » (« a-t-il »), « an interrogative form of je peux » (« puis-je »), « For
non-idiomatic uses see à and moi » (« à moi »), « For non-idiomatic uses: see faire (“do”) and ne …
que » (« ne faire que »), « Equivalent to “je vais” when used as a near-future tense » (« m'as »),
« used only in the simple past tense for s'en aller » (« s'en être »), « used only in the expression
comme de colin-tampon » (« colin-tampon »): 8 rows, 1,734 matches. « j'ai » and « ne faire que » keep
their meanings; « a-t-il », « puis-je » and « colin-tampon » keep nothing (149 matches); « à moi »,
« m'as » and « s'en être » are on the list too.

**« de l' »** (D3). 4,717 matches, 30 sampled: 29 mean "of the" (« de l'Arabie Saoudite », « de
l'équipe »), one the partitive (« si elle avait de l'étude »). Its section entry has one sense, « some;
the singular prevocalic partitive article »; « de la » has « of the » and « some; the feminine
partitive article », and its 39 sampled matches all read right. Lent « de la »'s first sense, « de
l' » reads 28 of the 30 right — the two left are « de l' » before an infinitive, a pronoun (« supplié
de l'épargner »), which change 44c's elision rule cannot tell from an article. Left out instead, 29
wrong matches go and the right one with them, and change 44c's probe « de l'eau » loses its partitive.

**The result** (all rules), on the corpora:

| Corpus | Words | Matches, `main` → this change |
|---|---|---|
| UD French-GSD (train, dev) | 323,269 | 17,564 → 15,959 |
| Gutenberg | 210,338 | 12,398 → 10,457 |
| Wikipedia | 379,182 | 19,582 → 18,032 |
| Tatoeba | 147,113 | 10,761 → 9,129 |
| The baseline's pages | 1,002 | 56 → 47 |
| **All** | **1,060,904** | **60,361 → 53,624** (56.9 → 50.5 per 1,000 words; lines meeting one 52.2 → 48.6 %) |

7,377 matches go — 7,197 of the list, 149 of the pointer-only headwords, 31 displaced by a match the
removals let through — and 640 come: a shorter or later expression a listed one hid (« en faisant du
bruit » meets « faire du bruit » "to make a lot of noise" instead of « en fait de »; « l'histoire de
la » « de la »; « ont lieu de juillet » « avoir lieu »). Of 40 of them, sampled, 38 read right, 1
partly, 1 wrong (« en toute chose » meets « tout chose »). **On the 300 sampled before**, the list
removes 29 of the 66 wrong, 8 of the 13 partly right and 2 of the 221 right (« de nouveau » once,
« faire avec » once), and « de l' »'s lent sense turns 27 wrong into right: 246 right, 5 partly, 10
wrong of the 261 left. **A fresh sample** drawn from the result (100 matches, seed 4903) reads **95
right, 2 partly, 3 wrong** (« s'empresserait de l'en informer » « de l' »; « au clair de lune » « au
clair » "clear"; « la tonalité de retour » « de retour » "back").

`mwe.tsv` goes from 17,635 rows to 17,521 (the builder keys 14,381; no key is shared with another
headword, so change 44e's choice moves nothing); the pack from 2,536,702 to 2,535,598 B. `gloss.tsv`,
`senses.tsv` and every table of `tables/fr/` are byte for byte.

**The golden.** `fr-en.golden` re-blessed over the prototype's tables: 2 lines move — `pack`
(2,536,702 → 2,535,598 B; its `pack_version` moves with the rule digest in the implementation) and
« de l’eau », whose « de l' » reads "of the; some; the singular prevocalic partitive ar" — and no other
probe. Every other test of `lingua-wasm` passes as committed: `fr_es_baseline`, `cross_native`, the
English, Spanish, es-en and en-es baselines among them.

**With 48c's table** (its prototype's candidates, the dump of 2026-10-02): 49 of the 111 have a row
in the French Wiktionary's table — « en fait » "in fact", « de rien » "you're welcome", « au plus »
"at most", « la ferme » "shut up", « histoire de » "in order to, so that" —, all met as the section's
are; none of the 111 shares a key with a table row 48c's key rule removed.

**Found while measuring** (not this change's; Non-Goals). The matcher's runs across a number or a
punctuation mark the tokeniser drops: 547 of the 60,361 matches, the 3 wrong of the sample's class.
A split word keyed through its verb meets other tenses: « j'ai » "I've, I have" on « j'avais »,
« j'aurais » (three of the fresh sample's five not right).

## Decisions

### D1 — A reviewed list: 111 of the section's expressions left out by name

`LEFT_OUT` gains the section's expressions read wrong in most of the contexts they are met in, each
with its reason — its gloss, how it is met, an example — as 48b's and 48c's rows give theirs: a
person's decision, as an override row is. The review read every expression met ten times or more
(*Measured*). The 111, by reason (matches on the corpora in parentheses):

- **A labelled sense met as plain words** (47; 3,002 matches): every sense of the entry carries a
  register or a region the plain words do not — « pas de » "without", Quebec (610: « il n'a pas
  d'enfants »), « dire que » "to think that" (355), « histoire de » "in order to" (256), « j'aime » "a
  like" (183), « à toute » "see you" (151), « fait que » "so, therefore", Quebec (121), « sa mère »
  "(so) fucking" (119), « de deux » "next, secondly" (117), « pas que » "not that" (108), « t'a » "tu
  as" (108), « sais pas » "dunno" (106), « espèce de » "you" (95), « m'as » (36), « vu que » (35),
  « de couleur » (32), « encore que » (28), « la faire » (26), « comme tout » (26), « du reste » (25),
  « au plus » (24), « et quelques » (24), « de première » (23), « du vent » (23), « à plus » (22),
  « qui parle » (21), « à la place » (21), « à d'autres » (19), « la ferme » (19), « tu parles » (19),
  « faire avec » (18), « des familles » (16), « bande de » (16), « d'aucuns » (16), « se prendre »
  (16), « y passer » (16), « et encore » (16), « en fait de » (15), « à la limite » (15), « rapport à »
  (14), « faire bien » (13), « s'en être » (13), « du diable » (12), « c'est cela » (12), « quoi
  faire » (12), « y rester » (10), « faire sans » (10), « de un » (10);
- **A headword met through another word's form** (21; 1,835): its key reads a word through its
  dictionary form, and the page writes another — « d'autre » "else" on « d'autres » (461), « en
  être » on « en est », « en sont » (345), « de nouveau » "again" on « de nouveaux » (228), « de fait »
  on « de faire » (219), « en fait » "actually" on « en faisant », « ce qui en fait » (175), « étant
  donné » on « est donné » (62), « en tout » (45), « tenir de » (36), « n'eût été » (35), « il ne
  fallait pas » "you shouldn't have" on « il ne faut pas » (34), « ceci étant » (32), « la tienne » on
  « la tenait » (26), « cela étant » (22), « la majeur » "A major" on « la majeure partie » (22),
  « attendu que » (21), « en tenue » (18), « en devenir » (13), « se passer de » (11), « à découvert »
  (10), « avoir lieu de » (10), « à l'anglaise » on « à l'anglais » (10);
- **Words met as they are, where the headword names a set phrase, a field's sense or a cry** (41;
  1,855): « être à » "to belong to" (456: « je suis au bout »), « nombre de » "quite a few" (259: « le
  nombre de »), « de plus » "moreover" (209: « de plus près », « une fois de plus »), « première
  fois » "first time (the first instance of sexual intercourse)" (146), « à moi » "help!" (50), « d'y »
  "thence" (48), « le temps de » (46), « de droit » (42), « au pied » "to heel!" (42), « de rien »
  "you're welcome" (36: « ne … de rien »), « bon dieu » (34), « au travail » "get to work!" (33),
  « c'est moi » (30), « un tel » "so-and-so" (27), « de service » "on duty" (26), « du siècle » (24),
  « de jour » (23), « tout fait » (19), « de confiance » (18), « qui sait » (18), « de cœur » (17),
  « d'esprit » (17), « de grâce » (17), « avoir mal » (16), « bien fait » (14), « le chercher » (14),
  « à suivre » (13), « de force » (13), « au reste » (13), « du temps de » (12), « au feu » "fire!"
  (12), « avoir raison de » (12), « pauvre de » (12), « de tête » (12), « d'un coup » (11), « au point »
  "ready" (11), « à la pointe » (11), « de concert » (11), « à l'horizon » (11), « au courant » (10),
  « le leur » (10);
- **A colloquial spelling met as the standard one** (2; 505): « m'a » "I'm going" (455: « il m'a
  sauvé »), « t'es » "you're" (50: « tu t'es accrochée »).

*Rejected — a list of what reads right* (the 623 others): it would freeze the table at this snapshot,
as 48c's D5 says; a later update brings new expressions, which this list leaves in and a later review
reads. *Rejected — a lower threshold*: the expressions met fewer than ten times hold 15 % of the
matches and, sampled, about one wrong in ten (5 of 51 in the 300).

### D2 — A sense that only notes a form is no meaning

`_POINTS`, 48b's rule for an expression's senses that only point (change 48 D11, 2), gains the notes
of *Measured*: a sense, or a piece of it between semicolons, opening on « compound of X and Y »,
« (the|an|a) (interrogative )?form of X … in the first|second|third-person », « an interrogative form
of », « for non-idiomatic uses », « used only in the », « equivalent to “ » goes; a sense no piece is
left to goes, and an expression no sense is left to is none, recorded as pointer-only — so 48c keeps
it out of its table too. The note is grammar the reader does not read as English (M5: a gloss is in
the reader's language and says what the words mean); the pieces' own rows say it (« a » "has", « il »
"he"). 8 rows: « j'ai » "I've, I have" (1,451 matches, its note gone), « ne faire que » "only, merely,
just", « a-t-il », « puis-je » and « colin-tampon » out (149 matches), « à moi », « m'as » and « s'en
être » on the list as well. *Rejected — « a form of » alone*: « a form of French gingerbread » (« pain
d'épice »), « a form of honey biscuit » are meanings.

### D3 — « de l' » reads "of the" first

A table of fr-en's own, `LENT_SENSES = {"de l'": "de la"}`, with its reason, as `LEFT_OUT`'s rows
give theirs: the headword's gloss opens on the lender's first sense when its own senses lack it —
« de l' » "of the; some; the singular prevocalic partitive article", cut as the shared rules cut a
gloss. One row, measured (*Measured*): 28 of 30 sampled matches read right instead of 1; « de l'eau »
keeps its partitive. *Rejected — `LEFT_OUT`*: 4,717 matches, « de l'eau » (change 44c's probe, the
owner's case) loses its meaning. *Rejected — every elided headword lent its full spelling's senses*:
of the section's headwords ending on an elided piece, two have their full spelling glossed too,
« de l' » and « putain d' », the second with the same meaning.

### D4 — The result, measured

*Measured*: 114 rows out of `mwe.tsv` (the 111, « a-t-il », « puis-je », « colin-tampon »), two
glosses shortened, one lent a sense; 60,361 → 53,624 matches; the fresh sample 95 right of 100. The
pack loses 1,104 B. `gloss.tsv`, `senses.tsv`, French's dictionary words, levels and coverage cannot
move: every rule reads an expression's entry alone (`is_expression`), as `LEFT_OUT` does.

### D5 — No structural rule

Each rule of *Measured* removes the wrong for a like share of the right, or — the labelled senses —
only what the list removes anyway, then three right for one wrong. None is taken, as 48c's D4 took
none for its table. *Rejected — matching an invariable headword as written only*: the engine's
(`match_expressions`), every French pack's; it would take « en fait » off « en faisant » but keep it on
« ce qui en fait la troisième ville » (of ten sampled contexts, seven write « en fait », and six of those read wrong), and move fr-es.

### D6 — 48c's table stays out of them

The order is 48c, then this change: its implementation (#893) is open. 48c's `translated_expressions`
leaves out a `LEFT_OUT` name and a headword `expression_senses` records as pointer-only (its D2, 3);
both are what D1 and D2 write, so the 49 table rows of the 111 (*Measured*) stay out with no rule
added. Its summary line counts them under « left out by name » instead of « the section glosses the
headword ». A table headword whose key a removed section expression held (48c's D2, 2) would come in:
0 measured. Built on 48c, the re-reduction re-measures every figure on 48c's tables (task 1.1).

### D7 — What moves, what cannot

- **Moves**: `tables/fr-en/mwe.tsv`; `manifest.json` (`pack_version`: the rule digest); `pin.json`
  (`reducer`, `pack`); `README.md` (*The expressions*: the list by reason, the figures);
  `fr-en.golden` (`pack`, « de l’eau »); `word-card-fr-en.txt` (« de l' — of the » for « de l' —
  some »).
- **No probe is added**: the phrase probes are shared with fr-es (`support/french.rs` `PHRASES`), and
  a probe more would add lines to `fr-es.golden` and `word-card-fr-es.txt`. The cases are tested on
  the reducer (task 2.1) and on the committed table (task 2.2).
- **Cannot move**: `tables/fr/` (the studied side reads `kaikki-French.jsonl`, GSD and wordfreq);
  `gloss.tsv`, `senses.tsv`; fr-es's tables, pin and pack (its reducer imports nothing of fr-en's, and
  its studied record reads `tables/fr/`); en-fr, es-fr, es-en and en-es — `reduce-fr-en.py` is in no
  other pair's rule digest (`pack_sources.rule_files`), no shared module (`reduce_common.py`, the three
  `reduce_edition_*.py`) and no engine file is edited. Measured: every `lingua-wasm` suite but
  `french_baseline` passes on the prototype's tables without re-blessing.

### D8 — Order and archive

| Change | Relation |
|---|---|
| 48 fr-en, 48b refinement | before (on `main`): `LEFT_OUT`, `_POINTS`, `expression_senses` |
| 48c translated expressions | before: implemented on its tables; it keeps D1's and D2's out of its table |
| 44e written as the page | on `main`; no listed headword shares a key |
| 49, 49b fr-es | none: fr-es does not move |
| 52 enable | after: French ships with these expressions |

`archiveAfter`: 44 (the keys), 48 (fr-en's glosses), 48b (`LEFT_OUT`, the pointers), 48c (the table
that reads `LEFT_OUT`). Sentences of open changes this change makes true of their time, to amend when
each is archived (nothing to decide): 48c's « `LEFT_OUT` gains the 51 » and its counts of `LEFT_OUT`
(53 / 2) read « … and this change's 111 »; 48b's list of the pointers `_POINTS` reads gains the notes
of D2.

## Risks / Trade-offs

- **[One judge]** → every figure of *Measured* is one reader's, ± 7 % on 100; the list goes to the
  owner with its contexts (task 4.1).
- **[The list is read on the corpora it is measured on]** → the confirmation sample was drawn after
  it; the expressions met fewer than ten times are not read one by one (5 wrong of 51 sampled).
- **[A right reading lost]** → « faire avec » "to make do" is right in « Fais avec. », « de nouveau »
  "again" in « se généralise de nouveau » — 2 of the 221 right sampled; the words' own rows stay.
- **[48c moves first]** → its re-reduction moves `mwe.tsv` and the counts; the re-measure on its
  tables is the one recorded (task 1.1), and a residual wrong expression met ten times goes on the
  list with its reason.
- **[A new snapshot]** → the next update brings new headwords this list does not read; the review is
  re-run on it, as 48c's.

## Migration Plan

Nothing to migrate: no reader holds a French pack. Rollback is a revert of the rules and the tables.

## Effort

1.5–2.5 ideal days: the 111 rows with their reasons, the pointer notes, the lent sense and their tests
0.75–1.25; the re-reduction on 48c's tables, README and the harness re-run with the owner's sample
0.5–0.75; the golden and the card snapshot 0.25–0.5.

## Open Questions

None. What moves is the owner's to approve (task 4.1).
