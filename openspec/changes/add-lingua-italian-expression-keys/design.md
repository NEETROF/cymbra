# Design

## Context

See proposal.md — Why. Where the seams stand on `main` after changes 5 to 11:

| Seam | Today |
|---|---|
| `crates/lingua-pack` `expression_key` | every language but English keys a headword through `reading_expression_key`; so an Italian headword is tokenised by Italian's pre-pass and each token resolved as a page's (change 5's seams). `expression_piece`'s Italian arm returns the lemma (English's arm), `expression_window` gives `EXPRESSION_WINDOW` (5). The names section and `expr.others` are written for French and Spanish (the `read` flag, an inline `matches!` for the others), and French's order among headwords of one key applies to them alone |
| `crates/lingua-core` `engine::match_expressions` | `whole_words()` is French and Spanish: the article of a contraction covered (`covered_end`), the as-written checks (`meets_as_written`, French only, reading the name through French's pre-pass when it holds an apostrophe or « au »/« aux » — `holds_french_contraction`), the choice among headwords (`written_as_the_page`, French and Spanish, Spanish's clitic guard). `expression_for_run`'s retry of `du`/`des` as `de` is French's. 44f's `gives_last_word` runs in the three languages, on `built_on_verb` (the pack's grammar) and `is_preposition` |
| `PageJoins` | `shares_span` (the second half of a split word), `elided` (`is_elided`: a span ending on an apostrophe), `hyphened`, computed for every language |
| Italian's pre-pass (change 6) | an elided piece is a token read as its word with a span holding the apostrophe (`l'` → `lo`, `d'` → `di`, `un'` → `una`, `dell'` → `dello`); `al`, `dal`, `nel`, `sul`, `col` and their forms split into preposition + article sharing the span, `all'`/`dall'`/`nell'`/`sull'` too; the `di` family whole; the truncations `po'`, `mo'`, `fa'`, `va'`, `da'`, `sta'`, `di'`, `be'` read as their words, the apostrophe inside the span |
| Italian's tables (change 9) | articles, pronouns and the `di` family as plain forms and lemmas of their own; `dai` a form of *dare* |
| `crates/lingua-wasm` `readable_gloss` | a French or Spanish expression card is read at the key its name reads as, and under its own headword; an Italian card at its lemma alone |
| The it-fr fixture (change 5) | a handful of expressions in `mwe.tsv`; the golden's phrase probes include « C'è un po' di pane », « all'alba », « Nel mezzo del cammin » (change 6) |

**The measurement** (proposal, Why): a scratch script over `kaikki-Italian.jsonl` with
`reduce_expressions`'s filters (a headword with a space, each word on an Italian token pattern,
no proper noun, a sense that is no form-of) and a Python copy of change 6's pre-pass written for
the count alone — 8,852 candidates, their pieces and their lengths. The figures are re-measured on
the implementation's base with the real builder over change 16's `mwe.tsv`, as 44's were after 41
and 43.

## Goals / Non-Goals

**Goals:**
- An Italian expression's key is what the analysis makes of a page holding it, as French's and
  Spanish's are; the card, the deck and the row name it as the dictionary writes it.
- An Italian match covers the written words it stands on, meets the page as its headword writes
  it, and is named by the headword written most like the page.
- English, Spanish and French do not move.

