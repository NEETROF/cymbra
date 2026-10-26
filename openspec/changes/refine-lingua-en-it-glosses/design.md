# Design — refine-lingua-en-it-glosses

## Context

See proposal.md (Why). What exists, on `main` at `c9a08c784` (change 36, #1081; its tables committed
by the owner's update of 2026.10.26, run 40324469757):

| Where | What |
|---|---|
| `reduce-en-it.py` | en-it's reducer: the native side alone, against `tables/en/` as committed — the Italian Wiktionary's English section (`without_letters` → `italian.read_as_meanings`), then `native_side`: the section's definitions, then the English Wiktionary's Italian table (direct), then the Italian Wiktionary's English translations read backwards (inverted), the first that glosses a lemma glossing it whole; en-es's table readers copied (change 36's D3) |
| `reduce_edition_it.py` | The Italian Wiktionary's rules (change 35): read by en-it alone among the committed pairs, and by fr-it and es-it (changes 47 and 51, proposed, not implemented) |
| `reduce_common.py` | The shared rules, every pair's digest: not edited here |
| `tables/en-it/` | 21,526 glossed lemmas (definitions 13,677, 55 through a pointer; the direct table 6,125; the inverted 1,724); 13,423 expressions; coverage **91.4 / 82.8 / 70.3 %** against `FLOORS["en-it"]` **89.4 / 80.8 / 68.3**; `gloss.tsv` `34ecdd39…`, `senses.tsv` `9e384393…`, `mwe.tsv` `646293cf…` |
| `tables/en/` | English's studied side, en-fr's, read as committed; no rule here writes it |
| The requirement this change touches | Change 36's *English is glossed in Italian from the Italian Wiktionary's English section and the two Wiktionaries' translation tables*, open, held by `add-lingua-pack-en-it`; changes 47 and 51 (proposals) each hold a MODIFIED copy of it |
| The lists for this change | Change 36's *Known defects, for change 37* (its D10), its README's; the specialist review of the committed tables (2026-10-26: error rates by band, the 23 wrong rows of the 300 commonest with the right Italian, the partly right rows worth fixing, systematic issues A–J, with its scripts and samples) |
| Change 36's open task 4.3 | `test/word-card-en-it.spec.ts` and `test/baseline/word-card-en-it.txt`, the full card in Italian over en-it's golden, and its line in `lingua-pack-update.yml`'s bless step: it waited for changes 33 and 34, both now on `main` (#1080, #1082) |

## Goals / Non-Goals

**Goals:**
- en-it's glosses open on the Italian word an English word's commonest use calls for, before change 45
  lists en-it — by rules of en-it's own modules, each measured alone on the committed tables and kept
  only where it reads better (the criteria below), and short reviewed tables for what no rule reaches.
- Every item of the lists above decided: a rule, a reviewed row, left with its reason, or put to the
  owner in plain words with examples (tasks group 0).
- Change 36's open task 4.3 done here: the card in Italian, its snapshot and its bless line.

**Non-Goals:**
- Any byte of `tables/en/` or of the other committed pairs (en-fr, es-fr, es-en, en-es, fr-en, fr-es,
  it-fr, it-en, it-es): no rule of `reduce_common.py` is edited, and `reduce_edition_it.py` only under
  the criteria's condition below.
- A new source: en-it's pin keeps its three files.
- The coverage floor: `FLOORS["en-it"]` stays 89.4 / 80.8 / 68.3.
- Shipping en-it (change 45).

## How this was measured

**Criteria, written on 2026-10-26 before any figure of this change's rules was read.** Change 36's own
figures (its design, its README) and the specialist review of its tables (2026-10-26) are this change's
inputs, read before: they are the list, not a measurement of a rule.

- *The base.* en-it's committed tables on `main` at `c9a08c784` (change 36, #1081: `gloss.tsv`
  `34ecdd39…`, `senses.tsv` `9e384393…`, `mwe.tsv` `646293cf…`), reproduced byte for byte by
  `reduce-en-it.py` over the pinned release `lingua-pack-sources-en-it-2026.10.26`, fetched by
  `pack_sources.py fetch-pinned` into a scratch folder and read as data, with Python 3.12 and
  `requirements-reduce.txt` (wordfreq 3.1.1). No figure is read before that reproduction holds.
- *The bands.* A row is counted by its lemma's rank in `tables/en/freq.tsv`, the file
  `gloss_coverage.py` and the review read: 1–1,000, 1,001–5,000, 5,001–20,000, past 20,000; the 300
  commonest are the review's band.
- *Each rule alone, then together.* Each rule is the committed reducer with that rule alone added, in a
  scratch copy; then every rule kept, together. Each is compared row by row with the committed
  `gloss.tsv`: rows changed by band, rows whose first sense changes, lemmas gained and lost,
  expressions changed, and the coverage of the 5,000 / 10,000 / 20,000 commonest.
- *What counts as better.* A row reads **better** when its first sense moves to the Italian word a
  bilingual dictionary gives first for the English word's commonest use — the review's « right Italian »
  where it names one —, or when it loses a sense that is wrong (not Italian, a description where a
  translation exists, a pointer's wording, a name's note on a common word, heraldry, a repeat, a
  misspelling, a false friend) and keeps a right one; **worse** when its first sense moves to a rarer,
  wrong or non-Italian word, or when it loses its only right sense or a common sense no other sense
  says; **no worse** otherwise (a reordering of right senses, a synonym for a synonym).
- *Which part of speech is the commonest.* en-it's pin holds no English treebank, and no rule reads one
  it does not record. The evidence a rule may use is what the pin and the committed tables hold: the
  part of speech of each sense run (`senses.tsv`), English's readings (`tables/en/grammar.tsv`), the
  words both translation tables list under each part of speech, and wordfreq's Italian list, already
  credited in en-it's notice. Each such evidence is measured as a variant, by the criteria below, and
  the commonest grammatical words, where no evidence is safe, are a reviewed table.
- *Who reads.* Every changed row among the 1,000 commonest, and, in each of 1,001–5,000 and
  5,001–20,000, every changed row when a rule changes at most 60 there, else 60 drawn by the seed
  20261026 (the review's). The reader is an agent, not a native speaker; the owner's specialist reads the
  gloss diff of the implementation (tasks).
- *What keeps a rule.* A rule is kept when, in every band read, its better rows outnumber its worse
  rows at least four to one, and no row of the 300 commonest reads worse unless a reviewed row (below)
  restores it; it is narrowed — and the narrowing recorded, with the rows it cost — when it fails that
  and a narrower wording passes; dropped otherwise. A variant of a rule (a threshold, a tie-break, a
  part of speech's handling, a band it is limited to) is measured the same way, and the variant kept is
  the one with the most better rows among those that pass.
- *What bars a rule, whatever it fixes.*
  - It takes the only gloss of a ranked lemma of the 20,000 commonest, unless that gloss is pure noise
    (a name's note on a common word, a description with no Italian word for the lemma, a word that is
    not Italian, another word's meaning, heraldry).
  - It puts the coverage of the 5,000 / 10,000 / 20,000 commonest under the floor, **89.4 / 80.8 /
    68.3** (`FLOORS["en-it"]`, settled by the owner, never lowered).
  - It needs `reduce_common.py`, or a reducer or module another committed pair loads, to change. Such a
    fix is reported, not made. `reduce_edition_it.py` may change only while en-it is its only committed
    reader, shown by every other committed pair's pin and rule digest, byte for byte; a change to it is
    measured on fr-it's and es-it's prototypes too (changes 47 and 51) and reported to them.
  - It reads, when the reducer runs, a source en-it's pin does not record, or another pair's committed
    tables (en-es's order, `tables/it/`): their next reduction would move en-it without its pin saying
    so. Such a table may be read by the measurement, outside the reducer, to list candidates for a
    reviewed row (`tables/it/forms.tsv` for misspellings), never by a rule.
  - It writes an Italian word no source gives, outside a reviewed row marked as a person's.
- *A reviewed row* (`FIRST_RUN`, `CORRECTIONS`, the false friends) is written only for a clear error of
  the 300 commonest, of an item change 36 or the review handed to this change, or of a class a rule
  cannot tell apart from right rows, that no kept rule reaches; each with its reason, keyed so that it
  fails or names itself when the source no longer writes what it corrects. Its words come from a source
  where one gives them; a gloss no source gives is **written by a person** and marked so, in the code
  and in the pull request, as it-es's `della` « De la » and it-fr's `dati` « Données », and needs the
  owner's approval. A matter of taste goes to the owner, with examples.
- *The precedents already settled apply*: a name opening a row is accepted when the word is only a name
  (it-en's D6); a given name read as its own language's name in short form (it-es's D7b, the owner's
  answer of 2026-10-25); a displaced first word that is the right cognate stays as a sense of its own
  (it-es's D2g, round 2); a list ordered by its sources' support, then by frequency, before it is cut
  (it-es's D7c); a repeat dropped where it is the same words (it-es's D9b), not where only a parenthesis
  differs.
- *The target.* On the review's judged rows — the 299 glossed rows of the 300 commonest, its 150 of
  301–5,000 and its 100 of 5,001–20,000 (seed 20261026), read again on the new tables by this design's
  reader —, the 300 commonest read wrong in at most **3 %** of their rows, the level of the review's
  bands past 300 (3.3 % and 3.0 %), against 7.7 % now; the rows read « partly right » fewer than now
  (19.7 %); the two samples past 300 no worse. A target missed is reported with the rows that miss it,
  not met by lowering a criterion.

**The data.** en-it's pinned release, `lingua-pack-sources-en-it-2026.10.26`, fetched by
`pack_sources.py fetch-pinned` into a scratch folder (every asset checked by the sha256 its pin records);
read as data, by scripts outside the repository, with Python 3.12 and wordfreq 3.1.1
(`requirements-reduce.txt`).

**The prototype.** A scratch copy of `reduce-en-it.py` with each rule as a switch. Every figure below is
re-measured by the real reduction at implementation (task 3.2).

**Reading.** Every changed row of the 1,000 commonest, and in each of 1,001–5,000 and 5,001–20,000
every changed row or 60 drawn by the seed 20261026, as the criteria say; a rule's variants read the
same way. The reader is an agent (Claude), not a native speaker: its readings decide which variant is
kept, and the owner's specialist reads the gloss diff of the implementation (task 4.1).

## Measured

The base held: `reduce-en-it.py` over the pinned release, with no rule on, wrote `gloss.tsv`,
`senses.tsv`, `mwe.tsv` and the `NOTICE` byte for byte (21,526 rows; 13,423 expressions).

**The classes, on the committed tables**, and what this design does:

| # | Class (the review's issue; change 36's D10) | Ranked rows | Examples | Decision |
|---|---|---|---|---|
| 1 | A minor part of speech first (A) | 199 rows two signals agree to reorder | `by` « passanti… », `but` « solo… », `make` « marca… », `go` « l'atto dell'andare… », `mean` « media… », `country` « campagnolo… », `away` « in trasferta… » | D2 |
| 2 | The main sense missing, or a row of descriptions (B, C) | 83 rows lacking the part of speech both signals name; 69 rows of descriptions | `can` without « potere », `prove` « prove », `treat`, `rescue`, `its` « possessivo di it », `should`, `trash` (the page copied from *thrash*) | D3 |
| 3 | The commonest grammatical words (A, B, C) | 33 reviewed rows (`FIRST_RUN`) | `that`, `this`, `an`, `me`, `can`, `will`, `do`, `very`, `even`, `still`, `down`, `big`, `course`, `take`, `feel`, `way`, `set`, `since`, `read`, `run` | D4 |
| 4 | A frame before the translation (C; D10's colon labels) | 59 rows | `my` « forma possessiva di I usata come aggettivo possessivo, il mio… », `he` « in funzione di soggetto: egli… », `first` « corrisponde al numero cardinale 1, primo », `yes` « risposta affermativa, sì », `american` « relativo ai Stati Uniti, americano… » | D5 |
| 5 | Names' notes (C; D10) | 73 rows: 52 given names read in short form, 21 notes off a common word | `john` « nome proprio di persona maschile, corrispondente all'italiano Giovanni », `summer` « estate; nome proprio… », `buddy`, `buzz`, `daisy`, `marshall`, `smith` « …; il più tipico cognome inglese » | D6 |
| 6 | Pointers left as text (D; D10) | 11 rows, 5 expressions | `r` « …, vedi are », `eh`, `troll` « troll (vedi in italiano) », `cellphone` « variante di cell phone, … », `hale` | D7 |
| 7 | Heraldry anywhere in a row, tinctures (E; D10) | 60 rows | `side` « …; fianco: porzione di uno scudo… », `party`, `a`, `b`, `or` « oro », `sun`, `chief`, `bar`, `skull` | D8 |
| 8 | Not Italian: expansions, wiki text, English sentences (F; D10) | 72 rows of expansions, 1 of wiki text; English sentences, by reviewed rows | `uk` « United Kingdom : Regno Unito », `usa`, `dna`, `btw` « By The Way: a proposito », `fyi`; `sandbox`; `media` | D9, D12 |
| 9 | Split forms and repeats (G, H; D10) | 1 split row; 369 rows repeating a whole sense | `mx` « signorə, signor, a », `one` « uno / uno », `national` « nazionale; nazionale », `few`, `well` « …, pozzo, …; pozzo » | D10 |
| 10 | Misspellings and non-words (F) | 262 words of 218 rows of the 5,000 unknown to `tables/it/forms.tsv` and rare in wordfreq, nearly all real Italian; 9 misspellings among them | « squaglia », « referenzia », « grupo », « sù », « colpo m », « Oileán Thoraigh », « stazine », « orundo », « armoia » | D12 |
| 11 | False friends (I) | 11 rows the review names | `prove`, `disposable` « disponibile », `large` « largo », `ultimately` « finalmente », `editor`, `fabric` « ordito », `firm` « comitiva », `passionate`, `sacred` « santo » | D12 |
| 12 | Contractions (D10) | 57 of the 5,000 unglossed, 6 of the 300; `doesn't` borrowing `don't`'s « contrazione di do not » | `you're`, `that's`, `he's`, `there's`, `they're`, `i'll` | D11, task 0.1 |
| 13 | US state codes (D10) | 21 rows, 18 of them the code's only sense | `de` « Sigla postale per il Delaware », `co`, `al` | D13 (3 rows) |
| 14 | Capital openings, inflected forms in the tables (D10) | 63 / 111 / 172 and 18 / 56 / 149 rows | `you` « tu; voi; Lei », `polar` « polare; polari » | D13, left |

**Each rule alone**, against the committed rows (rows changed among the ranks 1–1,000 / 1,001–5,000 /
5,001–20,000 / beyond, rows whose first sense changes, lemmas gained or lost; coverage 91.4 / 82.8 /
70.3 unless named); *rejected* variants in italics, each read as the criteria say:

| Rule | Rows changed | First sense | Gained / lost | Read (better / worse) | Examples |
|---|---|---|---|---|---|
| D2, the part of speech two signals agree on, a verb never yielding to a noun | 199 (33 / 80 / 69 / 17) | 199 | — | top 1,000: 24 better, 6 no worse, 3 worse (`that`, `former`, `none`, each a reviewed row, D4); 1,001–5,000: 80 read, 4 worse (`joint` « comune », `graph` « grafo », `cope`, `whatsoever`); 5,001–20,000: 60 drawn, 2 worse (`stir` « esaltare », `bugger`) | `by` « da; in, per, con; … », `but` « ma, però; … », `make` « fare, creare, fabbricare; … », `go` « andare; … », `mean` « voler dire, significare; … », `country` « paese, terreno, campagna; … », `state`, `health` « salute; … », `eat`, `walk`, `settle` « appianare, risolvere… » |
| *D2 with the two signals, a verb yielding* | 229 (38 / 87 / 85 / 19) | 229 | — | top 1,000: 5 more worse | `check` « assegno », `wish` « volontà », `beat` « battito », `wonder` « miracolo », `share` « parte » |
| *D2 on the English Wiktionary's page order alone* | 459 (100 / 181 / 149 / 29) | 459 | — | top 1,000: about 2 better for 1 worse | `on` « attivo, acceso », `like` « piacere », `please` « fare piacere », `phone` « telefonare », `former` « creatore », `result` « risultare » |
| *D2 on Italian frequency alone* | 383 (65 / 134 / 154 / 30) | 383 | — | top 1,000: about 2 better for 1 worse | `long` « molto, molto tempo », `want` « esigenza », `must` « obbligo », `wrong` « in errore… », `august` « augusto », `gold` « dorato » |
| *The sense holding the table's first word first* | 481 (44 / 161 / 220 / 56) | 481 | — | top 1,000: about 15 better, 12 worse | `just` « giusto », `only` « unico », `party` « parte », `early` « prematuro », `pick` « piccone », `issue` « emissione » |
| D3a, the missing part of speech, from the table's first sense | 83 (2 / 14 / 49 / 18) | 83 | — | 1,001–5,000: 14 read, 1 worse (`smart` « fare male »); 5,001–20,000: 49 read, 5 worse (`queer`, `goddamn`, `nag`, `flirt` « pomiciare… », `woof` « trama ») | `prove` « dimostrare, provare; … », `treat` « trattare; … », `rescue` « salvare; … », `concrete`, `sole` « unico, solo; … », `spare`, `reset`, `prom` « ballo di fine anno; … », `bravo`, `nerd` « secchione; … » |
| D3b, a row of descriptions opened on the table's first sense | 69 (2 / 8 / 40 / 19) | 69 | — | 1,001–5,000: 8 read, 0 worse; 5,001–20,000: 40 read, 2 worse (`millennial` « millenario », `duff` « sgangherato ») | `its` « suo; … », `should` « dovere; … », `trash` « spazzatura, immondizia, rifiuti; … », `volume`, `shareholder` « azionista; … », `fortnight` « due settimane; … », `mantle` « mantello; … » |
| *D3 for any row sharing no word with the tables* | 859 (14 / 131 / 488 / 226) | 859 | — | 1,001–5,000: 30 drawn, 20 better, 6 worse — under 4:1 | `chart` « carta… », `vintage` « vendemmia », `grain` « seme », `agenda` « agenda », `o'clock` « l'una… », `would` « We sat on the bench » |
| *D3 whenever the row lacks the table's first words* | 1,802 (92 / 394 / 928 / 388) | 1,802 | — | top 1,000: about as many worse as better | `and` « ed; e », `you` « vi, Loro; … », `how` « quanto », `room` « motivo, opportunità », `line` « corda, fune, cima » |
| D4, `FIRST_RUN` (33 rows) and D12, `CORRECTIONS` (21 rows), alone | 52 (34 / 11 / 6 / 1) | 46 | — | better | `that` « quello, quella; che, il quale… », `can` « potere; latta… », `down` « giù; abbasso… », `some` « qualche, alcuni, alcune; del, della », `editor` « redattore, curatore, editore » |
| D5, frames | 59 (19 / 20 / 19 / 1) | 55 | — | no worse after two guards | `my` « il mio, la mia, i miei, le mie », `he` « egli, lui, esso; … », `first` « primo; … », `those` « quelli, quelle », `mine` « il mio…; miniera… », `indian` « indiano; … » |
| *D5 without its guards* | 25 more | — | — | worse | `audio` « alla riproduzione sonora », `leader` loses « articolo di fondo » |
| D6, names | 73 (7 / 35 / 29 / 2) | 55 | — | better | `john` « Giovanni (nome) », `summer` « estate », `mark`, `buddy` « amico », `daisy` « margherita », `jack` « cricco; fante; spina » |
| D7, pointers | 11 (2 / 2 / 6 / 1) | 7 | — | better | `custom` « …; dogana, tasse doganali », `eh` « eh », `cellphone` « telefono cellulare », `millimeter` « millimetro », 5 expressions (`gun powder` « polvere pirica ») |
| D8, heraldry: its wording anywhere, the tinctures | 60 (12 / 19 / 22 / 7) | 5 | — | better; `savage` keeps « selvatico » | `side` « lato; faccia di una medaglia; allearsi… », `or` « o, ovvero », `a`, `sun`, `chief` « capo », `plain` « pianura, piana » |
| *D8 for every sense kaikki tags heraldic* | 201 (23 / 73 / 78 / 27) | 11 | — | worse | `crown` loses « corona », `capital` « capitello », `standard` « stendardo », `bit` « morso per cavalli », `square`, `coach` « carrozza » |
| D9, expansions | 72 (2 / 17 / 35 / 18) | 70 | `cqc` lost (past 20,000) | better; a list of four expansions or more stays as written | `uk` « Regno Unito », `usa` « Stati Uniti d'America », `dna` « acido desossiribonucleico », `btw` « a proposito », `fyi`, `esp` « percezione extrasensoriale » |
| *D9 dropping every English expansion* | 130 (2 / 22 / 72 / 34) | 127 | 42 lost | barred: 82.7 / 70.2, `un`, `ibm`, `ba` unglossed | — |
| *D9 on every sense, not only abbreviations'* | 7 | 3 | — | 3 worse of 7 | `format` loses « format », `crack` « crack (cocaina grezza) », `underpants` « slip » |
| D9, wiki text | 1 | 0 | — | better | `sandbox` « sabbionaia » |
| D10a, a whole sense repeated, « X / X » | 369 (37 / 104 / 141 / 87) | 13 | — | no worse | `national` « nazionale », `one` « uno; … », `well`, `few` « poche, pochi; i pochi », `law` |
| *D10a by comma item across the row* | 1,051 (136 / 373 / 385 / 157) | 13 | — | worse | `small` « …; basso; giovane », `wrong` « …; che non va », `set` « …; studiato », `god` « …; Iddio » |
| D10b, split forms | 1 | 1 | — | better | `mx` « signor/a » |
| D11, contractions read as pointers (task 0.1's (a)) | 59 (15 / 18 / 19 / 7) | 59 | 58 gained | better | `you're` « contrazione di you are », `that's` « contrazione di that is; contrazione di that has », `doesn't` « contrazione di does not » |
| D13, a state's code off a word another sense glosses | 3 (0 / 3 / 0 / 0) | 2 | — | better | `ma` « media mobile », `mo`, `ar` |

**Together** (every rule kept but D11): **1,005 rows change** (138 / 301 / 400 / 166), **587 in their
first sense** (87 / 183 / 241 / 76), 61 of the 300 commonest; no lemma gained or lost; 9 expressions
change (`dvd players` « lettori DVD », `south african` « sudafricano; … »). Coverage **91.4 / 82.8 /
70.3 %**, as now, against the floor **89.4 / 80.8 / 68.3**. Prototype tables: `gloss.tsv` `5bdaf57d…`,
`senses.tsv` `e05a357e…`, `mwe.tsv` `9767baf4…` (re-measured at implementation). With D11 (task 0.1's
answer (a)): 1,064 rows (153 / 319 / 419 / 173), 646 in their first sense, **58 lemmas gained**,
coverage **92.0 / 83.2 / 70.6**.

**The target, on the review's judged rows** (its tally, the same rows read again on the prototype's
tables, by this design's reader):

| Band | Wrong before | Wrong after | Partly right before | Partly right after |
|---|---|---|---|---|
| The 300 commonest (299 glossed) | 23, **7.7 %** | 2, **0.7 %** (`lot`, `doesn't`); 0 with tasks 0.1 and 0.2 answered (a) | 59, 19.7 % | about 34, **11 %** (`will` and `doesn't` among them) |
| 301–5,000 (the review's 150) | 5 | 2 (`cock`, `passionate`, `sacred` read right) | 20 | no worse: 14 rows change, none worse |
| 5,001–20,000 (the review's 100) | 3 | 1 (`cervix`, `inquisition` read right) | 17 | 12 rows change, 1 worse (`nag` « brenna, ronzino; assillare ») |

Of the 23 wrong rows of the 300, rules fix seven — `by`, `but`, `make`, `mean`, `country` (D2),
`something` (D2), `its` (D3b) —; the reviewed rows fourteen — `that`, `this`, `an`, `me`, `can`,
`will` (its auxiliary's description first, partly right), `do`, `very`, `even`, `still`, `down`, `big`,
`course` (D4), `some` (D12) —; `doesn't` reads « contrazione di does not » with task 0.1's answer
(a) (partly right), and `lot` opens on « molto, parecchio » with task 0.2's. Of the review's partly
right rows, the seventeen it names read right (`go`, `take`, `after`, `away`, `feel`, `money`, `way`,
`set`, `since`, `read`, `run`, `country`, `head`, `top`, `social`, `american`, `he`), and so do the
framed and heraldic rows of the 300 (`my`, `their`, `those`, `first`, `second`, `us`, `yes`, `she`,
`a`, `or`, `national`). The target (at most 3 %) is met on these rows as this reader reads them; the
specialist's reading of the gloss diff confirms it or names what misses (task 4.1).

## Decisions

### D1 — Every rule is en-it's, in its own module; no other pair moves

The rules live in a new module, `reduce_english_it.py` (English studied, Italian native, as
`reduce_italian_es.py` is it-es's), imported by `reduce-en-it.py` alone; en-it's rule digest becomes
`reduce-en-it.py`, `reduce_common.py`, `reduce_edition_it.py`, `reduce_english_it.py`. Nothing in
`reduce_common.py` or `reduce_edition_it.py` changes: change 35's rules stay as fr-it and es-it
(changes 47 and 51) will read them, and their prototypes' figures do not move. No new source: the
part of speech's evidence (D2) is what en-it's pin already holds — the two translation tables and
wordfreq's Italian list, credited in its notice. `tables/it/forms.tsv` is read by the measurement only,
to list the misspellings of D12, never by the reducer.

The passes, in this order:
1. **The section's entries** (after `italian.read_as_meanings`, before the shared rules): D6, D8, D9,
   D13, D7, D5 — each rewriting or dropping a sense, a dropped sense the word's only one kept (D7, D9,
   D13).
2. **The tables** (before `native_side`): D10b.
3. **The rows** (after `native_side`): D11, then, for a row the section glossed, D2, D3; then D4 and
   D12; then D10a last, so that a reviewed row's words are not repeated further on.

### D2 — The part of speech two signals agree on

A row the section glosses, of two parts of speech or more, opens on the part of speech that the
English Wiktionary's table lists first for the word **and** whose words, in either table, Italian
writes most (wordfreq's Italian Zipf, the highest of each part of speech's words), when that part of
speech's best word rates at least 0.5 above the row's first part of speech's, and the row holds it. Its
senses move together, in their order; the other senses follow in theirs. Parts of speech are compared
as classes: a determiner, an adjective, a pronoun and a numeral as one; the two conjunctions as one; a
verb and an auxiliary as one. **A verb never yields to a noun**: English verbs are as common as their
nouns or more, and Italian's nouns rate higher one by one.

Each signal alone fails the criteria: the page's order is the etymology's, not use's (`on`
« attivo, acceso », `please` « fare piacere »), and Italian frequency rates the words, not the English
use (`long` « molto, molto tempo », `want` « esigenza »). Together they pass at 8 to 1 in the 1,000
commonest. A margin of 0.5 is the one measured; `that`, `former` and `none`, which it reads worse, are
reviewed rows (D4).

### D3 — The main sense the section leaves out

The table's first sense — the Italian words the English Wiktionary's table lists under the page's first
translated sense, at most three, none English (D9's test), no suffix — opens a row the section glosses:
- **D3a**, when D2's agreed part of speech is not in the row at all, and rates 0.5 above the row's
  first (`can` « potere », `prove` « dimostrare, provare »);
- **D3b**, when the row's first sense describes rather than translates — a frame's opening (D5's
  words, « possessivo », « stesso significato », « indica », « esprime ») or every item four words or
  more — and no sense of the row shares a word with either table (`its` « suo », `should` « dovere »,
  `trash` « spazzatura, immondizia, rifiuti »).

Neither touches a lemma of one or two letters (`a`, `ar`, `ma`: the table's letters and codes). Wider
triggers fail the criteria (table above): the table's first sense is often a rarer one than the
section's.

### D4 — `FIRST_RUN`: the commonest grammatical words, reviewed

A table, keyed by lemma, naming the sense a row opens on — a sense the section writes, moved, or the
words a table gives, added —, each with its reason and its source; a row whose named section sense is
no longer written fails the test that holds the table. 33 rows: `that` « quello, quella; che, il
quale… », `this` « questo, questa », `an` « un, uno, una, un' », `me` « mi, me », `can` « potere »,
`will` (the auxiliary's description), `do` « fare, compiere », `very` « molto, assai… », `even`
« perfino; anche », `still` « ancora, tuttora; … », `down` « giù », `big` « grande, grosso », `course`
« corso, percorso, portata », `take`, `after`, `feel` « sentire », `money` « denaro, soldi, quattrini »,
`way` « via, modo, maniera », `set` « mettere, porre, collocare », `since` « da quando », `read`, `run`,
`about` « su, riguardo », `through` « attraverso, per », `former`, `none`, `cock` « gallo », `cervix`,
`inquisition` « inquisizione », `connection` « connessione, collegamento, coincidenza », `large`
« grande, ampio, vasto », `preservative` « conservante », `conductor`. Every word is a source's; none
is written by a person.

### D5 — A frame before the translation

A sense opening on a frame — « forma », « usato », « utilizzato », « articolo (in)determinativo »,
« complemento », « corrisponde », « relativo », « riguardante », « risposta », « plurale »,
« in funzione », « riferito », « pronome », « introduce » — loses it when what follows, after a colon
or the next comma or semicolon, is items of three words or fewer, none a frame and none opening on a
preposition: `my` « il mio, la mia, i miei, le mie », `he` « egli, lui, esso », `through`
« attraverso ». Its two guards keep `audio` (« relativo all'ascolto, alla riproduzione sonora ») and
`leader`'s « articolo di fondo ». A description Italian has no word for stays (`would`, `could`).

### D6 — Names

en-es's rule, transposed: a proper noun's note (« nome proprio di persona… », « cognome… ») goes from
a word that has a common entry holding a meaning (`summer`, `buddy`, `smith`), and a given name's note
naming its Italian equivalent reads as that name, short — « Giovanni (nome) » for « nome proprio di
persona maschile, corrispondente all'italiano Giovanni » —, the form the owner chose for it-es's given
names (change 28, task 0.3). A name alone keeps its note (`scott`, it-en's D6).

### D7 — Pointers

A parenthesis or item holding « vedi », « si veda », « cfr », « più su », a « (variante di X) », and an
item « variante di X » or « variante, … » go; a sense left with nothing goes, unless it is the word's
only one (`ashtray`). Resolving « variante di X » to X's gloss is left: two rows, both holding their
translation already (`cellphone` « telefono cellulare »).

### D8 — Heraldry

A sense whose text is heraldic — « araldic… », « blason… », « (al|come) cimiero », « dello scudo »,
« di/su uno scudo », « quando riferito a nobili / principi » — goes wherever it stands in its row, and
so does a tincture's sense (`or` « oro », `sable` « nero ») tagged heraldic; a sense whose first item,
before any comma or semicolon and with no colon, is a word of two or fewer that is not heraldic keeps
that item (`savage` « selvatico »). Change 35's rule, which moves such senses last, is unchanged. The
wider rule, every sense kaikki tags heraldic, takes ordinary meanings with it (the table above).

### D9 — Expansions and wiki text

In an abbreviation's entry, an English expansion goes: « United Kingdom : Regno Unito » reads « Regno
Unito », « abbreviazione di justice of the peace (giudice di pace) » « giudice di pace », a sense or an
item whose every word English writes 0.5 Zipf above Italian goes. It goes only when the word keeps a
sense holding an Italian word (Zipf 3 or more, at least English's), and a list of four expansions or
more stays as the source writes it (`csi`, `wc`, `zip`): what replaces it would be no better. A sense
naming a wiki goes when another sense stays (`sandbox`). English sentences in other entries are a
reviewed row's (D12, `media`): the rule on every sense took Italian loanwords with them.

### D10 — Repeats and split forms

- **D10a**: a sense whose every item an earlier sense of the row holds goes, and « X / X » reads X.
  By item across senses it takes the word that gives a sense its meaning (`small` « basso »).
- **D10b**: in the tables, an item holding « ə » goes, and a one-letter item after a word joins it
  (`mx` « signor/a »). « / » inside a word is not split today, and stays so.

### D11 — Contractions, read as pointers (task 0.1)

No source glosses `you're`, `that's`, `i'll`; the section writes the contractions it glosses as
pointers, « contrazione di do not ». With the owner's answer (a), a ranked contraction of a pronoun or
an adverb (`'re`, `'m`, `'ve`, `'ll`, `'s`, `'d`) or of a verb and *not* (`n't`, `can't`, `won't`) that
no source glosses reads the same way, from its full form — `he's` « contrazione di he is; contrazione
di he has » —, and `doesn't` reads its own entry's « contrazione di does not » rather than `don't`'s.
58 lemmas, 6 of the 300. No Italian word is written: the pointer says what the reader's other cards
gloss. A genitive (`women's`, `world's`) is not a contraction and stays unglossed.

### D12 — `CORRECTIONS`

A table of 21 rows, it-es's shape, each with its reason and its page, failing or naming itself when the
source no longer writes what it corrects:
- **The false friends** the review names, opened on the table's word, the misleading cognate dropped:
  `prove` « dimostrare, provare », `disposable` « monouso, usa e getta », `ultimately` « in definitiva,
  alla fine, in fin dei conti », `editor` « redattore, curatore, editore », `fabric` « tessuto,
  stoffa », `firm` « ditta; fermo, solido, stabile », `passionate` « appassionato », `sacred` « sacro;
  santo »; `large`, `preservative` and `conductor` are D4's.
- **The source's errors**: `some` « qualche, alcuni, alcune; del, della » (the page copied from
  *many*), `media` « media, mass media » (three English sentences), `tide` « marea » (« marea, bassa »),
  `whiplash` « colpo di frusta » (« colpo m di frusta »), `tory` without « Oileán Thoraigh ».
- **Misspellings**, the item dropped: « squaglia » (`scale`), « referenzia » (`he`'s sense goes),
  « grupo » (`social`), « sù » (`top`), « stazine » (`station`), « orundo » (`original`), « armoia »
  (`sort`); and `head` without « capoccia ». The census against `tables/it/forms.tsv` lists 262 words of
  the 5,000 commonest's rows unknown to it, nearly all real Italian — a rule would take them —, so the
  misspellings are reviewed rows, and the census is re-run on the implementation's tables.

Every word is a source's. With task 0.2's answer (a), `lot` opens on the expression *a lot*'s
« molto, parecchio », with « (a lot) » written by a person — the only gloss this change writes.

### D13 — Measured and left

- **US state codes**: a code's note goes from a word another sense glosses (`ma` « media mobile »,
  `mo`, `ar`); the 18 rows where it is the code's only sense keep it, a code's meaning.
- **Capital openings** (63 / 111 / 172): `Lei`, brands and acronyms are right; the descriptions that
  open on one are D5's and D3's. Left.
- **Inflected forms in the tables** (change 36's D10): telling them apart needs `tables/it/forms.tsv`,
  which en-it's pin does not record. Left, with the reason change 36 gave.
- **The rows the rules read worse** (D2, D3: `joint`, `graph`, `cope`, `whatsoever`, `stir`, `bugger`,
  `smart`, `queer`, `goddamn`, `nag`, `flirt`, `woof`, `millennial`, `duff`): within the criteria's 4 to 1, listed in the gloss
  diff for the specialist, a reviewed row each if the review asks.
- **Partly right in the 300** (about 34): `will`'s description, `so` « rafforzativo dell'aggettivo:
  così, molto », `lot`'s, `hop`'s order past 300. A word a person would write is the owner's (task 0.2).

### D14 — en-it re-reduced at its pin; what moves and what cannot

The implementation pushes `impl/refine-lingua-en-it-glosses` and opens no pull request; the
coordinator dispatches `lingua-pack-update` in mode `reduce` for en-it on it, from
`lingua-pack-sources-en-it-2026.10.26`. In `tables/en-it/pin.json` the rule digest (and its file list),
the pack and the layer move; the snapshot, the sources, the studied record and the base do not.
`gloss_coverage.py --pair en-it` passes against the floor. Byte for byte: `tables/en/`, every other
pair's tables, pins, packs and goldens, `reduce_common.py`, `reduce_edition_it.py`; fr-it's and
es-it's proposals' figures. `crates/lingua-wasm/tests/baseline/en-it.golden` and the en-it card
snapshot (D15) are re-blessed once, by the update workflow.

### D15 — The card in Italian (change 36's task 4.3)

`apps/lingua-extension/test/word-card-en-it.spec.ts` and `test/baseline/word-card-en-it.txt`: the full
card in Italian (change 33's interface, change 34's renderer, both on `main`) over en-it's golden, as
`word-card-en-es.spec.ts` renders en-es's; its line joins `lingua-pack-update.yml`'s bless step beside
`word-card-it-es.spec.ts`. The implementation commits it over the committed tables before the dispatch,
so that the dispatch's re-bless shows this change's moved lines alone. `some` (one of the golden's 40
lemmas) is among them.

### D16 — Order

After change 36 (on `main`, its requirement open), whose requirement this change modifies; before
change 45, which ships en-it. Changes 47 and 51 (proposals) each hold a copy of the same requirement:
whichever of 37, 47 and 51 is implemented later re-copies the earlier one's block whole and names it in
`archiveAfter` (the programme's rule). On `main` today neither 47 nor 51 is implemented, so this block
is change 36's.

## Settled by rule, not asked

- Given names' short form « X (nome) »: it-es's, the owner's choice (change 28, task 0.3).
- Every gloss of `FIRST_RUN` and `CORRECTIONS` is a source's: nothing for the owner to approve before
  the gloss diff.
- State codes, capitals and inflected forms: D13.

## Risks / Trade-offs

- **A reader that is not a native speaker** → the specialist reads the gloss diff (task 4.1), as for
  changes 23 and 28; the rows the rules read worse are listed for it.
- **Two signals for one order** → a verb guard and a margin, measured; a fourth signal (an English
  treebank) would need a new source in en-it's pin, not taken here.
- **Reviewed rows that age** → each fails or names itself when its source changes, as it-es's do.
- **Pointers in place of Italian for contractions** → the owner's choice (task 0.1); the pointer
  writes no Italian word.

## Migration Plan

Nothing deploys: no package lists en-it before change 45. Rollback is a revert of the reduction's pull
request.

## Effort

4–5.5 ideal days, against the programme's 2–4: the review widened the scope (sense order, the missing
main sense, false friends, misspellings) and change 36's task 4.3 joins it.

| Part | Ideal days |
|---|---|
| `reduce_english_it.py`: D5–D10, D13, their tests | 1–1.25 |
| D2, D3: the two signals, the table's first sense, their tests | 0.75–1 |
| D4, D12: `FIRST_RUN`, `CORRECTIONS`, the misspelling census, their tests | 0.5–0.75 |
| D11 and the owner's answers | 0.25 |
| The card in Italian (D15) | 0.25–0.5 |
| The dispatch, re-measuring, the README, `SOURCES.md` | 0.75–1 |
| The specialist's review, its round, OpenSpec | 0.5–0.75 |
