# Design — choose-lingua-overlapping-expressions

## Context

See proposal.md (Why). How the phrase gloss chooses among a selection's expressions today
(`origin/main` d02371ac: changes 39 to 51b, 43b, 44c and 44d merged, fr-en's and fr-es's tables as
43b re-reduced them):

| Seam | Today |
|---|---|
| `engine::match_expressions` | from each token, the longest run that is a key (2 to 5 tokens in English and Spanish, 7 in French; French's `du`/`des` retry, change 44's D5), French runs held to their headword's elisions and contractions (changes 44c, 44d), a French or Spanish match extended over the article of a written « au », « aux », « al », « del » (change 51's D12, change 44b's D5); **the next run starts past it**. The run that starts first wins, however short |
| French's pre-pass (`analysis/tokenize.rs`, change 40) | a hyphenated inversion (« est-ce », « a-t-il », « allez-y ») is read as pieces with spans of their own, the euphonic `t` dropped; the text between two pieces holds the hyphen. **No match looks at it**: « c'est ce que » and « Est-ce que » both read `être ce que`, and `est-ce que` meets both |
| `Pack::readings(lemma, form)` | a form's readings as the dictionary form the analysis read it as (`add-lingua-word-grammar`), from each language's studied tables (`tables/fr`, `tables/es`, `tables/en`), the same for both pairs of a studied language: `sont` → `être` VERB, `hizo` → `hacer` VERB, `went` → `go` VERB. Read by the word card, never by the phrase gloss until now |
| `analysis::function_words` | each language's closed classes, its prepositions among them (French's `FR_PREPOSITIONS`, change 41: `à`, `de`, `en`, `jusque`…; Spanish's `a`, `de`, `con`…; English's prepositions and particles `in`, `off`, `up`…); only `is_function_word` is public |

**How it was measured.** A scratch copy of `origin/main` at d02371ac, never committed, carrying
`match_expressions` behind switches for every rule and guard below, and change 44d's harness
extended to the six pairs: each pack built from the committed tables, every line of the corpora
glossed as one selection, the matches compared with today's.

| Corpus | Selections | Tokens |
|---|---|---|
| French: change 39's `baseline/pages-fr.txt` block by block, UD French-GSD train and dev, four Gutenberg novels by paragraph, French Wikipedia paragraphs, Tatoeba's French sentences (changes 42, 44c, 44d) | 51,790 | 1,085,056 |
| Spanish: Tatoeba's 442,407 Spanish sentences, 11,567 Spanish Wikipedia paragraphs, 5,341 UD Spanish sentences, 5,212 paragraphs of five Spanish-language Gutenberg books, change 3's `baseline/pages-es.txt` (changes 42b, 44d) | 464,574 | 4,160,681 |
| English: change 1's `baseline/pages.txt` block by block, UD English-PUD's 1,000 sentences, 10,000 quotations and 57,434 example sentences of the English Wiktionary's English entries | 68,484 | 1,614,394 |

Matches today: fr-en 60,360, fr-es 41,549, es-fr 205,743, es-en 236,765, en-fr 48,552, en-es
62,835. A change is a selection whose matches move, counted once over a language's two pairs.
Changes are judged by one judge, as the study's marks were — better, worse or neither (both wrong,
or both right) — on 30 drawn at random (seed 20261010, ± 6 points) and, for French and for every
guard, on all of them; every judged change is in [review.md](review.md).

**The overlaps today.** Where two expressions share a token, the first wins, and where it is built
on a verb it is mostly the wrong one:
- French: `être à` (« to belong to ») takes the `à` of « sont au nombre de », « est au courant », « est
  à la fois » (16 in fr-en), « est à l'origine de » (31); `tenir à` that of « tenait à la main »; `j'ai`
  the « ai » of « j'ai besoin de » (23, where `avoir besoin de` « to need » fits); `c'est` the « est »
  of « c'est le cas » (21, `être le cas`). But a compound preposition owns its `à`: « jusqu'au début »
  is `jusqu'à`, *until the beginning* (33 such places in fr-en, with `grâce à`, `par rapport à`,
  `quant à`).
