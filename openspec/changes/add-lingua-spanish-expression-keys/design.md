# Design — add-lingua-spanish-expression-keys

## Context

See proposal.md (Why). How Spanish expressions are keyed and found on `origin/main` bac5cc51 (change
44 merged as #835):

| Seam | Today |
|---|---|
| `tables/es-fr/mwe.tsv`, `tables/es-en/mwe.tsv` | 15,133 and 15,515 headwords, lowercase letters, spaces and hyphens, each with its gloss; written by each pair's reducer, untouched here |
| `crates/lingua-pack` `expression_key` | French: `french_expression_key` (change 44). English and Spanish: the headword split at its spaces, each word through `lemmatize`, every lemma a lemma of the lexicon; the lemmas joined. No window: a key of any length is kept. Among headwords of one key, the one written as the key, then (French only) the most tokens written as their key piece, then byte order |
| the pack | `expr`, `expr.zst`; `expr.names.zst` for a French pack only |
| `crates/lingua-core` `engine::match_expressions` | Spanish: runs of 2 to 5 tokens (`EXPRESSION_WINDOW`) whose lemmas, joined, are a key; the match reports the key, the status read on it |
| the Spanish pre-pass (*Spanish tokenisation pre-pass*, analyser `1.2.0`) | NFC; `al` → `a` + `el`, `del` → `de` + `el`, the two tokens sharing the written word's span; a hyphenated compound one token |
| `tables/es/forms.tsv` (M8) | `la`, `los`, `las` → `el`; `una`, `unos`, `unas` → `uno`; `al` → `al` and `del` → `del`, lemmas of their own; `esta`, `estos`… → `este`; `mis` → `mi`, `nuestra`… → `nuestro`; `cuenta` → `contar` |
| `crates/lingua-wasm` `readable_gloss` | a Spanish expression card is read in the table at its lemma |
| `apps/lingua-extension` `selection-card.ts` | a match covering the whole selection (`start == 0`, `end == tokens.length`) heads the card with `key`, keys the card, its status and its deck entry by it; a match inside a longer selection is a row whose form is `key` |

**The figures today.** es-fr keys 11,972 headwords on 11,851 keys, es-en 13,889 on 13,739. Of
those, a selection reaches only the keys of five tokens or fewer that hold no `al` or `del` as a
word: es-fr **11,199 headwords on 11,081 keys**, es-en **12,966 on 12,822**. The 526 es-fr keys and
607 es-en keys holding `al` or `del` as a word are never met, nor the 302 es-fr headwords keyed on
more than five tokens. 3,815 of es-fr's keys and 4,665 of es-en's are not their headword, and the
card shows them (`es-fr.golden`: `tener en contar`, `dar contar`, `a el` on « a la casa »).

**How it was measured.** A scratch copy of `origin/main` at bac5cc51, never committed, carrying a
prototype of this design (the core, the builder and review, about 120 lines), measured on the
committed `tables/es`, `tables/es-fr` and `tables/es-en` and on the Spanish baseline's corpus
(`crates/lingua-wasm/tests/baseline/pages-es.txt`, 13 pages), each block glossed as one selection
through the real `gloss_phrase`. Every variant below was built into real packs; the goldens were
re-blessed on the prototype, and each rule switched off once to see it in the diff (D8). Every
figure here was measured again on `origin/main` f43c6035 (changes 42 and 50 merged), the same,
the re-blessed goldens byte for byte the prototype's but for `fr-en.golden`, which moves on its
`beside es-en` line alone there too.

## Goals / Non-Goals

**Goals:**
- A Spanish expression's key is what the analysis makes of a page holding it, as change 44 made
  French's.
- The card, the deck and the row name a Spanish expression as the dictionary writes it.
- A reader who settled an expression before keeps it settled.
- The goldens show the repair; English, French and the en-es pair do not move.

**Non-Goals:**
- Glosses, and which headwords and senses the reducers offer (`mwe.tsv` is untouched, as are the
  reducers and their pins' rules).
- A one-lemma merge of two expressions (`dar cuenta`, `darse cuenta`), as for English and French
  (Risks).
- French's own `au`/`aux` at the end of a match (open question 3).
- Expression-aware page highlighting: `analyse_page` is untouched.

## Decisions

### D1 — The builder keys a Spanish headword through the core's Spanish reading

Change 44 wrote the reading for French: `headword_reading` (`analysis/pipeline.rs`) tokenises a
headword with the studied language's own pre-pass and resolves each token as a page's, and
`french_expression_key` (`engine.rs`) writes those tokens as key pieces, makes the window cut and
asks the lexicon whether every dictionary form is a lemma. `headword_reading` already serves every
language (its test reads `al pie de la letra` as Spanish). This change makes `french_expression_key`
one function for the languages read through their reading — `reading_expression_key(headword,
studied, lexicon)`, French's arm byte for byte, the list of written words and the window chosen by
the studied language — and the builder's Spanish arm calls it, as its French arm does. English
keeps the split at spaces, byte for byte.

| Where the key is made | es-fr: keyed / reachable (keys) | es-en: keyed / reachable (keys) |
|---|---|---|
| Today: words split at spaces, each lemmatised, window 5 | 11,972 / 11,199 (11,081) | 13,889 / 12,966 (12,822) |
| **The core's reading, D2's written determiners, window 7** | **11,887 / 11,887 (11,788)** | **13,826 / 13,826 (13,685)** |

The reading keys exactly the headwords today's keying does — every word of the tables' headwords
gives a token, so change 44's « every word a token » rule leaves none out — and makes all of them
reachable but the 85 (es-fr) and 63 (es-en) longer than seven tokens (D4), none of which a
selection reaches today. It reaches **688 headwords** es-fr does not reach today, **860** for es-en,
among them the `al`/`del` keys (495 and 590 keys within the window). It loses none.

*Rejected — `al` and `del` split in the builder, at spaces, as today.* A second copy of the
pre-pass's contraction rule, the drift `add-lingua-expression-table` D1 and change 44's D1 refused;
and the builder would still not read what the page's reading reads (NFC, a compound kept whole, a
single letter dropped). One function, the core's, for the key and the page.

### D2 — A Spanish key writes its determiners as the pre-pass gives them

Each token of a Spanish key is its dictionary form, except 35 words written in lowercase as the
pre-pass gives them (`SPANISH_KEY_WRITTEN`, sorted): the articles `el`, `la`, `los`, `las`, `lo`,
`un`, `una`, `unos`, `unas`; the possessive determiners `mi`, `mis`, `tu`, `tus`, `su`, `sus`,
`nuestro`, `nuestra`, `nuestros`, `nuestras`, `vuestro`, `vuestra`, `vuestros`, `vuestras`; the
demonstratives `este`, `esta`, `estos`, `estas`, `ese`, `esa`, `esos`, `esas`, `aquel`, `aquella`,
`aquellos`, `aquellas`. French's D2, transposed: the list holds what M8 files under another word
(`la` → `el`) and, for completeness, the forms that are their own lemma. The `el` the pre-pass
writes for « al » and « del » is written `el`, so `al menos` is `a el menos` and `a la vez` stays
`a la vez`.

Measured, four variants of the reading (window 7):

| Written as the pre-pass gives them | es-fr: keys several headwords reach | corpus matches | es-en: keys several headwords reach | corpus matches |
|---|---|---|---|---|
| nothing (every token its lemma) | 116 | 44 | 151 | 43 |
| the articles | 99 | 33 | 138 | 28 |
| **the articles and the possessive and demonstrative determiners** | **98** | **32** | **138** | **27** |
| the same and the clitic pronouns (`me`, `te`, `se`, `nos`, `os`, `le`, `les`) | 98 | 32 | 138 | 27 |

With every token lemmatised, `a la`, `a las` and `a los` stay one key (`a el`), `de las` and `de
los` another (`de el`), and the `al` split adds new merges: `a las armas` and `al arma`, `a la par
de` and `al par de`, `a la descubierta` and `al descubierto`. On the corpus, against every token
lemmatised, the determiners written take away, for es-fr, the 11 matches of an article entry on
another article or a contraction (`de las`/`de los` « Des » on « del », « de la », « de La »; `a la`
« À la » on « al ») and « en esta » answering `en este` (44 → 32); for es-en (43 → 27), its `de la`
« of the » on « del », « de las » and « de los » (10), `a la` on « al » and « a las » (3), `la
vieja` (« a three-note melody ») on « El viejo », `de esta` (« definitely this time ») on « de este
», and `los más` (« most ») on « las más ». The clitic pronouns change nothing measurable and are
left out, as French's list leaves out `me`, `te` and `se`: they are pronouns, not determiners; the
`la`, `lo`, `los` and `las` they share with the articles are written already.

The cost, as French's: an expression whose determiner varies in use is met only as the dictionary
writes it — on the corpus, `los más` on « las más », and es-en's `de la` « of the », a
function-word row, on « del ». The list is Spanish's alone and lives beside French's (D1).

### D3 — A Spanish expression is named by its headword

As change 44's D3, for Spanish: the pack carries, for each key unlike the headword that won it,
that headword in `expr.names.zst`; the phrase gloss reports it as the match's `key`, the status is
read on it, and the card, the deck and the row show it. The extension is unchanged.

| Pack | Keys | Named | `expr.names.zst` | Pack |
|---|---|---|---|---|
| es-fr | 11,788 | 3,750 (31.8 %) | 37,471 B | 2,190,188 → 2,224,439 B (+34,251) |
| es-en | 13,685 | 4,438 (32.4 %) | 42,380 B | 2,567,804 → 2,608,413 B (+40,609) |

(`expr` and `expr.zst` shrink slightly: es-fr 163,092 → 161,128 and 116,065 → 114,790 B, es-en
175,668 → 174,668 and 159,753 → 158,963 B.) Both stay far under the 5 MiB budget.

**The order among headwords of one key** becomes French's for Spanish: the headword written as the
key, then the one with the most tokens written as their key piece, then byte order. Measured, it
changes 4 winners, each to the dictionary's own spelling: es-fr `roma no se hizo en un día` over
`… un dia`; es-en `feo del baile` over `fea del baile`, `soy inglés` over `soy inglesa`,
`yo soy británico` over `yo soy británica`. Byte order alone would name those keys by the
feminine or the unaccented spelling.

**Names across natives.** A key is the lexicon's, which es-fr and es-en share (`tables/es`); a name
is the winning headword of one dictionary. Of the 3,757 keys both packs reach, **75** are named
differently (`dar cuenta` / `darse cuenta`, `aguas negras` / `agua negra`, `en ayunas` /
`en ayuno`, `así sea` / `así es`): a status set under es-fr's name is not read by an es-en engine.
It is change 44's cost for French (its open question 1, settled by the owner: a status keyed by the
name), here measured for Spanish.

