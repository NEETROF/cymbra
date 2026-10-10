# Design — choose-lingua-french-overlapping-expressions

## Context

See proposal.md (Why). How the phrase gloss chooses among a French selection's expressions today
(`origin/main` 9c88442a: changes 39 to 51b, 44c and 44d merged, French's analyser `1.2.0`, fr-en's
tables as 48b re-reduced them, fr-es's as 49b refined them):

| Seam | Today |
|---|---|
| `engine::match_expressions` | from each token, the longest run of 2 to 7 pieces that is a key (French's `du`/`des` retry, change 44's D5), kept only where `meets_as_written` finds its elisions and contractions as the headword writes them (changes 44c, 44d), then extended over a following token sharing its span (change 51's D12); **the next run starts past it**. A run never looks at the runs it overlaps: the one that starts first wins |
| `Pack::readings(lemma, form)` | a form's readings as the dictionary form the analysis read it as (`add-lingua-word-grammar`), from `tables/fr/grammar.tsv` (change 45), on the studied side, so fr-en and fr-es read the same: `sont` → `être` VERB, `tenait` → `tenir` VERB, `grâce` → NOUN. Read by the word card, never by the phrase gloss until now |
| `analysis::function_words` | French's closed classes (change 41's D3), `FR_PREPOSITIONS` among them: 34 lemmas, `à`, `de`, `en`, `sur`, `pour`, `par`, `jusque`, `sauf`…; `is_function_word` answers the union of the six tables. `du` and `des` are words of their own (M21), determiners, not prepositions |

**How it was measured.** A scratch copy of `origin/main` at 9c88442a, never committed, carrying
`match_expressions` behind a switch with every rule of D4, and change 44d's harness extended to the
six pairs: each pack built from the committed tables (`tables/<studied>`, `tables/<pair>`), every
line of the corpora glossed as one selection under each rule, the matches compared with today's.

| Corpus | Selections | Tokens |
|---|---|---|
| French: change 39's `baseline/pages-fr.txt` block by block, UD French-GSD train and dev (the two files fr-en's pin records), four Gutenberg novels by paragraph, French Wikipedia paragraphs, Tatoeba's French sentences (changes 42, 44c, 44d) | 51,790 | 1,085,056 |
| Spanish: Tatoeba's 442,407 Spanish sentences, 11,567 Spanish Wikipedia paragraphs, 5,341 UD Spanish sentences, 5,212 paragraphs of five Spanish-language Gutenberg books, change 3's `baseline/pages-es.txt` (changes 42b, 44d) | 464,574 | 4,160,681 |
| English: change 1's `baseline/pages.txt` block by block, UD English-PUD's 1,000 sentences, 10,000 quotations and 57,434 example sentences of the English Wiktionary's English entries | 68,484 | 1,614,394 |

Matches today: fr-en 60,359, fr-es 41,546, es-fr 205,743, es-en 236,765, en-fr 48,552, en-es
62,835. A rule's changes are judged on 30 of them drawn at random (seed 20261010) by one judge, as
the study's marks were (± 6 points on 30) — better, worse or neither —, all 30 per rule and
language in [review.md](review.md); and this design's rule is judged on every change, by kind.

**The overlaps today.** Where two expressions share a token, the first wins:
- **On the `à` of « au » or « aux »** (change 44d's open question 4): 92 fr-en selections and 32
  fr-es selections hold an expression starting on a « au » that one ending on its `à` keeps out.
  In fr-en, 47 of the 92 lose to an expression built on a verb — `être à` 42 times (« sont au nombre
  de », « est au courant », « était au bord de », « est au contraire »), `se mettre à` 5 (« se mit au
  lit ») —, and 43 to a compound preposition — `jusqu'à` 33 (« jusqu'au début », « jusqu'au milieu
  du »), `par rapport à` 4, `grâce à` 2, `sauf à` 2, `quant à`, `rapport à` —, and 2 to `en venir à`
  (« en venant au monde »). In fr-es, 31 of 32 lose to `être à` or `avoir à` (« avait au moins »), 1
  to `face à` (« face au vent »).
- **On a preposition written apart**: `être à` on « est à la fois » (16 in fr-en), « est à l'origine
  de » (31), « sont à la hauteur » (7), « sommes à la maison » (6); `tenir à` on « tenait à la main »;
  `venir de` on « viendra de temps en temps ».
- **On other words**: `c'est` on « c'est le cas » (`être le cas`, 21), `j'ai` on « j'ai besoin de »
  (`avoir besoin de`, 23), `se mettre` on « se met en place » (`mettre en place`); and the reverse,
  `c'est` and `ce que` on « c'est ce que » where `est-ce que` would read the yes-or-no question
  marker (23).

## Goals / Non-Goals

**Goals:**
- The owner's case: « Elles sont au nombre de sept » answers `au nombre de`, and every overlap like
  it, without breaking the overlaps today's order already gets right (« jusqu'au début », `jusqu'à`).
