# Design — add-lingua-french-expression-keys

## Context

See proposal.md (Why). How expressions are keyed and found today (`origin/main` 35faf774, change 40
merged):

| Seam | Today |
|---|---|
| `reduce_common.reduce_expressions` | a headword holding a space, each word matching the studied language's token pattern, not a proper noun, with a sense left after the edition's pointer filters → `mwe.tsv` (headword, gloss). Shared by every pair: its sha256 is in every pair's rule digest |
| `crates/lingua-pack` `expression_key` | the headword split at its spaces, each word through `lemmatize`, every lemma a lemma of the lexicon or the entry is dropped; the lemmas joined by a space are the key. Where two headwords reach one key, the one written as the key wins, then byte order (`add-lingua-expression-table` D2) |
| the pack | `expr` (FST: key → id) and `expr.zst` (glosses by id), optional and additive |
| `crates/lingua-core` `engine::match_expressions` | from each token of a selection, the longest run of 2 to 5 tokens whose lemmas, joined, are a key; a token in one match at most. `PhraseMatch.key` is that key, and the status is read on it (`knowledge.classify`) |
| `apps/lingua-extension` `selection-card.ts` | a match covering the selection heads the card with `key`, keys the card, its status and its deck entry by it (`expressionCard`); a match inside a longer selection is a row whose form is `key` (`rowsFor`); `cardGloss` stores the gloss the card showed when the card's lemma holds a space, and asks the single-lemma port otherwise |
| French, change 40 (merged, #821; French at `0.2.0`) | an elided piece is a token read as its word (`d'` → `de`, `qu'` → `que`, `s'` → `si`/`se`), `au`/`aux` → `à` + `le`/`les` sharing a span, `du`/`des` whole, an inversion read as words; D8 hands this change the keys holding `au`/`aux` or an elision, and adds `au revoir` and `coup d'œil` to the fixture |
| French, change 43 (proposed) | `au`/`aux` no word; `du`, `des` lemmas of their own; `la`, `les`, `l'` → `le` and `une` → `un` (M8); no plain word beginning with a piece is a form (`d'abord`, `c'est`, `l'on`), « the expressions among them are expression keys (change 44) » |
| French, change 41 (being proposed in parallel; not pushed when this was written or reviewed) | French's cascade, NFC in the pre-pass, closed classes (« pas »), names rule. Assumed here: it composes a token's text and changes how a token is lemmatised, never where a page is cut into tokens, and keeps `du`/`des` whole (M21); a headword goes through both, `tokenize` and `resolve_lemmas` (D1) |

**The Spanish precedent.** Spanish splits `al`/`del` into `a`/`de` + `el` (analyser `1.0.0`) and
its keys were never read that way: of es-fr's 15,133 expressions, 596 hold `al` or `del` (change
40's D8 count), 526 of them are keyed `al …`/`del …` — `al` and `del` being lemmas of
`tables/es` — and no selection reaches them. And 3,815 of the pack's 11,851 keys (3,936 of its
11,972 keyed headwords) are lemma chains unlike their headword, which its cards show:
`es-fr.golden` holds `tener en contar`, `haber que`, `dar contar`. Nothing of Spanish moves here
(open question 4).

