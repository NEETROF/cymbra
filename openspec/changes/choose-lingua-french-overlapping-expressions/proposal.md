# choose-lingua-french-overlapping-expressions — a verb's expression gives its preposition to a longer one

## Why

Row 44f of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes. The phrase gloss finds a selection's expressions from left to right: from
each token, the longest run that is a key wins, and the next run starts past it (`add-lingua-expression-table`'s
D3, change 44). When two expressions overlap, the one that starts first always
wins, however short. Change 44d (`match-lingua-french-contracted-pieces`, merged) measured the case
the owner then made a change of its own on 2026-10-10 (its open question 4): in « Elles sont **au**
nombre de sept », `être à` (« to pertain to; to belong to ») takes the `à` of « au », so the card
shows it, and `nombre de` (« quite a few »), where `au nombre de` (« numbering; among ») fits —
91 such places in fr-en, 33 in fr-es on its corpora.

Measured over change 44c's 51,790 French selections (1,085,056 tokens: change 39's corpus, UD
French-GSD, four Gutenberg novels, French Wikipedia, Tatoeba), on `main` 9c88442a:
- **The « au » case is two cases.** Of the 92 fr-en selections where an expression starting on a
  « au » loses to one ending on its `à`, 47 lose to an expression built on a verb — `être à` on
  « est au courant », « sont au nombre de », « était au bord de », `se mettre à` on « se mit au lit »
  —, and the other expression is the better reading 44 times, the worse once; 43 lose to a compound
  preposition — `jusqu'à` on « jusqu'au début » 33 times, `par rapport à`, `grâce à`, `quant à` —
  whose `à` it is: « jusqu'au début » means *until the beginning*, not *at the beginning* (and 2 to
  `en venir à`, design D2). fr-es: 31 of 32 are built on a verb (`être à`, `avoir à` on « avait au
  moins »).
- **The same holds where the preposition is written apart**: `être à` on « est à la fois » (16 in
  fr-en), « est à l'origine » (31), « sont à la hauteur », `tenir à` on « tenait à la main », `venir
  de` on « viendra de temps en temps ».
- **No simple order is right.** Nine rules measured, each over every selection and judged on 30
  sampled changes ([review.md](review.md), design D4), among them: the longer match wins — 490 selections move in fr-en, 20 better, 7
  worse (`à la fin` over `jusqu'à` on « jusqu'à la fin »); the most words covered — 13 better, 14
  worse; the expression covering the « au » whole — 21 better, 9 worse (every `jusqu'à`).

What separates the right reading from the wrong one is what the first expression is: a verb's
expression ends on the preposition its verb takes, which the next words may make part of a
longer expression; a compound preposition owns its last word.

## What Changes

- **A French expression built on a verb gives up the preposition it ends on to the expression that
  starts on it, when that expression is longer, or when the preposition is the `à` of a written
  « au » or « aux »** (design D1): « Elles sont au nombre de sept » answers `au nombre de`, « Il est
  à la fois sévère » `à la fois`, « Elle se mit au lit » `se mettre` and `au lit`. The giving
  expression is then tried shorter, as a run that does not meet its headword is (changes 44c, 44d).
  A compound preposition keeps its `à` (« jusqu'au début » `jusqu'à`), and so does a verb's
  expression when the one starting on its preposition is no longer (« Il venait de l'étranger »
  keeps `venir de`, not `de l'`).
- **Built on a verb** is read from the pack's grammar, on the studied side (design D2): a token
  before the expression's last that `tables/fr/grammar.tsv` reads as a verb or an auxiliary, with no
  preposition before it, so `à partir de` is not one. **A preposition** is one of French's closed
  classes (change 41). No pack, key, table, pin or analyser version moves.
- **Measured** (D1, D3): fr-en 173 selections move (190 matches lost, 181 gained), fr-es 83; judged
  whole by kind, fr-en 166 better, 6 neither, 1 worse (« le blanc l'est au noir » answering `au
  noir`), fr-es 83 better; a sample of 30 over both, 30 better. Gloss time +0.9 % (fr-en) and +0.5 %
  (fr-es).
- **What moves** (D5): five phrase probes added to the French golden; `fr-en.golden` and
  `fr-es.golden` gain 10 lines each and move none; `word-card-fr-en.txt` and `word-card-fr-es.txt`
  gain their cards, 16 lines each, their specs' count 38 → 43.
- **English and Spanish do not move** (D6): the rule runs for French alone, and their 533,058
  selections are unmoved. Measured on them, the same rule would move es-fr on 141 of 464,574 Spanish
  selections and en-fr on 61 of 68,484 English ones, better 25 of 30 sampled in each, worse 4 and 3
  (« Pese a que no » answering `a que no`): open question 1.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *A French expression built on a verb gives its preposition to a longer
  expression* (which of two overlapping French matches the phrase gloss keeps; English and Spanish
  unmoved).

No requirement is modified. The new one narrows, for French, the left-to-right choice that
*Expression lookup in a phrase gloss* (`add-lingua-expression-table`) and *French expressions are
found on French's reading of a selection* (change 44) make, beside *A French match never ends inside
a written word* (change 51), *A French expression's pieces meet the words as its headword writes
them* (change 44c) and *A French expression's contractions meet the words as its headword writes
them* (change 44d); those changes are open, so it says so rather than MODIFY them. It reads verbs
as *French's word grammar* (change 45) stores them, looked up as *A dictionary form is read as
itself* (change 41b) finds a lemma, and prepositions as *French closed classes* (change 41) lists
them. The changes whose requirements it builds on are in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: the choice in `engine::match_expressions`, and a preposition test
    beside `is_function_word` (`analysis/function_words.rs`); *consumed*: `Pack::readings`, French's
    closed classes, changes 44c's and 44d's checks, change 51's cover of the article.
  - `crates/lingua-wasm` — five phrase probes in the French scenario; `fr-en.golden` and
    `fr-es.golden` re-blessed by additions.
  - `apps/lingua-extension` — *consumed*, its code unchanged: the card shows what the phrase gloss
    reports. `word-card-fr-en.txt` and `word-card-fr-es.txt` gain the five probes' cards, their
    specs' count 38 → 43.

  ID, Music, Live, the back office, the site, the backend, the Apple host app, the agent plugin
  (which studies no French), every reducer, table and pack are untouched.
- **Release.** Silent: no listed pair studies French. It lands before change 52 lists fr-en, so no
  reader is shown `être à` on « au nombre de ».
- **Compatibility.** No stored format, wire field, table, pin, pack or version moves. A status or
  card a reader set on an expression is untouched (no reader studies French).
- **Effort, against no programme estimate (outside the 57)**: 0.5–1 ideal day (design, *Effort*).