- A rule that reads what the core already has: the pack's grammar, French's closed classes, the
  tokens' spans. No pack, key, table, pin or analyser version moves.
- English and Spanish do not move. The golden shows the rule on fr-en's and fr-es's real tables.

**Non-Goals:**
- Spanish and English (D6, open question 1).
- Overlaps whose first expression does not end on a preposition — `c'est` on « c'est le cas », `j'ai`
  on « j'ai besoin de » (D4, rule G; open question 2).
- A verb's `de` written « du » or « des » (« venu du côté de l'orient », `venir de` matched by change
  44's retry, `du côté de` starting on « du »): `du` is a word of its own (M21), not a preposition, so
  the expression keeps it, as today; 2 such places in fr-en.
- Overlaps of two tokens or more, and of a first expression ending on anything but a preposition:
  they keep today's order (rules A and B in D4 reach them, and do worse).
- Page highlighting: `analyse_page` is untouched, as in changes 44, 44c and 44d.

## Decisions

### D1 — A verb's expression gives its preposition to a longer one

A French match **gives up its last token to the expression that starts on it** when:
1. **it is built on a verb** (D2): one of its tokens before the last is a form the pack's grammar
   reads as a verb or an auxiliary, with no preposition before it;
2. **its last token is a preposition** of French's closed classes, or the `à` of a written « au » or
   « aux » (the token after the match shares its span);
3. **the expression that starts on that token** — the longest run that is a key and meets its
   headword there, as the scan would find it — **holds more tokens than the match, or starts on the
   `à` of « au » or « aux »**, which it then covers whole (change 44d: a run starting on that `à`
   meets only a headword writing « au »).

The match then does not match at that length: the shorter runs are tried, else the scan moves to the
next token, exactly as for a run that does not meet its headword (changes 44c, 44d). The expression
starting on the token is found there in turn. Anything else keeps today's order.

| Page | First expression | Starting on its last token | Today | This design |
|---|---|---|---|---|
| « Elles sont au nombre de sept » | `être à`, on the `à` of « au » | `au nombre de` | `être à`, `nombre de` | `au nombre de` |
| « Il est au courant de tout » | `être à`, on the `à` of « au » | `au courant` | `être à` | `au courant` |
| « Elle se mit au lit » | `se mettre à`, on the `à` of « au » | `au lit` (3 tokens, as many) | `se mettre à` | `se mettre`, `au lit` (the first tried shorter) |
| « Il est à la fois sévère » | `être à` | `à la fois` (3 > 2) | `être à` | `à la fois` |
| « Elle tenait à la main une rose » | `tenir à` | `à la main` | `tenir à` | `à la main` |
| « elle viendra de temps en temps » | `venir de` | `de temps en temps` | `venir de` | `de temps en temps` |
| « Il venait de l'étranger » | `venir de` | `de l'` (2, not more) | `venir de` | the same |
| « Il resta jusqu'au début de la nuit » | `jusqu'à`, no verb | `au début` | `jusqu'à` | the same |
| « grâce au travail du monteur » | `grâce à`, no verb | `au travail` | `grâce à` | the same |
| « Ce livre est au voisin » | `être à` | none | `être à` | the same |

