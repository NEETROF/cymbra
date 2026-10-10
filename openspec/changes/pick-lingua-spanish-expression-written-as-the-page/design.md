# Design — pick-lingua-spanish-expression-written-as-the-page

## Context

See proposal.md (Why). How a Spanish expression is stored and found on `origin/main` d02371ac
(changes 44b, 42b, 42c and 24c merged, Spanish's analyser `1.5.0`), and what change 44e
(`pick-lingua-expression-written-as-the-page`, proposed in #887, being implemented) brings French:

| Seam | Spanish today | 44e, for French |
|---|---|---|
| the key (`engine::reading_expression_key`, 44b) | the headword read by `headword_reading`, each token its dictionary form or a listed determiner as written: `hacer tiempo` and `hace tiempo` → `hacer tiempo`, `buen día` and `buenos días` → `bueno día`, `tener puesto` and `tenerlos puestos` → `tener puesto` | the same function |
| the builder (`lingua-pack` `build_pack`) | change 44's order among headwords of one key (written as the key, the most pieces written as their key piece, byte order), the first kept, the others dropped | the others kept, each an expression of its own under an id no key maps to, listed per key in the optional section `expr.others` |
| `engine::match_expressions` | the longest run of 2 to 7 tokens that is a key; the name is `Pack::expression_name` or the key; a match ending before the article of « al »/« del » covers it (44b D5); `settled_chain` reports the run's lemma chain where the reader holds a record on it and none on the name (44b D6) | among the key's headwords, the one written most like the run (`headword_reading`, the places where a piece's text is the token's), change 44's order between equals |
| review (`lingua-wasm` `readable_gloss`) | a Spanish expression card read at its lemma, else at the key its lemma reads as — the key's gloss, whichever headword the card is | the gloss of the headword named as the card's lemma (`Pack::expression_gloss_named`) |
| Spanish's enclitic rule (`analysis/spanish.rs` `enclitic_lemma`) | one or two clitic pronouns (`me te se nos os lo la los las le les`) after an infinitive, a gerund or an imperative, the base a form the lexicon lists: `darle` → `dar`, `hacerme` → `hacer`, `tócate` → `tocar`; the cascade runs it only on a form the forms table does not list, and the table lists many pronominal infinitives itself (`darse` → `dar`, `hacerse` → `hacer`) | — |

**How it was measured.** A scratch copy of `origin/main` at d02371ac, never committed, carrying 44e's
prototype (the builder keeping every headword of a French key, `expr.others`, the choice in
`match_expressions`, review by headword) and this design on top — the builder's condition opened to
Spanish (D1), the Spanish choice behind a switch whose settings are today, 44e's rule as it stands,
this design and each rejected variant of D3, the lemma-chain exception (D4), review (D5) —, and 44e's
harness extended to Spanish: es-fr's and es-en's packs built from the committed tables (`tables/es`,
`tables/es-fr`, `tables/es-en`), every line of change 44d's five Spanish corpora glossed as one
selection, every setting on the same pack. The today setting reproduces `main`'s matches to the
unit (es-fr 205,743, es-en 236,765), and 44e's rule as it stands reproduces 44e's Spanish figures
(D6 there: 584 and 1,604 renamed). The Spanish side of `main` has not moved since 44e's base
(9c88442a): no Spanish table, core or test file differs.

| Corpus | Selections | Tokens |
|---|---|---|
| change 3's `baseline/pages-es.txt`, block by block | 47 | 598 |
| UD Spanish, one sentence each | 5,341 | 130,104 |
| five Gutenberg books, by paragraph | 5,212 | 240,459 |
| Spanish Wikipedia paragraphs | 11,567 | 541,578 |
| Tatoeba's Spanish sentences | 442,407 | 3,247,942 |
| **All** | **464,574** | **4,160,681** |

**The tables.** es-fr: 11,887 keyed headwords on 11,788 keys. **98 keys are reached by several
headwords — 197 headwords, 99 of them not in the pack today**, one key by three (`echar a perder`:
`echar a perder`, `echado a perder`, `echarse a perder`). Change 44's order gives 74 of the 98 to the
headword written as the key (`hacer tiempo` over `hace tiempo`, `un poco` over `un poquito`, `red
social` over `redes sociales`), 18 to the most written (`buen día` over `buenos días`, `dar cuenta`
over `darse cuenta`) and 6 to byte order (`ella misma` over `ellos mismos`, `condenada a muerte` over
`condenado a muerte`). In 41 of the 98 every headword has the same gloss (`buen día` and `buenos días`
« Bonjour »); in the others they differ, often as two expressions: `hacer tiempo` « Atermoyer » and
`hace tiempo` « Jadis, belle lurette », `dar cuenta` « Rendre compte » and `darse cuenta` « Se rendre
compte », `servir de` « Faire office de » and `servirse de` « S'aider de ». es-en: 13,826 keyed
headwords on 13,685 keys; **138 keys reached by several headwords** (279 headwords, 141 not in the
pack today, three keys by three: `ir y venir`, `necesitar ayuda`, `primero plano`), 93 to the
headword written as the key, 20 to the most written, 25 to byte order, 51 of them one gloss.

**Headwords written with a clitic pronoun.** On 13 of es-fr's 98 shared keys and 26 of es-en's 138, a
headword carries a clitic pronoun on its verb as the enclitic rule reads it (D3): es-fr `cuidarse
de`, `darse cuenta`, `dándose la espalda`, `volverte loco`…; es-en `hacerse ilusiones`, `tenerlos
puestos`, `ponerse en contacto`, `puede ayudarme`… On 1 (es-fr: `venderse como pan caliente` and `…
panes calientes`) and 3 (es-en: `puede ayudarme`/`puedes ayudarme`, `jugarse todo a una carta`/
`jugárselo todo…`, `de agárrate que hay curva(s)`) every headword does.

## Goals / Non-Goals

**Goals:**
- The owner's rule of 2026-10-10 for Spanish: where several headwords share a key, the match is the one
  written as the page — « hace tiempo » answers `hace tiempo` « Jadis », « hacer tiempo » `hacer
  tiempo` « Atermoyer ».
- No misfire 44e's « closest spelling » would bring to Spanish: a headword written with a clitic
  pronoun the page does not write is not chosen (`tenerlos puestos` on « tenía puestos »).
- Before es-fr's readers get 44b's names (44b's 6.4), so no reader sets a status on a name that later
  moves; and no status set after that release read on another headword's match.
- The goldens show the rule on es-fr's and es-en's real tables; English and French do not move.

**Non-Goals:**
- **A headword the pack does not hold.** es-en has `un poco` and no `un poquito`, so « un poquito »
  keeps answering `un poco` there: a key one headword reaches is answered by it, as for any inflected
  form. Which headwords a pair offers is its reducer's ground.
- **Which headword wins a key.** Change 44's order (as 44b applies it to Spanish) still names the key's
  first headword, the one an older core answers and review falls back on.
- **Telling a reflexive pronoun from an object one** (« me di cuenta » is `darse cuenta`, « me sirve de
  luz » is `servir de`): the guard (D3) only keeps a headword written with a pronoun away from a page
  that writes none; it does not make it win where the page writes one (rejected variant, D3; open
  question 3).
- **A clitic pronoun required on every key**, a one-headword key included — the Spanish counterpart of
  44c's elisions (D3, rejected variant): a change of its own if the owner wants one.
- English, whose keys are not named; French, which 44e covers; page highlighting (`analyse_page` is
  untouched).

## Decisions

### D1 — A Spanish pack keeps every headword of a key, in ids of their own

44e's D1, opened from French to Spanish: for a pack studying French or Spanish, the builder keeps,
after the first headword of each key (whose id, gloss and name are as today), every other headword of
the key as an expression of its own — its gloss in `expr.zst`, its name in `expr.names.zst`, under an
id after the keys' ids, in key order and change 44's order within a key — and writes the optional
section `expr.others` (`count u32 | count × (key id u32, other id u32)`, little-endian) when a key has
others. Nothing in the layout is new: the section, its reader (`Pack::expression_headwords`) and its
refusal of a malformed one are 44e's.

| Pack | Today | This design | Growth | `expr.others` |
|---|---|---|---|---|
| es-fr | 2,231,951 B | 2,234,330 B | +2,379 B | 796 B (99 pairs) |
| es-en | 2,608,413 B | 2,612,075 B | +3,662 B | 1,132 B (141 pairs) |

Both stay far under the 5 MiB budget (es-en 49.8 % of it, as today to the tenth). **A core that
predates the section** answers every key with the headword, gloss and name it answers today: measured,
the today setting on the new packs reproduces `main`'s matches to the unit. `FORMAT_VERSION` stays 1
and `pack_version` stays, as 44b's MODIFIED *Versioned pack container, keyed by language pair* says of
a section the builder derives from committed tables; the two pins record the new sha256 and size, and
the pack sizes the es-fr and es-en READMEs and `SOURCES.md` state move with them (#852 and #883 moved
them the same way).

*Rejected — Spanish's other headwords in a section of their own*: the same data in another shape, a
second reader for one layout.

### D2 — The match is named by the headword written most like the page

44e's D2 for a Spanish run that is a key several headwords reach: each headword is read by
`headword_reading` — the function the key was made with, so its pieces line up with the run's one for
one — and the places where a piece's text is the run's token's text are counted, in lowercase (the
pre-pass gives both in NFC); the headword written as the page at the most places wins, change 44's
order between equals; its name is the match's key, its gloss the match's gloss, and the status is read
on the name (44b's D3, unchanged). There is no Spanish counterpart of 44e's D3 (44c's and 44d's
French elisions and contractions): every headword of the key is weighed, but for D3's guard. A key one
headword reaches is matched as today, without a reading.

« hace tiempo » answers `hace tiempo` « Jadis, belle lurette », « hacer tiempo » `hacer tiempo`
« Atermoyer »; « buenos días » `buenos días`; where none is written as the page, the closest: « dando
vueltas » `dar vueltas` (`vueltas` written as the page) rather than `dar vuelta`, « le dieron el
alta » `dar el alta` « to discharge » rather than `dar el alto` « to make someone stop ».

Measured over the 464,574 selections, with D3's guard:

| Pair | Matches | On a key several headwords reach | Renamed | of which the same gloss | of which by the closest | Gained | Lost | Selections moved |
|---|---|---|---|---|---|---|---|---|
| es-fr | 205,743 | 4,418 | **584** | 117 | 17 | 0 | 0 | 584 |
| es-en | 236,765 | 12,659 | **1,598** | 293 | 161 | 0 | 0 | 1,577 |

By corpus, es-fr's renamed: change 3's pages 0, UD 6, Gutenberg 19, Wikipedia 19, Tatoeba 540; es-en's
0, 31, 42, 146, 1,379. es-fr's 584, in 37 kinds: `un poco` → `un poquito` 111 (« Un peu » → « Un petit
peu »), `hacer tiempo` → `hace tiempo` 86 (85 « hace tiempo », once « Hace tiempos » by the closest),
`buen día` → `buenos días` 79 (one gloss), `red social` → `redes sociales` 59, `sí mismo` → `sí
mismos` 56, `ella misma` → `ellos mismos` 36, `dar cuenta` → `darse cuenta` 19 (« sin darse cuenta »),
`agencia de viaje` → `agencia de viajes` 17, `última palabra` → `últimas palabras` 13, `no fumador` →
`no fumadores` 11, `ella sola` → `ellos solos` 11, `condenada a muerte` → `condenado a muerte` 9, and
25 more. es-en's 1,598, in 83 kinds: `medio de comunicación` → `medios de comunicación` 101, `no ser
nada` → `no es nada` 99, `qué pasa` → `qué pasó` 91, `querer decir` → `quiero decir` 85, `buen día`
→ `buenos días` 78, `de día` → `de días` 77, `puede ayudarme` → `puedes ayudarme` 70, `dar vuelta` →
`dar vueltas` 67, `cómo está` → `cómo estás` 65, `red social` → `redes sociales` 59.

Read kind by kind, the renamed matches name what the page wrote, but:
- **three kinds are worse in es-fr**, the French Wiktionary glossing the plural otherwise: « una
  colección de muñecas rusas » answers `muñecas rusas` « Mise en abyme » where `muñeca rusa` « Poupée
  russe » answered (1); « Los vendedores ambulantes » `vendedores ambulantes` « À la sauvette » where
  `vendedor ambulante` « Colporteur » answered (4); « La tecnología por si misma » `por si misma`
  « Elle-même » where `por si mismo` « En tant que tel » answered (1);
- **some are as wrong as before**, another word read as an expression either way: es-fr « Ellos solo
  quieren » (`solo`, only) `ellos solos` « Eux-mêmes » where `ella sola` « Elle-même » answered (7, by
  the closest); es-en « un par de días » `de días` « a few days old » where `de día` « in the daytime »
  answered (77), « Ahora no puedes ayudar a Tom » `puedes ayudarme` where `puede ayudarme` answered
  (both « can you help me? »).

The gloss time does not move beyond the machine's noise: over UD, Gutenberg and Wikipedia (22,120
selections), median of 15 interleaved passes, es-fr 2.234 → 2.222 s and es-en 2.462 → 2.518 s (best
pass 2.100 → 2.091 and 2.165 → 2.174 s), on a machine shared with other sessions. The reading is made
only on a key several headwords reach (5 % of es-en's matches, 2 % of es-fr's).

**Across natives.** Unlike French (44e: 184 → 119), naming by the page parts es-fr and es-en more:
of the 115,262 places where both match the same key, 2,153 are named differently today and 2,450 with
this design — mostly a key one pair holds two spellings of and the other one (`un poquito` / `un poco`
111, `medio de comunicación` / `medios de comunicación` 101). A status set under one native language's
name is not read under the other's (44b's D3, settled by the owner); the gap closes where both packs
hold the page's spelling (`dar vueltas` / `dar vuelta` 101 → 34, `cómo estás` / `cómo está` 248 →
183).

*Rejected — the strict rule* (the headword written as the page, else change 44's winner): settled by
the owner for French on 2026-10-10 (44e's open question 1, the closest wins), and Spanish follows. It
names 17 es-fr and 161 es-en matches farther from the page (« dando vueltas » `dar vuelta`, « le
dieron el alta » `dar el alto`).

### D3 — A headword written with a clitic pronoun meets only a page that writes one

44e's closest spelling has a Spanish misfire French does not have: a headword whose verb carries a
clitic pronoun is often the reflexive or idiomatic twin of one without (`tener puesto` « to wear »,
`tenerlos puestos` « to have balls »), and on a page writing neither verb form the pronoun-bearing one
can be closer by another word (« tenía puestos unos audífonos »: `puestos` written as the page, so
`tenerlos puestos`, 6 times on the corpus).

So, in a Spanish run that is a key several headwords reach: **a headword one of whose words carries a
clitic pronoun is weighed only where the page writes a clitic pronoun for the run** — attached to one
of the run's words (« hacerme ilusiones », « Cuídate de ») or as the page's word just before the run
(« No te hagas ilusiones », « ¿Me puedes ayudar? », « Los tenía puestos »). **Where no headword of the
key is weighed so, every headword is**, as D2 alone weighs them: the guard chooses among headwords and
never makes a run match or stop matching. A key one headword reaches is matched as today.

*A word carries a clitic pronoun* where Spanish's enclitic rule reads it so — the cascade's own
`enclitic_lemma` (`analysis/spanish.rs`): one or two of `me te se nos os lo la los las le les` after an
infinitive, a gerund or an imperative whose base the lexicon lists — and the lexicon does not file the
word under another lemma (`parte` is no `par` + `te`; `darse`, which the forms table files under
`dar` itself, is `dar` + `se`). The same test reads a headword's words and the page's tokens. A clitic
pronoun before the run is the page's previous token, lowercased, one of those eleven words.

The guard never touches a headword written exactly as the page — a page written as `tenerlos
puestos` writes its pronoun —, so the owner's rule is untouched; it acts only on the closest spelling.

| Variant | es-fr against 44e's rule as it stands | es-en | Matches lost |
|---|---|---|---|
| 44e's rule as it stands (no guard) | — | — | 0 |
| **this design** (a pronoun on a word of the run, or the word before it) | **0** | **6 renamed**: `tenerlos puestos` → `tener puesto` | **0** |
| a pronoun on the word at the same place only | 0 | 12 renamed: the 6, and « No te hagas ilusiones » → `hacer ilusión` « to cause to get excited about something » 6 | 0 |
| a run no headword qualifies for does not match at that length | 1 (`venderse como pan caliente` → a shorter run) | 130 selections: the 6, and `puede ayudarme`/`puedes ayudarme` lost 125 times (« Puedo ayudar », « Tatoeba puede ayudar a… ») | es-fr 1, es-en 125 |
| the guard on every key, a one-headword key included | 759 selections | 763 selections | es-fr 768 (140 gained), es-en 760 (119) |
| the pronoun also counted as a place written as the page | 312 renamed | 141 renamed (the 6 included) | 0 |

The « same place » variant is the narrower reading and loses the proclitic, the place Spanish writes
the pronoun before a conjugated verb. The « no match » variant removes 125 wrong es-en matches (« Puedo ayudar »
is no « can you help me? »), but only on keys two pronoun-bearing headwords share: the same run on a
key one such headword reaches keeps matching, and the rule would read differently from one key to the
next. The guard on every key is that rule made consistent — the Spanish counterpart of 44c's French
elisions —: it removes many wrong matches (« las bibliotecas coleccionan escritos relacionados con »
`relacionarse con`, « Dado que no » `dile que no`) and some right ones, a pronoun placed before an
auxiliary being out of its sight (« Me voy a lavar las manos » `lavarse las manos`, « Se está haciendo
tarde » `hacerse tarde`); it is a change of its own, not this one's (Non-goals). Counting the pronoun
as written would make « me di cuenta » `darse cuenta` « Se rendre compte » (146 es-fr matches move to
the reflexive headword), but also « Ella me dio la espalda » `dándose la espalda` « Dos à dos », « me
sirve de luz » `servirse de` « S'aider de » and « me vuelven loco » `volverte loco` « Péter les
plombs »: an object pronoun is no reflexive one (open question 3).

### D4 — A lemma chain that is a headword's name is that headword's

44b's D6 keeps a status or a deck card set before Spanish expressions were named: where the reader
holds a record on the run's lemma chain (`tener en contar`) and none on the name, the match reports the
chain. With several names per key, a lemma chain can be one of them: of es-fr's 98 shared keys, **71
have a lemma chain that is their first headword's own name** (`hacer tiempo`, `un poco`, `red social`,
`sí mismo`), es-en 90 of 138; **395 of es-fr's 584 renamed matches fall on those keys**, 904 of
es-en's 1,598. Read as a chain settled before, a record on that name would answer every other
headword of the key: a reader who marks `un poco` known on « un poco » would see « un poquito »
answered `un poco`, known — the very defect this change removes, for every reader who ever marks the
first headword, and against the owner's rule that two spellings are two names (44e's open question
3, settled).

So `settled_chain` reports the lemma chain only where it is **the name of no headword of the run's
key**. A chain no headword is named keeps 44b's D6 whole (`tener en contar`, `a el vez`, `dar
contar`, `bueno día`): « buenos días », for a reader holding a record on `bueno día`, answers `bueno
día`, with `buenos días`'s gloss. A key one headword reaches is as today.

Measured on the prototype, a reader holding `hacer tiempo` known: « hacer tiempo » answers `hacer
tiempo`, known; « hace tiempo » answers `hace tiempo`, unknown (without the exception, `hacer tiempo`,
known, glossed « Jadis, belle lurette »).

The cost falls on a record set **before the release** that brings 44b's names: such a record is on a
lemma chain, and where the page wrote another headword of the key (« hace tiempo », shown `hacer
tiempo` « Atermoyer » until then), the reader now finds it under `hacer tiempo` only, and sees `hace
tiempo` unsettled. That is why this change lands before that release (44b's 6.4): from it on, every
record on such a chain is a record on a headword. The number of readers concerned is not known: es-fr is
read, and no record is visible to the project (open question 2).

*Rejected — 44b's D6 as it stands*: every pre-release record kept on every spelling, at the price of a
leak no later change could tell from a reader's choice.

### D5 — Review reads a Spanish card under its own headword

44e's D4 for review's Spanish branch: a Spanish expression card is read at its lemma, then at the key
its lemma reads as (44b's D7); this design reads, at that key, the gloss of the headword named as the
card's lemma (`Pack::expression_gloss_named`), the key's own gloss where none is. A card whose lemma is
itself a key — the first headword's name, or a lemma chain — is read there, as today. So a card `hace
tiempo` made in es-en and reviewed by a French-native engine shows es-fr's `hace tiempo` « Jadis,
belle lurette », not `hacer tiempo`'s « Atermoyer »; a card `hacer tiempo` shows « Atermoyer », and a
card `bueno día` made before names `buen día`'s « Bonjour », as today. No baseline holds such a card,
so no golden moves for it; a test in `lingua-wasm` holds it (measured on the prototype).

### D6 — What moves: the packs, their pins, five probes

Measured on the prototype with no probe added: `es-fr.golden` moves its `pack` line and « darse
cuenta » (`dar cuenta` « Rendre compte » → `darse cuenta` « Se rendre compte »: the page writes
`darse`), `es-en.golden` its `pack` line alone — none of the other phrase probes falls on a key several
headwords reach. So the Spanish scenario (`tests/support/spanish.rs`, shared by both baselines) gains
five phrase probes, after « al fin y al cabo »:

| Probe | es-fr today | es-fr with this design | es-en today | es-en with this design |
|---|---|---|---|---|
| « hace tiempo » | `hacer tiempo` « Atermoyer » | **`hace tiempo`** « Jadis, belle lurette » | none | none |
| « buenos días » | `buen día` « Bonjour » | **`buenos días`** « Bonjour » | `buen día` « good morning; a farewell… » | **`buenos días`** « good morning » |
| « un poquito » | `un poco` « Un peu » | **`un poquito`** « Un petit peu » | `un poco` | `un poco` (es-en holds no `un poquito`) |
| « No te hagas ilusiones. » | none | none | `hacer ilusión` « to cause to get excited about something » | **`hacerse ilusiones`** « to get one's hopes up » |
| « El guardaespaldas tenía puestos unos audífonos. » | none | none | `tener puesto` « to have on; to wear » | `tener puesto` (`tenerlos puestos` without D3's guard) |

Re-blessed on the prototype: **`es-fr.golden` 648 → 658 lines** — the `pack` line (2,231,951 →
2,234,330 bytes, `pack_version` and `analyzer_version` unchanged), « darse cuenta », and 10 lines
added —; **`es-en.golden` 748 → 758** — the `pack` line (2,608,413 → 2,612,075) and 10 lines added —;
every other line of both byte for byte, every probe's tokens included. **`fr-en.golden` moves its
`beside es-en` line alone** (2,608,413 → 2,612,075 bytes; its `pack` line is 44e's) — « the beside
pack's update », a reason *A French invariance baseline runs beside the English and Spanish ones*
names, as 44b's re-bless moved it. The pins `tables/es-fr/pin.json` and `tables/es-en/pin.json` record
the packs' new sha256 and size; `committed_tables` builds the four other packs to their pins.

**Not moved**, measured on the prototype without re-blessing: `en-fr.golden` and `en-es.golden`;
`fr-es.golden`, against 44e's own re-bless (its `beside en-es` line included); `fr-en.golden` but its
`beside es-en` line, against the same; `cross_native`, `parity`, `card_gloss_language`,
`spanish_expression_keys`, `statuses`, `deck`, `languages`, `soft_hyphens`, `reprofile`,
`english_baseline`, `en_es_baseline`, `fr_es_baseline`; `committed_tables` fails on the es-fr and
es-en pins alone (and fr-en's and fr-es's, 44e's), en-fr and en-es building to theirs.
**`word-card-es-en.txt` does not move**: `word-card-es-en.spec.ts` renders the es-en golden's
`word-grammar` probes alone, and no `word-grammar` line moves (nor does any `analyse` or `gloss`
probe); no extension snapshot renders a Spanish phrase probe. Measured: `word-card-es-en.spec.ts` and
`word-grammar-es.spec.ts` pass without `-u` on the re-blessed `es-en.golden`. **Spanish's analyser version stays**,
as 44b (its D9) and 44e (its D5) read *An analyser version per studied language*: no token, lemma or
page analysis moves; the pack gains an additive section, and an older core with the new pack answers
as before (D1).

**The goldens' reasons.** `spanish_baseline.rs` and `es_en_baseline.rs` let es-fr's and es-en's
outputs move for a dictionary update or an analyser change; a builder change that moves a pack and
probes a change adds are what 44b re-blessed for, with the owner's approval (its 6.1). Moving es-fr's
output needs the owner's approval here too (task 6.1), and the pull request says why.

### D7 — English and French cannot move

- The builder keeps the other headwords for a pack studying French or Spanish, so no English pack
  carries `expr.others`, and the Spanish choice runs on Spanish runs alone: en-fr and en-es are built
  byte for byte and matched as today. French is 44e's, untouched here.
- 44e's checks that say « no Spanish pack carries `expr.others` » (its tasks 1.3 and 4.1) become « no
  English pack », and its scenario *English and Spanish packs do not move* is read, for Spanish, as
  this change's *The committed Spanish packs*.
- Measured on the prototype: every `lingua-core` and `lingua-pack` unit test passes unchanged (507 and
  79), the English, Spanish and French fixtures byte for byte; `committed_tables` (39 tests) passes
  once the es-fr and es-en pins move (and fr-en's and fr-es's, 44e's).

### D8 — Order, and what other changes take from here

| Change | Takes |
|---|---|
| 44e `pick-lingua-expression-written-as-the-page` (proposed, being implemented) | lands first: this change opens its builder condition, adds a Spanish arm to its choice and to its review lookup; its accessors are this change's (renamed with it if its implementation renames them) |
| 44b's 6.4, the release that brings es-fr's readers their named expressions | waits for this change (task 6.3), so no status is set on a name that later moves (D4) |
| 34 `enable-lingua-english-speakers` (not started) | es-en is listed with this rule |
| 44f `choose-lingua-french-overlapping-expressions` (proposed), 43b, 43c, 43d, 48c | French only: no shared line; `fr-en.golden` re-blessed by whichever lands second (here its `beside es-en` line alone) |
| a change requiring a written clitic pronoun on every Spanish key (if the owner wants one, D3) | `carries_enclitic`, the proclitic test |

## Risks / Trade-offs

- [Two spellings of one expression, two names] → A status set on `buen día` is not read on « buenos
  días », and the deck can hold both; 41 of es-fr's 98 shared keys (51 of es-en's 138) give every
  headword one gloss, and 117 of es-fr's 584 renamed matches keep their gloss. The owner settled it for
  French (44e's open question 3); Spanish follows.
- [A pre-release status on a chain that is a headword's name] → Read on that headword's spelling only
  (D4; open question 2).
- [A pronoun before an auxiliary] → « Me voy a lavar las manos » writes no pronoun just before the
  run: a pronoun-bearing headword gives way there, as it does today when it does not win the key.
- [An object pronoun before the verb] → It lets a pronoun-bearing headword be weighed (« Los tenía
  puestos » `tenerlos puestos`); it does not make it win (D3).
- [A dictionary's plural gloss] → `muñecas rusas` « Mise en abyme », `vendedores ambulantes` « À la
  sauvette » (5 on the corpus): a reducer's ground.
- [Names across natives] → 2,153 → 2,450 of 115,262 places (D2).
- [An older core] → Answers as today (D1); a pack ships with the core built beside it.
- [Cost] → es-fr +2,379 B, es-en +3,662 B; gloss time within noise (D2).

## Migration Plan

Nothing to migrate: no stored format, wire field, table or `pack_version` moves, and no record is
rewritten. The Spanish packs gain an optional section and their pins move. A record set before the
release keeps answering where 44b's D6 reads it, but for a lemma chain that is a headword's name,
which answers that headword (D4). Rollback is a revert, which returns every key to its first headword.

## Effort

0.5–1 ideal day: the builder's condition 0.05; Spanish's clitic test and the guard in the choice
0.15–0.25; the lemma-chain exception 0.05–0.1; review 0.05; their tests 0.1–0.25; the probes, the
goldens, the pins and the READMEs 0.1; spec and programme 0.05.

## Open Questions

For the owner, none blocking — each with the design's recommendation:

1. **The guard on pronouns (D3).** Without it, « El guardaespaldas tenía puestos unos audífonos »
   (« the bodyguard was wearing headphones ») would show `tenerlos puestos` « to have balls » in es-en,
   because its word `puestos` is spelled as on the page. Recommended: a dictionary entry whose verb
   carries a pronoun (`tenerlos`, `hacerse`, `ayudarme`) is only picked when the page has such a
   pronoun, stuck to a verb (« hacerme ilusiones ») or just before it (« No te hagas ilusiones ») — it
   moves only those 6 matches. The alternatives: no guard (the 6 stay wrong), or a stricter guard that
   only counts a pronoun stuck to the verb (« No te hagas ilusiones » would then show `hacer ilusión`
   « to cause to get excited about something », 6 times).
2. **A status set before the release (D4).** A reader of es-fr who marked « hace tiempo » known before
   this release saw it labelled `hacer tiempo`, so the mark is stored under `hacer tiempo` — which is
   also the name of the other expression, « to play for time ». Recommended: from the release, that
   mark belongs to `hacer tiempo`; « hace tiempo » shows `hace tiempo`, not yet marked. Otherwise,
   anyone who marks `un poco` known would see « un poquito » labelled `un poco` forever (71 of es-fr's
   98 doubled entries are like this). The alternative keeps old marks on every spelling and accepts
   that leak.
3. **The pronoun as a hint (D3, Non-goals).** « Me di cuenta » (« I realised ») keeps showing `dar
   cuenta` « Rendre compte » in es-fr, as today, rather than `darse cuenta` « Se rendre compte ».
   Counting the pronoun as part of the spelling would fix it (146 matches) but would also label « Ella
   me dio la espalda » (« she turned her back on me ») `dándose la espalda` « Dos à dos », and « me
   sirve de luz » `servirse de` « S'aider de », because « me » there is not reflexive. Recommended:
   leave it out; a change that tells « me » the reflexive from « me » the object could do it later.