- Spanish: `ser de` (« to be one for ») takes the « de » of « es de lo más », « son del mismo »,
  « es del todo » (87 together); `hacer de` (« to pretend to ») that of « lo hizo de todos modos »;
  `me llamo` (« my name is ») the « llama » of « me llama la atención » (17); `es más` (« what's more »)
  the « más » of « es más o menos » (29).
- English: `stand in` (« to substitute ») takes the « in » of « standing in front of »; `go over`
  (« to examine ») the « over » of « went over the top »; `be on` the « on » of « was on account of ».

## Goals / Non-Goals

**Goals:**
- The owner's case and its kind, in the three languages: « Elles sont au nombre de sept » answers
  `au nombre de`, « Lo hizo de todos modos » `de todos modos`, « went over the top » `over the top`.
- Keep what today's order gets right (« jusqu'au début », `jusqu'à`), and cut the rule's misfires
  with measured guards.
- A French headword's hyphen held to the page, so « c'est ce que » never answers `est-ce que`.
- Read what the core already has: the pack's grammar, the closed classes, the tokens' spans and the
  text between them. No pack, key, table, pin or analyser version moves.

**Non-Goals:**
- Overlaps whose first expression is not built on a verb (« de faire la connaissance », where `de
  fait` is keyed `de faire`; « comme tout le monde »): they keep today's order (D5's rule A reaches
  them, and does worse).
- The misfires no guard measured here removes (D3): an English idiom read literally (« took off the
  record » answering `off the record`), a Spanish `es` given to `es lo que hay` in « ¿Qué es lo que
  hay? ».
- Page highlighting: `analyse_page` is untouched, as in changes 44, 44c and 44d.

## Decisions

### D1 — An expression built on a verb gives its last word to a longer one

In French, Spanish and English, a match **gives up its last token to the expression that starts on
it** when:
1. **it is built on a verb** (D2): one of its tokens, its last included, is a form the pack's grammar
   reads as a verb or an auxiliary, with no preposition before it;
2. **the expression that starts on its last token** — the longest run that is a key and meets its
   headword there, as the scan would find it — **holds more tokens than the match**, or (French,
   Spanish) starts on the `à` of « au »/« aux » or the `a`/`de` of « al »/« del » the match ends on,
   which it then covers whole;
3. **neither guard of D3 holds.**

The match then does not match at that length: the shorter runs are tried, else the scan moves to the
next token, exactly as for a run that does not meet its headword (changes 44c, 44d); the expression
is found on that token in turn. Anything else keeps today's order.