**What names show that keys hid.** A key two expressions reach shows one name: es-fr holds
`dar cuenta` (« Rendre compte ») and `darse cuenta` (« Se rendre compte »), both keyed `dar contar`
(`darse` read `dar` by the enclitic rule); « darse cuenta » answered `dar contar` « Rendre compte »
and now answers `dar cuenta` « Rendre compte » — the same gloss, a truer heading (Risks).

### D4 — Spanish's window is seven tokens

Reading `al` and `del` as two tokens lengthens keys: « al fin y al cabo » is five words and seven
tokens. Of the keyed headwords, by tokens:

| Tokens | 2 | 3 | 4 | 5 | 6 | 7 | more |
|---|---|---|---|---|---|---|---|
| es-fr headwords | 5,858 | 4,041 | 1,324 | 410 | 172 | 82 | 85 |
| up to here | 48.9 % | 82.7 % | 93.7 % | 97.2 % | 98.6 % | 99.3 % | 100 % |
| es-en headwords | 6,431 | 5,173 | 1,511 | 374 | 224 | 113 | 63 |
| up to here | 46.3 % | 83.5 % | 94.4 % | 97.1 % | 98.7 % | 99.5 % | 100 % |

English's five tokens hold 98.9 % of its table (`add-lingua-expression-table` D3); six hold 98.6 %
of es-fr's, seven 99.3 %. Spanish reads seven, as French does, and the builder leaves a longer key
out (85 es-fr headwords, 63 es-en, mostly proverbs and long names, none of which a selection
reaches today). English keeps five.