Why these three conditions. A verb's expression ends on the preposition its verb takes (`être à`
« to belong to », `tenir à` « to be attached to », `avoir à` « to have to »), and the words after it
often make that preposition the head of a longer expression of their own: « être au courant » is
`au courant` after `être`, not `être à` before « le courant ». A compound preposition (`jusqu'à`,
`grâce à`, `par rapport à`) owns its last word: « jusqu'au début » is *until the beginning*, and
`au début` (« at first ») would misread it. The length condition keeps a verb's preposition from an
expression no longer than the verb's (`venir de` keeps « de » from `de l'`); at a « au » the
expression starting on it takes a written word the first only half covers, so it wins at any length
(`au lit` over `se mettre à`).

**Measured** (selections are counted once per pair; the moves split between the two branches of
condition 3):

| Pair | Matches today | Lost | Gained | After | Selections moved | At a « au »/« aux » | Written apart |
|---|---|---|---|---|---|---|---|
| fr-en | 60,359 | 190 | 181 | 60,350 | 173 | 47 | 126 |
| fr-es | 41,546 | 85 | 84 | 41,545 | 83 | 31 | 52 |

By corpus, fr-en moves 0 selections in change 39's corpus, 48 in GSD, 33 in Gutenberg, 46 in
Wikipedia and 46 in Tatoeba; fr-es 0, 18, 22, 20 and 23. Of change 44d's 92 fr-en selections at a
« au », 47 move (every one built on a verb but `en venir à`, D2) and 45 keep today's reading; of
fr-es's 32, 31 move and `face à` on « face au vent » keeps it.

**Judged on every change**, by kind (72 kinds in fr-en, 33 in fr-es, each read on its selections):
- fr-en: **166 better, 6 neither, 1 worse.** The worse: « le blanc l'est au noir » (as white is to
  black) now answers `au noir` (« off the books »). Neither: « le soleil est au plus haut » (`au
  plus` « at most », `être à` both wrong), « nous sommes à moins de la moitié » and « à moins d'un
  an » (`à moins de` « unless »), « se tient à la maison pour tous », « ils se mirent au travail »
  (`se mettre à` « to start » and `au travail` « get to work! » both fit), « on fait avec les moyens
  du bord » (both fit). The commonest: `être à` → `à l'origine` 31 (« fut à l'origine du projet »),
  → `à la fois` 16, → `au nombre de` 8, → `à la hauteur` 7, → `au courant` 6, → `à la maison` 6.
- fr-es: **83 better.** `être à` → `à la fois` 17, → `à peu près` 7, → `au courant` 6, → `à cause de`
  5; `avoir à` → `au moins` 6; `être pour` → `pour la plupart` 4 (« furent pour la plupart
  condamnés »).
- The sample of 30 over both pairs' 199 distinct selections: 30 better ([review.md](review.md), F).

### D2 — « Built on a verb » from the pack's grammar; « a preposition » from French's closed classes

The pack carries no part of speech for an expression (`mwe.tsv` is a headword and a gloss); adding
one would move both packs and their pins. It carries the readings of every form on the studied side
(change 45), which the word card already reads: `Pack::readings(lemma, form)` on a token's lemma
and its lowercased surface answers whether the form is a verb as the analysis read it. One form,
one lemma (M8): « été » read as `être` is a verb; « été » the noun, read as `été`, is not.

Three readings of « built on a verb », measured:

| Reading | fr-en moved | fr-es moved | Differs |
|---|---|---|---|
| a verb among the tokens before the last | 175 | 83 | `en venir à` on « en venant au monde » (2): `venir au monde` (« to be born ») wins, better; in Spanish `a partir de` on « a partir del cual » and `a pesar de` on « a pesar del tiempo » give « del » to `del cual`, `del tiempo`, worse |
| **a verb before the last, no preposition before it (this design)** | 173 | 83 | `à partir de`, `a pesar de` are prepositions built on an infinitive, not a verb's expression |
| the first token a verb | 168 | 83 | loses `se mettre à` (« se mit au lit », a pronoun first) |

The second reads what the words are — a verb's expression is led by its verb, a pronoun before it
at most —, and it is the reading D6 measures Spanish and English with. It costs fr-en the 2 `en venir à`
moves: `en` is in French's prepositions (as the pronoun « en » is in its pronouns), and « en venant
au monde » keeps `en venir à`, wrong today and still.

**A preposition** is a lemma of French's closed classes' prepositions (`FR_PREPOSITIONS`, change
41), exposed by `is_preposition(lemma, studied)` beside `is_function_word`, which each language
answers from its own table. `du` and `des` are not in it (Non-goals).

A pack without a grammar section reads no verb, so no match gives way: today's order. Every French
pack has one (change 45).

### D3 — Where the matcher asks, and what it costs

