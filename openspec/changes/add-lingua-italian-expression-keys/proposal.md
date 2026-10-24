# add-lingua-italian-expression-keys — Italian expressions keyed as Italian is read, named as the page writes them

## Why

Change 12 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. French's
expressions are keyed through the core's own French reading and named by their headword (matrix
change 44), their elided and contracted pieces meet the page as the headword writes them (44c, 44d),
and a key several headwords reach is named by the one written most like the page (44e); Spanish
followed (44b, 44g). Italian has the first half by accident and none of the rest: since change 5
the builder keys every language but English through its reading, so an Italian headword is already
tokenised by Italian's pre-pass and each token resolved as a page's — but each piece is its lemma
(English's arm), the window is English's five tokens, the pack carries no name, a key several
headwords reach keeps one, and the matcher reads an Italian run as English's: no article of an
articulated preposition covered, no piece held to the headword's writing, no choice among headwords.

Measured on the English Wiktionary's Italian section (the file change 9 reduces), the headwords
change 16's reducer would offer — `reduce_common.reduce_expressions`'s filters, an Italian token
pattern —, read by a copy of change 6's pre-pass: **8,852** candidates with a space, and **77** words
without one that the pre-pass splits into two tokens or more (`d'accordo`, `all'improvviso`,
`senz'altro`, `dov'è`, `un'altra`), expressions by change 44's rule. Of the 8,852:
- **553 hold an articulated preposition the pre-pass splits** (`al dente`, `alla romana`, `allo
  spiedo`, `col legno`, `penne all'arrabbiata`), read as `a` + `il` …, which a page reads the same
  way: they meet already, but the key writes `a il dente`, and the article is a lemma — `alla` and
  `al` would share a key where M8 files `la` under `il`;
- **449 hold the `di` family**, whole lemmas (`amico del cuore`, `livello del mare`): they meet;
- **318 hold an elision** (`acino d'uva`, `fare l'amore`, `sala d'aspetto`), read as `di` + `uva`:
  they meet, but `fare l'amore` and a page's « fare lo amore » (never written) would too, and
  « d'un » meets `di un` — 44c's defects;
- **744 hold an article**, 17 a possessive, 19 a demonstrative: the words 44's D2 writes as the
  pre-pass gives them;
- **by tokens**: 2: 4,491 (50.7 %), 3: 3,175 (86.6 %), 4: 695 (94.5 %), 5: 247 (97.2 %), 6: 145
  (98.9 %), 7: 58 (99.5 %), 8 and more: 41 (proverbs: « non dire gatto se non ce l'hai nel sacco »).
  English's window of five leaves 244 keyed headwords unreachable; seven leaves 41;
- **105 end on `di`** (`a causa di`, `per mezzo di`, `al posto di`, `da parte di`), 31 on `a`, 7 on
  `da`, 7 on `su`, 5 on `con`, 5 on `per`: the `di` family stays whole (I7), so « a causa del
  ritardo » never meets `a causa di` — French's `du`/`des` case (44's D5); the `a`, `da`, `su`, `con`
  ones meet « al », « dal », « sul », « col » by the split, their article covered or not (44b's D5).

The card is the key: an Italian match today reports `a il dente`, `fare lo amore`, `tenere in
contare`-like lemma chains as the expression's dictionary form, which the reader would review.

## What Changes

- **An Italian key writes its determiners as the pre-pass gives them** (design D2): the articles
  `il`, `lo`, `la`, `i`, `gli`, `le`, `un`, `uno`, `una`, the possessive and the demonstrative
  determiners, 41 words in `ITALIAN_KEY_WRITTEN`, every other token its dictionary form: `al dente`
  is `a il dente`, `alla romana` `a la romano`, `fare l'amore` `fare lo amore` — never `a il romano`.
- **Italian's window is seven tokens** (D3), holding 99.5 % of its headwords; a longer key is left
  out at build. The headwords that are expressions are 44's D6: two to seven tokens, every word a
  token — the 77 single words the pre-pass splits included, once change 16's reducer offers them.
- **An Italian expression is named by its headword** (D4): the pack carries the names section and
  44e's `expr.others`, the match reports the name and the status is read on it; the order among
  headwords of one key is French's; the match is named by the headword written most like the page.
- **An Italian match never ends inside a written word** (D5): the article of « al », « dal »,
  « nel », « sul », « col » a match ends on is covered, as Spanish's (44b's D5) and French's (51's
  D12); « a causa del » answers `a causa di` by the `di` family read once more as `di` when no key
  matches the run — French's `du`/`des` retry (44's D5) for `del`, `dello`, `della`, `dei`, `degli`,
  `delle` —, where that `di` closes a governing frame: the key's first piece a preposition, or the
  run built on a verb (the owner's decision at implementation, D5).
- **Italian pieces meet the page as the headword writes them** (D6): an elided piece meets only an
  elided word and a piece written in full only a word in full, the last piece in full meeting either
  (44c); two pieces the headword writes as one articulated preposition meet only a written one, and
  two it writes apart only words apart, the run's edges the page's (44d); the headword is read by
  Italian's pre-pass from its name, which the pack carries wherever it differs from the key. A
  truncation (`po'`) reads as an elided piece by its span, and a piece written in full meets a
  truncation that stands for another word (`fare` « fa' ») but never the one that stands for the
  piece itself, so « un po' » is `un po'` and « un poco » is `un poco`, each named as written —
  measured, the dictionary writing both (amended at review, D6).
- **The overlap rule reads Italian too** (D7): change 44f's rule — an expression built on a verb
  gives its last word to a longer one — runs for Italian as for Spanish once change 13's grammar
  says what a verb is, its guard on « que » reading « che ».
- **Review finds an Italian expression card by its name** and under its own headword (D8).
- **What moves** (D9): the it-fr fixture gains expressions and forms, its `pack_version`; the
  Italian golden re-blessed on the pack line and its phrase probes, with probes added; Italian's
  analyser version does not move (the keys are pack data, the matching is the engine's at reading
  time, as 44's D7); English, Spanish, French and the six goldens do not move.
- **Not here**: which headwords and senses it-fr offers, and their glosses — change 16 (as 48 for
  French), with the pointer senses left out; a one-word selection over several pieces opening the
  whole-selection card, where `d'accordo` answers — change 18 (as 51).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *An Italian pack keys its expressions as Italian is read* (the
  reading, the determiners, the window, the names, every headword kept, which headwords are
  expressions).
- `lingua-analysis`: ADDED — *Italian expressions are found on Italian's reading of a selection*
  (the window, the article covered, the `di` retry, the pieces as written, the name written as the
  page, the overlap rule, the name reported and the status read on it).
- `lingua-decks-review`: ADDED — *Review finds an Italian expression card by its name*.

For Italian, the ADDED requirements take the place of the key and match rules of
`add-lingua-expression-table`'s *Multi-word expression table* and *Expression lookup in a phrase
gloss*, and of the sentences of changes 44, 44b, 44c, 44d, 44e, 44g, 44f and 51's requirements that keep
every other language as it was; those changes are open, so each requirement here says so rather
than MODIFY them (44b's precedent), and they are in `archiveAfter` with the Italian changes 5, 6
and 7. Of those sentences only the base table's rules and 44f's « in French, Spanish or English »
bind Italian today — the others name English, Spanish or French —, so the displacement is stated
for the archive's reading, not because a rule moves. Change 16, once proposed, joins it: the reducer's expressions are its.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/engine.rs` — *new*: `ITALIAN_KEY_WRITTEN`, `ITALIAN_EXPRESSION_WINDOW`,
  Italian's arms of `expression_piece` and `expression_window`, `whole_words` for Italian, the `di`
  retry in `expression_for_run`, `meets_as_written` reading an Italian name through Italian's
  pre-pass (the contraction test per language), the `as_written()` gate of `ExpressionScan::run`
  for Italian, 44f's G2 guard widened with `whole_words()` and its word by language; *consumed*: `headword_reading`, Italian's pre-pass and cascade, `Pack::expression_name`,
  `expression_headwords`, `is_elided`, `shared_spans`.
- `crates/lingua-pack/src/lib.rs` — the names and `expr.others` sections written for an Italian pack,
  French's order among headwords for Italian; English, Spanish and French byte for byte.
- `crates/lingua-wasm/src/lib.rs` — `pack_gloss`'s Italian arm; `tests/{italian_baseline.rs,
  support/italian.rs}` and `baseline/it-fr.golden`, re-blessed once.
- `scripts/lingua-data/testdata/it-fr/` — expressions, forms, `pack_version`.
- `apps/lingua-extension` — *consumed*, unchanged: the card shows the key the phrase gloss hands it.
- Untouched: the six pairs' tables, pins and goldens; every reducer and committed table; the
  extension's bundle; the agent; the server; the site.
- **Effort, against 2–4 ideal days**: 2.5–3.5 (design, *Effort*).