### D5 — A Spanish match never ends inside a written word

The `a` of « al » and the `de` of « del » are tokens of their own, so an expression ending on `a` or
`de` is found without a retry (`cerca de` in « cerca del río »): the opposite of French, where
`du`/`des` stay whole and need change 44's D5. But the match covers `cerca`, `de` and stops before
the `el` that shares `del`'s span, so a selection of the written words — « después del » — is
three tokens and a match of two: the card shows a row, not the expression's answer, though the
reader selected exactly it. A Spanish match whose last token is followed by a token sharing its
span (the article of `al`/`del`) covers that token too.

Measured: on the corpus, 5 of es-fr's 32 matches end on the first piece of a contraction
(« antes del », « Vamos al », « después del », « al final del », « cerca del ») and 2 of es-en's; no
match starts on the second piece. The cost: an expression beginning with `el` can no longer start
on the article of a contraction a match ends on — none on the corpus. The phrase gloss knows which
tokens share a span (it tokenises the selection); `PhraseToken` and the JSON are unchanged, the
match's `end` alone moves.

### D6 — An expression the reader settled before keeps its key

es-fr is read. Until this change the phrase gloss reported a Spanish expression by its run's
lemmas, joined — the key — and the reader's status, and the deck card the « + Deck » gesture makes
(it sets « learning »), were written on that string. Measured: of the 11,199 headwords es-fr
reaches today, **3,538** would be reported under another string (3,499 distinct strings today);
es-en, not shipped, 4,333 of 12,966 (4,282 strings). Glossing every headword as a selection through
today's and the prototype's `gloss_phrase` gives the same strings: 3,543 and 4,337 headwords, the
extra 5 and 4 reached today only through another headword's key (« al arma » answering
`a las armas`'s `a el arma`). A reader who marked `tener en contar` known would see
`tener en cuenta` unknown, and « + Deck » would make a second card.