**How it was measured.** A scratch copy of `origin/main` at 35faf774 (change 40 merged, French at
`0.2.0`), never committed, carrying a prototype of this design — first written on change 40's
implementation branch (`1d67576a`), whose code differs from the merged one by doc comments only,
and re-run on the merge with the same figures; change 43's prototype tables
(`tables/fr/forms.tsv` 124,013 forms, `freq.tsv` 60,000 lemmas); the English Wiktionary's French
section change 43 measured (`kaikki-French.jsonl`, derived on 2026-10-08 from the English dump of
2026-10-03, 510,058,226 B). The candidates are the expressions change 48's reducer would offer:
`reduce_common.reduce_expressions` as it is, with the English edition's rules and a French token
pattern (change 43's D3) — **15,524** headwords with a space — plus the headwords without a space
that hold an apostrophe or a hyphen and are no form of the prototype's table, through the same
sense filters — **1,999** — so **17,523**. The corpus is change 39's `pages-fr.txt`, each block
glossed as one selection. The fr-es cross-check reads the Spanish Wiktionary's French section
(its raw dump as change 38 records it, 103,226,106 B, regenerated 2026-10-02): 749 candidates.

## Goals / Non-Goals

**Goals:**
- A French expression's key is what the analysis makes of a page holding it, by construction.
- The card, the deck and the row name a French expression as the dictionary writes it.
- The golden shows the repair; English and Spanish do not move.

**Non-Goals:**
- Glosses. French expressions' glosses, and which headwords and senses a French reducer offers,
  are changes 48 (fr-en) and 49 (fr-es); this change keys whatever `mwe.tsv` holds.
- The card: a one-word selection over several pieces, and storing the gloss of a name without a
  space, are change 51's (D9).
- Discontinuous expressions (« il s'est **rendu compte** », `ne … pas`), as for English.
- Spanish's `al`/`del` keys and its lemma-chain names (open question 4).
- Expression-aware page highlighting: `analyse_page` is untouched.

## Decisions

### D1 — The builder keys a French headword through the core's French reading

The key is made where the reading lives: `expression_key`, for a pack studying French, hands the
headword to a core function (`analysis/pipeline.rs`, `headword_reading`) that tokenises it with
French's pre-pass and resolves each token as `analyse_document` and `gloss_phrase` do
(`resolve_lemmas`); `engine::french_expression_key` writes those tokens as D2 does and makes D4's
and D6's cuts, and the builder's French arm is that function. The phrase gloss already joins the
same tokens' readings, so a key and a selection holding its words meet by construction, as
`add-lingua-expression-table` D1 decided for the lemmatiser: one implementation, the core's. The
dependency is the one the builder already has — `lingua-pack` calls `lingua_core::analysis`'s
`lemmatize` today. English and Spanish keep the split at spaces, byte for byte (D7).

Three places were measured, on the 17,523 candidates and the corpus:

| Where the key is made | Keyed headwords | Keys | Corpus matches |
|---|---|---|---|
| Today: words split at spaces, each lemmatised (`0.2.0`) | 12,428 | 12,381 | 55 |
| A — the reducers rewrite `mwe.tsv`'s headwords as change 40 reads them | as B, if the copy is exact | | |
| **B — the builder, through the core's reading** | **14,586** | **14,512** | **69** |
| C — the matcher, on written words: a word the pre-pass splits kept as written (`d'abord`), every other word lemmatised, on both sides | 14,586 | 14,528 | 53 |

B keys 2,158 headwords today's keying cannot — 1,564 holding an elision, 457 holding `au` or `aux`
(23 of them an elision too, counted here), 101 words without a space, 36 holding an inversion — and
loses none. French's window (D4) then leaves 173 of the 14,586 out, 65 of them keyed today but
longer than today's five tokens, so never met: the pack keys 14,413 headwords.

*Rejected — A, in the tables.* The reducers would carry a Python copy of change 40's pre-pass: the
elided forms and what each reads as (`s'` → `si` before `il`), `au`/`aux`, the pronouns of an
inversion and the euphonic `t`, the runs the pack lists whole, the single-letter rule. A copy
written from change 40's spec, here, agreed with the Rust tokeniser on 17,512 of the 17,523
headwords; the 11 others (`au-dedans`, `au-devant`, `jusqu'au-boutiste`, `de l'`, `vélib'`…) are
exactly the drift `add-lingua-expression-table` D1 refused for the lemmatiser, and every later
French rule (41's NFC, any version bump) would be copied again, `reduce_common.py` being off limits
(every pair's digest). And the table would hold `à le revoir` instead of the dictionary's spelling,
which D3 needs.

*Rejected — C, at match time.* The same headwords are keyed, but a word the pre-pass splits is
compared as written, so its pieces never meet an expression on their own: an expression whose last
word is elided onto the next word is missed — « Qu’est-ce qu’il attend », « Bien qu'il pût
venir », « il n'y a » on the corpus; 365 keyed expressions end on an elidable word (`de` 258,
`que` 94) — and an inflected split word is another key (« c'était » never reaches `c'est`). The
matcher would also need the tokens' spans to rebuild written words, which `PhraseToken` does not
carry, and the builder the same rule a second time.

### D2 — A French key writes the determiners as the pre-pass gives them

Each token of a French key is its dictionary form — `bonne` → `bon`, `a` → `avoir`, `mains` →
`main`, so « il y avait » meets `il y a` as `gave up` meets `give up` — except the articles, the
possessive and demonstrative determiners and the pronouns that share their forms, written in
lowercase as the pre-pass gives them: `le`, `la`, `les`, `un`, `une`, `ce`, `cet`, `cette`, `ces`,
`mon`, `ma`, `mes`, `ton`, `ta`, `tes`, `son`, `sa`, `ses`, `notre`, `nos`, `votre`, `vos`,
`leur`, `leurs`. The pre-pass already writes the article it reads in a contraction or an elision
(`au` → `à` + `le`, `aux` → `à` + `les`, `l'` → `le`), so `au revoir` is `à le revoir` and
`à la` stays `à la`.

M8 files `la`, `les` and `l'` under `le`, `une` under `un`, and in change 43's prototype tables
`cette` and `ces` under `ce`, `sa` and `ses` under `son`, `nos` under `notre`, `vos` under `votre`,
`leurs` under `leur` — right for a word and wrong for an expression, measured:
- with every token lemmatised, `à la` (« in the style or manner of ») is keyed `à le` and answers
  every « au » and « aux »: on the corpus, 10 matches of `à la`, 7 of them on a contracted « au »
  or « aux »; written, the 3 on « à la » stay and the 7 go;
- 73 keys are reached by several headwords with every token lemmatised, 60 with the determiners
  written; the 13 kept apart include distinct expressions — `faire la course` (« race ») and
  `faire les courses` (« do the shopping »), `haut la main` and `haut les mains`, `dans le temps`
  (« in the old days ») and `dans les temps` (« on time »), `faire l'affaire` and
  `faire les affaires`, `faire la belle` and `faire le beau`;
- a key unlike its headword falls from 6,375 headwords to 5,477.

The cost: an expression whose determiner varies in use is met only as the dictionary writes it; on
the corpus, none. `du`, `des` and `de` are lemmas of their own (change 43), unaffected. The list is
French's alone and lives beside the matcher, which the builder calls (D1).

### D3 — A French expression is named by its headword

The key is a lookup, not a name. The pack carries, for each French key, the headword that won it
(D6) — written as the dictionary writes it, lowercased by the reducer — and the phrase gloss
reports it as the match's `key`: the expression's dictionary form, which the card heads, the
status is read on, and the deck and the row show. The extension is unchanged: it already shows the
`key` it is handed. A key reached by « il y avait » is named `il y a`; « Au revoir », `au revoir`;
« de bonne heure », `de bonne heure` where `fr-en.golden` shows `de bon heure` today.

**The section.** `expr.names.zst`, optional and additive as the container requires (*Versioned
pack container, keyed by language pair*): zstd, offset-indexed by expression id, in the layout of
`expr.zst`, holding only the names that differ from their key — a key that is its own headword has
none, and the phrase gloss falls back on the key. Written for a pack studying French only, so no
English or Spanish pack gains a byte. Measured on the prototype: 5,260 of the 14,352 keys (36.6 %)
carry a name, 49,990 B; every name stored would be 120,240 B. `Pack::expression_name(key)` reads
it. No name is another key's name or key (measured: none), so a name stands for one expression of
a pack, as a key does.

**Review.** A card is created with the name as its lemma, and *Review shows a gloss the reader can
read* (M4) reads an expression card's gloss in the table at that lemma (`readable_gloss`,
`crates/lingua-wasm`): `au revoir` is no key, so a French card glossed in another language would
show its own text where the pack holds a gloss. For a French card whose lemma holds a space, review
reads the table at the key its name reads as — `french_expression_key` on the pack's lexicon, the
builder's own function — and every other card as before; no baseline holds such a card, so no
golden moves. A name without a space (`d'abord`) is no expression to `Card::is_expression`: with
its stored gloss, change 51's (D9).

**What a name costs.** A key is the lexicon's, which every French pair shares (*A studied
language's tables are kept once*); a name is the winning headword of one dictionary, and fr-en and
fr-es are written from two Wiktionaries. Where they win different headwords for one key, a status
or a card made under one native language's name is not found under the other's. Measured: of the
426 keys that both fr-en's 17,523 and fr-es's 749 candidates reach, 8 are named differently —
`s'il vous plait` and `s'il vous plaît`, `allez-y` and `vas-y`, `ça ira` and `ça va`,
`œuf brouillé` and `œufs brouillés`… (open question 1).

*Rejected — keeping the lemma chain, as English and Spanish do.* `à le revoir`, `il y avoir`,
`de bon heure` and `coup de œil` are not French, and a learner would review them in the deck.
*Rejected — a third column in `mwe.tsv`.* The headword is already the first column; the key is the
builder's.

### D4 — French's window is seven tokens

`EXPRESSION_WINDOW` (5) was chosen to hold 98.9 % of the English table (`add-lingua-expression-table`
D3). French's pieces make keys longer — 198 headwords of five written words or fewer read as more
than five tokens (`au fur et à mesure`: six). Of the 14,525 French keys:

| Tokens | 2 | 3 | 4 | 5 | 6 | 7 | more |
|---|---|---|---|---|---|---|---|
| Keys | 5,196 | 5,984 | 1,908 | 679 | 428 | 157 | 173 |
| Up to here | 35.8 % | 77.0 % | 90.1 % | 94.8 % | 97.7 % | 98.8 % | 100 % |

French reads seven (98.8 %), English and Spanish keep five. A key longer than the window is never
met, so the French builder leaves it out — 173 keys — rather than carry it, as the lexicon test
already leaves out a key no reading can produce. A selection is at most 120 characters: two more
probes per token.

### D5 — `du` or `des` closing a run may stand for `de`

M21 keeps `du` and `des` whole, so the 258 keyed expressions ending on `de` (`à cause de`,
`au bord de`, `près de`, `au lieu de`) never meet « à cause des » or « au bord du », the
contraction of their `de` with the article that follows. In a French run that no key matches and
whose last token reads `du` or `des`, that token is read once more as `de`; a match covers it. The
retry is the phrase gloss's, the token keeps its lemma, and nothing of a page's analysis moves.

Measured on the corpus: of its 18 « du » and « des » after a word, the retry adds the two that
close such an expression (« à cause des », « au bord du ») and none on a partitive or indefinite
article (« acheter du », « mangé des », « bu du »): after a word that governs `de`, French drops
the partitive and the indefinite plural (« beaucoup de vin »), so `du` and `des` there are the
contraction. The known exception is an expression ending on `de` before an indefinite `des` that
French keeps — `pas de` in « ce ne sont pas des enfants » — none on the corpus.

### D6 — Which headwords are expressions, and which wins a key

A French entry is kept when its headword reads as **two to seven tokens**, every token's dictionary
form a lemma of the lexicon, and **every word of the headword gives a token**. So:
- a word without a space that the pre-pass splits is an expression: `d'abord` (`de abord`),
  `c'est`, `allez-y`, `jusqu'à`, `s'agir` — 101 of the 1,999 such candidates are keyed;
- a word read as one token never is: a compound the pack lists (`peut-être`, `c'est-à-dire`) or not
  (`abat-jour`), `aujourd'hui`;
- a headword part of which the analysis drops is left out — a single letter the lexicon does not
  list, or a word with a digit: 5 headwords, all single letters (`compte en t`, `vitamine c`,
  `fiche s`…), of which `compte en t` would otherwise key `compte en`.

Which headwords a reducer offers is the reducer's (change 48, 49): `reduce_expressions` takes
headwords with a space, and the 101 single words come from candidates it does not read. Among the
14,413 kept headwords, 43 only point at other words — 26 spell out their pieces (`qu'elle`
« que + elle », `jusqu'au` « jusque + au », `s'est` « se + est »), 14 point at a spelling
(« post-1990 spelling of crème fraîche »), 3 at an inverted form (`puis-je`) —, 29 of them among the
101 single words. Their senses are pointers the English edition's rules do not read today, and
change 48's reducer leaves them out with them (D9).

**Several headwords, one key.** The headword written as the key keeps it, as for English; then the
one with the most tokens written as their key piece (`boîte à gants` over `boite à gants`, whose
`boite` is a spelling of `boîte`); then the first in byte order. Measured: of the 60 keys reached by
several headwords, 29 go to the headword written as the key, 12 to the most written, 19 to byte
order. Of those 19, 12 are equally right (a phrase's two persons, `dis donc` and `dites donc`), and
7 pick the less usual headword: three post-1990 spellings whose only sense points at the
traditional one (`crème fraiche`, `s'il vous plait`, `s'il te plait`), which change 48 leaves out so
that the traditional spelling stands alone; `s'occuper de ses ognons` over `… oignons`; and three
tenses of one phrase over its present (`quel que fût`, `quoi qu'il en fût`, `ça ira` over `ça va`),
where one lemma merges two headwords (Risks).

The fr-es cross-check: of the Spanish Wiktionary's 749 French candidates, today's keying keeps 480,
this design reads 619 (601 keys) and keeps the 611 within French's window (593 keys) — the same
rule, whichever reducer feeds it.

### D7 — What does not move, and the version

- **English and Spanish.** The builder's French arm and the matcher's French rules run for French
  alone; the order among headwords of one key is unchanged for the others. Measured on the
  prototype, without re-blessing: `english_baseline` (en-fr), `spanish_baseline` (es-fr),
  `es_en_baseline`, `en_es_baseline`, `cross_native` and `parity` pass, 25 tests in 6 suites;
  `committed_tables` builds the four committed packs to the sha256 their pins record; every
  lingua-core and lingua-pack test passes.
- **French's analyser version does not move.** No token and no lemma changes: `analyse_page` is
  untouched, and no `analyse` probe of the golden moves. The keys are pack data built by the core
  that ships with the pack (*Packs stay embedded, one package per store*), and the names section is
  additive, so the container's rule applies — `pack_version` moves, `analyzer_version` stays, as
  `add-lingua-expression-table` D5 decided for the table itself. A French pack built by this
  change and read by a core before it would miss the expressions holding a written determiner,
  longer than five tokens or reached through `du`/`des`, and show lemma chains; no French pack is
  published before change 52, and none is read by another core than the one built beside it. The
  container needs no new format: `Pack::load` ignores a section it does not know
  (`a_pack_carrying_an_unknown_section_loads_unchanged`), and `FORMAT_VERSION` stays 1. This
  change names no version number, so changes 41 and 42's bumps cannot contradict it.
- **The baseline's reason.** *A French invariance baseline runs beside the English and Spanish ones*
  asks a pull request moving the golden to say why, and names three reasons: a French rule that
  bumps French's analyser version, the fixture replaced by the committed tables, the beside pack's
  update. This change is none of them — a rule of the builder and the phrase gloss that leaves the
  analysis, hence its version, alone, and three expressions and three forms added to the fixture —
  so *A French rule lands* is not its case either. Change 39's design expects changes 40 to 47 to
  re-bless the golden over the fixture; the requirement is changes 39 and 40's, both open, so it is
  not MODIFIED here: the pull request says why, and the wording is the owner's (open question 5).
- **The requirements this one narrows.** `add-lingua-expression-table` (open) holds *Multi-word
  expression table* — a key is « the sequence of dictionary forms of its words », by the core's
  lemmatiser — and *Expression lookup in a phrase gloss* — runs of « dictionary forms », « over a
  bounded number of tokens ». For a French pack D1, D2, D4 and D5 replace both rules. A requirement
  another open change holds is not MODIFIED, so this change's two ADDED requirements say, each,
  that they take the place of those rules for French, every other rule of theirs still applying
  (open question 5).

### D8 — The fixture, the probes, and what `fr-en.golden` shows

The fixture (`scripts/lingua-data/testdata/fr-en/`): `mwe.tsv` gains `à la` (« in the style of »),
`d'abord` (« first, at first ») and `au fur et à mesure` (« as one goes along »); `forms.tsv` gains
`abord`, `fur`, `mesure`; `manifest.json`'s `pack_version` becomes `0.0.2-fixture` (a new section).
The scenario's phrases gain « D’abord », « au fur et à mesure », « il y avait » and « à la maison ».

Re-blessed on the prototype, against `fr-en.golden` as change 40 merged it (35faf774, French at
`0.2.0`): **6 of its 141 probes move, 4 are added, 135 are byte for byte.**
- `pack`: 5,139 → 5,549 bytes, `pack_version "0.0.2-fixture"`;
- « il y a »: `il y avoir` → `il y a` (D3);
- « Longtemps, je me suis couché de bonne heure. »: `de bon heure` → `de bonne heure` (D3);
- « Au revoir »: none → `au revoir` (D1);
- « un coup d’œil »: none → `coup d'œil`, covering `coup` to `œil` (D1);
- « à cause des »: none → `à cause de` (D5);
- added: « D’abord » → `d'abord` (D6), « au fur et à mesure » → one match over six tokens (D4),
  « il y avait » → `il y a` (D3), « à la maison » → `à la` (D2) — the rule, not the sense: « in
  the style of » is no reading of « à la maison », and which such entries a reducer keeps is
  change 48's;
- byte for byte: every `analyse`, `gloss` and `word-grammar` probe, the levels, the review, the
  exports, the backup, and « au marché » and « jusqu'au soir », which answer no `à la`.

Each rule shows in the diff, measured by switching it off: without D2, « au marché » and « jusqu'au
soir » answer `à la` « in the style of »; without D4, « au fur et à mesure » answers nothing;
without D5, « à cause des » answers nothing.

On the real data the corpus, glossed block by block, finds 62 expressions with this design, against
55 with today's keys: `c'est`, `qu'est-ce que`, `jusqu'à ce que`, `d'ici là`, `d'abord`,
`s'en aller` (« s'en va »), `y a-t-il`, `est-ce que`, `au bord de` (« au bord du »), `à cause de`
come, the 7 `à la` on « au » and « aux » go, and the cards read `de bonne heure`, `salade verte`,
`entendre parler` (« entendu parler »). Eight of the 62 fall on pointer entries change 48 leaves
out (`qu'elle`, `s'est`, `j'ai`…, D6).

The figures are re-measured on the implementation's base, after changes 41, 42 and 43: 41's cascade
may lemmatise a piece differently, which moves a key and the reading together.

### D9 — What later changes take from here

| Change | Takes |
|---|---|
| 41 analysis | nothing to do: the keys follow its NFC and cascade through `tokenize` and `resolve_lemmas`; its function words leave the keys alone (a match covers function words; the rows leave them out) |
| 48 fr-en | the expressions' glosses; French's candidates — the headwords with a space, and the words without one the pre-pass splits (D6) —, read by its own reducer since `reduce_common.py` cannot be edited; the 43 pointer senses left out (« X + Y », « post-1990 spelling of », an inverted form), and the entries made of function words whose sense a page rarely means (`à la` « in the style of », D8); the golden's hand-over to the committed tables, which shows the real expressions |
| 49 fr-es | the same, from its sources |
| 51 word card | a one-word selection over several pieces opens the whole-selection card (change 40's D7, as carried), where `d'abord` and `c'est` answer; `cardGloss` stores the gloss the card showed for an expression whose name holds no space (today it asks the single-lemma port, which knows no `d'abord`), and review reads such a card as an expression (D3) |
| 52 enable | the dogfood: an expression card on real pages in fr-en and fr-es |

## Risks / Trade-offs

- [A determiner varies inside an expression] → Met only as the dictionary writes it (D2); none on
  the corpus. The alternative merges distinct expressions and answers every « au » with `à la`.
- [`pas de` before an indefinite `des`] → Read as `pas de` (D5); none on the corpus, and the
  expression's row is « without » beside the sentence the reader holds.
- [A lemma merges two meanings] → `de fait` (« in fact ») is keyed `de faire` (M8: `fait` →
  *faire*) and answers « de faire »; `ça ira` and `ça va` share `ça aller`. The same cost as
  English's `breaking point`/`break point`, and change 43's M8 report names the nouns it touches.
- [Pointer senses become reachable] → 43 of 14,413 (D6); left out by change 48's reducer. Until
  then only the fixture is keyed.
- [A French pack read by a core before this change] → Lemma-chain names and the written
  determiners' keys missed (D7); no French pack is published before change 52.
- [Change 41 moves a lemma] → Key and reading move together; the golden's re-bless in 41 or 44,
  whichever lands second, shows it.
- [Two headwords on one key, the wrong one named] → 7 of the 60 such keys (D6): three post-1990
  pointers change 48 leaves out, one post-1990 spelling, three tenses of a phrase that one lemma
  merges.
- [fr-en and fr-es name one key differently] → A status or a card made under one native language is
  not found under the other: 8 of the 426 keys both reach (D3); open question 1.
- [A French card reviewed in another language] → Read at its name's key (D3); a name without a
  space shows the card's own text until change 51.

## Migration Plan

Nothing to migrate: no stored format, wire field, table, pin or committed pack moves, and no reader
studies French. A French pack gains an optional section. Rollback is a revert.

## Effort

2–2.75 ideal days, against the programme's 1–2.5: the reading of a headword, the key pieces, the
window, the `du`/`des` retry and the names in the core 0.5–0.75; the builder's French arm, the
names section, the order and the cut 0.5–0.75; review's lookup by name and its test 0.25; their
tests 0.25–0.5; the fixture, the probes and the re-bless 0.25; spec and programme 0.25.

## Open Questions

For the owner, none blocking:
1. **Names (D3).** A French expression card headed by the dictionary's spelling (`au revoir`,
   `il y a`) rather than its lemmas (`à le revoir`, `il y avoir`), as English and Spanish cards are
   headed — at the cost of a status keyed by a dictionary's headword: 8 of the 426 keys fr-en and
   fr-es both reach are named differently, and a status set under one is not read under the
   other.
2. **Determiners written (D2).** `à la` no longer answers « au », and `haut la main` and
   `haut les mains` stay two; an expression whose determiner varies is met only as written.
3. **`du`/`des` read as `de` at the end of an expression (D5).** « à cause des » answers
   `à cause de`; `pas de` before an indefinite `des` is the known misreading.
4. **Spanish.** The same rule would let es-fr's 526 keyed headwords holding `al`/`del` be reached
   and name its 3,815 lemma-chain keys by their headword (`tener en cuenta`, not `tener en contar`);
   it moves es-fr and es-en output, so it would be a change of its own, re-blessing both with the
   owner's approval — not in the programme's 57.
5. **Held requirements' wording (D7).** *Multi-word expression table* and *Expression lookup in a
   phrase gloss* (`add-lingua-expression-table`) state their key and match rules for every pack, and
   *A French invariance baseline runs beside the English and Spanish ones* (changes 39, 40) names
   three reasons for moving the golden, none of them this change's. Their changes are open, so
   nothing here MODIFIES them: this change's ADDED requirements take the place of the first two's
   rules for French, and its pull request says why the golden moves. Whether their wording is
   narrowed when they archive is the owner's.
