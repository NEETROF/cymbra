# Design — refine-lingua-fr-es-expressions

## Context

See proposal.md (Why). What exists, on `main` at `311f8fe3`:

| Where | What |
|---|---|
| `tables/fr-es/mwe.tsv` | 12,174 expressions: 587 from the Spanish Wiktionary's French section (the section), 10,683 from the French Wiktionary's Spanish translations (the direct table), 904 from the Spanish Wiktionary's French translations read backwards (the inverted table), in that order of precedence (`native_side`: `common.reduce_expressions`, `label_expressions`, `common.fallback_expressions`). The builder keys 9,472; 64 headwords share a key with another and lose it by the builder's order |
| `reduce-fr-es.py` | change 49's reduction and 49b's own rules; `CORRECTIONS` (49b D7: a source's text corrected or set aside by name, each with its reason and the page to report, the summary line naming one that fires on nothing); no list of expressions left out — fr-en's `LEFT_OUT` (48b, 48c) is fr-en's |
| fr-es's pin | snapshot `2026.10.10`, `lingua-pack-sources-fr-es-2026.10.10` (the Spanish dump of 2026-10-02 12:12, the French dump of 2026-10-02 00:10), UD French-GSD's two files, wordfreq 3.1.1; pack 1,969,484 B |
| The matcher | `match_expressions` (44, 44c, 44d): from each token the longest run that is a key and whose pieces meet the page as written; the next run starts after it. A key is read through the core's lemmas: « de faire » is the key `de faire` « de fait » reads as |
| The goldens | `fr-es.golden` (change 51) renders the French scenario's probes on the real fr-es pack; its phrase probes are `support/french.rs` `PHRASES`, shared with `fr-en.golden`; `word-card-fr-es.txt` and `word-card-fr-en.txt` render every phrase probe |
| Open changes reaching fr-es's expressions | 44e (`pick-lingua-expression-written-as-the-page`: every headword of a key kept, the page's spelling named; re-pins fr-es), 44f (`choose-lingua-overlapping-expressions`: an expression built on a verb gives its last word; probes), 48c (fr-en's translated expressions; adds « une crème brûlée » and « sur le champ de bataille » to `PHRASES`), 43d and 43e (French's forms table; fr-es re-pinned on it) |
| The owner's decision (2026-10-10) | 48c's open question 4: fr-es's expression defects, a change of their own; look for fr-en's classes too |

## Goals / Non-Goals

**Goals:**
- fr-es shows an expression where French text means it, and nothing where it means something else.
- Each choice measured where a reader meets it: matches of the engine on French text, read in
  context, as 48c measured fr-en's.
- fr-es's glosses of words, its coverage, `tables/fr/` and every other pair byte for byte.

**Non-Goals:**
- The matcher. Two defects found here are its own (D5): an invariable expression met by its words'
  other forms (« en fait » by « en faisant »), and « ce que » met inside « est-ce que ».
- New expressions. fr-es keeps its three sources; nothing is written by a person but the four
  corrections' words, which the sources supply (D3).
- fr-en: 48b and 48c settle its expressions; its section's own (48c's open question 3) are not this
  change's.

## Measured

A prototype in the scratchpad (never committed). Its inputs: fr-es's pinned files — the section and
the inverted table (`kaikki-es-Frances.jsonl` `2e724e39…`, `kaikki-es-traductions.jsonl`
`bf44ebb4…`), the direct table (`kaikki-fr-traductions.jsonl` `bd14a5c4…`), UD French-GSD's two
sections —, wordfreq 3.1.1 and `reduce-fr-es.py` of `main`, which reproduced `gloss.tsv`,
`senses.tsv`, `mwe.tsv`, `NOTICE` and `manifest.json` byte for byte. A harness (48c's, a scratch example
of `lingua-pack`) builds the pack with `build_pack` — 1,969,484 B, the pin's — and runs the engine's
`gloss_phrase` on every line of 48c's five corpora: UD French-GSD's training and development
sentences, Gutenberg, French Wikipedia, a Tatoeba sample and the baseline's 13 pages, 51,790 lines,
1,085,056 tokens. A second build of the harness carries change 44f's engine (its proposal's
prototype, applied on `main`) and moves 86 fr-es selections, 44f's own figure.