So, in a Spanish match: when the reader holds a record on the run's lemma chain — a status, or a
withdrawn one: an explicit status or a sync stamp, whatever its time, as `KnowledgeState`'s
`is_withdrawn` reads it (the prototype's `status_updated_at != 0` misses a withdrawal stamped 0) —
and none on the name, the match reports the lemma chain as its key and reads its class there. The
chain is computed on the run before D5's extension, so it is exactly what the phrase gloss reported
for those tokens before. The card is then headed, keyed and acted on as it was: its status, its
deck card, « Je connais » and the retirement of the card all reach the same record. Every expression the reader
never settled is named (D3). No record is rewritten and nothing new is synced; the knowledge model
is per studied language, so the rule is Spanish's, whatever the native language.

Across devices: a status pulled from the server goes through `applyStatusChanges` →
`KnowledgeState::apply_status_lww`, which stores it, or a withdrawal's stamp, exactly as a local
gesture does, and the server's rows are keyed by `(user_id, language, lemma)`
(`backend/lingua` `word_statuses`), the native language no part of them; a sync applies the
statuses before the cards (`sync.ts`). So a chain settled before the update, or on a device not yet
updated, is found on every device. Measured on the prototype with records pulled through
`apply_status_lww`: `tener en contar` known keeps « tener en cuenta » on it, known; `a el vez`
withdrawn keeps « a la vez » on `a el vez`; `a el` learning answers « a los » as `a el`.

Measured on the prototype, a reader holding `tener en contar` known and `a el vez` learning:
« tener en cuenta » answers `tener en contar`, known; « a la vez » answers `a el vez`, learning (the
key is now `a la vez`, D2, the record is still found); a reader holding neither sees `tener en
cuenta` and `a la vez`, unknown. A record on a chain D2 splits into several keys (`a el`, which
`a la`, `a las` and `a los` shared) answers all of them, as today.

*Rejected — a migration of the stored records.* A card's `client_id` is its lemma: renaming
`tener en contar` to `tener en cuenta` is a removal and a creation on every device and on the
server, the card's review history at risk, for a reader who may have none. *Rejected — the class
read on the chain, the key reported as the name.* The card's gestures act on the key it shows:
« Je connais » would write on the name, and the old card would keep coming due. *Accepting the loss*
is open question 1.

### D7 — Review finds a Spanish expression card by its name

A card made after this change is named (D3): `tener en cuenta` is no key, so *Review shows a gloss
the reader can read* (M4), which reads an expression card's gloss in the table at its lemma, would
show the card's own text. For a Spanish card whose lemma holds a space, `readable_gloss` reads the
table at the lemma, then at the key the lemma reads as (`reading_expression_key`, D1): a card made
before (its lemma today's key) is found as before, a named card through its key. Measured: of the
11,081 keys es-fr reaches today, 10,334 are still keys; the other 747 (es-en: 1,134 of 12,822) held
an article D2 now writes (`a el vez`, now `a la vez`): a card made before on one of them, reviewed by
an engine of another native language, shows its own text, as a gloss the pack has not. No baseline
holds such a card, so no golden moves.

### D8 — What moves, what does not, and what the goldens show

**`es-fr.golden`**, re-blessed on the prototype: **8 of its 133 sections move — its `pack` line and
7 phrase probes —, 4 probes are added, 125 sections are byte for byte.**
- `pack`: 2,190,188 → 2,224,439 bytes (`pack_version` and `analyzer_version` unchanged);
- « tener en cuenta »: `tener en contar` → `tener en cuenta` (D3);
- « darse cuenta »: `dar contar` → `dar cuenta`, the same gloss (D3);
- « al aire libre »: `a el` « À la » over two tokens → `al aire libre` « En plein air, à la belle
  étoile » over four (D1);
- « Fue a la casa de su abuela. »: `a el` → `a la` (D2);
- « En un lugar de la Mancha », « El vino tinto vino de La Rioja. », « del barrio »: their `de el`
  « Des » goes (D2);
- added: « al menos » → `al menos` (today `a el` « À la »), « a la vez » → `a la vez` (today
  `a el vez`), « después del » → `después de` over its three tokens (today two), « al fin y al
  cabo » → one match over seven tokens (today `a el` « À la » twice).

**`es-en.golden`**: **8 of its 172 sections move, the same 4 probes are added, 164 are byte for
byte.** The
`pack` line (2,567,804 → 2,608,413 bytes); `tener en cuenta`; « darse cuenta » → `darse cuenta`
(es-en holds no `dar cuenta`); « al aire libre » → `al aire libre` « outdoors (in the open air) »
over four tokens, where `a el` and `aire libre` answered; `de el` → `de la` « of the » on « de la
Mancha » and « de La Rioja »; `a el` → `a la` on « a la casa »; « del barrio »'s `de el` « of the »
goes.

**`fr-en.golden`**: its `beside es-en` line alone (2,567,804 → 2,608,413 bytes) — « the beside
pack's update », a reason *A French invariance baseline runs beside the English and Spanish ones*
names.

**Pins**: `tables/es-fr/pin.json` and `tables/es-en/pin.json`, the `pack`'s sha256 and size only
(prototype: `3a52b7d1…`, 2,224,439; `f13020a3…`, 2,608,413), as #690 moved es-fr's pin.

**Not moved, measured on the prototype without re-blessing:** every `analyse`, `gloss` and
`word-grammar` probe and the tokens of every phrase probe (each moved probe's `tokens` byte for
byte); `english_baseline` (en-fr), `en_es_baseline`, `cross_native`, `parity`,
`card_gloss_language`, `statuses` and every other lingua-core, lingua-pack and lingua-wasm test but
two unit tests that assert Spanish is keyed as before, rewritten here (the Spanish line of
`a_french_key_writes_its_determiners_and_lemmatises_the_rest`, and
`an_english_or_spanish_pack_carries_no_names`);
`committed_tables` once the two pins move, en-fr's, en-es's and fr-en's pins included; the English and
Spanish fixtures (`pipeline_testdata`: the Spanish fixtures hold no expression); the reducers' 405
Python tests. The extension's snapshots — `word-card-es-en.txt` (the es-en golden's `word-grammar`
probes), `selection-rows-fr.txt` (the committed glosses), `word-card-en-es.txt` and
`voice-ranking.txt` — were run on the prototype's goldens and pass without `-u`: they read nothing
this change moves.

**Each rule shows in the diff**, measured by switching it off on the prototype: without D2,
« del barrio » answers `de las` « Des » (es-fr) and `de la` « of the » (es-en), and « de la Mancha »
and « de La Rioja » answer es-fr's `de las`; with a window of five, « al fin y al cabo » answers
`al fin` (es-fr) or `al fin` and `al cabo` (es-en); without D5, « después del » covers two tokens of
three. D6 shows on no probe — the baselines' reader holds no expression — and is pinned by its own
tests (tasks 1.4, 4.4).

**On the corpus**, es-fr finds 32 expressions against 45 today, es-en 27 against 43: the article
entries' matches on another article or a contraction go (D2), `al final de` answers « al final
del » where `a el` and `de el` did, 5 (es-fr) and 2 (es-en) matches cover the article of the
contraction they end on (D5), and `hacerse tarde`, `buen tiempo`, `nos vemos`, `en las últimas`
head the cards where `hacer tarde`, `bueno tiempo`, `nos ver`, `en el último` did. The figures are
re-measured on the implementation's base.

### D9 — The version, `pack_version`, and the requirements this one narrows

- **Spanish's analyser version does not move.** No token and no lemma changes (D8); the keys and the
  names are pack data built by the core that ships with the pack (the programme's *Packs stay
  embedded, one package per store*), and D5 and D6 change which tokens a match covers and which
  string it reports, not the analysis. A pack built here read by a core before it would miss the keys holding a written
  determiner or longer than five tokens and show lemma chains; no such pairing ships. This change
  names no version number.
- **`pack_version` does not move, and the container says so.** *A pack says which dictionary it is*
  identifies the tables, and the tables do not move (*Two releases, one dictionary*); the reducer
  writes `pack_version` from the pin's snapshot and rules (`pack_sources.py version`, checked by
  `test_pack_sources.py`), so a builder change cannot move it without inventing a version the
  tables do not have. What records the new bytes is the pin's `pack` (*Reproducible offline build*:
  a pull request that changes the builder so that the pack's sha256 no longer holds fails until the
  pin moves), as #690 moved es-fr's sha256 with no new `pack_version`. But *Versioned pack
  container, keyed by language pair* says that adding an additive table « SHALL bump
  `pack_version` », and the names section is one (an older core ignores it, `FORMAT_VERSION` 1):
  keeping es-fr's and es-en's `pack_version` would break it the day this change merges, so it is
  not left to archive. No open change holds that requirement, so this change MODIFIES it: a table
  added with new tables bumps `pack_version`, as new tables do — the expression table and the
  grammar tables did, and change 44's names came with new fixture tables (`0.0.1-fixture` →
  `0.0.2-fixture`) —; a section the builder derives from committed tables leaves it alone, its
  bytes recorded by the pin. Nothing but the goldens and the checks reads `pack_version`: the core
  loads a pack on its `analyzer_version` alone, and no surface chooses or caches a pack by it.
