# Design — match-lingua-french-elided-pieces

## Context

See proposal.md (Why). How a French expression is found today (`origin/main` cbef3788: changes 39
to 51b merged, French's analyser `1.1.0`, fr-en's and fr-es's tables committed):

| Seam | Today |
|---|---|
| French's pre-pass (`analysis/tokenize.rs`, change 40) | an elided word (`l'`, `d'`, `qu'`…) is split off the word it is joined to, a token of its own whose **span holds its apostrophe** and whose text is the word it stands for (`L'` → `Le` [0, 2)); `au`/`aux` → `à` + `le`/`les` sharing one span; `du`/`des` whole |
| the key (`engine::reading_expression_key`, change 44) | the headword read by `headword_reading` (the pre-pass, then `resolve_lemmas`), each token written by `expression_piece` — its dictionary form, or a listed determiner as written —, joined by spaces: `de l'` → `de le`, `coup d'œil` → `coup de œil`, `d'abord` → `de abord` |
| the name (`expr.names.zst`, `Pack::expression_name`, change 44) | the headword that won a key, carried wherever it differs from the key; the match reports it and the status is read on it |
| `engine::gloss_phrase` | tokenises the selection, computes `shares_span` from the tokens' spans (change 44b's D5, change 51's D12), then builds `PhraseToken`s — whose `surface` is the token's text, the read word (`le`), not the written one (`l’`) |
| `engine::match_expressions` | from each token, the longest run of 2 to 7 pieces that is a key (French's du/des retry, D5 of change 44), extended over a following token sharing its span; no look at how a piece was written |

**How it was measured.** A scratch copy of `origin/main` at cbef3788, never committed, carrying the
rule in `match_expressions` behind a switch with four settings — today; the rule as worded, every
piece; this design (D1, the last piece written in full meeting either); and elided pieces only (an
elided headword piece needs an elided token, a full one takes either) —, and a harness building
fr-en's and fr-es's packs from the committed tables (`tables/fr`, `tables/fr-en`, `tables/fr-es`)
and glossing every line of five French corpora as one selection, as change 44 glossed its corpus:

| Corpus | Selections | Tokens | Elided pieces |
|---|---|---|---|
| change 39's `baseline/pages-fr.txt`, block by block | 66 | 1,066 | 66 |
| UD French-GSD train and dev, the two files fr-en's pin records (sha256 `4b9a87b1…`, `9221e508…`), one sentence each | 15,926 | 330,125 | 16,097 |
| four Gutenberg novels change 42 measured (Lemonnier, Cherbuliez, Fréchette, Hémon), by paragraph | 4,393 | 226,086 | 13,883 |
| French Wikipedia paragraphs change 42 measured | 11,405 | 370,775 | 17,549 |
| Tatoeba's French sentences, change 42's sample | 20,000 | 157,004 | 10,048 |
| **All** | **51,790** | **1,085,056** | **57,643 (5.3 %)** |

The Spanish cross-check glosses 11,768 Spanish selections (`baseline/pages-es.txt`, change 42's
AnCora and Tatoeba samples) through es-fr and es-en.

**The tables.** fr-en: 17,479 expressions, 14,373 keyed, 14,322 keys. **1,554** keyed headwords hold
an elided piece and 1,541 keep their key: the piece is the first in 282 (`d'abord`, `c'est`,
`qu'est-ce que`, `l'un`), inside the expression in 1,291 (`coup d'œil`, `tout d'un coup`), the last
in one, `de l'`; 61 of them are one written word. 375 keys end on a piece written in full that French
elides before a vowel (`de`, `que`, `si`, `ce`, `je`…: `parce que`, `pas de`, `même si`). fr-es: 12,177
expressions, 9,410 keys, 877 keyed headwords holding an elided piece, 869 keeping their key (first
piece 95, inside 781, last none); 168 keys end on an elidable piece written in full.

## Goals / Non-Goals

**Goals:**
- The owner's rule: a piece written elided meets an elided word, a piece written in full a word
  written in full — « de le faire » no longer answers `de l'`.
- No pack, key, pin, table or version moves; English and Spanish do not move.
- The golden shows the rule on fr-en's real table.

**Non-Goals:**
- Contractions. A piece the headword writes as « au » or « aux » (`au fait`) still meets « à le »
  written in two words (« prêt à le faire »): the owner's rule is about elision (D7, open
  question 3).
- Which headwords a reducer offers. `de un` (« first, first up ») and the colloquial `c'qui` are
  fr-en's data, changes 48 and 48b's ground; the rule makes `de un` harmless without removing it.
- The words a page drops between two tokens (a number, a single letter), which make tokens adjacent
  that are not (« de 1278 entre »): the rule removes the elided cases only.
- Page highlighting: `analyse_page` is untouched, as in change 44.

## Decisions

### D1 — The rule, piece by piece, the last piece written in full meeting either

Each piece of a matched run is compared with the headword's piece at its place:

| Headword's piece | Page's token | Meets |
|---|---|---|
| elided (`l'` of `de l'`) | elided (« l’eau ») | yes |
| elided | written in full (« le faire ») | **no** |
| written in full, not the last (`de` of `de un`) | written in full | yes |
| written in full, not the last | elided (« d’un ») | **no** |
| written in full, the last (`que` of `parce que`) | either (« parce que », « parce qu’il ») | yes |

French elides a word by the word that follows it. Inside an expression that word is the expression's
own, so the headword says how the piece is written; after its last piece the word is the page's, so
a last piece written in full says nothing about it. A last piece written elided (`de l'`, the one
such headword in fr-en) does say something — « before a vowel » — and is kept to it.

A run whose pieces do not meet does not match at that length; the shorter runs are tried as before,
so a token freed by a refused match can start another (« d'un peu plus »: `de un` refused, `un peu`
found).

Measured, of fr-en's 62,803 matches today over the 51,790 selections:

| Today's match | Count | Examples | The rule as worded | This design |
|---|---|---|---|---|
| every piece as the headword writes it | 58,264 | `coup d'œil`, `d'abord`, `jusqu'à` on « jusqu'au » | stays | stays |
| an elided piece, not the last, on a word in full | 327 | `c'est` on « ce sont » 100, « ce fut » 77, « ce soit » 43, « ce serait » 29, « ce sera » 25, 8 more; across a dropped word: « de 1278 entre » `d'entre` 12, « de 1980 avec » `d'avec` 12, « de 1888, où » `d'où`; an inversion: « Pourrais-je avoir » `j'ai`, « secoue-le un peu » `l'un`; English: « That's For Me, a » `m'a` | gone | gone |
| the last piece elided, on a word in full | 165 | `de l'` on « offert de le payer », « l'attention de M. le ministre », « Jeux olympiques de 1960, le stade » | gone | gone |
| a piece in full, not the last, on an elided word | 1,526 | `de un` « first, first up » on « d’un » 1,513 (« Personne au village ne se souvenait d’un hiver »); `ça a été` on « ç'a été » 5, `de la` on « d'la » 3, `coup de pied` on « coup d'pied », `ce qui` on « c'qui », `je suis enceinte` on « J'étais enceinte » | gone | gone |
| the last piece in full, on an elided word | 2,521 | 201 expressions: `ce que` 284 (« ce qu'il »), `dire que` 150, `pas de` 134 (« pas d'argent »), `ainsi que` 119, `bien que` 97, `parce que` 95, `alors que` 85; change 39's own « Bien qu'il pût venir » and « Qu’est-ce qu’il attend » | **gone** | stays |

| Pair | Matches today | This design: lost | gained | after | Selections moved | The rule as worded: lost | gained |
|---|---|---|---|---|---|---|---|
| fr-en | 62,803 | 2,021 | 57 | 60,839 | 1,774 | 4,547 | 269 |
| fr-es | 41,568 | 13 | 1 | 41,556 | 13 | 2,013 | 64 |

By corpus, this design:

| Corpus | fr-en today | lost | gained | fr-es today | lost | gained |
|---|---|---|---|---|---|---|
| change 39's | 55 | 1 (`de un` on « d’un hiver ») | 0 | 45 | 0 | 0 |
| UD French-GSD | 18,299 | 593 | 14 | 12,556 | 3 | 0 |
| Gutenberg | 13,213 | 657 | 21 | 7,952 | 4 | 0 |
| Wikipedia | 20,351 | 547 | 13 | 14,846 | 3 | 0 |
| Tatoeba | 10,885 | 223 | 9 | 6,169 | 3 | 1 |

fr-en's 2,021 lost are the 2,018 mismatches of the first table's rows 2–4 and 3 matches displaced:
once `c'est` no longer takes « ce sont » or « ce sera », `être à` takes the « sont au » or « sera à »
that follows, and `au contraire`, `à cause de` and `à présent` lose their `à`. The 57 gained are the shorter runs a refused match frees: `un peu` 17
(« d'un peu plus »), `un autre` 17, `être à` 7, `un tel` 6, `un jour` 4, `être le cas` 3 (« Ce fut le
cas »), 3 more. fr-es holds neither `de l'` nor `de un`: it loses `c'est clair`, `c'est dire`,
`c'est du gâteau` on « ce soit », « ce sera » (3), `langue d'oc` on « Langue de Oc », `jaune d'œuf`
across a number, « d'la » (4), `coup de pied`, `ce qui`, and `se altérer` on « s'altèrent » (2, a
headword the Spanish Wiktionary writes without its elision); it gains `du gâteau`.

The rule as worded would also refuse the last piece written in full on an elided word: 2,521 right
matches in fr-en and 1,996 in fr-es, the « Bien qu'il » and « Qu’est-ce qu’il » change 44's D1
rejected matching on written words to keep (open question 1).

*Rejected — elided pieces only* (an elided headword piece needs an elided word, a piece in full takes
either): it keeps the 1,513 `de un` on « d’un », the commonest wrong match measured; it loses 495
matches in fr-en against this design's 2,021, every one of them among this design's.

*Rejected — an inner elided piece meets a word in full before a consonant* (« ce sont » would answer
`c'est`, « de 1278 entre » still not `d'entre`): it needs French's aspirated-h words, which no table
holds (« le héros » is never elided, « l'homme » always), for 282 matches of one expression.

### D2 — The page's elision is read from the token's span

An elided token's text is the word it stands for (`le` for « l’ »), so neither `Token::text` nor
`PhraseToken::surface` says it was elided. Its span does: *French tokenisation pre-pass* gives an
elided piece a span holding its apostrophe (`L'` [0, 2)), written alone (« l’ homme ») or not, and no
other token's span ends on one — UAX #29 never ends a word on an apostrophe, and an edge apostrophe
is outside the word's span. Measured: of the corpus's 1,085,056 tokens, the 57,643 whose span ends on
`'` or `’` are each one of the fourteen elided forms.

`gloss_phrase` reads it from the tokens it has just made, beside `shares_span`, and hands
`match_expressions` one flag per token; a function beside the pre-pass,
`analysis::tokenize::is_elided(text, token)`, says it once for the selection and for the headword.

*Rejected — a field on `Token` set by the pre-pass.* Every place that builds a token (English,
Spanish, compounds, contractions) would carry it, and `Token` is serialised; the span already holds
the answer.

### D3 — The headword's elision is read from its name, already in the pack

At match time the run has a key (`de le`), which no longer says how its pieces were written. The
headword does, and the pack already carries it: a headword holding an elided piece always differs
from its key — the piece is written `l'` in the one and `le` in the other —, so change 44's names
section holds it. Measured: fr-en's 1,541 kept headwords holding an elided piece are all named, and
fr-es's 869. A key the pack does not name is its own headword, written without an elided piece.

So the matcher reads the match's name (`Pack::expression_name`, or the key where there is none)
through `headword_reading` — the function the builder made the key with, on the same lexicon, so
the pieces line up one for one — and takes each piece's elision from its span (D2). A name holding
no apostrophe is written in full and is not read. Measured: glossing the 51,790 selections natively
takes 2.75 s today and 2.77–2.82 s with the prototype, which reads every match's name.

*Rejected — keys that spell the elision* (`de l'` keyed `de l'`, `coup d'œil` `coup d' œil`). The
builder and the matcher would share it through `expression_piece`, but the keys of 1,554 of fr-en's
keyed headwords and 877 of fr-es's would move, with both packs and their pins; the last piece written in full would need a
retry like change 44's `du`/`des`; and the gain is three keys several headwords of different
elisions reach in fr-en (`ce qui` and `c'qui`, `ce que` and `c'que`, `putain de` and `putain d'`),
one match on the corpus (« c'qui »).

*Rejected — a section recording each key's elided pieces.* A pack format change for what the names
already say.

### D4 — What the rule costs

The rule refuses some matches whose meaning was right, measured above:
- **`c'est` on its forms written in full** — « ce sont » 100, « ce fut » 77, « ce soit » 43, « ce
  serait » 29, « ce sera » 25 and 8 more in fr-en (282), « que ce soit clair » and « Ce sera du
  gâteau » in fr-es. `ce être` is one key for « c'est » and « ce sont », as `il y a` is one for « il y
  avait »; the rule tells them apart by the written `c'`. « c'était », « c'eût été » and every
  elided form still answer `c'est`. Some of the 282 were wrong anyway: « que ce soit » is « whether »,
  not « it is ».
- **An inner piece whose elision follows an inflected word** — `je suis enceinte` on « J'étais
  enceinte » (1); **a literary or colloquial elision** — « ç'a été » (5), « d'la » (3), « coup
  d'pied », « c'qui »; **a word written in full where the dictionary elides** — « Langue de Oc ».

Against them, the rule removes 1,513 `de un` on « d’un », 165 `de l'` on « de le », and 44 more
wrong ones: 34 across a dropped number (« de 1278 entre » `d'entre`), 5 across an inversion or a
markup (« secoue-le un peu » `l'un`, « Pourrais-je avoir » `j'ai`), 3 `c'est …` phrases on another
form (« ce fut selon eux » `c'est selon`), « That's For Me, a » `m'a` and « en forme de Y » `d'y` —
the 45th is the right « Langue de Oc ». The owner's rule is taken as written for every piece but the
last (open question 2).

### D5 — What moves: four probes, added

The French golden shows the rule only where a probe holds it; none of its 30 phrase probes does.
Measured: with the rule and no probe added, `fr-en.golden` and `fr-es.golden` are byte for byte, as
are `en-fr.golden`, `es-fr.golden`, `es-en.golden`, `en-es.golden` and the `cross_native`,
`parity`, `card_gloss_language` and `spanish_expression_keys` suites (57 tests in 10 suites); every
`lingua-core` test passes. So the French scenario (`tests/support/french.rs`) gains four phrase
probes, as changes 44 and 44b added theirs:

| Probe | fr-en today | fr-en with the rule | fr-es, both |
|---|---|---|---|
| « de l’eau » | `de l'` | `de l'` | none |
| « Il a décidé de le faire » | `de l'` | **none** | none |
| « parce qu’il pleut » | `parce que` | `parce que` | `parce que` |
| « d’un hiver » | `de un` | **none** | none |

Re-blessed on the prototype: `fr-en.golden` 789 → 797 lines and `fr-es.golden` 799 → 807, 8 lines
added each and none removed, 30 → 34 phrase probes. `word-card-fr-en.txt` and `word-card-fr-es.txt`
(`apps/lingua-extension/test/baseline/`), which render the golden's phrase probes, gain the four
probes' whole-selection cards, every block before byte for byte; their specs count 34 phrase probes.
No `analyse`, `gloss`, `word-grammar` or review probe, no pack, pin, table, `pack_version` or
fixture moves.

**French's analyser version stays.** *An analyser version per studied language* bumps the version of
a language whose output a change can alter; as change 44 (its D7) and change 51 (its D12) read it for
the phrase gloss's matching, no token, lemma or page analysis moves and no pack byte does: the
matching is the engine's at reading time, a pack and its core ship together, and an older core with
the same pack answers as before.

**The baseline's reason.** *A French invariance baseline runs beside the English and Spanish ones*
(changes 39, 40, 41, open) names three reasons for moving the golden, none of them probes a change
adds; changes 44 and 51 added theirs and said so in their pull requests, and so does this one. The
test's doc comment (`french_baseline.rs`) lists this change among those that add probes.

### D6 — English and Spanish cannot move

- The check runs for a French selection alone, after a French run has matched a key; English and
  Spanish runs go through `match_expressions` as before.
- **Spanish has no elision.** Its pre-pass splits no apostrophe, and no Spanish token's span ends on
  one: 0 elided tokens in 11,768 Spanish selections, and no es-fr or es-en headword holds an elided
  piece. Its contractions « al » and « del » are split into pieces sharing a span — change 44b's
  ground, like French's « au »/« aux » (D7) —: on the same selections, no contracted headword piece
  met « a el » or « de el » written apart (0 of es-fr's 5,854 matches and es-en's 6,634). The other
  way, 132 and 123 matches meet a contraction with pieces written apart: « después del » answering
  `después de` (44b's D5, right), and 3 in es-fr and 25 in es-en starting on the article of « al » or
  « del » (« al que » answering `el que`), as French's on « aux » (D7).
- No builder, key, name or table moves, so `committed_tables` builds the six committed packs to their
  pins, and the four other goldens pass without re-blessing (measured, D5).

### D7 — Contractions are not this change's

The same question stands for French's contractions: `au fait` (« by the way; informed »), keyed `à le
faire`, answers « Il est prêt à le faire », `le` a pronoun. Measured on the corpus: 10 such matches
in fr-en and 10 in fr-es, every one `au fait` on « à le faire ». The owner's rule is about elision;
a piece written as « au » or « aux » could be held to a contraction on the page by the same reading
of the name — its two pieces sharing a span — in a change of its own (open question 3). Measured
alongside: 39 matches in fr-es and 1 in fr-en start on the article of « au » or « aux » (`le même`
on « au même », `les deux` on « aux deux »), which change 51's D12 does not cover since it only
extends a match's end; not this change's either.

### D8 — What later changes take from here

| Change | Takes |
|---|---|
| 48b `refine-lingua-fr-en-glosses`, 49b `refine-lingua-fr-es-glosses` | the four probes in their re-bless; 48b's removals (`que de`, `et si`, `un coup`…) move a few of D1's counts, not `de l'` nor `de un` |
| 52 `enable-lingua-french` | the dogfood: « de l'eau » and « de le faire » on a real page in fr-en |

## Risks / Trade-offs

- [A right match refused] → `c'est` on « ce sont », « ce fut », « ce sera » (282), a literary or
  colloquial elision (10): D4, open question 2. A status set on `c'est` is still read on « c'était ».
- [The last piece's exception lets a wrong match through] → One among fr-en's 2,521, whose last
  pieces are all `que`, `de` or `si` but this one: `sur ce` across a dropped number (« sur 5 352 km2.
  C’est »). Elsewhere the next word is the page's, and French elides before it by rule.
- [A name that does not line up with its key] → The name is read by the function and lexicon the key
  was made with; should the two ever differ in length, the run is matched as today. A test on the
  committed tables holds every French headword with an elided piece named.
- [Tokens joined across a dropped word] → The rule removes those where the headword's piece is
  elided and the page's is not (34 matches, « de 1278 entre » `d'entre` among them); a run of words
  written in full joined across a dropped number stays as today (Non-goals).
- [Cost] → One reading of a name holding an apostrophe per French match; within 2 % of the corpus's
  gloss time with every name read.

## Migration Plan

Nothing to migrate: no stored format, wire field, table, pin or pack moves, and no reader studies
French. Rollback is a revert.

## Effort

0.5–1 ideal day: `is_elided` and the check in the core, with a test per scenario, 0.25–0.5; the
committed-tables test 0.1; the four probes, the two goldens and the two snapshots re-blessed 0.1–0.25;
spec and programme 0.1.

## Open Questions

For the owner, none blocking — each with the design's recommendation:
1. **The last piece written in full (D1).** Your rule, word for word, would also stop « parce qu’il
   pleut » answering `parce que`, « pas d’argent » `pas de` and « Bien qu’il pût venir » `bien que`:
   the `que` is elided because « il » follows, and « il » is not part of the expression. That is
   2,521 right matches in fr-en and 1,996 in fr-es on the corpus. Recommended: a last piece written
   in full meets either form; a last piece written elided (`de l'`) still needs « l’ ».
2. **`c'est` and the other forms written in full (D4).** With the rule, « Ce sont mes amis », « ce fut
   le cas » and « que ce soit » no longer show the card `c'est` (282 matches in fr-en); « C’était
   l’hiver » still does. Likewise « J’étais enceinte » no longer shows `je suis enceinte`, and
   « ç’a été » `ça a été`. Recommended: accept, as the rule says; the alternative needs a list of
   French's aspirated-h words for one expression.
3. **Contractions (D7).** « Il est prêt à le faire » shows `au fait` (« by the way »), and will
   still: the rule is about elision, and « au » is a contraction. 10 matches per pack on the corpus.
   Recommended: a small change of its own if you want it, holding a piece written « au »/« aux » to
   a contraction on the page the same way; not here.