**Where the matches come from.** 41,549 matches on `main`, 21,086 lines meeting one (40.7 %):
19,208 from the section's expressions, 20,244 from the direct table's, 2,097 from the inverted
table's. « de la » alone, a section expression, is 8,757.

**A judged sample.** 200 matches drawn at random (seed 49), each read in its sentence by one judge (the
prototype's author): **126 right, 7 partly, 67 wrong**. By source: the section 35 / 2 / 48 — 41 of the
48 « de la » —, the direct table 85 / 5 / 14, the inverted table 6 / 0 / 5.

**The classes 48c named, on fr-es:**

| Class | fr-es | Matches |
|---|---|---|
| A partitive read where the page means « of the » | « de la » « Algo de, un poco de, una cierta cantidad de »; no « de l' », « du » or « des » expression (one token each) | 8,757, all 41 sampled wrong (« le maire de la ville », « les sucs de la terre ») |
| A shorter idiom swallowing the start of a longer compound noun | the harness asks, for every match, which expression the engine meets from each token inside it; 1,345 matches cut one off, nearly all rightly (« lors de » before « de la »). Read: « sur le champ » / « champ de bataille » (4 of its 6 matches), and a few right ones either way (« à l'époque » / « époque moderne » 5 of 76, « du coup » / « coup d'état » 2 of 9) | 4 |
| A post-1990 spelling keyed with the traditional one | a word whose circumflex on i or u restored (or « ognon » written « oignon ») is a form of the same lemma in `tables/fr/forms.tsv`, commoner by wordfreq: 11 table headwords; 8 listed beside their traditional spelling (« boite aux lettres », « chaine de blocs », « connaitre comme sa poche »), which loses the key to it; 3 alone (« crème brulée », « clé de voute », « ognon de roscoff »), met under that name by a page writing « crème brûlée » | 2 |

None of 48c's other merge rules has a case here: 4 keys are shared between a section headword and a
table headword, the section's winning all four (« coup d'œil », « metteur en scène », « premier ministre », « aller
et venir » over « allées et venues », both right, change 44e's case); 177 expressions are
glossed by themselves, the Spanish writing them alike (« statu quo », « a priori », « foie gras »,
« in extremis ») but for « à la » "A la" — the Spanish « a la » of « à la maison » too, and read right
in every sampled match.

**The review.** Every expression met at least five times — 822 of the 2,571 met, 38,467 matches
(92.6 %) — read in four of its sentences (three under ten matches), drawn at random. An expression read
wrong there was read again in every sentence when met fewer than 15 times, in at least twelve more
otherwise, or counted by the form the page writes where that tells (« de fait »: 202 of its 223
matches are « de faire »). 65 read wrong in at least 60 % of their matches (D2). Read and kept,
below that line: « comme pour » "Como si, como para" (8 of 14 wrong: « comme pour réparer un
oubli » is right), « à point » "Al punto" (4 of 9: « à 14 points »), « avoir mal » "Dolerle a" (8 of
14), « bien entendu », « que je sache », « bonne nuit », « bonne chance », « je vous en prie » (half,
or fewer than five matches), « en fait » (100 of 196, met by « en faisant », « en faire », « en
font »), « de nouveau » (122 of 231, by « de nouveaux », « de nouvelles »), « le même » "Lo mismo"
(neuter for « le même homme », but read).

**The result**, all rules (D2–D4):

| Corpus | Words | Matches, `main` → this change | Lines meeting one |
|---|---|---|---|
| UD French-GSD (train, dev) | 330,125 | 12,554 → 9,076 | 8,025 → 6,507 |
| Gutenberg | 226,086 | 7,947 → 6,305 | 2,648 → 2,446 |
| Wikipedia | 370,775 | 14,841 → 9,702 | 5,180 → 4,240 |
| Tatoeba | 157,004 | 6,161 → 5,286 | 5,201 → 4,559 |
| The baseline's pages | 1,066 | 46 → 41 | 32 → 30 |
| **All** | **1,085,056** | **41,549 → 30,410** (38.3 → 28.0 per 1,000) | **21,086 → 17,782** (40.7 → 34.3 %) |

8,052 selections move. 11,533 matches go — the list's 11,393 and the renamed or corrected ones —, 394
come: 130 renamed or corrected (« se lever » « Levantarse » 84, « se réveiller » « Despertarse » 18,
« huile d'olive » 12, « grand public » 9, « effet spécial » 5, « crème brûlée » 2) and 264 an
expression the removed one hid (« Vous étiez à la campagne » meets « à la » where it met « être à »,
« c'est à peu près l'heure » « à peu près », « sur le champ de bataille » « champ de bataille »).

- **A fresh sample** of 100 matches (seed 4949, drawn on the prototype's first pass, which differs
  from the result by « comme pour » and « à point » kept, 39 matches, and « laissez faire » left out,
  8; none drawn): **91 right, 5 partly, 4 wrong** (« d'autant qu'il » meeting « autant que », « en
  fait » in « la prohibition en fait une destination », two « ce que » inside a question). The 200
  sampled on `main` would read, the list applied, 124 / 7 / 10 of 141 (57 wrong and 2 right taken out:
  « la glace venait de descendre », « elle me regarda de côté »).
- **What the change newly shows**, 50 of the 264 drawn (seed 4950): 48 read better than before, 2 the
  same (« à 4 jours » meeting « à jour », where « être à » was as wrong), none worse.
- **With change 44f**: 41,547 → 30,410 matches, 8,005 selections moved; on this change's tables 44f
  moves 13 fr-es selections (« avoir à » 11, « se mettre » 2), where it moves 86 on `main` — 73 of
  them were « être à » or « venir de » yielding a word, both left out here. 44f does not reach « sur
  le champ de bataille » (no verb in « sur le champ »): measured, it meets « sur le champ » with and
  without 44f on `main`, « champ de bataille » with and without it here.

`mwe.tsv` 12,174 → 12,108 rows (65 left out, « huile d' olive » merged into the direct table's
« huile d'olive »), 9,472 → 9,406 keyed; the pack 1,969,484 → 1,968,375 B. `gloss.tsv` and
`senses.tsv` byte for byte: coverage 83.0 / 70.7 / 56.7 %, the floor unmoved. On the prototype's
tables every test of `lingua-wasm` and `lingua-pack` passes but those naming fr-es's pin
(`committed_tables.rs`'s two) and `fr-es.golden`, which moves its `pack` line alone.

## Decisions

### D1 — fr-es's own rules, in its reducer

The list, the four corrections and the spelling pass are in `reduce-fr-es.py`, run by `main` after
`native_side` and before `with_etc_period`, on the expressions of all three sources together. No shared
module (`reduce_common.py`) and no edition module (`reduce_edition_es.py`) is edited: fr-es's rule
digest moves, and no other pair's (`test_reduce_editions.py`). fr-en's `LEFT_OUT` is not imported — a
reducer loads no other pair's reducer (49's rule) and fr-en's list names English glosses' faults:
« les autres » "other people" is wrong, fr-es's « les autres » is no expression; « à la » is fr-en's
first row, fr-es's « A la » reads right.

*Rejected — the rules in the builder or the matcher*: every pair would move, for fr-es's data.

### D2 — A reviewed list: 65 expressions left out by name

`LEFT_OUT`, a dictionary of headword → reason, as fr-en's: a person's decision, as a correction is.
An expression it names is left out of `mwe.tsv` after the three sources are merged, so a later source
cannot gloss what an earlier one was left out for; a name no source glosses any more is reported on
the summary line, to be removed, as `CORRECTIONS` reports one firing on nothing. Its line: wrong in
at least 60 % of the matches read (*Measured*, *The review*) — not a bare majority, so that « en
fait » (51 %) and « de nouveau » (53 %), common and right half the time, stay; not two thirds, so that
« venir de » (64 %: « Il vient de Paris » met as « Acabar de ») goes. The 65, by reason (matches on
the corpora in parentheses):

- **A partitive read where the page means « of the »** (1): « de la » (8,757).
- **A shorter idiom swallowing a compound noun** (1): « sur le champ » (6).
- **Words met as they are, where the source names one use or a set phrase** (41): « être à » "Ser
  de, pertenecer" (571: « Il est à Paris »), « venir de » (320), « histoire de » "Con la finalidad
  de" (262: « l'histoire de la ville »), « être pour » "Estar por" (147: « c'est pour ça »), « mais
  que » "(Canadá) Cuando" (55), « mon cœur » "Amor" (53), « et bien » "Pues" (40: « facile et bien
  écrit »), « de qui » "Cuyo" (36), « dans l'ensemble » "En general" (32: « dans l'ensemble des
  sites »), « de rien » "De nada" (32: « peur de rien »), « c'est moi » "A usted" (30), « un tel »
  "Citano, fulano" (27), « encore que » "Aunque" (26: « n'est encore qu'un village »), « laisser
  tomber » "Dejarlo" (24: « se laissa tomber »), « à plus » "Hasta luego" (23: « vendu à plus de »),
  « à l'aide » "¡Auxilio!" (23: « à l'aide d'un théodolite »), « faire avec » "Es lo que hay" (22),
  « à d'autres » "¡a otro con ese asunto!" (20), « de côté » "Soslayo" (20), « de laquelle » "Cuyo"
  (20), « de sable » "Sable" (17: the heraldic black, met as sand), « au vent » "Barlovento" (15),
  « à suivre » "Continuará" (14), « la ferme » "Cállate el hocico" (14), « de force » "Por fuerza"
  (14), « avoir raison de » "Poder con" (13), « dire quelque chose » "Sonarle" (10), « au mieux » "A
  lo sumo" (10), « quatre heures » "Merienda" (10), « convenir de » "Acordar" (10: « il convient
  de »), « à neuf » "De nuevo" (9), « on ne peut plus » "De padre y muy señor mío" (8), « du gâteau »
  "Pan comido" (8), « bon vieux » "De toda la vida" (8), « à vendre » "Se vende" (8), « à table » "A
  comer" (8), « laisser faire » and « laissez faire » "Dejarlo", "Laissez faire" (8: one key —
  without the second, « laisse faire » meets the French « Laissez faire »), « loin de là » "Ni de
  lejos" (7: « non loin de là »), « au goût » "Al gusto" (5), « grand pont » "Autopase" (5).