- **The requirements this one narrows.** `add-lingua-expression-table` (open) holds *Multi-word
  expression table* (« the sequence of dictionary forms of its words ») and *Expression lookup in a
  phrase gloss* (runs of « dictionary forms »), and *An expression is the card's answer* (« key the
  card and its status by the expression's dictionary form », which D6 keeps as the lemma chain for
  a settled expression); change 44 (open) holds *A French pack keys its expressions as French is
  read* (« A pack studying English or Spanish SHALL be keyed as before and carry no such section »)
  and *French expressions are found on French's reading of a selection* (« English and Spanish
  selections SHALL be matched as before, over runs of up to five tokens, on their lemmas ») and
  *Review finds a French expression card by its name* (« Every other card SHALL be looked up as
  before »). None is MODIFIED: this change's ADDED requirements each say they take the place of
  those sentences for Spanish, and the four changes are in `archiveAfter`; change 44's own
  `archiveAfter` orders the French changes that hold *A French invariance baseline runs beside the
  English and Spanish ones*, whose `beside es-en` line moves here. Change 48's *fr-en is committed at
  its studied tables' snapshot…* (« es-fr's, es-en's … pins, packs and goldens SHALL NOT move »)
  speaks of fr-en's own commit, not of a later change, and is not narrowed. The wording at archive
  is the owner's, as change 44's open question 5 was settled.