| Page | First expression | Starting on its last token | Today | With the rule |
|---|---|---|---|---|
| « Elles sont au nombre de sept » | `être à` | `au nombre de` (on the `à` of « au ») | `être à`, `nombre de` | `au nombre de` |
| « Il est à la fois sévère » | `être à` | `à la fois` (3 > 2) | `être à` | `à la fois` |
| « Elle se mit au lit » | `se mettre à` | `au lit` (on « au ») | `se mettre à` | `se mettre`, `au lit` (tried shorter) |
| « J'ai besoin de toi » | `j'ai`, the verb last | `avoir besoin de` (3 > 2) | `j'ai` | `avoir besoin de` |
| « Il resta jusqu'au début » | `jusqu'à`, no verb | `au début` | `jusqu'à` | the same |
| « Il venait de l'étranger » | `venir de` | `de l'` (2, not more) | `venir de` | the same |
| « Lo hizo de todos modos » | `hacer de` | `de todos modos` | `hacer de` | `de todos modos` |
| « Van al final del álbum » (es-fr) | `ir a` | `al final de` (on « al ») | `ir a` | `al final de` |
| « Pese a que no tengo dinero » | `pese a` | `a que no` | `pese a` | the same (G2) |
| « Ella se iba al día siguiente » (es-fr) | `ir a` | `al día` | `ir a`, `día siguiente` | the same (G3) |
| « The whole thing went over the top » | `go over` | `over the top` | `go over` | `over the top` |
| « He was standing in front of the house » | `stand in` | `in front of` | `stand in` | `in front of` |

**Measured** (base d02371ac):

| Pair | Matches today | Lost | Gained | After | Selections moved |
|---|---|---|---|---|---|
| fr-en | 60,360 | 368 | 295 | 60,287 | 320 |
| fr-es | 41,549 | 93 | 90 | 41,546 | 90 |
| es-fr | 205,743 | 308 | 229 | 205,664 | 228 |
| es-en | 236,765 | 453 | 391 | 236,703 | 389 |
| en-fr | 48,552 | 73 | 71 | 48,550 | 71 |
| en-es | 62,835 | 140 | 124 | 62,819 | 124 |

French's figures hold G1's own moves (D3): 49 fr-en selections (4 of them the rule moves too) and 4
fr-es selections the hyphen check moves on today's order, before any overlap.

**Judged:**

| Language | Changes | Judged | Better | Worse | Neither |
|---|---|---|---|---|---|
| French | fr-en 320 | all, by kind | 285 | 18 | 17 |
| French | fr-es 90 | all, by kind | 85 | 4 | 1 |
| French | 346 distinct | sample of 30 | 28 | 0 | 2 |
| Spanish | 567 distinct (es-fr 228, es-en 389) | sample of 30 | 22 | 2 | 6 |
| English | 160 distinct (en-fr 71, en-es 124) | sample of 30 | 22 | 2 | 6 |

French's 18 worse in fr-en are 11 of G1's own (« Est ce que vous… », « n'est ce pas », « Vois tu »
written without their hyphen lose their expression, D3) and 7 of the rule's (`pas de quoi` on
« savent pas de quoi » 3, `garde à vous` on « prendre garde à vous », `au noir` on « le blanc l'est au
noir », `avoir la peau de` on « J'ai la peau du ventre », `faire le jeu de` on « Tout compte fait, le
jeu »); its 17 neither are places where both readings are wrong or both right (« se mettre d'accord »
5, « se mettre en route » 2, « est au plus haut », « à moins de la moitié »). fr-es's 4 worse are G1's
(« Qu'est ce que » without its hyphen). Spanish's 2 worse in the sample: `es lo que hay` (« it is
what it is ») on « ¿Qué es lo que hay sobre la cama? » and `sea lo que sea` on « seguir siendo lo que
somos »; English's: `out of gas` and `out of the wood`, idioms read on a page that means them
literally. The neither are mostly a literal reading both expressions miss (`down the road` over `run
down`, « sabías que yo sabía »).

Of change 44d's 92 fr-en selections where an expression starting on « au » lost to one ending on its
`à` (first round, `main` 9c88442a), 47 move — every verb's expression but `en venir à`, whose « en » D2 reads as a preposition —
and the 43 compound prepositions keep their `à`; fr-es: 31 of 32 (`face à` on « face au vent »
keeps it).

### D2 — « Built on a verb »

The pack carries no part of speech for an expression (`mwe.tsv` is a headword and a gloss); adding
one would move every pack and pin. It carries the readings of every form on the studied side
(change 45 for French, the archived word-grammar changes for English and Spanish), which the word
card reads already: `Pack::readings(lemma, form)` on a token's lemma and lowercased surface says
whether the form is a verb as the analysis read it. One form, one lemma (M8): « été » read as `être`
is a verb. A pack without grammar reads no verb, and keeps today's order.

Readings of « built on a verb », measured in French (fr-en / fr-es selections moved), the « au »
and longer conditions of D1 as they are:

| Reading | French | Differs |
|---|---|---|
| a verb among the tokens before the last (the first proposal's first variant) | 175 / 83 | gives « del » away in Spanish's `a partir de` on « a partir del cual », `a pesar de` on « a pesar del tiempo », prepositions built on an infinitive |
| a verb before the last, no preposition before it (the first proposal) | 173 / 83 | keeps `j'ai` and `c'est`, whose verb is the last token |
| **a verb, the last token included, no preposition before it (this design, the owner's « any last word »)** | 298 / 86 without D3's guards | `j'ai` gives « ai » to `avoir besoin de` (23), `c'est` « est » to `être le cas` (21) — and to `est-ce que` on « c'est ce que » (23), which G1 removes |
| the first token a verb | 168 / 83 | loses `se mettre à` (« se mit au lit »: a pronoun first) |

**A preposition** is one of the studied language's closed classes' prepositions (English's
prepositions and particles), exposed by `is_preposition(lemma, studied)` beside `is_function_word`.
`du`, `des`, `del` and `al` are not in them: « venu du côté de » keeps `venir de` (2 such places in
fr-en).

### D3 — The guards, each measured

Each guard was measured against the rule with every other guard on; « better » means the guard is
right where it keeps today's reading, every selection judged ([review.md](review.md), section 2).

| Guard | Languages | Selections it changes | Better | Worse | Neither | Kept |
|---|---|---|---|---|---|---|
| **G1 — a French headword's hyphen meets only a hyphen** (in the overlap rule) | French | 23 (fr-en) | 23 | 0 | 0 | yes |
| G1, on its own (today's order, before any overlap) | French | 53 (fr-en 49, fr-es 4) | 38 | 15 | 0 | yes, with it |
| **G2 — a preposition the page follows with « que » stays** | French, Spanish | 8 (es-fr 8, es-en 7; French 0) | 8 | 0 | 0 | yes |
| **G3 — the expression next is not cut**, if it holds a word that is not a function word | all three | 10 (Spanish 7, English 3; French 0) | 7 | 2 | 1 | yes |
| G3 cutting any expression, `de la` included | all three | 14 more | 0 | 13 | 1 | no |
| S1 — a verb the headword writes inflected, written so on the page (« pese », « debido »), keeps its words | Spanish | 36 | 10 | 25 | 1 | no |
| S3 — a last `que` stays | French, Spanish | 18 | 6 | 3 | 9 | no |
| the verb written as its dictionary form (« pesar »), else no verb | Spanish | es-fr 242 → 190, es-en 397 → 259 | — | most | — | no |

**G1 — hyphens.** French's pre-pass splits « est-ce », « a-t-il », « allez-y » into pieces with
spans of their own, so a headword's hyphen was invisible to the matcher: `est-ce que` met « c'est ce
que » and « Est ce que » alike. The owner's guard: a headword joining two pieces with a hyphen —
read by `headword_reading`, a hyphen with the euphonic `t` between (`-t-`) counted as one — meets
only a page joining them with a hyphen too, read without soft hyphens (change 42d). It is a matching
rule beside changes 44c's and 44d's, checked in the same reading of the name, and it moves matches
on its own, before any overlap: on fr-en 49 selections, 38 better — `est-il` no longer reads « À
l'Est, il » or « comme il est, il », `puis-je` « et puis je lui ai dit », `allez-y` « on va y
arriver », `a-t-il` « qui les ont. Il » — and 11 worse, where the page writes the expression without
its hyphen (« Est ce que vous envisagez », « n'est ce pas », « Vois tu »); fr-es 4, all worse (its
table holds only the `qu'est-ce` family of these). Measured, the narrower guard — the hyphen checked
only for the expression that would take a word — removes the same 23 `est-ce que` and moves nothing
on its own; this design keeps the owner's (open question 1).