- **A form of another word met as the headword** (12; the key reads the page's word as the headword's
  lemma): « de fait » (223: « de faire »), « étant donné » (72: « est donné »), « tenir de » "Tirar de"
  (52: « compte tenu de »), « la tienne » (26: « la tenue », « la tenait »), « bien fait » (19: « bien
  faire »), « qui sait » (18: « qui savait »), « ayant droit » (11: « ont droit »), « garder de » (11),
  « à l'anglaise » (10: « à l'anglais »), « à l'ancienne » (9: « à l'ancien »), « en conserve » (5:
  « en conservant »), « de conserve » (5: « de conserver »).
- **A wrong translation no other source corrects** (9): « se passer » "Pasarse" (62), « en ce
  moment » "Al momento, al instante, inmediatamente" (32), « se reposer » "Posar" (21), « se moquer »
  "Jugar" (15), « être défini » and « être sec » "Estar en definición" (15, 6), « se poser » "Posar"
  (14), « parti pris » "Compromiso" (7), « plaque tournante » "Mesa giratoria" (6).
- **A name** (1): « de gaulle » "Apellido" (7).

Together 11,393 matches (27.4 %). The words of each stay glossed: a reader of « le maire de la ville »
reads `de` and `la`.

*Rejected — a structural rule*: by source, the inverted table's expressions (6 right of 11 sampled)
would take « plus de » "Más" (596), « un autre » (165), « se trouver » (285) with them; by shape, a
preposition and an article (« de la », « à la ») would take « à la », right. *Rejected — a list of what
reads right*: it would freeze fr-es at this snapshot; an update's new expressions are read when met.