### D10 — What other changes take from here

| Change | Takes |
|---|---|
| 34 `enable-lingua-english-speakers` | es-en ships keyed and named as Spanish is read if this lands first; if not, its readers' records are D6's too (one knowledge model per studied language) |
| 48, 49 (fr-en, fr-es) | nothing: French keeps change 44's rules byte for byte |
| 51 French word card | D5's French counterpart, if the owner wants it (open question 3); `french_expression_key`, which it names, stays — French's call of `reading_expression_key` |
| `fix-lingua-lemma-lookup` (outside the 57) | both re-bless `es-fr.golden` and `es-en.golden`, it on the lines that read another word's entry, this change on its `pack` line and phrase probes: whichever lands second re-blesses them and lists its own lines only; neither moves the other's keys (`reading_expression_key` asks `contains_lemma`, which it leaves alone) |
| 54, 56 wording | nothing: no interface text moves |

## Risks / Trade-offs

- [A determiner varies inside an expression] → Met only as the dictionary writes it (D2): on the
  corpus `los más` no longer answers « las más », and es-en's `de la` « of the » no longer answers
  « del ». The alternative keeps every article entry answering every article.
- [One lemma merges two expressions] → `dar cuenta` and `darse cuenta` share `dar contar`, `hacer
  tiempo` answers « hace tiempo »: today's merges, now named by the headword that wins (D3), the
  gloss unchanged. As English's `break point` and French's `ça ira`.