**G2 — « que » after a preposition.** `pese a` (« despite ») and `debido a` (« due to ») are
prepositions made of a verb form, and with « que » after them conjunctions: « Pese a que no tengo
dinero » would answer `a que no` (« I bet you won't »). The guard keeps a match's last token when it
is a preposition the selection's next token, `que`, follows: the 8 selections it changes (« pese a
que » 4, « debido a que » 3, « apuntan a que no habrá ») are all better, and none other moves.

**G3 — no cut.** `ir a` giving the « a » of « iba al día siguiente » to `al día` (« up to date »)
would cut `día siguiente`, which the scan finds right after today. The guard keeps the match's last
token when the expression that would take it ends inside the run the scan would take next after the
match, and that run holds a token that is not a function word. Changed: « iba al día siguiente » 2,
« ido al fin del mundo », « dejado de la mano de Dios » 2, and in English `up and run` and `on the way` kept out — better —;
« las cosas no van del todo bien » (`del todo` « entirely » right) and `all in a day's work` kept out
— worse —; « sabe que yo sé que » — neither. Cutting any next run, `de la`
and `de l'` included, would block 14 right moves more (« c'est à cause de la souris »).

*Rejected — S1, S3, the dictionary form.* Each removes a few misfires (`debido al uso`, `es que lo
que` → `que lo que`) for more right moves (`es más` keeping « más » from `más o menos` 20 times;
`me llama la atención` 17 under the dictionary-form test).

*English: none found.* Its misfires are idioms read on pages that mean them literally — `off the
record` over `take off`, `out of gas` over `phase out`, `in the red` over `park in` — and share no
shape the matcher sees that its right moves (`over the top` over `go over`, `on account of` over `be
on`) do not.

On the first round's samples (D5, F shared), the guards turn Spanish's 25 better, 4 worse, 1
neither into 25, 1, 1 with 3 left as today (G2 « pese a que », « debido a que »; G3 « ido al fin del
mundo »), and leave English's 25, 3, 2 as they were.

### D4 — Where the matcher asks, and what it costs

In `match_expressions`' scan, at a start, the runs are tried longest first as today. A run that is
a key and meets its headword is then checked: the expression starting on its last token is looked up
the way the scan looks one up (the longest run that is a key and meets its headword, without asking
whether that one would in turn give way); then condition 2; then G2 (a lemma); then whether the run
is built on a verb (its tokens' readings); then G3 (one more lookup). `gloss_phrase` computes the
page's hyphen joins once per selection, from the text between the tokens' spans, beside
`shares_span` and `elided`.

Measured, the median of interleaved passes over each language's selections, natively, on a machine
shared with other sessions: fr-en 2.861 → 2.959 s (+3.4 %), fr-es 2.732 → 2.762 s (+1.1 %), es-fr
9.342 → 9.551 s (+2.2 %), es-en 8.828 → 9.056 s (+2.6 %), en-es 2.872 → 2.898 s (+0.9 %), en-fr 2.976 → 2.964 s (within the noise).

The giving match is treated as a run that does not match at that length, so change 51's D12 never
covers an article for it, change 44d's D4 (a match starting on the article of « au ») is untouched,
and a match's key, name, gloss and status are what they are without the rule. Spanish's settled
chain (change 44b's D6) is read on the run that wins, as before.

### D5 — The first round: rules measured on main 9c88442a

The first proposal measured nine rules over the French corpus (and the shared ones over Spanish and
English), each judged on 30 sampled changes (French / Spanish / English, better / worse / neither;
[review.md](review.md), section 3):

| Rule | fr-en / fr-es moved | French | Spanish, shared | English, shared |
|---|---|---|---|---|
| A — the longer match wins (every run of every start, longest first) | 490 / 123 | 20 / 7 / 3 | 17 / 6 / 7 | 23 / 5 / 2 |
| B — the most tokens covered | 557 / 141 | 13 / 14 / 3 | 11 / 18 / 1 | 20 / 8 / 2 |
| C — the « au » covered whole | 92 / 32 | 21 / 9 / 0 | 17 / 12 / 1 | — |
| D — C, to a match covering more written words | 38 / 11 | 17 / 13 / 0 | — | — |
| E — C, when the giving match holds a verb | 49 / 31 | 28 / 0 / 2 | 25 / 5 / 0 | — |
| F — E for prepositions written apart too, to a longer match (the first proposal) | 173 / 83 | 30 / 0 / 0 | 25 / 4 / 1 | 25 / 3 / 2 |
| G — F for any last word (this design without guards) | 323 / 98 | 26 / 4 / 0 | — | — |
| H — any match holding a verb, to one reaching further | 1,009 / 397 | 15 / 14 / 1 | — | — |
| I — any match, to one reaching further | 2,521 / 1,262 | 7 / 21 / 2 | — | — |

A and B are wrong where a compound preposition is first (`à la fin` over `jusqu'à` on « jusqu'à la
fin »), or split a right expression (« en vue de la » `en vue` and `de la`); C and D give every
`jusqu'à` away; H and I give a last word to any later expression, however short (`pas que` on « je
ne pense pas que »). G, judged on all its fr-en changes, was 276 better, 31 worse — 23 of them
`est-ce que` on « c'est ce que », which G1 removes — and 16 neither; the owner chose it, with G1
(2026-10-10).

### D6 — What moves: probes added in the six goldens

Measured on the prototype with the rule and the guards on and no probe added: the six goldens are
byte for byte, as are the `cross_native`, `parity`, `card_gloss_language`, `spanish_expression_keys`,
`soft_hyphens`, `languages`, `deck`, `statuses` and `reprofile` suites (98 tests in 15 suites), and
every `lingua-core` test passes. So the scenarios gain probes:

| Probe | Today | With the rule |
|---|---|---|
| **French** (`support/french.rs`, after « à mon age ») | fr-en; fr-es | fr-en; fr-es |
| « Elles sont au nombre de sept. » | `être à`, `nombre de`; `être à` | **`au nombre de`**; `être à` (fr-es holds no `au nombre de`) |
| « Il est à la fois sévère et amical. » | `être à`; `être à` | **`à la fois`**; **`à la fois`** |
| « Il est au courant de tout. » | `être à`; `être à` | **`au courant`**; **`au courant`** |
| « Elle se mit au lit. » | `se mettre à`; `se mettre` | **`se mettre`, `au lit`**; `se mettre` |
| « grâce au travail du monteur » | `grâce à`; `grâce à` | the same |
| « J'ai besoin de toi. » | `j'ai`; `avoir besoin de` | **`avoir besoin de`**; the same |
| « C'est ce que tu veux. » | `c'est`, `ce que`; `ce que` | the same (G1) |
| « À l'est, il fait froid. » | `est-il`, `faire froid`; none | **`faire froid`**; none (G1) |
| **Spanish** (`support/spanish.rs`, after « La ciutat de Barcelona ») | es-fr; es-en | es-fr; es-en |
| « Lo hizo de todos modos. » | `hacer de`; `hacer de` | **`de todos modos`**; **`de todos modos`** |
| « No es del todo cierto. » | none; `ser de` | none; **`del todo`** |
| « Pese a que no tengo dinero, compré el libro. » | `pese a`; `pese a` | the same (G2) |
| « Van al final del álbum. » | `ir a`; `al final` | **`al final de`**; `al final` |
| « Ella se iba al día siguiente. » | `ir a`, `día siguiente`; `al día` | the same (G3) |
| **English** (`support/english.rs`, after « El faro se alzaba ») | en-fr; en-es | en-fr; en-es |
| « The whole thing went over the top. » | `go over`; `go over` | **`over the top`**; **`over the top`** |
| « He was standing in front of the house. » | `stand in`; `stand in` | **`in front of`**; **`in front of`** |
| « They were picking up and running off. » | `pick up`; `pick up`, `run off` | **`up and run`** (en-fr holds no `run off`, so G3 has nothing to keep); the same (G3) |

Re-blessed on the prototype, lines added only: `fr-en.golden` 813 → 829 and `fr-es.golden` 823 → 839
(16 each), `es-fr.golden` 648 → 658 and `es-en.golden` 748 → 758 (10 each), `en-fr.golden` 655 → 661
and `en-es.golden` 763 → 769 (6 each). `word-card-fr-en.txt` 737 → 761 and `word-card-fr-es.txt` 601
→ 625 gain the eight French probes' cards (24 lines each, every block before byte for byte) — in
fr-en « au nombre de — numbering », « à la fois — at a time, at the same time, at once », « au
courant — up to date, abreast », « se mettre — to place oneself in a place » and « au lit — in bed »,
« avoir besoin de — to need », no row for `est-il`; their specs count 50 phrase probes (49 phrases and
the reader's). `word-card-es-en.txt` and `word-card-en-es.txt` render grammar probes only and do not
move. No `analyse`, `gloss`, `word-grammar` or review probe, pack, pin, table, `pack_version` or
fixture moves; every analyser version stays, as changes 44, 51, 44c and 44d read *An analyser version
per studied language* for the phrase gloss's matching.

The invariance baselines name reasons for moving a golden, none of them probes a change adds;
changes 44, 51, 44c and 44d added theirs and said so in their pull requests, and so does this one.

### D7 — es-fr and en-fr move, approved

es-fr and en-fr ship today; the programme's rule is that their output does not move. The owner
approved it moving on 2026-10-10 (tasks 0.2), told that the shared rule fixes 25 of 30 sampled
changes and breaks 3 to 4 in Spanish and English, and that es-fr and en-fr move (141 and 61
selections with the first proposal's rule). The rule as settled moves es-fr on 228 of 464,574 Spanish
selections and en-fr on 71 of 68,484 English ones, es-en and en-es on 389 and 124; it lands with the
next release that carries the core, and a reader's statuses and cards are untouched (a match's key
is what it is without the rule). The goldens show the new probes; no earlier probe moves.

### D8 — What later changes take from here

| Change | Takes |
|---|---|
| 52 `enable-lingua-french` | the dogfood: « Elles sont au nombre de sept », « Il est à la fois sévère », « jusqu'au début », « C'est ce que tu veux » on a real page in fr-en |
| 44e `pick-lingua-expression-written-as-the-page` (proposed) | the probes, in whichever re-bless of `fr-en.golden`, `fr-es.golden` and the two French snapshots lands second (both add phrase probes and move the snapshots' count) |
| a change that reads more of a page's punctuation | G1 reads hyphens only; « Est ce que » without its hyphen loses `est-ce que` (11 places) until a page's informal spelling is read as the dictionary's |

## Risks / Trade-offs

- [A right reading given up] → French 7 of 320 in fr-en through the rule; Spanish and English about
  2 in 30 each, sampled (D1).
- [G1 on informal pages] → « Est ce que », « n'est ce pas », « Vois tu » written without their hyphen
  lose their expression: 11 places in fr-en, 4 in fr-es (D3, open question 1).
- [A preposition made of a verb] → `pese a`, `debido a` before « que » are kept by G2; before a noun
  phrase (« debido al uso habitual ») they still give « al » to `al uso` (2 places), which S1 would
  keep at the cost of 25 right moves.
- [A form the grammar does not read] → no verb is found and today's order is kept: the rule can only
  miss, never invent.
- [Chains] → the expression a match gives way to is not asked whether it would give way in turn; the
  scan finds it, or a shorter one, when it reaches it. Measured in D1's figures.
- [Shipped pairs move] → es-fr and en-fr readers see other expressions on 228 and 71 of the
  selections measured; approved (D7), and read by the owner in the pull request (4.2).
- [Cost] → +0.9 % to +3.4 % of the gloss time (D4).

## Migration Plan

Nothing to migrate: no stored format, wire field, table, pin or pack moves. A status a reader set on
an expression is read on the match's key as before; where the rule now shows another expression, the
reader sees that expression's status. Rollback is a revert.

## Effort

1–2 ideal days: `is_preposition`, the hyphen join and G1 in the French check, 0.25–0.5; the rule, its
two guards and the shared lookup in `match_expressions`, with a test per scenario on inline packs
holding expressions, names and readings in the three languages, 0.5–0.9; the probes, six goldens and
two snapshots re-blessed, 0.15–0.3; spec, doc comments and programme 0.1–0.3.

## Open Questions

**Settled by the owner on 2026-10-10 (in session):** all three languages in this change, es-fr's and
en-fr's output moving (tasks 0.2); the rule widened to any last word now, with a guard holding a
French headword's hyphen to the page; compound prepositions keep their « à »; the five French probes
approved (4.1).

For the owner, none blocking, each with the design's recommendation:
1. **G1 as the owner worded it moves 53 selections on its own (D3).** It removes `est-il` from « À
   l'Est, il » and `puis-je` from « et puis je lui ai dit » (38 better), but « Est ce que vous… »
   written without its hyphen loses `est-ce que` (15 worse, 4 of them in fr-es). The narrower guard —
   the hyphen checked only where an expression would take a word — removes the same 23 `est-ce que`
   on « c'est ce que » and moves nothing else. Recommended: keep the owner's; the narrower one is a
   one-line change in 1.4 if he prefers it.