### D3 — Four slips corrected, not left out

Where another source glosses the expression right, the wrong text is set aside or spelled right by
49b's `CORRECTIONS`, keyed by the source's text as the pinned files write it, each reported upstream:

| Source | Headword | Text | Corrected | Then |
|---|---|---|---|---|
| section | « se lever » (verb) | « Estar de pie uno. » | none | the direct table's « Levantarse » (84 matches: « Elle se leva ») |
| direct | « se réveiller » (verb) | « despedirse » | none | the inverted table's « Despertarse » (18) |
| direct | « grand public » (noun) | « grande publico » | « gran público » | 9 |
| direct | « effet spécial » (noun) | « efecto especiale » | « efecto especial » | 5 |

*Rejected — leaving them out*: « se lever » is met 84 times, and a source glosses it right.

### D4 — Written as French pages write it

After the merge, before the list:

1. **A post-1990 spelling alone** is written in its traditional spelling: a headword one of whose
   words, its circumflex restored on an i or a u (or « ognon » written « oignon »), is a form of the
   same lemma in `tables/fr/forms.tsv` that wordfreq rates commoner, when the tables do not list that
   spelling already — « crème brulée » → « crème brûlée », « clé de voute » → « clé de voûte »,
   « ognon de roscoff » → « oignon de roscoff ». The key does not move (the lexicon reads both
   spellings as one lemma); the card's name does. A headword listed beside its traditional spelling
   stays (8): it loses the key to it today, and change 44e names it where the page writes it so.
