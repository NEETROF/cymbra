# Design — match-lingua-french-contracted-pieces

## Context

See proposal.md (Why). How a French expression is found today (`origin/main` f3585580: changes 39
to 51b, 48b and 44c merged, French's analyser `1.2.0`, fr-en's tables as 48b re-reduced them, fr-es's
as change 49 committed them):

| Seam | Today |
|---|---|
| French's pre-pass (`analysis/tokenize.rs`, change 40) | `au`/`aux` → `à` + `le`/`les`, **the two tokens sharing the written word's span** (`Au` → `À` [0, 2) + `le` [0, 2)); `du`/`des` whole (M21); an elided piece a token of its own whose span holds its apostrophe; an inversion's pieces each with a span of its own. So **the halves of « au » and « aux » are the only French tokens sharing a span** (change 51's D12) |
| the key (`engine::reading_expression_key`, change 44) | the headword read by `headword_reading`, each token written by `expression_piece`, joined by spaces: `au fait` → `à le faire`, `au revoir` → `à le revoir`, `armé jusqu'aux dents` → `armer jusque à les dent` |
| the name (`expr.names.zst`, `Pack::expression_name`, change 44) | the headword that won a key, carried wherever it differs from the key |
| `engine::gloss_phrase` | tokenises the selection, computes `shares_span` (token `i` shares its predecessor's span) and `elided` (change 44c's D2) from the tokens' spans, hands both to `match_expressions` |
| `engine::match_expressions` | from each token, the longest run of 2 to 7 pieces that is a key (French's du/des retry, change 44's D5), kept only where `meets_as_written` finds its elided pieces as the headword writes them (change 44c), then extended over a following token sharing its span (change 51's D12); no look at whether two pieces were one written word |

**How it was measured.** A scratch copy of `origin/main` at f3585580, never committed, carrying the
check in `match_expressions` behind a switch with five settings — today; the rule as worded, piece by
piece (a piece the headword writes as half of « au » needs half of « au » on the page, a piece
written apart a word written apart); the last piece's `à` meeting the `à` of « au » too; **this
design**, the run's two edges the page's (D1); and contracted headword pieces only (a piece written
apart meets either) — and change 44c's harness extended: fr-en's and fr-es's packs built from the
committed tables (`tables/fr`, `tables/fr-en`, `tables/fr-es`), every line of change 44c's five
French corpora glossed as one selection:

| Corpus | Selections | Tokens | Halves of « au »/« aux » |
|---|---|---|---|
| change 39's `baseline/pages-fr.txt`, block by block | 66 | 1,066 | 18 |
| UD French-GSD train and dev, the two files fr-en's pin records, one sentence each | 15,926 | 330,125 | 6,388 |
| four Gutenberg novels change 42 measured, by paragraph | 4,393 | 226,086 | 2,792 |
| French Wikipedia paragraphs change 42 measured | 11,405 | 370,775 | 6,996 |
| Tatoeba's French sentences, change 42's sample | 20,000 | 157,004 | 1,488 |
| **All** | **51,790** | **1,085,056** | **17,682 (8,841 written « au » or « aux »)** |

The Spanish cross-check (D6) glosses 464,574 Spanish selections, 4,160,681 tokens — Tatoeba's
442,407 Spanish sentences, 11,567 Spanish Wikipedia paragraphs, 5,341 UD Spanish sentences (AnCora,
GSD, PUD, COSER) and 5,212 paragraphs of five Spanish-language Gutenberg books, change 42b's
corpora, and change 3's `baseline/pages-es.txt` — through es-fr and es-en.

**The tables.** fr-en: 14,432 keys. **448** winning headwords hold a contraction, all named (the
headword writes « au », the key `à le`): the contraction is the first word in 180 (`au fait`, `au
revoir`, `aux aguets`) and inside in 268 (`aller au lit`, `tirer au sort`, `armé jusqu'aux dents`);
none ends on one. **No headword writes « à le » or « à les » apart**: the 154 keys holding `à le` or
`à les` without a contraction are all elided (`mal à l'aise`, `d'un jour à l'autre`), change 44c's
ground; and no key is reached both by a headword holding « au » and by one writing it otherwise. 104
keys start on `le`/`les` written alone (`le même`, `les miennes`) and 29 end on `à` (`grâce à`,
`jusqu'à`). 805 hold `du` or `des`, words of their own. fr-es: 9,410 keys, 157 holding a contraction
(first 57, inside 100), all named, none written apart; 56 start on `le`/`les`, 24 end on `à`; 557
hold `du` or `des`.

**The exposure.** 24 of fr-en's 448 (7 of fr-es's 157) are keyed `à le`/`à les` before a verb's
dictionary form, the noun after « au » read as a form of a verb: `au fait` (`à le faire`), `au
revoir`, `au pouvoir`, `au reste` (`à le rester`), `au dire de`, `au vu de` (`à le voir de`), `au
jugé`, `aux commandes` (`à les commander`), `tirer au sort` (`à le sortir`), `être aux prises` (`à
les prendre`), `écouter aux portes` (`à les porter`). French writes « à le » and « à les » apart
where `le` or `les` is a pronoun before an infinitive (« prêt à le faire », « continuer à les
voir »), so these are the headwords such a page answers today.

