# Design — pick-lingua-expression-written-as-the-page

## Context

See proposal.md (Why). How a French expression is stored and found today (`origin/main` 9c88442a:
changes 39 to 51b, 48b, 49b, 44c and 44d merged, French's analyser `1.2.0`):

| Seam | Today |
|---|---|
| the key (`engine::reading_expression_key`, change 44) | the headword read by `headword_reading`, each token written by `expression_piece` — its dictionary form, or a listed determiner as written —, joined by spaces: `ça va` and `ça ira` → `ça aller`, `quel que soit` and `quel que fût` → `quel que être`, `allez-y`, `allons-y` and `vas-y` → `aller y` |
| the builder (`lingua-pack` `build_pack`) | sorts the keyed headwords by key, then « written as the key », then the most tokens written as their key piece (`written_pieces`), then byte order (change 44's D6), and **keeps the first of each key** (`dedup_by`): the others, and their glosses, are not in the pack |
| the pack | `expr` (FST: key → id), `expr.zst` (gloss by id), `expr.names.zst` (the winning headword by id, where it differs from the key, French and Spanish packs) — all three in one id space, one entry per key |
| `engine::match_expressions` | from each token, the longest run of 2 to 7 pieces that is a key (`expression_for_run`, French's du/des retry); the name is `Pack::expression_name` or the key; a French run is kept only where `meets_as_written` finds the name's elisions (change 44c) and contractions (change 44d) as the page writes them; the match reports the name and the key's gloss, and the status is read on the name |
| review (`lingua-wasm` `readable_gloss`) | a French card glossed in another language than the engine's is read in the table at the key its lemma — the name — reads as (change 44's D3; without a space, change 51's D8): `ça va` reads `ça aller`, so `ça ira`'s gloss |

**How it was measured.** A scratch copy of `origin/main` at 9c88442a, never committed, carrying
this design — the builder keeping every headword of a key (D1), the pack reading them, the choice in
`match_expressions` behind a switch with three settings (today; the headword written as the page,
else change 44's winner; **this design**, D2), review by headword (D4) — and change 44c's harness
extended: fr-en's and fr-es's packs built from the committed tables (`tables/fr`, `tables/fr-en`,
`tables/fr-es`), every line of change 44c's five French corpora glossed as one selection, every
setting on the same pack (the today setting reproduces `main`'s matches to the unit: fr-en 60,359,
fr-es 41,546, change 44d's figures):

| Corpus | Selections | Tokens |
|---|---|---|
| change 39's `baseline/pages-fr.txt`, block by block | 66 | 1,066 |
| UD French-GSD train and dev, the two files fr-en's pin records, one sentence each | 15,926 | 330,125 |
| four Gutenberg novels change 42 measured, by paragraph | 4,393 | 226,086 |
| French Wikipedia paragraphs change 42 measured | 11,405 | 370,775 |
| Tatoeba's French sentences, change 42's sample | 20,000 | 157,004 |
| **All** | **51,790** | **1,085,056** |

The Spanish measure (D6, open question 2) builds es-fr and es-en from their committed tables with
the same rule and glosses change 44d's 464,574 Spanish selections (4,160,681 tokens: Tatoeba, Spanish
Wikipedia, UD Spanish, five Gutenberg books, change 3's pages); the today setting reproduces es-fr's
205,743 and es-en's 236,765 matches.

**The tables.** fr-en: 14,493 keyed headwords on 14,432 keys. **60 keys are reached by several
headwords — 121 headwords, 61 of them not in the pack today**, one key by three (`aller y`:
`allez-y`, `allons-y`, `vas-y`). Change 44's order gives 31 of the 60 to the headword written as
the key (`peu importer` over `peu importe`, `grand air` over `grands airs`), 9 to the most written
(`droit de l'homme` over `droits de l'homme`) and 20 to byte order (`ça ira` over `ça va`, `quel que
fût` over `quel que soit`, `quoi qu'il en fût` over `quoi qu'il en soit`, `ne fût-ce que` over `ne
serait-ce que`, `les miennes` over `les miens`). In 23 of the 60 every headword has the same gloss
(`fais de beaux rêves` and `faites de beaux rêves` « sweet dreams », `les miens` and `les miennes`
« mine »); in the others they differ, often as two expressions:
`fruit de mer` « a piece of seafood » and `fruits de mer` « seafood », `pied de mouche` « paragraph
mark » and `pieds de mouche` « chicken scratch », `petit mal` and `moindre mal` « lesser evil »,
`demander en mariage` « to propose » and `demande en mariage` « proposal », `grand air` « open
air » and `grands airs` « supercilious attitude ». Three keys hold headwords whose pieces read
alike and differ only in an elision (`ce qui` and `c'qui`, `ce que` and `c'que`, `putain de` and
`putain d'`). fr-es: 9,471 keyed headwords on 9,407 keys; **64 keys reached by two headwords** (41
to the one written as the key, 17 to the most written, 6 to byte order: `les miennes` over `les
miens`, `tous les deux` over `toutes les deux`), 27 of them one gloss; `ça va` alone on `ça aller`.

## Goals / Non-Goals

**Goals:**
- The owner's rule: where several headwords share a key, the match is the one written as the page —
  « Ça va ? » answers `ça va` « how are you? », « Ça ira. » `ça ira`.
- Every headword a key holds is in the pack, with its gloss, at a cost an older core does not see.
- The golden shows the rule on fr-en's and fr-es's real tables; English and Spanish do not move.

**Non-Goals:**
- **A headword the pack does not hold.** fr-es's dictionary has `ça va` and no `ça ira`, so « Ça
  ira. » keeps answering `ça va` « ¿Cómo estás? » in fr-es, and « ça allait » `ça ira` in fr-en:
  a key one headword reaches is answered by it, written as the page or not, as for any inflected
  form (« il y avait » `il y a`). Which headwords a pair offers is its reducer's ground (changes 48,
  48b, 49, 49b).
- **Which headword wins a key.** Change 44's order (D6 there) still names the key's first headword,
  the one an older core answers and review falls back on; it is not reopened.
- **Spanish** (D6, open question 2) and English, whose keys are not named.
- Page highlighting: `analyse_page` is untouched, as in changes 44, 44c and 44d.

## Decisions

### D1 — The pack keeps every headword of a key, in ids of their own

The key stays the reading — that is what lets « il y avait » meet `il y a` — so two headwords that
read alike share it, and the pack has to hold both to answer either. For a pack studying French, the
builder keeps, after the first headword of each key (the one change 44's order names, whose id, gloss
and name are as today), every other headword of the key:

- each is an expression of its own — its gloss in `expr.zst`, its name in `expr.names.zst` — under an
  id after the keys' ids (`keys.len()` onwards, in key order and change 44's order within a key),
  which no key of `expr` maps to. Every one is named: only one headword can be written as its key, and
  change 44's order gives it the key;
- a new optional section, **`expr.others`**, lists them: `count u32 | count × (key id u32, other id
  u32)`, little-endian, sorted by key id and then in change 44's order — fr-en 61 pairs (492 B), fr-es
  64 (516 B). Written only for a pack studying French in which a key has other headwords.

| Pack | Today | This design | Growth | `expr.others` |
|---|---|---|---|---|
| fr-en | 2,537,386 B | 2,538,483 B | +1,097 B | 492 B |
| fr-es | 1,971,289 B | 1,972,957 B | +1,668 B | 516 B |

**A core that predates the section** ignores it (`Pack::load` passes over a section it does not know,
`a_pack_carrying_an_unknown_section_loads_unchanged`), and the extra ids are never reached through
`expr`: it answers every key with the headword, gloss and name it answers today. Measured: the today
setting on the new pack reproduces `main`'s matches to the unit. `FORMAT_VERSION` stays 1, and
`pack_version` stays, as change 44b's *Versioned pack container, keyed by language pair* says of a
section the builder derives from committed tables; the two pins record the new sha256 and size.

*Rejected — one section holding each shared key's other headwords and glosses as text* (`name TAB
gloss` lines by key id): measured +1,956 B and +1,709 B, and a text layout of its own beside three
sections already keyed by id.
*Rejected — the same id twice in `expr.zst` and `expr.names.zst`.* The layout allows it and needs no
new section, but `parse_glosses` keeps the last entry of an id, so an older core would answer
whichever headword was written last, and a key that is its own headword (31 of fr-en's 60) carries
no name, so an older core would read another headword's name for it.
*Rejected — changing change 44's order instead* (the present before the future, `ça va` over `ça
ira`): « Ça va ? » would be right and « Ça ira. » wrong; one gloss per key cannot answer both.

### D2 — The match is named by the headword written most like the page

For a French run that is a key several headwords reach, the matcher reads each headword through
`headword_reading` — the function the key was made with, so its pieces line up with the run's one for
one — and counts the places where a piece's text is the run's token's text, compared in lowercase,
the typographic apostrophe read as the straight one (as `expression_piece` writes a key). The
headword written as the page at the most places wins; between equals, the first in change 44's
order. Its name is the match's key and its gloss the match's gloss; the status is read on the name
(change 44's D3, unchanged). A key one headword reaches is matched as today, without a reading.

The owner's rule is the case where a headword is written as the page at every place: « Ça va ? »
`ça va`, « Ça ira. » `ça ira`, « Allons-y ! » `allons-y`, « les miens » `les miens`. Where none is,
the closest is: « quelle que soit la distance » answers `quel que soit` (`que` and `soit` written as
the page) rather than `quel que fût` (`que` alone); and where none is closer, change 44's winner:
« ça allait » answers `ça ira` (`ça` in both).

Measured over the 51,790 selections:

| Pair | Matches | On a key several headwords reach | Renamed | of which the same gloss | Gained | Lost | Selections moved |
|---|---|---|---|---|---|---|---|
| fr-en | 60,359 | 1,398 | **143** | 41 | 1 | 0 | 139 |
| fr-es | 41,546 | 220 | **47** | 15 | 0 | 0 | 47 |

By corpus, fr-en's renamed: change 39's 0, GSD 57, Gutenberg 19, Wikipedia 20, Tatoeba 47; fr-es's
0, 7, 20, 4, 16. fr-en's 143: `droit de l'homme` → `droits de l'homme` 43 (« droits de l'hommes »
once, by the closest), `ça ira` → `ça va` 19, `quel que fût` → `quel que soit` 20 (13 of them «
quelle que soit », by the closest), `peu importer` → `peu importe` 12, `allez-y` → `vas-y` 9 and
`allons-y` 3, `quoi qu'il en fût` → `quoi qu'il en soit` 7, `les miennes` → `les miens` 5, `grand
air` → `grands airs` 4 (« ses grands airs la rendaient redoutable »), `fruit de mer` → `fruits de
mer` 4, `les tiennes` → `les tiens` 3, `demander en mariage` → `demande en mariage` 3, `école de
langue` → `école de langues` 2, and 9 once each (`à flots`, `petites annonces`, `ne serait-ce que`,
`ayant droit`, `va savoir`…). fr-es's 47: `les siennes` → `les siens` 13, `tous les deux` → `toutes
les deux` 10, `les miennes` → `les miens` 6, `fruits de mer` 5, `coup d'œil` → `coup d'oeil` 4, `les
tiens` 3, and `première ministre`, `championne olympique`, `cousine germaine`, `condamnée à mort`,
`tueuse en série`, `pleins pouvoirs`, `à torrents`, `serpents à sonnettes` once each.

Read one by one, the renamed matches name what the page wrote, but:
- **one is worse**: « Quand tu es venue au monde » answers `venue au monde` « coming into the world,
  birth », the noun, where `venir au monde` « to be born » answered — the page's participle is
  written as the noun;
- **four are as wrong as before**, a verb read as an expression either way: « Je les tiens »
  `les tiens` « yours » (fr-en and fr-es), « on va savoir ce que c'est » `va savoir` « who knows? »,
  « Tu vas y réfléchir » `vas-y` « go ahead »;
- **four in fr-es take the Spanish Wiktionary's other entry**: « coup d'oeil », written without the
  ligature, answers `coup d'oeil` « Intuito » (an old word for a look) where `coup d'œil` « Ojeada »
  answered.

The cross-native gain: of the 30,423 places where fr-en and fr-es match the same key, 184 are named
differently today — a status set under one native language's name is not read under the other's,
change 44's open question 1 — and 119 with this design (`ça ira` and `ça va` 20 → 1, `droit de
l'homme` and `droits de l'homme` 43 → 0).

*Rejected for now — the headword written as the page, else change 44's winner* (open question 1):
the same as this design where a headword is written as the page at every place; where none is, the
winner instead of the closest — 15 matches in fr-en (the 13 « quelle que soit » answered `quel que
fût`, « droits de l'hommes », « écoles de langues ») and 1 in fr-es (« serpents à sonnettes ») named
farther from the page, none nearer. It matters more once « ca » without its cedilla reads as `ça`
(change 43b, proposed): « ca va » is then written as neither `ça va` nor `ça ira`, and measured on a
copy of the tables reading `ca` as `ça`, this design answers `ça va` (`va` written as the page) and
the strict rule `ça ira`.

### D3 — Each headword of the key meets the page by its own writing

Changes 44c and 44d keep a French run only where the headword's elisions and contractions meet the
page's (`meets_as_written`, read on the name). With several headwords on a key, each is held to it:
the headwords whose pieces do not meet the run are not candidates, and a run none of them meets does
not match at that length (the shorter runs are tried, as today). So a key whose winner does not meet
the page but another headword does now matches with that one: `ce qui` and `c'qui`, whose pieces
read alike, are told apart by the elision of « c' » — fr-en gains « j'sais ben à qui c'qui ça fait
point d'peine » answered `c'qui` (1 match), and loses none, since the winner, when it meets, is
still a candidate.

### D4 — Review reads a French card under its own headword

A French card is created with the match's name as its lemma (change 44's D3). Review shows the
pack's gloss of a card glossed in another language than the engine's (*Review shows a gloss the
reader can read*) by reading the table at the key the lemma reads as — today the key's gloss,
whichever headword the card is. So `readable_gloss` reads, at that key, the gloss of the headword
named as the card's lemma (`Pack` reads it through `expr.others`), and the key's own gloss where none
is: a card `ça va` made in fr-es and reviewed by an English-native engine shows `ça va`'s fr-en gloss,
not `ça ira`'s. Both of review's French branches do it — a lemma with a space and one without
(change 51's D8: `allons-y` reads `aller y`, whose first headword is `allez-y`). No baseline holds
such a card, so no golden moves for it; a test in `lingua-wasm` holds it.

### D5 — What moves: the packs, their pins, five probes

Measured on the prototype with no probe added: `fr-en.golden` and `fr-es.golden` move their `pack`
line alone — no phrase probe of the French scenario falls on a key several headwords reach. So the
scenario (`tests/support/french.rs`) gains five phrase probes, after change 44d's:

| Probe | fr-en today | fr-en with this design | fr-es today | fr-es with this design |
|---|---|---|---|---|
| « Ça va ? » | `ça ira` | **`ça va`** | `ça va` | `ça va` |
| « Ça ira. » | `ça ira` | `ça ira` | `ça va` | `ça va` (fr-es holds no `ça ira`) |
| « Allons-y ! » | `allez-y` | **`allons-y`** | none | none |
| « les miens » | `les miennes` | **`les miens`** | `les miennes` | **`les miens`** |
| « Quelle que soit la distance » | `quel que fût` | **`quel que soit`** | `quel que soit` | `quel que soit` |

Re-blessed on the prototype: `fr-en.golden` 805 → 815 lines and `fr-es.golden` 815 → 825 — the `pack`
line (2,537,386 → 2,538,483 and 1,971,289 → 1,972,957 bytes, `pack_version` and `analyzer_version`
unchanged) and 10 lines added each, every probe before byte for byte; 38 → 43 phrase probes.
`word-card-fr-en.txt` (724 → 743 lines) and `word-card-fr-es.txt` (590 → 607), which render the
golden's phrase probes, gain the five cards, every block before byte for byte — fr-en « expression:
ça va » « how are you? how's it going? how are thing; things are going fine; OK, all right »,
« expression: ça ira », « expression: allons-y » « let's go; let's go there; come with me, follow
me », « expression: les miens » « mine », the row « quel que soit — whatever, regardless of what, no
matter wh… »; fr-es the same with « (coloquial) ¿Cómo estás?… » twice, the row « aller — Ir o
viajar… », « Los míos » and « quel que soit — Cual que sea ». Their specs count 43 phrase probes.
The pins `tables/fr-en/pin.json` and `tables/fr-es/pin.json` record the packs' new sha256 and size;
`committed_tables` builds the four other packs to their pins.

**French's analyser version stays**, as changes 44 (its D7), 51 (its D12), 44c (its D5) and 44d read
*An analyser version per studied language*: no token, lemma or page analysis moves; the pack gains an
additive section, and an older core with the new pack answers as before (D1).

**The baseline's reason.** *A French invariance baseline runs beside the English and Spanish ones*
(changes 39, 40, 41, open) names three reasons for moving the golden; probes a change adds and a pack
that gains a section are none of them. Changes 44, 51, 44c and 44d said so in their pull requests,
and so does this one; the test's doc comment (`french_baseline.rs`) lists this change among those
that add probes.

### D6 — English and Spanish cannot move; Spanish has the same defect

- The builder keeps the other headwords for a pack studying French alone, so no English or Spanish
  pack carries `expr.others`, and the matcher chooses only among the headwords a pack lists: en-fr,
  es-fr, es-en and en-es are built byte for byte and matched as today. Measured on the prototype,
  without re-blessing: `english_baseline`, `spanish_baseline`, `es_en_baseline`, `en_es_baseline`,
  `cross_native`, `parity`, `card_gloss_language`, `spanish_expression_keys`, `statuses` and `deck`
  pass (62 tests in 10 suites); `committed_tables` (38 tests) passes once fr-en's and fr-es's pins
  move; every `lingua-core` and `lingua-pack` test passes, the English, Spanish and French fixtures
  byte for byte.
- **Spanish is keyed the same way since change 44b, and has the same defect, larger.** Measured with
  the same rule on es-fr and es-en:

  | Pair | Keys several headwords reach | Matches on them | Renamed | Same gloss | Pack |
  |---|---|---|---|---|---|
  | es-fr | 98 (99 other headwords) | 4,418 of 205,743 | 584 | 117 | 2,231,951 → 2,234,330 B |
  | es-en | 138 (141) | 12,659 of 236,765 | 1,604 | 293 | 2,608,413 → 2,612,075 B |

  es-fr: `un poco` → `un poquito` 111 (« Un peu » → « Un petit peu »), `hacer tiempo` « Atermoyer » →
  `hace tiempo` « Jadis, belle lurette » 85, `buen día` → `buenos días` 79, `red social` → `redes
  sociales` 59; es-en: `medio de comunicación` → `medios de comunicación` 100, `no ser nada` → `no es
  nada` 99, `qué pasa` → `qué pasó` 91. `es-fr.golden` would move its `pack` line and « darse cuenta »
  (`dar cuenta` « Rendre compte » → `darse cuenta` « Se rendre compte »), `es-en.golden` its `pack`
  line, `en-es.golden` nothing; es-fr's and es-en's pins. The closest headword (D2) costs more in
  Spanish than in French: of the 17 (es-fr) and 167 (es-en) matches it names otherwise than the strict
  rule, es-en's `dar vueltas` 52, `dar el alta` 17 (« le dieron el alta », discharged), `dar gracias` 8 and
  `hacerse ilusiones` 7 are right, and `tenerlos puestos` « to have balls » on « tenía puestos unos
  audífonos » 6 is wrong.
- es-fr is read, and its output does not move without the owner's approval (the programme's rule), so
  Spanish is left out here (open question 2). Its readers' statuses are not at stake yet: no release
  has brought them change 44b's names (its task 6.4), so no reader has set a status on a name this
  rule would move; one who sets one after that release, on a name not written as the key (`buen día`
  on « buenos días »), would see the expression unsettled once Spanish follows.

### D7 — What later changes take from here

| Change | Takes |
|---|---|
| 43b `read-lingua-french-ca-as-ca-cedilla` (proposed) | « ca va » read as `ça aller` is answered `ça va` (D2); whichever of the two lands second re-blesses `fr-en.golden`, `fr-es.golden` and the word-card snapshots |
| 43c `keep-lingua-french-acronyms-in-capitals` (proposed) | the goldens, whichever lands second |
| 52 `enable-lingua-french` | the dogfood: « Ça va ? » and « Ça ira. » on a real page in fr-en |
| a Spanish change, if the owner wants one (open question 2) | the builder's French condition opened to Spanish, es-fr's and es-en's pins and goldens |

## Risks / Trade-offs

- [Two spellings of one expression, two names] → A status set on `les miennes` is not read on « les
  miens », and the deck can hold both; 23 of fr-en's 60 shared keys (27 of fr-es's 64) give every
  headword one gloss, and 41 of fr-en's 143 renamed matches keep their gloss (open question 3). No
  reader studies French, so no status is under a French name yet.
- [A participle written as a noun headword] → « es venue au monde » answers `venue au monde` « birth »
  (1 on the corpus); « ayant droit », « demande en mariage » are right on the corpus (4), and would be
  as wrong on « il la demande en mariage ».
- [A pack's own spelling defect surfaces] → fr-es's `coup d'oeil` « Intuito » (4); a reducer's ground.
- [The closest headword is wrong] → None in French on the corpus; in Spanish `tenerlos puestos` (D6).
- [An older core] → Answers as today (D1); no French pack is published before change 52, and a pack
  ships with the core built beside it.
- [Cost] → Pack +1,097 and +1,668 B; one reading per headword of a shared key per match that reaches
  it: gloss time 2.595 → 2.604 s over the 51,790 selections (median of 15 interleaved passes, +0.3 %,
  within the machine's noise).

## Migration Plan

Nothing to migrate: no stored format, wire field or table moves, and no reader studies French. The
French packs gain an optional section and their pins move. Rollback is a revert.

## Effort

0.75–1.25 ideal day: the builder's other headwords and `expr.others` 0.15–0.25; the pack's reader and
accessors 0.1–0.15; the choice in `match_expressions` 0.15–0.25; review 0.05–0.1; their tests
0.15–0.25; the probes, the goldens, the snapshots and the pins 0.1; spec and programme 0.05.

## Open Questions

For the owner, none blocking — each with the design's recommendation:

1. **When no dictionary spelling is written exactly as the page (D2).** « Quelle que soit la
   distance » is written neither `quel que soit` nor `quel que fût` (the page says « quelle »).
   Recommended: the spelling that shares the most words with the page, `quel que soit` (`que` and
   `soit`), and the usual winner only between equals (« ça allait » → `ça ira`). The alternative is
   the usual winner every time: « quelle que soit » would keep showing `quel que fût` (13 times on
   the corpus), and once « ca » reads as « ça » (change 43b), « ca va » would show `ça ira` « it will
   be all right » again.
2. **Spanish (D6).** The same mix-up exists in Spanish, larger: « hace tiempo » (« long ago ») shows
   `hacer tiempo` « Atermoyer » (« to play for time ») in es-fr, 85 times on the corpus; 584 es-fr
   matches and 1,604 es-en ones would change name. es-fr is read, so its output moving needs your
   approval. Recommended: a change of its own right after this one, landing before the release that
   brings es-fr's readers their named expressions (change 44b's 6.4), so no status is ever set on a
   name that later moves. The alternatives: Spanish in this change (the same code, es-fr's and es-en's
   pins and goldens re-blessed: their `pack` lines and « darse cuenta »), or Spanish left as it is.
3. **Two spellings of one expression become two (Risks).** `les miens` and `les miennes` (« mine »),
   `dis donc` and `dites donc` are each named as the page writes them, so a status set on one is not
   read on the other. Recommended: accept it — the card shows what the page wrote, as you asked. The
   alternative, renaming only where the two spellings have different glosses, keeps one status for
   `les miens`/`les miennes` but also leaves `quel que fût` on « quel que soit » (one gloss for both):
   102 of fr-en's 143 renames instead of 143.