2. **An elided piece written apart** from its word is written against it — a piece of French's
   elisions (`c' d' j' l' m' n' s' t' qu' jusqu' lorsqu' puisqu' quoiqu'`) followed by a space: the
   inverted table's « huile d' olive », which won the key and showed that name to « de l'huile
   d'olive » (12 matches), is the direct table's « huile d'olive », whose row is kept.

*Rejected — the rectified spelling left out when alone*: « crème brûlée » would lose its gloss.
*Rejected — every rectified spelling rewritten*: a page writing « boite aux lettres » would lose its
own spelling once 44e names a key's headwords by the page.

### D5 — Measured and left to others

- **The matcher's two** (*Non-Goals*): « en fait » met by « en faisant », « en faire », « en font »
  (100 of 196), « de nouveau » by « de nouveaux », « de nouvelles » (122 of 231), « bien que » by
  « bien qu' » rightly — an invariable expression meeting its words' other forms is a rule of
  `match_expressions` (as 44c and 44d made elision and contraction one), for every pair; « ce que »
  « Lo que » met in « Est-ce que tu viens ? » (250 of its 940 matches): fr-es has no « est-ce que »
  — none of its sources glosses one —, and the hyphen inside « est-ce » is 44f's guard's matter. Named
  in the pull request for a change of their own.
- **What the review did not read**: the 1,749 expressions met fewer than five times (3,082 matches,
  7.4 %). By the fresh sample, about one match in twenty-five of the result reads wrong.

### D6 — The pin and the pack

`build.sh --reduce fr-es` from `lingua-pack-sources-fr-es-2026.10.10` and UD French-GSD's two files,
nothing fetched: `mwe.tsv`, `manifest.json` (`pack_version` `2026.10.10+<digest[:7]>.<studied>`) and
`pin.json` (`reducer`, `pack`) move; the snapshot, the studied record and every source record byte for
byte. 1,968,375 B in the prototype.

### D7 — What moves, what cannot

