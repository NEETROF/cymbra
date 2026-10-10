# Design — refine-lingua-fr-es-glosses

## Context

See proposal.md (Why). What exists, on `main` at `101b684d` (change 49's implementation, #862, and
48b's proposal, #861; nothing of `scripts/lingua-data` that fr-es reads has moved since `59867512`,
where this design was first measured):

| Where | What |
|---|---|
| `reduce-fr-es.py` | fr-es's reducer, a reader pair of French (change 49 D1). Section: `straight_apostrophes` → `common.without_letter_senses` → `spanish.read_as_meanings` (24b: senses marked obsolete or outdated after the others of their entry, one typography) → `french_entries` (a name's note off a common word; possessives and demonstratives, and their forms, as determiners) → `native_side` (`common.reduce_gloss`, `no_self_definition`, then the direct table in its order, then the inverted table by Spanish frequency; expressions alike). Tables: `translation_words` (letters out, `’` read as `'`) → `common.read_translations` (lowercases the French word) → `in_typography` (direct) or `listed_once` (inverted: a Spanish word once, under the first part of speech French's readings name). Rule digest: `reduce-fr-es.py`, `reduce_common.py`, `reduce_edition_es.py` |
| `reduce_edition_es.py` | The Spanish edition's rules, read by en-es and fr-es: editing it re-pins both. `reduce_common.py` is every pair's. Neither is edited here |
| kaikki's Spanish senses | A sense carries the edition's labels twice: as kaikki's English `tags` (`colloquial` 164 senses of the French section, `figurative` 121, `obsolete` 94, `vulgar` 71, `outdated` 66, `slang` 43, `derogatory` 41, `rare` 32, `Quebec` 29…) and as the edition's own `categories`, in Spanish (« FR:Términos coloquiales », « FR:Términos malsonantes », « FR:Términos anticuados », « FR:Quebec », « FR:Bélgica »…). A `contraction` entry (`des`, `du`) has no part of speech of UD's: the shared rules read it as `X` |
| The translation files | Direct (`kaikki-fr-traductions.jsonl`): per French entry, its Spanish translations' `word` and `sense`, the French headword as written (« DS »). Inverted (`kaikki-es-traductions.jsonl`): per Spanish entry, its `pos` and the French words it lists, as written (« US », « Luc », « lOrient », and « fr », the template's language code, in `calabaza` and `imperial`) |
| `tables/fr-es/` | 19,050 glossed lemmas (4,560 definitions, 13,195 direct, 1,295 inverted), 12,177 expressions; coverage 83.2 / 70.8 / 56.8 % (4,161 / 7,082 / 11,359) against `FLOORS["fr-es"]` 81.4 / 68.8 / 54.5; pinned at `2026.10.10` from `lingua-pack-sources-fr-es-2026.10.10`, `pack_version` `2026.10.10+3f293e8.e18e7e8`, the pack 1,973,407 B. Its README's *Known data defects* and change 49's Open Questions 2–8 are this change's input |
| Change 48b | `refine-lingua-fr-en-glosses` (proposed, #861): fr-en's rules in fr-en's reducer; French's dictionary words (`tables/fr/lexical.tsv`) leave out the lemmas fr-en glosses as names alone; a French level only for a dictionary word (`level.tsv`). Both tables are in fr-es's pin's `studied` record |
| UD French-GSD | Its training and development sections, pinned at commit `94d5b68e…` in `pack_sources.PINNED["fr-en"]` and recorded in fr-en's pin (`gsd-train`, `gsd-dev`, sha256 `4b9a87b1…`, `9221e508…`), read by fr-en's reducer for which lemma a form takes; 48b's D5 reads them again for the part of speech a function word's row opens on. fr-es reads no treebank today |
| Changes 51 and 52 | 51 (`add-lingua-french-word-card`, proposed) lists fr-es's defects in its *Known data defects* and writes fr-es's golden; 52 (`enable-lingua-french`, proposed) lists fr-es once this change has merged (its Open Question 4, settled 2026-10-09) |

## Goals / Non-Goals

**Goals:**
- The owner's four decisions of 2026-10-10, and his answers the same day to this proposal's four
  questions, applied to fr-es, each measured on the whole table and on the top 10,000, with what it
  costs, by rules of fr-es's own reducer.
- The defects changes 49, 51 and 52 list for fr-es fixed, or left with their counts and where their
  fix belongs.
- Each of 48b's decisions read for fr-es: applied, or already reaching fr-es through French's
  tables.
- fr-es at or above its floor.

**Non-Goals:**
- Any byte of en-fr, es-fr, es-en, en-es or fr-en, or of `tables/fr/`: no rule of `reduce_common.py`
  or of an edition module, though some of these rules would suit en-es (24b's Q1, Q4).
- A new fetch beyond UD French-GSD's two pinned files (D8): fr-es keeps its snapshot and its three
  kaikki files.
- The card's wording (change 51), French's studied side (changes 43–46, 48b), and the seeding of a
  review deck by level, settled by the owner as a core change of its own (*For the owner*).

## Measured

A prototype in the scratchpad (never committed): `reduce-fr-es.py` of `main`, each rule behind a
switch, its inputs the three files of `lingua-pack-sources-fr-es-2026.10.10` (sha256 `2e724e39…`,
`bf44ebb4…`, `bd14a5c4…`, the bytes the pin records), UD French-GSD's two sections (the sha256 fr-en's
pin records), `tables/fr/` as committed, wordfreq 3.1.1 under Python 3.12. With every switch off it reproduces the committed `gloss.tsv`, `senses.tsv` and `mwe.tsv`
byte for byte. Packs built by this branch's `lingua-pack-build`, which rebuilds the committed pack to
its pin's sha256. Rows are the lemmas of `gloss.tsv`; « top 10k » those ranked 1–10,000 in
`tables/fr/freq.tsv`.

**The defects, and what this design does:**

| # | Class | Rows / top 10k | Examples | Decision |
|---|---|---|---|---|
| 1 | A sense's register, age or place not shown | 202 / 109 rows hold a labelled sense — register 137 / 78 (210 senses), age 59 / 26 (66), place 32 / 18 (39); 85 / 42 open on one | « baiser » « Coger (sexualmente); …; Besar; Beso… », « mec » « …; Proxeneta », « on » « …; Nosotros; Yo; Tú », « vous » « …; Usted » | Shown (D2) |
| 2 | One Spanish word under two runs of the direct table | 654 / 222 rows; 415 / 128 repeat a run word for word (change 49 counted 421 / 129) | « parti » « Partido; Partido », « européen », « extra », « renaissance »; « jeune » « Joven; Joven, chaval, muchacho » | Listed once (D3) |
| 3 | The inverted table's names, acronyms and other senses | about a dozen of its 265 rows of the top 10,000 | « us » « EEUU », « fr » « Imperial, calabaza », « luc » « San Lucas », « matthieu » « San Mateo », « pa » « Autopiloto », « lorient » « Oriente », « rap » « Secuestro », « pilote » « Controlador », « merlin » « Meollar », « teint » « Teñido », « excité » « Jarioso » | Left out (D4) |
| 3b | The direct table's other-sense word | 1 / 1 | « ds » (4,547) « Tiburón »: the French Wiktionary's « DS », the car | Left out (D4) |
| 4 | The French word given as its own gloss | 24 rows where wordfreq rates it a hundred times commoner in French than in Spanish, and 8 expressions | « arnaque » « Arnaque », « gâche » « Gâche », « retraite » « Retraite, jubilación, retiro », « mutuel » « Mutuel »; loans: « diaporama », « raï » | French dropped, loans kept (D5) |
| 5 | An infinitive's noun before its verb | 2 / 2 | « être » « [sustantivo] Ser; [verbo] (être + participio) Haber; Estar; … », « devoir » « [sustantivo] Deber » | The verb's (D6) |
| 6 | A contraction with no part of speech | 6 / 5 | « des » « [determinante] Algunos…; [—] Contracción de la preposición de y el artículo les… », « du », « duquel », « audit » | A preposition (D6) |
| 7 | The part of speech named again after the meaning | 2 / 2 | « qui » « Quién. (Pronombre nominativo.); Que. (Pronombre nominativo.) », « quoi » | Out (D6) |
| 8 | « etc » without its period | 5 / 4 | « avec », « adresse », « regard », « cochon » | « etc. » (D6) |
| 9 | The sources' own slips | 4 rows and 1 expression, all of the top 10,000 | « rien » « Pequeño cantidad de algo », « amie » « Amia o lamia » (the fish), « el » « Ella, ello o él », « hall » « Explanada », « il y a » « Hace » | Corrected here, reviewed (D7) |
| 10 | A quantifier or a possessive headed as a pronoun | 3 / 3 | « autre » « Otro », « peu » « Pocos », « leur » « Suyo, suya » | Left (D9) |
| 11 | A note before the meaning | 46 / 23 (change 49) | « être » « (être + participio) Haber », « chien » « (Canis lupus familiaris) Perro » | Kept (D9) |
| 12 | A function word's row opening on another part of speech | 9 / 9 by UD French-GSD, as 48b reads it | « pas » « Paso; No », « pendant » « Pendiente; Durante », « autour » « Halcón; Alrededor », « un », « que » | The treebank's order (D8) |
| 13 | A French level with no Spanish gloss | 1,188 of French's 8,302 levelled lemmas (22 at A1, 67 at A2) | `parce`, `part`, `lors`, `afin`, `taux` | Seeding skips them: a core change of its own, before 52 (*For the owner*) |
| 14 | A labelled sense of an expression | 59 expressions, 70 senses | « fils de pute », « mal aux cheveux » | Shown, outside the shared cut (D2) |

**Each rule alone, then together** (rows / top 10k):

| Rule | Where | Rows | First sense | Lemmas | Expressions | Examples |
|---|---|---|---|---|---|---|
| Labels (D2) | `with_labels` | 202 / 109 | 85 / 42 | — | — | « baiser », « mec », « rien », « ça » « (coloquial) Eso, esto, aquello » |
| Labels on expressions (D2) | `label_expressions` | — | — | — | 59 changed (70 senses) | « fils de pute » « (vulgar) Hijo de puta, hijoputa o máncer; … » |
| Listed once (D3) | `listed_once_direct` | 654 / 222 | 113 / 44, 19 / 6 opening on another word | — | 1 changed | « parti » « Partido », « russe » « Ruso, rusa », « critique » « Crítica, crítico » |
| Names and acronyms read backwards (D4) | `translation_words` | 14 / 8 | 1 / 0 | 13 / 8 lose a gloss | — | « us », « fr », « luc », « pa », « lorient »; « usa » lost |
| Other senses, by name (D4) | `OTHER_SENSES` | 6 / 6 | — | 6 / 6 lose a gloss | — | « rap », « pilote », « merlin », « teint », « excité », « ds » |
| The French word (D5) | `without_french_words` | 12 / 4 | 3 / 2 | 8 / 2 lose a gloss | 5 changed, 3 lost | « retraite », « mutuel », « conjugal »; « arnaque », « gâche » |
| An infinitive's noun (D6) | `french_entries` | 2 / 2, runs alone | — | — | — | « être », « devoir » |
| A contraction of a preposition (D6) | `french_entries` | 6 / 5, runs alone | — | — | — | « des », « du », « duquel », « audit », « ès », « dudit » |
| The part of speech named again (D6) | `french_entries` | 2 / 2 | 2 / 2 | — | — | « qui » « Quién; Que », « quoi » |
| « etc. » (D6) | `with_etc_period` | 5 / 4 | 1 / 1 | — | — | « cochon » « Cerdo, marrano, guarro, cochino, etc. » |
| The sources' slips (D7) | `CORRECTIONS` | 4 / 4 | 2 / 2 | 1 / 1 loses a gloss (`el`) | 1 changed | « rien », « amie » « Amiga; Amia o lamia », « hall », « il y a » « Hay; Hace » |
| The treebank's order (D8) | `treebank_order` | 9 / 9 (2 / 2 runs alone) | 7 / 7 | — | — | « pas » « No; Paso », « pendant » « Durante; … », « toutefois » « Todavía; … » (worse) |
| **Together** | | **912 / 371** (9 / 8 runs alone) | **214 / 100** | **28 / 17 lose a gloss, none gained** | **66 changed, 3 lost** | coverage 83.2 / 70.8 / 56.8 → **83.0 / 70.7 / 56.7 %** |

**Coverage, against the floor:**

| Lemmas | Committed | This design | Floor | Margin |
|---|---|---|---|---|
| top 5,000 | 83.2 % (4,161) | **83.0 %** (4,150) | 81.4 % | +1.6 |
| top 10,000 | 70.8 % (7,082) | **70.7 %** (7,065) | 68.8 % | +1.9 |
| top 20,000 | 56.8 % (11,359) | **56.7 %** (11,340) | 54.5 % | +2.2 |
| all 60,000 | 31.8 % (19,050) | 31.7 % (19,022) | — | — |

19,022 glossed lemmas: 4,559 definitions (24.0 %), 13,185 direct, 1,278 inverted; of the glossed top
10,000 (7,065), 2,849 definitions, 3,963 direct, 253 inverted — 59.7 % from a table (59.8 %
before). 12,174 expressions. The pack: 1,971,286 B (−2,121). No row of the card is empty or ends on an
opening mark under the row cut (`rowGloss`, measured on the 19,022 rows).

## Decisions

### D1 — Every rule is fr-es's, in its reducer

The rules run in `reduce-fr-es.py` and nowhere else, as 48b's run in fr-en's:
- `french_entries` (the section's pre-pass, change 49 D5) gains three readings (D6) — an
  infinitive's noun after its verb, a contraction of a preposition as a preposition, a part of speech
  named again after the meaning —, and `with_labels`, a pass after it, writes the labels (D2);
- `translation_words`, which still sees the French word as each table writes it, leaves out the
  inverted table's names and acronyms and the named other-sense words (D4);
- `read_translated` lists the direct table's words once (D3) and leaves out the French word
  (D5);
- `with_etc_period`, a post-pass over the reduced glosses and expressions (D6), and
  `label_expressions`, one over the section's expressions after the shared rules cut them (D2);
- `CORRECTIONS`, the sources' slips corrected as the section and the direct table are read (D7);
- `treebank_order`, a pre-pass over the section's entries after `french_entries`, from a module of
  French's treebank counts, `reduce_french_treebank.py` (D8).

*Why not the Spanish edition.* Labels, the translation tables' words and « etc. » would read better in
en-es too (24b's Q1 and Q4 are open), but a rule of `reduce_edition_es.py` re-pins en-es, which must
not move here; where one suits en-es it is named for en-es's own refinement. *Why not
`reduce_common.py`.* Every pair's digest. fr-es's digest is `reduce-fr-es.py`, `reduce_common.py` and
`reduce_edition_es.py`, and this change adds `reduce_french_treebank.py`, which no other pair loads:
only fr-es re-pins.

### D2 — A sense shows its register, its age and its place, in the edition's words

**Where the labels come from.** The Spanish Wiktionary's French section files each labelled sense in
its own categories, in Spanish: « FR:Términos coloquiales », « FR:Términos anticuados », « FR:Quebec ».
kaikki also renders the labels as English tags, but the categories are the edition's words and hold
more: measured, they give 19 labels the tags lack (« Bélgica » 9 times, among them « baiser »'s
« Besar » and « avec »'s « También »; « República Democrática del Congo » and « Ruanda » 4 times each
on « nonante » and « septante »; « África »; one « anticuado ») and none the tags have and they lack.
An editor's template that named the wrong language files « ES:Términos anticuados » (« maîtresse »,
« item »): read alike.

**Which.** The three kinds the owner named, from closed tables:
- **register**: « coloquial » (98 senses of the committed rows), « malsonante » (40), « jergal » (36),
  « despectivo » (24), « literario » (17), « vulgar » (7), « eufemístico » (7), « formal » (5),
  « infantil » (5), « irónico » (1), « jocoso » — each the category « Términos … » it comes from, in the
  singular;
- **age**: « anticuado » (37), « obsoleto » (29);
- **place**: « Quebec » (13), « Francia » (12), « Suiza » (6), « Canadá » (5), « Bélgica » (4),
  « Provenza » (4), « República Democrática del Congo » (2), « Ruanda » (2), « Toulouse », and the
  continents (« Europa » 1, « América », « África ») — a place inside another shown alone: « Quebec »,
  not « América, Canadá, Quebec ».
Not shown: « en sentido figurado » (a sense in 55 / 31 rows) and « infrecuente » (5 / 3), which say how
a meaning is used, not its register, age or place; a sense that only points at another word, which
glosses nothing.

**How.** The labels open the sense in parentheses, register first, then age, then place, each in its
table's order: « baiser » « (malsonante) Coger (sexualmente); (malsonante) Culear, follar, fornicar,
joder o realizar el coito; (malsonante) Dominar o joder; (malsonante) Quebrar o romper; (jergal)
Grapar; (anticuado, Canadá, Bélgica) Besar; Beso, besuqueo u ósculo », « mec » « (coloquial, jergal,
Francia) Hombre enérgico; …; (coloquial, jergal, anticuado, Francia) Proxeneta », « rien » « Nada;
Poca cosa; (obsoleto) Algo; Pequeño cantidad de algo; (coloquial, irónico) Muy; Mucha, muchas, mucho o
muchos », « vous » « Vosotros, ustedes; (formal) Usted », « ça » « (coloquial) Eso, esto, aquello ».
Each labelled sense carries its own labels, repeated (« on » « Se, alguien o uno…; (coloquial)
Nosotros; (coloquial) Yo; … »): a label written once for several senses would read as the first
one's. 202 / 109 rows, the first sense of 85 / 42; no lemma gains or loses a gloss. Four labelled
first senses run past the row's 80 characters and are cut as before (« merde », « nonante »); none
ends on an opening mark.

**The order is not changed for a label** — the owner asked whether a row should open on an unlabelled
sense; measured, no:
- labelled senses after the unlabelled ones of their entry: 16 / 13 rows, and « cul » opens on
  « Fundo (de un objeto) », « trouille » on « Alheña, aligustre », « dame » on « Cualquier mujer »: a
  colloquial word's colloquial sense is what it means;
- a row opening on an entry holding an unlabelled sense when the first entry holds none: 9 / 7 rows;
  « baiser » would open on « Beso, besuqueo u ósculo », « item » on « Elemento o ítem », but « coucou »
  on « Cuco », the bird, and « grégeois » on a commune.
The label says what the order cannot. The senses the edition marks outdated or obsolete stay after the
current ones of their entry (24b's D5, already fr-es's): « rien »'s « Algo » after « Poca cosa ».

**Expressions too, outside the cut** (the owner, 2026-10-10). The cut of an expression is the pack's,
not the card's: `reduce_common.reduce_expressions` keeps 42 characters of each sense, three senses,
and 80 characters of the joined gloss; the card shows an expression's gloss whole on its card (pages of
160 characters, `gloss-pages.ts`) and its first sense as a row (`rowGloss`, 80 characters). Labels
written into the section before the shared rules read it would count in the 42: measured, 39 of the 59
labelled expression senses would then lose words (« mal aux cheveux » « (anticuado) Resaca, caña,
chaqui, chuchaqu »). So the section's expressions are reduced unlabelled, as today, and
`label_expressions` labels them afterwards: it rebuilds the shared rules' choice of senses — the same
round-robin, the same 42-character cleaning — and walks the joined gloss by their lengths, opening
each sense, or the cut remnant of the last, on its source sense's labels. Measured: 59 expressions, 70
senses labelled; stripped of their labels, all 59 glosses are the committed ones byte for byte, so the
label costs the meaning no character — « mal aux cheveux » « (anticuado) Resaca, caña, chaqui,
chuchaqui, cruda, go » keeps the 42 characters it had, « fils de pute » « (vulgar) Hijo de puta,
hijoputa o máncer; (malsonante, despectivo) Bastardo, hijo de la chingada, hijo de la; (malsonante,
despectivo) Obje » the 80 it had, « bon sang » « (coloquial) Caramba, diablos, joder, maldición,
maldit ». The labelled gloss runs past the 80 characters in 13 of the 59 (the longest 139), as 48b's
« etc. » runs one past: no part of the pack bounds an expression's gloss, none reaches the card's
160-character page, and no expression's row is cut by the card's 80 characters. The shared 42 and 80
still cut the meanings they cut before; changing them is every pair's (`reduce_common.py`).

### D3 — A word of the direct table listed once

The French Wiktionary's translators write a Spanish word under each part of speech of the French
entry, and the table glosses each part of speech with its first three words: « parti » « Partido;
Partido », « jeune » « Joven; Joven, chaval, muchacho ». `listed_once_direct`, on the words a run
shows:
1. a run whose shown words another run of the word shows too goes — of two equal runs, the later:
   « parti » « Partido », « européen » « Europeo », « extra » « Extra », « russe » « Ruso, rusa »,
   « jeune » « Joven, chaval, muchacho »;
2. a word still shown by two runs stays in the first that shows it and leaves the others, whose next
   words come up: « clair » « Claro, luminoso, límpido; Claramente », « sérieux » « Serio, formal;
   Seriedad »; a run left with no word goes.

654 / 222 rows; no lemma gained or lost; one expression. The heading of the first run changes in 113 /
44 rows, the run with more words keeping its part of speech (« jeune » « [sustantivo] Joven, chaval,
muchacho »); the opening word in 19 / 6 (« critique » « Crítica, crítico », « propriétaire » « Dueño,
propietario »). « parti » keeps the adjective's heading the table lists first — the French Wiktionary's
heraldic adjective — and the sample names it.

*Rejected — each word in the first run that shows it, no run dropped* (step 2 alone): 54 / 19 rows are
left with a run holding only the other gender — « russe » « Ruso; Rusa », « chinois » « Chino; China »,
the second read as the country. *Rejected — each word under the first part of speech French's
readings name* (change 49's rule for the inverted table): 29 / 8 rows change their opening, among
them demonyms split the other way, opening on the feminine — « libyen » « Libia; Libio », « malaisien »
« Malasia; Malasio », « palestinien » « Palestina; Palestino », each a country's name.

### D4 — The inverted table's names and acronyms, and its other-sense words, left out

The Spanish Wiktionary lists French translations under its Spanish entries; read backwards, a French
word written as a name or an acronym, or a translation of another sense, glosses the French lemma
spelled like it. `translation_words` sees the French word as the entry writes it (`read_translations`
then lowercases it), and leaves out:
- **« fr »**, the translation template's language code, which kaikki reads as a French word in
  `calabaza` and `imperial`: « fr » « Imperial, calabaza »;
- **an acronym's other word**: a French word of two letters or more, all capitals, glosses its lemma
  only through itself, the letters compared without dots, spaces or case (« ONU » « ONU », « HTTP »,
  « AEC » « A. e. c. »): « us » « EEUU », « pa » « Autopiloto »;
- **a saint for a first name**: a Spanish « San », « Santa » or « Santo » glosses no one-word French
  name: « luc » and « lucas » « San Lucas », « matthieu » « San Mateo »; a name translated by a name
  stays (« pâques » « Pascua », « toussaint » « Día de Todos los Santos »);
- **an elided article read into a word**, a lower-case letter followed by a capital: « lOrient »,
  kaikki's « l'Orient », for the town `lorient`.
14 / 8 rows; 13 / 8 lemmas lose their gloss — the seven above, and beyond the top 10,000 « cci »
« CCO », « pnb » « PNV », « fsh » « MSH », « fsf » « MSM »; « tls » keeps « TLS » without « SSL ».
Lost though right: « usa » (1,759) and « éu » « EEUU », an acronym translated by Spanish's.

And, by name, **the other-sense words of the top 10,000 no measured rule tells from a translation**,
in `OTHER_SENSES`, each pair a French word and a Spanish word with its reason:

| French | Spanish | Why |
|---|---|---|
| rap (3,698) | secuestro | « secuestro » lists « rap » for « rapt » |
| pilote (1,694) | controlador | a device's driver, not the pilot |
| merlin (6,894) | meollar | a rope on a ship, not the maul or the wizard |
| teint (5,261) | teñido | the participle of *teindre*, not the complexion |
| excité (6,028) | jarioso | a regional word for one sense |
| ds (4,547) | tiburón | the direct table's « DS », the car's Spanish nickname, for `ds`, French shorthand for *dans* |

6 / 6 rows, each losing its gloss: neither table has another word. A pair fires on nothing once the
source changes it.

**What is lost**: 19 lemmas / 14 of the top 10,000 are left unglossed, two of them rightly glossed
before (« usa », « éu »); 0.14 points of the top 10,000's coverage; 8 of them carry a French level
(`us`, `pilote` A2; `rap`, `pa`, `ds` B1; `teint`, `excité`, `merlin` B2), so a card seeded from them
would carry no gloss, until the seeding change skips them (D10). No gloss is better than another sense's word (M5's « written by a
person » holds; « Secuestro » is not what `rap` means).

*Rejected*, measured on the inverted table's 265 rows of the top 10,000 unless said:
- **every French word written with a capital** glossing only through a word spelled alike: 33 / 11
  lemmas and 84 expressions lose their gloss — « pâques » « Pascua », « pentecôte » « Pentecostés »,
  « cendrillon » « Cenicienta », « mer Rouge » « Mar Rojo » among them;
- **a French word Spanish also writes** (wordfreq ≥ 3.0 Zipf) glossed by another word: 19 rows, 11 of
  them right (« sobre » « Escueto », « river » « Remachar », « ferry » « Transbordador »);
- **only the French word a Spanish entry lists first**: 41 rows, about 30 of them right (« cet »
  « Este », « thème » « Tema », « patate » « Patata »);
- **24b's floor on the Spanish word's frequency** (under 2.5 Zipf): 22 rows, 3 of them wrong (« pa »,
  « excité », « merlin »).
None of them catches « rap »: hence the list.

### D5 — The French word is no Spanish gloss

The French Wiktionary's translators sometimes list the French word itself as Spanish (« arnaque »,
« gâche ») or among Spanish words (« retraite » « Retraite, jubilación, retiro »): M5 and *A gloss is
in the reader's language, written by a person*. `without_french_words` leaves out of the direct table
a Spanish word spelled as the French headword, case aside, that wordfreq rates at least two Zipf points
— a hundred times — commoner in French than in Spanish, unless it is in `SPANISH_ALIKE`, the loanwords
Spanish writes alike: « diaporama », « redingote », « aguerrir » (in the RAE's dictionary); « raï »,
« poutine », « tartiflette », « andouillette », « navarin », « savate », « calanque », « joual » (the
names Spanish texts give a French music, dish, sport, coast or dialect); « vivarium » (Latin) — twelve,
reviewed by the owner. A word left out is replaced by the table's next word, and a word left with none
falls to the inverted table, as a letter does.

12 / 4 rows: « retraite » (965) « Jubilación, retiro, pensión », « mutuel » « Mutuo », « conjugal »
« Conyugal », « négligé » « Descuidado, desaliñado »; 8 / 2 lemmas lose a gloss that was French —
« arnaque » (4,603), « gâche » (9,447), « adage », « crêperie » (Spanish writes « crepería »),
« opex », « désinscrire », « smala », « cauchois ». 5 expressions change (« herbes de Provence »
« Hierbas de Provenza », « parti pris » « Compromiso ») and 3 lose their gloss (« bouchée à la
reine », « contrôle continu », « gueule cassée »). The candidates a later update brings are dropped
until the owner lists them, and the update's pull request names them.

*Rejected — no Spanish frequency at all* (Zipf 0, change 49's measure): 20 rows, missing « retraite »
(Spanish 1.23, French text in the Spanish corpus), « mutuel » and « conjugal ». *Rejected — ten times
commoner*: 75 rows, about 50 of them words Spanish writes alike (« portable », « canapé », « frigo »,
« brioche », « fondue »): a list four times longer for few more French words.

### D6 — Changes 51's and 52's defects

- **An infinitive's noun after its verb.** The section enters `être`'s noun (« Ser », a being) before
  its verb (« Ser », « Estar »); the shared round-robin keeps the first of two identical senses, so the
  verb lost « Ser » and the row opened on a noun. `french_entries` writes a noun entry after the verb
  entries when every sense of it is, word for word, a sense of the verb's and French's readings
  (`grammar.tsv`) name the word a verb: `être` « Ser; (être + participio) Haber; Estar; (être +
  participio) ser », every run a verb's; `devoir` « Deber », a verb's. 2 / 2 rows, the runs alone.
  *Rejected — any entry another entry repeats*: 27 / 17 rows, « jaune » « Amarillo » and « moi » « Yo »
  read as nouns, « pendant » opening on « Pendiente ».
- **A contraction of a preposition is a preposition.** A `contraction` entry whose sense opens
  « Contracción de la preposición » is read as `prep` (ADP, the card's « preposición »): `des`
  « [determinante] Algunos, algunas, unos o unas; [preposición] Contracción de la preposición de y el
  artículo les, de las o de los », `du`, `duquel`, `audit`, `ès`, `dudit`. 6 / 5 rows, runs alone. The
  section's other contractions (« c'est », « qu'il ») are no lemma French's tables hold.
- **The part of speech named again.** A parenthesis after the meaning's period that opens on
  « Pronombre » — what the card's heading already says — goes with the period: `qui` « Quién; Que »,
  `quoi` « Qué; Algo; …; Cual cosa, lo que etc. ». 2 / 2. The section's other notes after a meaning
  stay: they say something the heading does not (« (Plural exclusivo.) » under « nous autres »,
  « (Usado para solo una persona.) » under « s'il te plaît », « (Literalmente: …) », `il`'s « (No tiene
  traducción al español. No existe en español.) »).
- **« etc. »**, fr-es's copy of 48b's D8: after the shared rules, « etc » not followed by a period is
  written « etc. », as the RAE writes it — the shared cleaning takes off a sense's final period, so a
  pre-pass cannot keep it. 5 / 4 rows (« avec », « adresse », « regard », « cochon », « moucher »), no
  expression. The shared fix for every pair stays 24b's D8.
- **`rien`** shows its labels (« (obsoleto) Algo », « (coloquial, irónico) Muy »); its « Pequeño
  cantidad de algo » is the Spanish Wiktionary's own agreement slip, corrected (D7).

### D7 — The sources' slips, corrected here

The owner settled on 2026-10-10 that the five rows the sources themselves write wrong are corrected in
fr-es's reducer, reviewed, each with its reason, and reported upstream. `CORRECTIONS` holds them, each
keyed by the source's text as the pinned files write it — the entry or the translation it replaces —,
so that it fires on nothing once the page is corrected:

| Row | The source writes | Read as | Why | Report to |
|---|---|---|---|---|
| `rien` (64), the noun | « Pequeño cantidad de algo. » | « Pequeña cantidad de algo. » | an agreement slip | the Spanish Wiktionary's « rien » |
| `amie` (1,250), the noun | « Amia o lamia. » (the fish) alone; the friend only as « Forma del femenino singular de ami », a pointer the shared rules skip when the word has a meaning | « Amiga. », then « Amia o lamia. » | the commonest meaning missing; « amiga » is the French Wiktionary's own Spanish for `amie` | the Spanish Wiktionary's « amie » |
| « il y a », the expression | « Hace. » | « Hay. », then « Hace. » | « there is » missing; « hay » is the French Wiktionary's own Spanish for it | the Spanish Wiktionary's « il y a » |
| `el` (1,033) | the section's pronoun « Ella, ello o él. », and the French Wiktionary's table « elle » | no gloss | no French dictionary word: met in French text as the article of a Spanish or Arabic name (« El Niño »); the table's word is French | both |
| `hall` (2,973) | the French Wiktionary's one Spanish word, « explanada » | « vestíbulo », « recibidor » | an esplanade is no entrance hall; the one correction no source supplies, its two words written here | the French Wiktionary's « hall » |

4 / 4 rows (`rien`, `amie` « Amiga; Amia o lamia », `hall` « Vestíbulo, recibidor », and `el`, which
loses its gloss) and one expression (« il y a » « Hay; Hace »). A correction is a gloss written by a
person and read by the owner (M5's « written by a person »); three take their words from a source
(the French Wiktionary's « amiga », « hay ») or from the slip itself (« Pequeña »). The pull request
lists the five pages to correct on the Wiktionaries; once a page is corrected, the update that takes
it in finds its correction firing on nothing, says so in its summary, and removes it.

*Rejected — a rule.* No measured rule tells these from right glosses: the 111 / 77 rows holding a
pointer beside a meaning are nearly all verb forms beside a noun (« école », « groupe »), where the
meaning is right.

### D8 — A function word's row opens on the part of speech UD French-GSD reads it as

The owner settled on 2026-10-10 that fr-es takes 48b's D5. The section enters `pas`'s noun « Paso »
before its adverb « No », `pendant`'s adjective « Pendiente » before its preposition « Durante »,
`autour`'s noun « Halcón » (the goshawk) before its adverb « Alrededor ». `treebank_order` applies 48b's
rule as 48b writes it, threshold and boundary alike: UD French-GSD's training and development sections
counted — the auxiliary as a verb, a token inside a fixed expression for none, a noun, verb, adjective or
proper noun under its lemma, any other part of speech under its own form, in lower case and NFC —; a
headword's entries of its commonest part of speech written first when the treebank reads it **at least
10 times and twice as often** as the part of speech of its first entry, and that part of speech is a
**function word's** (`ADP`, `DET`, `PRON`, `CCONJ`, `SCONJ`, `PART`, `ADV`) or the first entry is a
**proper noun's**; a proper noun never moved first, the other entries in their order.

Measured: **9 / 9 rows**, all of the top 10,000 — the first sense of 7: `un` « Un; Un o uno; … »
(the article first), `pas` « No; Paso », `pendant` « Durante; Pendiente; Juego… », `aucun` « Ninguno;
Ninguno, nada », `autour` « Alrededor; Halcón », `envers` « Hacia; Con; El otro lado… », and
`toutefois` « Todavía; sin embargo, no obstante », **the one that reads worse**: « todavía » means
« nevertheless » only in a literary use the RAE records, and a reader takes it for « still »; the runs
alone of 2: `que` « [conjunción] Que; [pronombre] Qué », `donc` « [adverbio] Pues, entonces… ». No row
opening on a name moves: no section entry of fr-es qualifies. No lemma gains or loses a gloss.

**Shared, without moving fr-en.** 48b's rule is not implemented yet; its design writes it in
`reduce-fr-en.py`. fr-es cannot load that file — fr-en's every rule would enter fr-es's digest (change
49 D1) —, and moving it out after 48b would edit fr-en's digest. So the counting and the decision are a
module of their own, `reduce_french_treebank.py` (`gsd_pos_counts`, `commonest_first`, the thresholds
and the set as named constants), which fr-es imports now and fr-en does not, so fr-en's digest does not
move; 48b's implementation is handed the module to import instead of writing the rule again — fr-en is
re-pinned by 48b anyway, so sharing it then moves nothing more. If 48b's implementation lands first with
the rule in `reduce-fr-en.py`, this change's module is written from it, and the two copies are named in
the pull request as one rule for fr-en's next refinement.

**The source.** fr-es's pin records the two treebank files as fr-en's does: `pack_sources.PINNED`
gains `fr-es` with the same `gsd-train` and `gsd-dev` (URL at the pinned commit, sha256), which the pin's
`sources` record beside its kaikki files, untouched; a re-reduction fetches them, the reduce job's cache
holding them once for both pairs. `pack_sources.py` is in no digest.

### D9 — Measured and left

- **A quantifier or a possessive headed as a pronoun**: « autre » « [pronombre] Otro » (the section's
  adjective repeats it and the round-robin keeps the first), « peu » « [pronombre] Pocos » (the section
  has no adverb), « leur » « Suyo, suya ». French's readings name invariable words under no part of
  speech, and the treebank order reorders entries, not runs the round-robin merged, so no rule here can
  tell the commoner heading; 24b's Q3 (quantifiers) is the same question for en-es.
- **A note before the meaning** (46 / 23 rows, change 49): « (être + participio) Haber » says when
  « Haber » translates `être`; without it the sense would read as a wrong meaning. Kept, with change
  24's Q5 for en-es.
- **48b's other decisions**: D10.

### D10 — 48b's decisions, read for fr-es

| 48b | For fr-es |
|---|---|
| D1 — rules in the pair's reducer | The same: D1 here |
| D2 — names are no French dictionary words | Reaches fr-es through `tables/fr/lexical.tsv`, which fr-es's pack carries as French's dictionary words: fr-es glosses 511 of the 3,586 lemmas fr-en glosses as names alone (297 of the top 10,000: « françois », « marie »), which stay glossed and stop counting, as in fr-en. Nothing here |
| D3 — a level only for a dictionary word | French's levels are French's: fr-es reads them as committed. But 1,188 levelled lemmas have no Spanish gloss (1,198 after this change); a Spanish speaker's deck seeded from a level would hold cards with none. Not fixable in fr-es's tables without making French's levels depend on a pair that is not French's reference: the owner settled on 2026-10-10 that seeding a deck by level skips a lemma the reader's pack does not gloss — a core change of its own, for every pair, not this one, required before change 52 ships French |
| D4 — a pointer that carries its meaning | The Spanish edition writes few: « mieux » « Comparativo irregular de bien: mejor o más bien », « pis » « Comparativo de mal: peor », superlatives already glossed by the direct table (« rarissime » « Rarísimo »). Read as meanings, 2 rows would move (`mieux` « Mejor » under an adverb rather than the direct table's noun, `pis` gaining « Peor »). Left |
| D5 — a function word's row opens on UD French-GSD's part of speech | Taken, the same threshold and boundary, from a module both pairs can share: D8 |
| D6 — no name under a function word | Nothing to do: no function word's row of fr-es holds a name's run (`le` « El », `on`) |
| D7 — expressions left out or lent | None of `et des`, `que de`, `sur ce`, `un coup` is an expression of fr-es; `et si` « Y si » reads right. Of the six post-1990 spellings 48b lends a gloss, fr-es's tables gloss « à postériori », « et cétéra » and « être sur son trente-et-un » already; « à priori » and « sur son trente-et-un » meet nothing (« a priori » reads « A priori »). Lending them is 48b's rule for fr-en; 2 expressions, left |
| D8 — notes, openers, citations, « etc. » | « etc. »: D6 here. The English edition's notes and openers do not occur in Spanish; the Spanish section's « (Pronombre …) » note is D6's |

### D11 — fr-es re-pinned alone, at its snapshot

`build.sh --reduce fr-es` from `lingua-pack-sources-fr-es-2026.10.10` and UD French-GSD's two pinned
files (D8), nothing else fetched. The pin keeps its `snapshot`, its `studied` record and its three
kaikki files' records byte for byte, and its `sources` gain `gsd-train` and `gsd-dev`, as fr-en's
record them; its `reducer` digest moves (`reduce-fr-es.py`, and `reduce_french_treebank.py` added to its
`files`), with `pack_version` (`2026.10.10+<digest[:7]>.e18e7e8`, the studied digest unchanged), the
pack's sha256 and size.
`gloss_coverage.py --pair fr-es` holds `FLOORS["fr-es"]` (*Measured*: 83.0 / 70.7 / 56.7 %). `tables/fr-es/README.md` —
the figures, the rules and what each moves, the three lists, *Known data defects* rewritten as fixed and
left — and `SOURCES.md` (fr-es reads UD French-GSD) follow.

### D12 — What moves, what cannot, and in which order

- **What moves**: fr-es's `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `manifest.json`, `pin.json`,
  `README.md`; `pack_sources.PINNED` gains fr-es's treebank files. No golden or snapshot reads fr-es's glosses: `fr-en.golden` is an English-native
  reader's (fr-en beside es-en); change 51's `fr-es.golden` is not written; `selection-rows-fr.txt`
  pins en-fr's and es-fr's rows. `cross_native.rs` (`maison` « Casa », `quant` glossed and no
  dictionary word, the studied sections alike) and `committed_tables.rs` (`maison`, `et` « Y, e »,
  fr-es's pack against its pin) pass with the new pin; `row-gloss-tables.spec.ts` runs as the gate.
- **What cannot move**: en-fr, es-fr, es-en, en-es and fr-en — no file of their rule digests is
  edited, and the new module is in fr-es's alone; `tables/fr/` — fr-es reads it; the builder, the core,
  the extension.
- **48b** re-reduces fr-es too (its pin's `studied` record: `lexical.tsv`, `level.tsv`); none of
  this change's rules reads either table, so fr-es's glosses are the same either way, and whichever of
  the two lands second re-records fr-es's pin and pack on its branch. The treebank module: D8.
- **The seeding change** (the owner, 2026-10-10): a core change of its own, for every pair, before 52
  ships French; it moves no table of this change.
- **51**: if its fr-es golden and `word-card-fr-es.txt` land first, they are re-blessed here, each
  moved line with its rule; otherwise 51 draws them from these tables.
- **52** lists fr-es once this change has merged (its Open Question 4), and its `archiveAfter` gains
  this change.

### D13 — Every moved row read

The pull request carries the prototype's comparison over the committed tables: every changed row of
the top 10,000 (371 in the prototype) with its first differing sense and the rule that moved it, the
changed and lost expressions (the 59 labelled ones among them), the 28 lemmas left unglossed, the rows
D2's labels open (85), the rows whose heading D3 changes (« jeune », « parti »), the treebank's 9 rows
(« toutefois » named), and the three lists — `SPANISH_ALIKE`, `OTHER_SENSES` and `CORRECTIONS`, with the
pages to report upstream — for the owner, who approves the re-pin and reads them.

## For the owner

**Settled on 2026-10-10 (in session)**, and this design's: labels shown, in the edition's Spanish
words (Q2 of change 49); a direct-table word once (Q3); the inverted table's other-sense words out by
a rule, what is lost measured (Q4); the French word dropped, loans kept (Q8). The design decided what
they left open: which labels and where from (D2), no reordering for a label (D2), which run keeps a
word (D3), the rule and the list (D4), the threshold and the loanwords (D5).

**This proposal's four questions, answered by the owner the same day:**
1. **Labels on expressions — yes**, against the recommendation: written after the shared cut, so the
   meaning keeps every character it had (D2).
2. **The sources' five slips — corrected here**, reviewed, each with its reason, and reported upstream
   (D7).
3. **French levels with no Spanish gloss — seeding a review deck by level skips a lemma the reader's
   pack does not gloss**: a core change of its own, for every pair, **not this one**, required before
   change 52 ships French (D10).
4. **Function words by the treebank — yes, for fr-es too**, 48b's threshold and boundary, from a module
   fr-en can share without moving (D8).

Still the owner's: approving what moves and reading the moved rows (task 5.1), and the three lists
(5.2). A word moved from one side of a list to the other, or a correction reworded, joins this change
before it merges, fr-es re-pinned again.

## Risks / Trade-offs

- **[A label crowds a short gloss]** → labels repeat per sense (« on »); measured, no row of the card
  ends on an opening mark and four first senses are cut as before; the sample shows them.
- **[A heading changes with a dropped run]** (« jeune » « [sustantivo] Joven, chaval, muchacho ») →
  the words read right; the alternatives split genders (D3); named in the sample.
- **[A right gloss lost to the acronym rule]** (« usa » « EEUU ») → two, named; a list names the rest
  instead of a broader rule (D4).
- **[A loanword dropped as French]** → the candidates are a closed, measured class (24 rows), every
  one read by the owner, and a new one named in its update's pull request.
- **[A list ages]** → each entry is keyed by the source's words and fires on nothing once the source
  changes; the README names the three lists, and an update names a correction that no longer fires.
- **[A correction written here is a gloss no source wrote]** (`hall`) → one, read by the owner, its
  page reported upstream; the others take their words from a source.
- **[The treebank reorders a row for the worse]** (`toutefois` « Todavía ») → one of nine, named; 48b's
  threshold and boundary kept, so fr-en and fr-es read their function words alike.
- **[A labelled expression runs past 80 characters]** → 13 of 59, at most 139; nothing bounds it, the
  card pages at 160.
- **[48b, 51 and this change re-record fr-es]** → none reads another's output but through the
  committed tables; whichever lands second re-reduces fr-es (D12).

## Migration Plan

No reader holds a French pack before change 52: nothing to migrate. A rollback reverts
`reduce-fr-es.py`, its tests and `tables/fr-es/`.

## Effort

2–3 ideal days: the rules and their tests 1.25–1.75 (the expressions' labels, the corrections and the
treebank module among them); the treebank's pin and the re-pin, the README and `SOURCES.md` 0.5; the
comparison and the owner's sample 0.25–0.75.
