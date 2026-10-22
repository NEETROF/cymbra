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