- **Moves**: `tables/fr-es/mwe.tsv`, `manifest.json`, `pin.json`, `README.md` (the figures, the list
  and the four corrections, *Known data defects*); `SOURCES.md`; `fr-es.golden`'s `pack` line. Six
  phrase probes join `support/french.rs` `PHRASES`: « sur le champ de bataille » (fr-es: « champ de
  bataille » "Campo de batalla", where `main` shows « sur le champ » "En el acto"), « une crème
  brûlée » (« crème brûlée », where « crème brulée »), « le maire de la ville » (nothing, where « de
  la » "Algo de…"), « Elle se leva. » (« se lever » "Levantarse", where "Estar de pie uno"), « Il vient
  de Paris. » (nothing, where « venir de » "Acabar de"), « de l'huile d'olive » (« huile d'olive »,
  where « huile d' olive »). The first two are 48c's: whichever change lands first adds them, the
  second re-blesses their fr-es lines. `fr-es.golden` and `fr-en.golden` gain the six (fr-en's answers
  its own, none of its existing lines moving), `word-card-fr-es.txt` and `word-card-fr-en.txt` their
  cards, the specs' probe counts following.
- **Cannot move**: `gloss.tsv`, `senses.tsv`, coverage; `tables/fr/` (read, not written); fr-en,
  en-fr, es-fr, es-en and en-es — no file of their rule digests is edited — their tables, pins, packs
  and goldens but the probe lines above; the engine; every other test of `lingua-wasm` and
  `lingua-pack` (`cross_native.rs`'s fr-es assertions hold: measured).

### D8 — Order and archive

| Change | Relation |
|---|---|
| 49, 49b | before (on `main`): fr-es's sources and pin, and `CORRECTIONS`, which this change extends |
| 44e | either side: it re-pins fr-es (`expr.others`); whichever lands second re-reduces and re-pins. Without the list, 44e would keep « laissez faire » as another headword of `laisser faire`; with it, neither |
| 44f | either side: on this change's tables 44f moves 13 fr-es selections, not 86; if 44f lands first, its fr-es probes « Elles sont au nombre de sept. » (« être à ») and « Il resta jusqu'au début de la nuit. » (« de la ») move here to nothing; if second, it blesses them so |
| 48c | either side: the two probes (D7) |
| 43d, 43e | either side: they move `tables/fr/forms.tsv`, which D4 reads; whichever lands second re-reduces fr-es and re-runs D4's census |
| 52 | after: fr-es ships with this change, as fr-en with 48c |

`archiveAfter`: 43 (French's forms table, read by D4), 44 (the keys the list is measured through),
49 (fr-es), 49b (`CORRECTIONS`). Sentences of open changes this change makes true of their time, to
amend when each is archived (nothing to decide): 49's figure « 12,174 expressions » and 49b's
« 4 rows and 1 expression » of `CORRECTIONS` (now 8 rows).

## Risks / Trade-offs

- **[One judge]** → every figure of *Measured* is one reader's; the list goes to the owner with its
  reasons and contexts (task 5.2), the samples with it.
- **[Right matches lost]** → about 3 % of the list's matches read right (2 of 59 sampled); « venir de »
  is the costly one: « Il vient de partir » loses « Acabar de » (about 115 of its 320 matches precede an
  infinitive). Kept out by the line D2 draws; a matcher rule that needs the infinitive is not fr-es's.
- **[The list is read on the corpora it is measured on]** → the fresh sample was drawn after it; an
  update's new expressions are read when they are met often.
- **[An update rewords a source]** → a listed headword no source glosses is reported, as a correction
  firing on nothing; a listed headword glossed right after an update stays out until a person reads it.
- **[The spelling heuristic]** → it rewrites 3 headwords; the pull request runs it and lists every
  headword it reads, so a word it should not reach is seen.

## Migration Plan

Nothing to migrate: no reader holds a French pack. Rollback is a revert of the rules and the tables.

## Effort

1–1.75 ideal days: the list, the corrections, the spelling pass and their tests 0.5–0.75; the
re-reduction, pin, README and `SOURCES.md` 0.25–0.5; the probes, goldens and snapshots 0.25; the
harness re-run and the comparison 0.25.

## Open Questions

None. Settled by the owner on 2026-10-10 (in session): fr-es's expression defects are a change of
their own, fixed in fr-es's reducer by a reviewed list and rules, the classes 48c found in fr-en looked
for (48c's open question 4). Shipping before change 52 follows the owner's answer for fr-en (48c's
question 2). The owner reads the list and the corrections (task 5.2).