- [A reader's records under a lemma chain] → Kept (D6). A record set on a chain after this change
  ships — by a device not yet updated — is read too.
- [Devices on different releases] → The stores do not publish on the same day (the Safari host app
  waits for App Store review): until every device is updated, a status set on a name by an updated
  device is not read by one that is not, which still shows the lemma chain. It resolves with the
  update; nothing is lost.
- [Names differ between es-fr and es-en] → 75 of 3,757 keys (D3); a status set under one native
  language's name is not read under the other's.
- [A card made before, reviewed in another language] → 747 of es-fr's keys moved (D7): such a card
  shows its own text.
- [A key the window leaves out] → 85 es-fr and 63 es-en headwords, none reachable today (D4).
- [Readers see the cards change] → es-fr readers do, on the next release: the owner approves the
  re-bless (task 6.1) and dogfoods it (6.3).

## Migration Plan

No stored format, wire field, table or `pack_version` moves, and no record is rewritten: a reader's
statuses and cards under a lemma chain are read where they are (D6). The packs gain an optional
section. Rollback is a revert, which returns the cards to their lemma chains; a record set on a
name meanwhile is then not read, as a record set under the other native's name is not (D3).

## Effort

1.5–2.5 ideal days: the generalised reading key, Spanish's written words and window 0.25; D5 and D6
in the matcher 0.5; the builder's Spanish arm, its names and order 0.25; review's lookup 0.25; the
tests 0.5; the probes, the re-bless and the pins 0.25; spec and programme 0.25.

## Open Questions

For the owner, none blocking:
1. **Settled expressions (D6).** Keep a status or card set before this change on the lemma chain
   it was set under (recommended), or accept that a reader's settled Spanish expressions read as
   unsettled — 3,538 of the 11,199 expressions es-fr reaches today change string.
2. **Determiners written (D2).** The 35 words, and the measured cost: an expression whose article
   or determiner varies is met only as written (`los más` on « las más »), and es-en's `de la`
   « of the » no longer answers « del ».
3. **The article of `al`/`del` covered (D5)**, so that « después del » is answered whole. French's
   `au`/`aux` share their span the same way (an expression ending on `à` matched in « … au » would leave `le` outside): the
   same rule for French would move `fr-en.golden`, a change of its own or change 51's.
4. **Names across natives (D3).** The rule change 44 settled for French, applied to Spanish as
   decided on 2026-10-09: 75 of the 3,757 keys es-fr and es-en both reach are named differently, so
   a status set under one is not read under the other.
5. **Held wording (D9).** *Multi-word expression table*, *Expression lookup in a phrase gloss* and
   *An expression is the card's answer* (`add-lingua-expression-table`), and change 44's three
   requirements' sentences keeping Spanish as it was: reworded when they are archived, as change
   44's open question 5 was settled. The container's « adding it SHALL bump `pack_version` » is not
   deferred: no open change holds it, and this change MODIFIES it (D9) — the owner may prefer a
   new `pack_version` scheme naming the builder, a larger change of its own.