In `match_expressions`' scan, at a start, the runs are tried longest first as today. A French run
that is a key and meets its headword is checked, cheapest first: its last token is a preposition or
the `à` of « au »/« aux » (a lemma and a span flag); then the expression starting on that token is
looked up the way the scan looks one up (the longest run that is a key and meets its headword,
without asking whether that one would in turn give way); then condition 3; then whether the run is
built on a verb (its tokens' readings). Only runs ending on a preposition pay the lookup.

Measured, the median of 15 interleaved passes over the 51,790 selections, natively, on a machine
shared with other sessions: fr-en 2.583 → 2.606 s (**+0.9 %**), fr-es 2.627 → 2.639 s (**+0.5 %**).
Rule A (D4), which looks at every run of every start, costs fr-en +4.4 %.

The giving match is treated as a run that does not match at that length, so change 51's D12 never
covers an article for it, change 44d's D4 (a match starting on the article of « au ») is untouched,
and a match's key, name, gloss and status are what they are without the rule.

### D4 — The rules measured, and why not the others

Each rule ran over every selection; « moved » counts selections, per pair (fr-en / fr-es, then
es-fr / es-en and en-fr / en-es where the rule would be shared); the sample is 30 of the language's
changes, better / worse / neither ([review.md](review.md)).

| Rule | fr-en / fr-es moved | French sample | Spanish, shared: moved, sample | English, shared: moved, sample |
|---|---|---|---|---|
| A — the longer match wins (every run of every start, longest first, ties to the left) | 490 / 123 | 20 / 7 / 3 | 336 / 1,118; 17 / 6 / 7 | 113 / 258; 23 / 5 / 2 |
| B — the most tokens covered | 557 / 141 | 13 / 14 / 3 | 513 / 1,850; 11 / 18 / 1 | 163 / 294; 20 / 8 / 2 |
| C — the « au » whole: a match ending on its `à` gives way to any starting on it | 92 / 32 | 21 / 9 / 0 | 39 / 113 (« al », « del »); 17 / 12 / 1 | — |
| D — C, only to a match covering more written words | 38 / 11 | 17 / 13 / 0 | — | — |
| E — C, only when the giving match holds a verb | 49 / 31 | 28 / 0 / 2 | 20 / 66; 25 / 5 / 0 | — |
| **F — this design (D1)** | **173 / 83** | **30 / 0 / 0** | 141 / 224; 25 / 4 / 1 | 61 / 106; 25 / 3 / 2 |
| G — F without « ends on a preposition »: a verb's expression gives any last token to a longer one | 323 / 98 | 26 / 4 / 0 | — | — |
| H — any match holding a verb gives its last token to one reaching further | 1,009 / 397 | 15 / 14 / 1 | — | — |
| I — any match gives its last token to one reaching further | 2,521 / 1,262 | 7 / 21 / 2 | — | — |

*Rejected — A, the longer wins.* Right where a verb's expression is first, wrong where a compound
preposition is: `à la fin` over `jusqu'à` on « jusqu'à la fin » (19 in fr-en), `au début` on
« jusqu'au début » (13), `au temps de` on « par rapport au temps », `est-ce que` on « c'est ce que »
(22); +4.4 % gloss time.

*Rejected — B, the most tokens.* Splits a right expression to cover one more word: « en vue de la »
answers `en vue` and `de la`, « à côté de la chapelle » `à côté` and `de la`.

*Rejected — C and D, the « au » whole.* The owner's suggestion read literally: right on every verb's
expression, wrong on every compound preposition (`jusqu'à` 33 times); D's count of written words
trades the one for the other (« jusqu'au milieu du » `au milieu de`, « est au courant » kept wrong).
A rule « pieces written as on the page » is C again: at a « au » it is the expression ending on its
`à` whose piece the page does not write alone.

*Kept as F's first branch — E, a verb's « au ».* F extends it to prepositions written apart, where
the same verbs make the same mistake (`être à` on « est à la fois »), with the length condition
(D1).

*Not now — G, a verb's expression gives any last token.* Judged on all 323 fr-en changes by kind:
276 better, 16 neither, 31 worse — 23 of them `est-ce que` on « c'est ce que », where `c'est` gives
`est` to an expression the page does not write: « est-ce » is hyphenated, and the matcher does not
hold a hyphen to the headword yet. Its 150 changes beyond F's are 110 better, 30 worse and 10
neither; the better (`avoir besoin de` on « j'ai besoin de », `être le cas` on « c'est le cas »,
`mettre en place` on « se met en place », `faire la connaissance` on « de faire la connaissance »)
are mostly a right reading replacing a right but vaguer one (open question 2).

*Rejected — H and I.* A last token is given to any later expression, however short: `pas que` on « je
ne pense pas que », `de l'` on « au sein de l'Union », `de la` on « lors de la ».

### D5 — What moves: five probes, added

The French golden shows the rule only where a probe holds it; none of its phrase probes does.
Measured on the prototype with the rule and no probe added: `fr-en.golden` and `fr-es.golden` are
byte for byte, as are `en-fr.golden`, `es-fr.golden`, `es-en.golden`, `en-es.golden` and the
`cross_native`, `parity`, `card_gloss_language`, `spanish_expression_keys`, `soft_hyphens`,
`languages`, `deck`, `statuses` and `reprofile` suites (98 tests in 15 suites), and every
`lingua-core` test passes. So the French scenario (`tests/support/french.rs`) gains five phrase
probes, after change 44d's:

| Probe | fr-en today | fr-en with the rule | fr-es today | fr-es with the rule |
|---|---|---|---|---|
| « Elles sont au nombre de sept. » | `être à` over `sont`, `à`, `le`; `nombre de` | **`au nombre de`** over `à`, `le`, `nombre`, `de` | `être à` | the same: fr-es holds no `au nombre de` |
| « Il est à la fois sévère et amical. » | `être à` over `est`, `à` | **`à la fois`** over `à`, `la`, `fois` | `être à` | **`à la fois`** |
| « Il est au courant de tout. » | `être à` over `est`, `à`, `le` | **`au courant`** over `à`, `le`, `courant` | `être à` | **`au courant`** |
| « Elle se mit au lit. » | `se mettre à` over `se`, `mit`, `à`, `le` | **`se mettre`** over `se`, `mit`; **`au lit`** over `à`, `le`, `lit` | `se mettre` | the same |
| « grâce au travail du monteur » | `grâce à` over `grâce`, `à`, `le` | the same (no verb) | `grâce à` | the same |

Re-blessed on the prototype: `fr-en.golden` 805 → 815 lines and `fr-es.golden` 815 → 825, 10
lines added each and none removed. `word-card-fr-en.txt` and `word-card-fr-es.txt`
(`apps/lingua-extension/test/baseline/`), which render the golden's phrase probes, gain the five
probes' cards, 16 lines each, every block before byte for byte — in fr-en « au nombre de —
numbering » and « sept — seven »; « à la fois — at a time, at the same time, at once », « sévère —
severe, harsh », « amical — friendly, amicable »; « au courant — up to date, abreast »; « se mettre
— to place oneself in a place » and « au lit — in bed »; « grâce à — thanks to », « travail — work,
labor », « monteur — editor (film, TV, etc.) »; in fr-es « être à — Ser de, pertenecer… », « nombre
— Número », « sept — Siete »; « à la fois — A la vez… », « sévère — Severo », « amical — Amistoso »;
« au courant — Al corriente »; « se mettre — Ponerse », « lit — Cama »; « grâce à — Gracias a »,
« travail — Trabajo ». Their specs count 43 phrase probes (42 phrases and the reader's). No
`analyse`, `gloss`, `word-grammar` or review probe, no pack, pin, table, `pack_version` or fixture
moves.

**French's analyser version stays**, as changes 44 (its D7), 51 (its D12), 44c (its D5) and 44d (its
D5) read *An analyser version per studied language* for the phrase gloss's matching: no token,
lemma or page analysis moves and no pack byte does.

**The baseline's reason.** *A French invariance baseline runs beside the English and Spanish ones*
(changes 39, 40, 41, open) names three reasons for moving the golden, none of them probes a change
adds; changes 44, 51, 44c and 44d added theirs and said so in their pull requests, and so does this
one. The test's doc comment (`french_baseline.rs`) lists this change among those that add probes.

### D6 — English and Spanish cannot move; the same rule measured on them

- The check runs for a French run alone. English and Spanish runs go through `match_expressions` as
  before; `is_preposition` is a new function no English or Spanish path calls.
- Measured with the rule on, the 464,574 Spanish selections keep es-fr's 205,743 and es-en's 236,765
  matches, and the 68,484 English selections en-fr's 48,552 and en-es's 62,835.
- **Would Spanish and English want it?** Run on them (F shared, Spanish's prepositions and « al »,
  « del », English's prepositions and particles): es-fr would move 141 selections and es-en 224, en-fr
  61 and en-es 106. Sampled: Spanish 25 better, 4 worse, 1 neither — `ser de` on « es del todo », « son
  de lo más », `hacer de` on « lo hizo de todos modos », `ir a` on « van al final del álbum » give way;
  but `pese a` and `debido a`, prepositions made from verb forms, give « a » to `a que no` (« I bet
  you won't ») in « Pese a que no tengo dinero », and `ir a` gives « al » to `al día` (« up to date »)
  in « iba al día siguiente »; English 25 better, 3 worse — `go over` on « went over the top » gives
  way to `over the top`, but `take off` on « took off the record » to `off the record`. The goldens do
  not show it (no probe holds such an overlap; measured, they pass with the shared rule on), so es-fr's
  and en-fr's shipped output would move with nothing pinning it. Left out, es-fr's output not moving
  (the programme's rule): open question 1.

### D7 — What later changes take from here

| Change | Takes |
|---|---|
| 52 `enable-lingua-french` | the dogfood: « Elles sont au nombre de sept », « Il est à la fois sévère », « jusqu'au début » on a real page in fr-en |
| any change adding French phrase probes in flight (row 44e) | the five probes in its re-bless of `fr-en.golden`, `fr-es.golden` and the two snapshots, whichever lands second |
| a Spanish counterpart, if the owner wants one (open question 1) | D1's rule and D2's reading, with Spanish's verb-made prepositions (`pese a`, `debido a`) held apart |

## Risks / Trade-offs

- [A right reading given up] → 1 of 173 in fr-en (« le blanc l'est au noir »), none in fr-es.
- [A compound preposition read as a verb's expression] → one built on a verb form with no
  preposition before it and ending on a preposition gives its last word away: `quitte à` (« quitte »
  a form of `quitter`) once on the corpus, rightly (« il en était quitte à bon compte » answers `à bon
  compte`). French's others start on a preposition or a noun (`à partir de`, `grâce à`, `face à`).
- [A form the grammar does not read] → no verb is found and today's order is kept: the rule can only
  miss, never invent.
- [fr-es's table] → where fr-es holds no longer expression (`au nombre de`), `être à` stays (« Ser de,
  pertenecer »): the rule cannot show what the table does not hold.
- [Chains] → the expression a match gives way to is not itself asked whether it would give way; the
  scan then finds it, or a shorter one, in its turn. Measured in D1's figures.
- [Cost] → +0.9 % (fr-en) and +0.5 % (fr-es) of the corpus's gloss time.

## Migration Plan

Nothing to migrate: no stored format, wire field, table, pin or pack moves, and no reader studies
French. Rollback is a revert.

## Effort

0.5–1 ideal day: `is_preposition` and the check in `match_expressions`, with a test per scenario on
an inline French pack holding expressions, names and readings, 0.3–0.5; the five probes, the two
goldens and the two snapshots re-blessed 0.1–0.2; spec, doc comments and programme 0.1.

## Open Questions

For the owner, none blocking — each with the design's recommendation:
1. **Spanish and English (D6).** The same mistake exists there: « Lo hizo de todos modos » shows
   `hacer de` (« to act as ») where `de todos modos` (« anyway ») fits, « es del todo cierto » `ser de`
   where `del todo` (« entirely ») fits. The rule fixes 25 of 30 sampled Spanish changes and 25 of 30
   English ones, but gets 4 and 3 wrong: « Pese a que no tengo dinero » would show `a que no` (« I bet
   you won't ») instead of `pese a` (« despite »). It would change what es-fr and en-fr readers see
   today, on 141 of 464,574 Spanish selections and 61 of 68,484 English ones. Recommended: not in this
   change; a Spanish change of its own if you want it, which holds `pese a` and `debido a` apart.
2. **The wider rule (D4, G).** Letting a verb's expression give any last word, not only a
   preposition, would also show `avoir besoin de` (« to need ») for « j'ai besoin de » instead of
   `j'ai` (« I have »), and `être le cas` for « c'est le cas » — 150 more places in fr-en, 110 better,
   30 worse — but « C'est ce que tu veux » would show `est-ce que` (the yes-or-no question marker),
   23 of those 30, because the matcher does not yet check the hyphen of « est-ce ». Recommended: not
   now; first a change that holds a hyphen to the headword, as changes 44c and 44d did for the
   apostrophe and « au », then measure this again.
3. **Compound prepositions keep their « à » (D1).** « Il resta jusqu'au début de la nuit » keeps
   `jusqu'à` (« until ») and never shows `au début` (« at first »); likewise `grâce à` on « grâce au
   travail », `par rapport à` on « par rapport au nombre » — 43 places in fr-en. Recommended: keep;
   the « à » is theirs.