## Goals / Non-Goals

**Goals:**
- The owner's rule: « au » and « aux » meet only « au » and « aux », pieces written apart only words
  written apart — « prêt à le faire » no longer answers `au fait`.
- Change 44c's mechanism: the page's side read from the spans, the headword's from its name, one
  reading for both checks; no pack, key, pin, table or version moves; English and Spanish do not
  move.
- The golden shows the rule on fr-en's and fr-es's real tables.

**Non-Goals:**
- `du` and `des`. M21 keeps them whole, so a headword's `du` is keyed `du` and the page's « de le »
  reads `de le`: the two never meet today, and the rule has nothing to hold. Change 44's `du`/`des`
  retry (« à cause des » → `à cause de`) reads a whole word, no span shared.
- Spanish's « al » and « del » (D6).
- Which of two overlapping expressions wins. An expression ending on the `à` of « au » takes the
  article (change 51's D12), so an expression starting on that « au » cannot start there: `être à`
  on « elles sont au nombre de sept » keeps `au nombre de` out, `jusqu'à` on « jusqu'au début »
  `au début`. Measured, 91 of fr-en's 505 such matches and 33 of fr-es's 334 (`au début` 14, `au
  nombre de` 10, `au milieu de` 10; `au moins` 6, `au courant` 6). It is change 44's longest-first
  order, not how a piece is written (open question 4).
- Tokens joined across a word the page drops (a number, punctuation): the rule removes those where
  a headword's « au » meets « à » and « le » written apart, none on the corpus; others stay.
- Page highlighting: `analyse_page` is untouched, as in changes 44 and 44c.

## Decisions

### D1 — A run is joined where its headword is, and only there; its edges are the page's

French contracts `à` with the article after it. Each place between two adjacent tokens of a matched
run is compared with the place between the headword's two pieces at the same position:

| Headword | Page | Meets |
|---|---|---|
| two pieces written as one, « au » or « aux » (`au` of `au fait`) | « au », « aux » (« Au fait ») | yes |
| « au », « aux » | `à` and `le` written apart (« prêt à le faire ») | **no** |
| two pieces written apart, `à` and `le` (no headword in the tables) | « au », « aux » | **no** |
| the last piece, `à` (`grâce à`) | the `à` of « au », its article the page's next word (« grâce au soleil ») | yes — change 51's D12 then covers the article |
| the first piece, `le` or `les` (`les miennes`) | the article of « aux », its `à` the page's word before (« aux miennes ») | yes (D4) |

Inside an expression both words of a contraction are the expression's, so the headword says how
they are written. At its edges one of the two is the page's — the article after the expression's
last `à`, the `à` before its first article — and the expression says nothing about it. This is the
owner's rule read as change 44c's D1 reads elision (the last piece follows the page): the owner's
« à le » is a pair of pieces, and a pair is the expression's only when both halves are.

A run whose places do not meet does not match at that length; the shorter runs are tried as before.
Measured on the corpus, no shorter run takes the place of a refused one.

Today's matches, by how they meet:

| Today's match | fr-en | fr-es | Examples | Piece by piece | This design |
|---|---|---|---|---|---|
| joined where the headword is | 59,850 | 41,173 | « Au revoir », « au fur et à mesure », « aller au lit » | stays | stays |
| « au » on `à` and `le` written apart | 10 | 10 | `au fait` on « Elle n'arrive pas à le faire », « qui commençait à le faire souffrir », « continueront à le faire » | gone | **gone** |
| the last `à` on the `à` of « au », « aux » | 505 | 334 | 18 and 12 expressions: `jusqu'à` 180, `être à` 91 and 120, `grâce à` 62 and 63, `face à` 42, `par rapport à` 34 and 35, `quant à` 26 | **gone** | stays |
| the first `le`, `les` on the article of « au », « aux » | 4 | 39 | `les miennes` « Compare tes réponses aux miennes », `le sien` « en comparant leur sort au sien »; `le même` « au même titre que » 21, `les deux` « aux deux ministres » 13 | **gone** | stays |
| a place inside the run, joined on one side only, anything else | 0 | 0 | | | |

| Pair | Matches today | This design: lost | gained | after | Selections moved | Piece by piece: lost | gained | Last piece only: lost | gained |
|---|---|---|---|---|---|---|---|---|---|
| fr-en | 60,369 | 10 | 0 | 60,359 | 10 | 537 | 106 | 14 | 0 |
| fr-es | 41,556 | 10 | 0 | 41,546 | 10 | 383 | 35 | 49 | 0 |

By corpus, this design loses in each pair 0 matches in change 39's corpus, 1 in GSD, 1 in Gutenberg,
2 in Wikipedia and 6 in Tatoeba, each `au fait` on « à le faire ».

*Rejected — piece by piece, the rule as worded.* It refuses the 505 and 334 matches ending on the
`à` of « au », among them `jusqu'à` on « jusqu'au soir », which the owner settled for change 51's D12,
and `grâce à` on « grâce au soleil »; it frees 106 and 35 shorter or later runs, 91 and 33 of them
an expression starting on that « au » (Non-goals).

*Rejected — the last piece only* (the run's first token held to the rule): it refuses the 4 and 39
matches starting on the article of « au » or « aux », all but one right (D4), and frees nothing.

*Rejected — contracted headword pieces only* (a piece written apart meets either). Same figures on
the committed tables, which write no « à le » apart; but a headword written apart, which a future
table may hold (« apprendre à le connaître »), would answer « au », the other half of the owner's
rule.

### D2 — The page's contractions are read from `shares_span`

`gloss_phrase` already computes, from the tokens it has just made, whether each token shares its
predecessor's span (change 44b's D5, change 51's D12), and hands it to `match_expressions`, which
extends a match over the article. In French only the halves of « au » and « aux » share a span
(change 40): an elided piece, an inversion's piece and every other word has a span of its own,
measured — the corpus's 17,682 tokens sharing a span are 8,841 pairs, `à` and `le` of « au »
(6,755) or `à` and `les` of « aux » (2,086).
So the run's places are `shares_span[start + 1 .. start + len]`, and `match_expressions` hands
`meets_as_written` that slice beside the elision flags. No token, field or JSON changes.

### D3 — The headword's contractions are read from its name, already in the pack

At match time the run has a key (`à le faire`), which no longer says the two pieces were one word.
The headword does, and the pack carries it: a headword holding « au » or « aux » differs from its key
— `au` in the one, `à le` in the other —, so change 44's names section holds it. Measured: fr-en's
448 winning headwords holding a contraction are all named, and fr-es's 157. A key the pack does not
name is its own headword, written apart.

So `meets_as_written` (change 44c) reads the match's name — `Pack::expression_name`, or the key —
through `headword_reading`, the function the builder made the key with, on the same lexicon, so the
pieces line up one for one, and takes the headword's places from the reading's spans as
`gloss_phrase` takes the page's. It reads it when the name holds an apostrophe (change 44c) or the
word « au » or « aux » in any case, after an elision too (« jusqu'aux »); one reading serves both
checks. A name holding neither joins no pieces: the run's places must all be apart. Measured: every
winning headword holding a contraction holds the word « au » or « aux » (448 and 157); the check
reads 1,979 names in a pass over the 51,790 selections in fr-en and 772 in fr-es, and glossing them
natively takes 2.717 s today and 2.724 s with it (median of 15 interleaved passes, +0.3 %).

*Rejected — keys that spell the contraction* (`au fait` keyed `au faire`). The builder and the
matcher would share it through `expression_piece`, but 448 of fr-en's keys and 157 of fr-es's would
move, with both packs and their pins, and the run's last `à` would need a retry on « au » like change
44's `du`/`des`; no key is reached by two writings, so nothing is gained.

*Rejected — a section recording each key's contractions.* A pack format change for what the names
already say.

### D4 — The matches starting on the article of « au » or « aux » are kept

Change 44c's D7 found them, and change 51's D12, which covers an article after a match, does not
reach an article before one. Measured: 4 in fr-en — `les miennes` « Compare tes réponses aux
miennes » and « semblables aux miens », `les vôtres` « je crois aux vôtres », `le sien` « en
comparant leur sort au sien » — and 39 in fr-es — `le même` 21 (« au même titre que le catalan »,
« au même endroit »), `les deux` 13 (« aux deux ministres », « aux deux bouts »), `les miennes` 2,
`les vôtres`, `les siennes`, `le sien`. All but one are right: the expression is the article and the
word after it, and the `à` is the page's; « aux deux tiers » answering `les deux` (« both »,
« Ambos ») is not.

The card shows such a match as a row beside the `à`, which, being a function word, has none
(`rowsFor`): « au même titre » shows `le même` and `titre`. It is never the whole selection's answer,
which needs a match from the selection's first token.

Three ways, measured:
- **keep them** (this design): the run's first token meets the article of « au »;
- refuse them — the rule piece by piece at the run's start, *a French match never starts inside a
  written word*: 4 and 39 right matches lost, none gained; « au même titre » shows `même` and `titre`;
- cover the `à` too, as D12 covers the article: « au même » selected alone would be answered `le
  même` as a whole — « the same » for « at the same » —, the match covering a word its key does not
  hold.

Recommended: keep them (open question 2). The probe « aux miennes » holds the choice in the golden.

### D5 — What moves: four probes, added

The French golden shows the rule only where a probe holds it; none of its 34 phrase probes does.
Measured on the prototype: with the rule and no probe added, `fr-en.golden` and `fr-es.golden` are
byte for byte, as are `en-fr.golden`, `es-fr.golden`, `es-en.golden`, `en-es.golden` and the
`cross_native`, `parity`, `card_gloss_language` and `spanish_expression_keys` suites (57 tests in 10
suites), and every `lingua-core` test passes. So the French scenario (`tests/support/french.rs`)
gains four phrase probes, after change 44c's:

| Probe | Today, fr-en and fr-es | With the rule |
|---|---|---|
| « Il est prêt à le faire » | `au fait` over `à`, `le`, `faire` | **none** |
| « Au fait, tu viens ? » | `au fait` over `À`, `le`, `fait` | the same |
| « grâce au soleil » | `grâce à` over `grâce`, `à`, `le` | the same |
| « aux miennes » | `les miennes` over `les`, `miennes` | the same (D4) |

Re-blessed on the prototype: `fr-en.golden` 797 → 805 lines and `fr-es.golden` 807 → 815, 8 lines
added each and none removed, 34 → 38 phrase probes. `word-card-fr-en.txt` and `word-card-fr-es.txt`
(`apps/lingua-extension/test/baseline/`), which render the golden's phrase probes, gain the four
probes' cards, 11 lines each, every block before byte for byte — in fr-en « prêt — ready » and
« faire — to do »; « au fait — au fait » (fr-en's first sense, the English borrowing) and « venir —
to come… »; « grâce à — thanks to » and « soleil — sun (star) »; « les miennes — mine » —, in fr-es
« prêt — Dispuesto, listo, presto o preparado » and « faire — Hacer »; « au fait — Dicho sea de paso,
de paso, por cierto » and « venir — Venir »; « grâce à — Gracias a » and « soleil — Sol »; « les
miennes — Las mías ». Their specs count 38 phrase probes. No `analyse`, `gloss`, `word-grammar` or
review probe, no pack, pin, table, `pack_version` or fixture moves.

**French's analyser version stays**, as changes 44 (its D7), 51 (its D12) and 44c (its D5) read *An
analyser version per studied language* for the phrase gloss's matching: no token, lemma or page
analysis moves and no pack byte does.

**The baseline's reason.** *A French invariance baseline runs beside the English and Spanish ones*
(changes 39, 40, 41, open) names three reasons for moving the golden, none of them probes a change
adds; changes 44, 51 and 44c added theirs and said so in their pull requests, and so does this one.
The test's doc comment (`french_baseline.rs`) lists this change among those that add probes.

### D6 — English and Spanish cannot move; Spanish's « al » and « del » left out

- The check runs for a French run alone, beside change 44c's; English's split words (`don't` → `do`
  + `not`, sharing a span) and Spanish runs go through `match_expressions` as before.
- **Does a headword's « al » meet « a el » written apart?** Yes, in principle: change 44b keys `al
  menos` `a el menos`, and nothing tells « a el » from « al ». Measured over the 464,574 Spanish
  selections: 1 of es-fr's 205,743 matches (`del cual` on « en la década de 1820, el cual »), 5 of
  es-en's 236,765 (`al tiempo` on « una entrevista a El Tiempo », `del mismo` on « el 28 de septiembre
  de 1936, el mismo día » and two more dates, `al uso` on « a) el uso »). Spanish writes « a el » and « de el »
  apart only before a title or a name (« El Tiempo »), and the rest join across a number or a
  bracket the tokeniser drops. Applied to Spanish, this design would lose those 6 matches and gain
  `el cual` once; it would also hold the 5 es-fr headwords and the 1 es-en headword that write « de
  el » apart (`búfalo de el cabo`, `pingüino de el cabo`, `mochuelo de el cabo`…, the dictionaries'
  spelling) to « de el » written apart, none matched on the corpus. The matches starting on the
  article of « al » or « del » are Spanish's counterpart of D4: 67 in es-fr (`el mío` 30, `el tuyo`
  14, `el prójimo` 12) and 792 in es-en (`el que` 426 on « al que », « del que », `el extranjero`
  202, `el juego` 84), kept either way.
- So Spanish is left out: es-fr's output does not move (the programme's rule), and es-en's and
  en-es's are held the same way, for 6 matches in 464,574 selections (open question 3).
- No builder, key, name or table moves, so `committed_tables` builds the six committed packs to their
  pins, and the four other goldens pass without re-blessing (measured, D5).

### D7 — What later changes take from here

| Change | Takes |
|---|---|
| 49b `refine-lingua-fr-es-glosses` (proposed) | the four probes in its re-bless of `fr-es.golden` and `word-card-fr-es.txt`, whichever of the two lands second |
| 52 `enable-lingua-french` | the dogfood: « prêt à le faire » and « Au fait » on a real page in fr-en |

## Risks / Trade-offs

- [A right match refused] → None on the corpus: the 10 refused are `au fait` on « à le faire »,
  `le` a pronoun each time. A headword written « à le » apart would no longer answer « au »; the
  tables hold none.
- [A wrong match kept at an edge] → « aux deux tiers » answering `les deux` (1 of 43, D4); and `être
  à` on « est au », as on « est à » (Non-goals).
- [A name that does not line up with its key] → The name is read by the function and lexicon the key
  was made with; should the two ever differ in length, the run is matched as today (change 44c's
  guard). A test on the committed tables holds every French headword with a contraction named and
  holding the word « au » or « aux ».
- [Cost] → One reading of a name holding « au » or « aux » per French match that reaches the check:
  +0.3 % of the corpus's gloss time.

## Migration Plan

Nothing to migrate: no stored format, wire field, table, pin or pack moves, and no reader studies
French. Rollback is a revert.

## Effort

0.25–0.5 ideal day: the check in `meets_as_written` and its places from `match_expressions`, with a
test per scenario, 0.15–0.25; the committed-tables test 0.05; the four probes, the two goldens and
the two snapshots re-blessed 0.05–0.15; spec and programme 0.05.

## Open Questions

For the owner, none blocking — each with the design's recommendation:
1. **The expression's edges (D1).** Your rule, word for word, would also stop « jusqu'au soir »
   answering `jusqu'à` (180 times on the corpus in fr-en), « grâce au soleil » `grâce à` and « face
   aux enjeux » `face à`: the expression ends on `à`, and the `le` glued to it belongs to the page's
   next word. That is 505 matches in fr-en and 334 in fr-es; « jusqu'au » is the case you settled on
   2026-10-09 (change 51's D12). Recommended: at an expression's edge, the contraction is the page's.
2. **« au même titre » answering `le même` (D4).** The expression starts on the `le` hidden in
   « au »: « Compare tes réponses aux miennes » shows `les miennes` (« mine »), « au même titre que
   le catalan » `le même` (« Lo mismo »). 4 matches in fr-en, 39 in fr-es, all but one right
   (« aux deux tiers » shows `les deux`, « both »). Recommended: keep them. Refusing them loses the
   43 and gains nothing (the card shows `même` and `titre` instead); making them cover « au » whole
   would answer « au même » with « the same », hiding the « at ».
3. **Spanish's « al » and « del » (D6).** The same gap exists — « a El Tiempo » answers `al tiempo`
   —, but 6 times in 464,574 Spanish selections, each before a title or across a dropped number.
   Recommended: leave Spanish out, so es-fr, es-en and en-es do not move.
4. **`être à` taking « au » from `au nombre de` (Non-goals).** « Elles sont au nombre de sept »
   shows `être à` (« to belong to ») where `au nombre de` (« among ») fits; 91 such places in fr-en,
   33 in fr-es. It is about which of two overlapping expressions wins, not how a piece is written.
   Recommended: not here; a change of its own if you want it.