**Non-Goals:**
- Which headwords and senses it-fr offers, their glosses, the pointer senses left out (change 16).
- A one-word selection over several pieces opening the whole-selection card (change 18).
- A settled-chain rule (44b's D6): no Italian pack has shipped, no reader holds a status.
- Discontinuous expressions; expression-aware page highlighting.

## Decisions

### D1 — The key is the core's Italian reading, as it already is; the pieces, the window and the names follow

`reading_expression_key` already keys Italian through `headword_reading`; this change gives
`expression_piece` and `expression_window` their Italian arms (D2, D3), writes the names and
`expr.others` for an Italian pack (the `read` flag and the inline `others` test become French,
Spanish and Italian), and
applies French's order among headwords of one key (D4). The builder's English arm stays byte for
byte, and a French or Spanish pack is built as before, measured: the six committed packs at their
pins.

### D2 — An Italian key writes its determiners as the pre-pass gives them

`ITALIAN_KEY_WRITTEN`, sorted, 41 words: the articles `il`, `lo`, `la`, `i`, `gli`, `le`, `un`,
`uno`, `una` (the pre-pass writes `lo` for `l'`, `una` for `un'`, the article of a split
preposition in full); the possessives `mio`, `mia`, `miei`, `mie`, `tuo`, `tua`, `tuoi`, `tue`,
`suo`, `sua`, `suoi`, `sue`, `nostro`, `nostra`, `nostri`, `nostre`, `vostro`, `vostra`, `vostri`,
`vostre`, `loro`; the demonstratives `questo`, `questa`, `questi`, `queste`, `quello`, `quella`,
`quelli`, `quelle`, `quel`, `quei`, `quegli` (the pre-pass writes `questo`, `quello` for `quest'`,
`quell'`); `codesto` and its forms, archaic and Tuscan, none among the 19 measured, are left out. French's and Spanish's D2, transposed: the list holds what M8 may file under another
word and the forms that are their own lemma. Measured on the candidates: 744 headwords hold an
article, 17 a possessive, 19 a demonstrative; with every token lemmatised, `alla romana` and a
headword `al romano` would share a key wherever the tables file `la` under `il`, and `a la`
(« à la ») would answer every « al ». The `di` family are lemmas of their own, unaffected. The
clitic pronouns (`mi`, `ti`, `ci`, `lo`, `la`, `ne`…) are left out, as Spanish's: pronouns, not
determiners, and the `la`, `lo`, `le` they share with the articles are written already.

### D3 — Italian's window is seven tokens; which headwords are expressions

Of the 8,852 candidates, by tokens: 2: 4,491, 3: 3,175, 4: 695, 5: 247, 6: 145, 7: 58, 8: 23,
9 and more: 18 — seven holds 99.5 %, as French's 98.8 % and es-fr's 99.3 %; the 41 longer are
proverbs (« non vendere la pelle dell'orso prima di averlo ucciso »), left out at build, as the
lexicon test leaves out a key no reading can produce. A headword is an expression when Italian reads
it as two to seven tokens, every word giving a token (44's D6): the 77 words without a space the
pre-pass splits (`d'accordo`, `all'improvviso`, `senz'altro`, `mezz'ora`, `dov'è`, `sott'olio`)
are expressions once change 16's reducer offers them beside the headwords with a space; a compound
read as one token (`perché`, `pomodoro`, a hyphenated word — 10 among the candidates) is not.

### D4 — An Italian expression is named by its headword, every headword kept, the closest named

44's D3 and 44e's D1 and D2 for Italian: the pack carries each key's winning headword wherever it
differs from the key (`expr.names.zst`) and every other headword of the key as an expression of its
own (`expr.others`); the phrase gloss reports the name, the status is read on it, and where several
headwords reach the run's key the one written as the run's tokens at the most places wins, the
pack's order between equals. The order among headwords of one key is French's: the headword written
as the key, then the most tokens written as their key piece, then byte order. Measured at
implementation: how many keys several headwords reach, the names' bytes.

### D5 — An Italian match never ends inside a written word; the `di` family read once more as `di`

Two rules, one from each sibling:
- **Spanish's** (44b's D5, 51's D12): the `a` of « al », the `da` of « dal », the `in` of « nel »,
  the `su` of « sul », the `con` of « col » are tokens of their own, so an expression ending on
  them is found without a retry (`vicino a` in « vicino al mare »); the match then covers the
  article sharing the span, so a selection of the written words is answered whole. `whole_words()`
  becomes Spanish, French and Italian; `covered_end` is unchanged.
- **French's** (44's D5): the `di` family stays whole (I7), so the 105 keyed expressions ending on
  `di` (`a causa di`, `per mezzo di`, `al posto di`, `da parte di`) never meet « a causa del
  ritardo » or « al posto dell'altro » — `dell'` is read `dello`, one piece. In an Italian run no key
  matches whose last piece is `del`, `dello`, `della`, `dei`, `degli` or `delle`, that piece is read
  once more as `di`; the match covers it; the token keeps its lemma. The known misreading is a
  run whose last `del` belongs to the next phrase, not to an expression ending on `di`: « la prima
  del film », where `prima del` matches no key and the retry answers `prima di` (*before*) with
  `prima` the noun; measured on the corpus at implementation and named in the pull request.

### D6 — Italian pieces meet the page as the headword writes them

44c and 44d for Italian, on the same seams: `meets_as_written` reads the match's name through
Italian's pre-pass (`headword_reading`, the function the key was made with, so the pieces line up
one for one) when the name holds an apostrophe or a word of the split families — the contraction
test becomes per language, `holds_contraction(name, studied)`, French's « au »/« aux », Italian's
`al`, `allo`, `alla`, `ai`, `agli`, `alle`, `dal`…, `nel`…, `sul`…, `col`, `coi`, `all'`, `dall'`,
`nell'`, `sull'`, which the apostrophe test sends through the reading anyway —; each piece's elision from its span (`is_elided`, which change 6 widened to
Italian's apostrophes), its contractions from the reading's `shared_spans`. The rules are 44c's and
44d's — `ExpressionScan::run`'s two French gates, which choose the as-written arm over Spanish's
clitic-weighed arm and gate `meets_as_written`, become an `as_written()` predicate true for French
and Italian —: an elided piece meets only an elided token and a piece in full only a token in full, the last
piece in full meeting either (Italian elides before a vowel, by the next word, as French does); two
pieces written as one articulated preposition meet only a written one, two written apart only words
apart; the run's edges are the page's (its first token on the article of « al », its last on the
`a` of « al », then covered by D5). Measured on the candidates: 318 headwords hold an elision and
553 an articulated preposition, all named (a piece written elided or contracted always differs
from its key).

**A truncation reads as an elided piece, and is not the next word's doing.** `po'`, `mo'`, `fa'`,
`va'`, `da'`, `sta'`, `di'`, `be'` keep the apostrophe inside their span (change 6's D5), so
`is_elided` says yes for them, on the page and in a name; and a token, as a headword's piece, is
elided when its source span ends on an apostrophe — an elided form of the pre-pass, `dell'` read as
`dello`, both halves of `all'`, `dall'`, `nell'`, `sull'`, a truncation. One rule more than 44c's:
the last piece written in full meets an elided token only where that token is an elision, not a
truncation (told apart by change 6's truncation list on the token's span) — Italian elides a word
by the word that follows, but truncates it on its own. Without it, with both `un po'` and `un poco`
in the pack (both key `un poco`, `un poco` winning it as written as the key), « un po' » would be
met by `un poco` too — its last piece `poco` in full meeting either — and the choice among headwords,
which compares the tokens' text (`poco` for « po' »), would tie and name `un poco`. With it: « un
po' » meets `un po'` alone and « un poco » meets `un poco` alone, each named as written; a pack
holding `un po'` alone meets « un po' » and not « un poco ». 4 of the candidates hold a truncation.

*Pointer (2026-10-24):* `refine-lingua-italian-reading` (change 11a) adds `coll'` and the truncated articulated prepositions `a'`, `co'`, `da'`, `ne'`, `su'` to the split families and reads `da'` as the verb `dai` by its context; if 11a is on main first, this change's `holds_contraction` and truncation test take them (11a's D4, its task 1.7 the other way round).

### D7 — The overlap rule reads Italian

44f's rule runs « in French, Spanish or English » on `built_on_verb` (the pack's grammar reads a
token as a verb) and `is_preposition`; Italian gains its closed classes with change 7 and its
grammar with change 13. The rule runs for every language already; its G2 guard — a preposition the selection's next token
« que » follows — is gated on `whole_words()`, which D5 widens to Italian, and its word becomes
« que » or « che » by the language; G1 (a French headword's hyphens) stays French's; G3 is
language-neutral. Before change 13 no Italian token is read as a verb,
so the rule never fires on an Italian run; after it, the rule decides as for Spanish. Measured at
change 13 on the corpus, as 44f measured the three languages.

### D8 — Review finds an Italian expression card by its name

`pack_gloss`'s Italian arm (the per-language match `readable_gloss` delegates to): a card whose lemma holds a space is read at its lemma, then at the
key the lemma reads as (`reading_expression_key` for Italian), and under its own headword where
several reach the key (44's D3, 44b's D7, 44e's D4). No baseline holds such a card, so no golden
moves for it; a test in `lingua-wasm` holds it.

### D9 — What moves, what does not, and the version

- **The fixture** (`scripts/lingua-data/testdata/it-fr/`): `mwe.tsv` gains `al dente`, `alla
  romana`, `fare l'amore`, `a causa di`, `un po'`, `un poco`, `d'accordo`, `anch'io`, `valere la pena` and `vale la pena`
  (both keyed `valere la pena`); `forms.tsv` the words they need; `manifest.json`'s `pack_version` moves (new rows).
- **The golden** (`it-fr.golden`): its `pack` line; the phrase probes that hold an expression —
  « C'è un po' di pane » (`un po'` named), « all'alba » if the fixture keys it — and the probes
  added: « cotto al dente », « vicino al mare », « a causa del ritardo », « anch'io » and « anche io »,
  « un poco di pane », « vale la pena », « d'accordo »; every `analyse`, `gloss` and `word-grammar` probe byte for
  byte. The pull request lists what moved, as 44's did; *An Italian invariance baseline runs beside
  the English, Spanish and French ones* names the reasons a golden moves and this change's is 44's
  (a rule of the builder and the phrase gloss, probes added): the pull request says so.
- **Italian's analyser version does not move** (44's D7, 44b's D9): no token and no lemma changes;
  the keys and the names are pack data built by the core that ships with the pack, and the
  matcher's rules are the engine's at reading time. `pack_version` of the fixture moves for its new
  rows; a committed it-fr pack does not exist yet.
- **English, Spanish and French do not move**: the Italian arms run for Italian alone; the six
  committed packs build to their pins and the six baselines pass without re-blessing (task 5.1);
  the doc comments that say « French and Spanish » (`whole_words`, `match_expressions`) and « English
  and Spanish tokens are never elided » (`is_elided`) are rewritten; the builder's
  `an_english_pack_carries_no_names` holds as it is.

### D10 — What later changes take from here

| Change | Takes |
|---|---|
| 13 grammar tables | D7's rule fires on Italian runs; measured there |
| 16 it-fr pack | which headwords and senses are offered — the headwords with a space and the 77 words the pre-pass splits —, the pointer senses left out, the glosses; the golden's hand-over to the committed tables, which shows the real expressions; the figures of D2–D6 re-measured on the real `mwe.tsv` |
| 17 refine | the expression defects the dogfood finds |
| 18 word card | a one-word selection over several pieces opening the whole-selection card (`d'accordo`), as 51 did for French |
| 20 enable | the dogfood: an expression card on real pages in it-fr |

## Risks / Trade-offs

- [A determiner varies inside an expression] → Met only as the dictionary writes it (D2); the
  alternative merges `alla romana` with `al romano` and answers every « al » with `a la`.
- [`di` before a partitive] → Read as `di` (D5) where no key matches the run; measured on the
  corpus at implementation.
- [A truncation written in full on the page] → Not met by the truncated headword (D6); the
  dictionary writes both where both exist.
- [One lemma merges two expressions] → Named by the headword written as the page (D4); the same
  cost as French's `ça ira`/`ça va` before 44e.
- [Figures from a copy of the pre-pass] → The count's copy, never the rule (44's D1 rejected a copy
  in the reducers); re-measured with the real builder at change 16.

## Migration Plan

Nothing to migrate: no committed table, pin or pack moves, no reader studies Italian. Rollback is a
revert with the fixture's and the golden's.

## Effort

2.5–3.5 ideal days, against the programme's 2–4: the written list, the window, the names and
`expr.others` for Italian, `whole_words`, the `di` retry, the per-language contraction test and
Italian's reading in `meets_as_written`, the `as_written()` gate for Italian, 44f's G2 gate 1–1.5; the
builder's arms and order 0.25–0.5; review 0.25; their tests 0.5–0.75; the fixture, the probes and
the re-bless 0.25; spec and programme 0.25.

## Implementation — what ships, what departs, and the figures (2026-10-24)

Implemented on `main` after changes 5 to 11, 11a (analyser `1.4.0`), 13 and 14. Italian's analyser
version does not move (D9): no token and no lemma changes — the golden's `analyse`, `gloss` and
`word-grammar` probes are byte for byte, and a test holds an Italian selection's tokens equal with
and without the expression table.

### What departs from D1–D9

- **D6, an apostrophe that stands for an accent.** `read-lingua-italian-unmarked-spellings` (change
  10, its D1) reads `perche'`, `piu'`, `E'` as `perché`, `più`, `È`, their span ending on the
  apostrophe, and named for this change the cost that the as-written rule would then read them as
  elided pieces: `più o meno` would never meet « piu' o meno ». The implementation reads what the
  apostrophe stands for, on the page and in a name, by the token's text
  (`tokenize::italian_apostrophe`): a **truncation** when the written word, without its
  apostrophe, is one of change 6's truncations or 11a's truncated articulated prepositions — told by
  the span, so `da'` is one whether its context reads the verb `dai` or `da` + `i`, and `mo'`
  whether it reads `modo` or `mò` —; an **accent** when the read word is the written word with its
  last vowel accented, which is a word written in full; an **elision** otherwise. The
  `lingua-analysis` requirement says so, with a scenario (« piu' o meno »).
- **D6, an elision and a truncation told apart on both sides.** A piece the headword writes elided
  meets only an elision and one it writes truncated only a truncation, where the design reads both
  as « elided ». The requirement's « only an elided token » holds, narrowed. No elided form and
  truncation of change 6's and 11a's tables read as the same word, so the narrowing decides nothing
  today; it keeps the two apart as « never a truncation » already does for the last piece.
- **D9, the fixture.** `mwe.tsv` gains `vicino a` beside D9's rows: the probe « vicino al mare »,
  which D9 adds, needs it. Ten rows (`a causa di` was there), and in `forms.tsv` the ten words they
  need: `dente`, `romana`, `romano`, `amore`, `accordo`, `pena`, `valere`, `vale`, `valeva`,
  `vicino`; `pack_version` `0.0.2-fixture`.
- **D9, the golden.** Beyond the probes D9 names, two more phrase probes move: « ad alta voce »,
  which now reports its name `ad alta voce` where it reported its key `ad alto voce` (D4), and
  « a causa dello sciopero », which now meets `a causa di` by the `di` family read once more as
  `di` (D5). « all'alba » does not move: the fixture keys no expression on it.
- **D6's pointer.** Change 11a was on `main` first: `holds_contraction` holds `coll'` and the
  truncated articulated prepositions `a'`, `co'`, `da'`, `ne'`, `su'`, and a `da'` the pre-pass
  reads as the verb `dai` sends its headword through the reading, which says it is one token (a
  truncation's, not a contraction's two). The Italian test lives beside change 6's tables
  (`tokenize::holds_italian_contraction`); `engine::holds_contraction(name, studied)` is the
  per-language entry, French's « au »/« aux » unchanged.

### The figures over the fixture (task 4.2)

The real builder over `scripts/lingua-data/testdata/it-fr/`, read back by the core; the full figures
are change 16's, on its `mwe.tsv`.

| D | Figure |
|---|---|
| D1 | 16 headwords, 16 kept; the pack 7,051 bytes (6,485 before) |
| D2 | 7 headwords hold a determiner the key writes (`al dente`, `alla romana`, `fare l'amore`, `un po'`, `un poco`, `vale la pena`, `valere la pena`); 6 keys differ from the every-token-lemmatised key (`a la romano` for `a il romano`, `fare lo amore` for `fare il amore`, `un poco` for `uno poco` twice, `valere la pena` for `valere il pena` twice) |
| D3 | by tokens: 2 → 8, 3 → 7, 4 → 1; none beyond seven, none left out; two single words split, `d'accordo` and `anch'io` |
| D4 | 8 headwords named (`ad alta voce`, `al dente`, `alla romana`, `anch'io`, `d'accordo`, `fare l'amore`, `un po'`, `vale la pena`); 2 keys several headwords reach, `un poco` (`un poco`, `un po'`) and `valere la pena` (`valere la pena`, `vale la pena`), the one written as the key winning each; `expr.names.zst` 122 bytes, `expr.others` 20 bytes |
| D5 | 1 key ends on `di` (`a causa di`); « a causa del ritardo » and « a causa dello sciopero » meet it by the retry, covering « del », « dello » |
| D6 | 3 headwords hold an elision (`anch'io`, `d'accordo`, `fare l'amore`), 1 a truncation (`un po'`), 2 an articulated preposition (`al dente`, `alla romana`); every one named |

### The `di` retry, measured (D5, task 6.1)

**Keys.** The headwords change 16's reducer would offer, as change 12's proposal counted them
(`reduce_common.reduce_expressions` over the English Wiktionary's Italian section of 2026-10-23,
an Italian token pattern): 8,535 candidates, keyed by the real builder with the committed it-fr
tables — 7,461 keyed, 102 of them ending on `di`. An approximation of change 16's `mwe.tsv`, not
it.

**Corpus.** Change 11a's blocks (its D5: MarkIT train and dev, `pages-it`, *I promessi sposi*,
*Pinocchio*, change 8's Wikipedia sample, the whole Tatoeba export; never UD Italian-PUD nor MarkIT's
test section): 1,011,235 blocks, each glossed as a selection by this branch's core — 183,279
matches, **3,217 of them by the retry** (a match whose last token is of the `di` family and whose
run, as written, is no key), on 80 expressions.

**Read.** Every occurrence of 71 of the 80 expressions (1,156 occurrences); the other nine — `avere
bisogno di` 571, `a causa di` 561, `nel corso di` 270, `da parte di` 220, `all'interno di` 207,
`nei confronti di` 111, `al di fuori di` 50, `al di là di` 42, `al di sotto di` 29 — through a
seeded sample of 100 of the 3,217 (`random.Random(20261024).sample`, in corpus order), which holds
one misreading (« riflettere sulla bellezza della tua vita »), and a search of `da parte di` after a
verb of setting aside and of `nel corso di` before a river.

**Misreadings: 130 of 3,217 (4.0 %), in 17 expressions**, of three kinds:
- *the next phrase's `di`* — 87: the expression is a noun phrase whose `di` the page gives to the
  noun that follows: `la bellezza di` (*the whole sum of*) 71 of 71, « la bellezza della natura »;
  `merito di` 12 of 13, « si sono presi il merito del taglio delle tasse », « Te lo meriti del
  relax » (the verb's `meriti` filed under `merito`); `cazzo di` 3 of 3; `a fine di` 1 of 1,
  « venire a fine della sua bella impresa »;
- *the literal phrase* — 29: the words read for what they say: `in via di` 8 of 8, street names
  (« in via dei Fori Imperiali »); `alla luce di` 5 of 13, « alla luce della luna », « alla luce del
  giorno »; `alle spalle di` 4 of 4, « al secondo posto alle spalle del Real Madrid »;
  `sull'orlo di` 3 of 17, « sull'orlo della strada »; `alla faccia di` 2 of 12, « alzato
  lo sguardo alla faccia del padre »; `a fronte di` 2 of 3, « Sopra il ponte a fronte del fonte »;
  `sul filo di` 2 of 2; `sulla pelle di` 2 of 3; `all'insegna di` 1 of 4, « All'insegna del pesce
  d'oro » (a publisher);
- *a partitive* — 14: `del`, `dei`, `delle` is *some*: `da parte di` 2 of 220, « Sta mettendo da parte
  dei soldi », « Ho tenuto da parte del cibo »; `fuori di` 3 of 35, « Tom versò fuori dell'acqua »;
  `pur di` 7 of 7, « Prenditi pure della frutta » (the tables file `pur` under `pure`); `a prova di`
  2 of 2, « disposto a provare delle cose nuove » (they file `prova` under `provare`).

D5's own example, « la prima del film », is not met: the candidates hold no `prima di`. Every other
occurrence read is the expression the page means (« a causa della pioggia », « nel corso dei
secoli », « per amor del cielo », « in balia della tempesta »). The retry ships as designed; which
of the 17 change 16 offers, and how, is change 16's and change 17's.
