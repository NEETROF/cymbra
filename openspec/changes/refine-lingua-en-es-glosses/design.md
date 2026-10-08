# Design — refine-lingua-en-es-glosses

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `reduce-en-es.py` | `read_studied` (`tables/en/forms.tsv`, `freq.tsv`, `level.tsv`) → `without_letters` (`common.without_letter_senses`, `spanish.ES`) → `native_side`: `common.reduce_gloss` and `reduce_expressions` over the Spanish Wiktionary's English entries, then `fallback_glosses` over the direct table (the English Wiktionary's Spanish translations, in the table's order) and the inverted one (the Spanish Wiktionary's English translations read backwards, the commonest Spanish word first); `read_translated` reads each table through `without_letter_translations` and `common.read_translations` |
| `reduce_edition_es.py` | `ES`: `_FORM_OF` (an untagged pointer followed by « de »/« del »), `_NOTES` (sense-link subscripts, « Véase también »), `_LETTER`; `capitalised` true, `long_parenthesis` 0. No pre-pass. Loaded by en-es alone: en-fr's and es-fr's pins name `reduce_edition_fr.py`, es-en's `reduce_edition_en.py` (`git grep reduce_edition_es`; each pin's `reducer.files`). fr-es (the programme's change 49) will load it for the Spanish Wiktionary's French section |
| `reduce_common.py` | `_read_entries` reads a sense by `glosses[0]`, skips it when `_is_form_of`; `clean_gloss` (the edition's notes, whitespace, `.strip(" ;,").rstrip(".:")`, the cut); `_join_senses_by_pos` (round-robin across a word's entries in file order, up to eight, a sense identical to one already picked not picked again, grouped by part of speech); `_acronym`; `kaikki_upos` (`adj` → ADJ, `det` → DET, `name` → PROPN); `read_translations` (a `name` entry glosses nothing), `native_words`, `translation_gloss` (up to three words per part of speech, one sense each). Every pair loads it: editing it re-pins en-fr, es-fr, es-en and en-es |
| kaikki's Spanish senses | A sense carries the edition's labels as `tags` (`obsolete` 322, `outdated` 130, `colloquial` 430, `slang` 202, `UK` 175, `vulgar` 146, `US` 145…), `raw_tags` (« Arcaico », « obsoleta », « dialectal »…) and `topics` (`heraldry`…); an entry carries its section's kind as `tags` (an `adj` entry `possessive`, `demonstrative`, `indeterminate`); a proper noun is an entry of `pos` `name` under its capitalised headword (`Will`, `Smith`), read for the lower-case lemma like any other entry |
| The translation files | Direct, `kaikki-en-traductions-es.jsonl`: per English entry, its translations' `word` and `sense`, the translator's note inside `word` (« hall (hol) », « orquestra (disused) », « [4] a favor »). Inverted, `kaikki-es-traductions-en.jsonl`: per Spanish entry, its `word`, its `pos` and the English words it lists — `largo`'s adjective, noun, interjection and verb each list « lengthy » |
| `tables/en-es/` | 21,965 glossed lemmas (10,247 from the Spanish Wiktionary, 11,214 from the direct table, 504 from the inverted one), 17,096 expressions; pinned at `2026.10.08` from `lingua-pack-sources-en-es-2026.10.08`, `pack_version` `2026.10.08+75631d7.e1915ca`; coverage 93.0 / 85.0 / 71.7 % against `FLOORS["en-es"]` 91.4 / 83.2 / 69.9 (proposed; change 22's task 5.1, the owner's, is open); the pack 1,690,595 B. Its README's *Known noise, for the owner's 5.1* lists the translation tables' notes, the inverted table's regional words, its duplicated runs and the name notes |
| Change 24 | `crates/lingua-wasm/tests/en_es_baseline.rs` + `baseline/en-es.golden` (the reference's corpus and probes and 40 lemmas, among them `to`, `a`, `that`, `will`, `about`, `up`, `her`, `its`, `go`, `well`; `EXPEDITION_FIRST_PAGE` « Expedición »); `test/word-card-en-es.spec.ts` + `test/baseline/word-card-en-es.txt`; `test/row-gloss-tables.spec.ts` (no row of any pair ends on an opening mark; the French rows pinned in `test/baseline/selection-rows-fr.txt`); `lingua-pack-update` re-blesses all of them on a dictionary update's branch. Its design's *Known data defects* is this change's input; its *For the owner's review (4.2)* is open |
| Change 38 | `migrate-lingua-pack-sources-to-raw-dumps`, merged (#804): re-pins no pair; en-es's `kaikki-en` names the English edition's dump from en-es's next update on. Derived from that dump (2026-10-03 08:24) instead of the extract en-es pins, the direct table makes 186 glosses and 55 expressions take their words in another order or another third word, no row added or removed: "en-es's next update carries them, and the owner judges the 186 glosses there" (`SOURCES.md`, *Extract and dump are measured against each other*) |

## Goals / Non-Goals

**Goals:**
- en-es's glosses read as the Spanish Wiktionary's meanings and the translators' words, in one
  typography, before change 35 ships them — by rules of the Spanish edition and of en-es's reducer,
  measured on the whole table and the top 10,000.
- Every defect class of change 24's list decided — fixed, for the owner, or left — with its counts and
  where its fix belongs.

**Non-Goals:**
- Any rule of `reduce_common.py`; any other pair's bytes; the studied side (`tables/en/`).
- New data in the packs (labels as a field, a part-of-speech frequency); the card's wording (changes
  24, 33).
- A new fetch: en-es keeps its snapshot and its sources.
- Change 22's floor (task 5.1): the owner's, composed with below (D11).

## Measured

Rows are the lemmas of `tables/en-es/gloss.tsv` (21,965). "top 10k" are the first 10,000 lemmas of
`tables/en/freq.tsv` by rank, as `gloss_coverage.py` and change 22's share count them (8,495 glossed:
6,287 by the Spanish Wiktionary, 2,135 by the direct table, 73 by the inverted one). Text classes are
counted on the committed table; structural ones (the entry a sense comes from, its tags) on the pinned
sources as `reduce-en-es.py` cuts them, each kept sense traced to its entry. Each rule's effect is
measured on the real reduction: the shared rules (`common.reduce_gloss`, the fallbacks, the
expressions) over the cut files, with a scratch prototype of D2–D7 — which, with no rule, reproduces
the committed `gloss.tsv`, `senses.tsv` and `mwe.tsv` byte for byte. Classes overlap.

| # | Class | Rows: whole / top 10k | Examples | Decision |
|---|---|---|---|---|
| 1 | A surname's or a given name's note on a word's card | 754 / 484 rows hold one; 603 / 366 hold nothing else (the word is a name: « david », « wayne » « Apellido »); 61 / 53 hold one beside the senses of a word that has an entry in lower case, 16 / 12 opening on it; 90 / 65 beside a proper noun's other senses (« spain » « España; Apellido ») | « will » (37) « …; Apellido; Hipocorístico de William »; « smith » (1083) « Apellido; Herrero »; « mike » (1285) « Hipocorístico de Michael; Hipocorístico de Michaela; Micro, micrófono »; « small » (242), « white » (243), « king » (562) « …; Apellido » | Fix the 61 / 53 (D3); a name's own row kept (D8) |
| 2 | The edition's notes to its readers | 6 / 4 rows, 1 expression | « stage » (674), « unit » (964) « .^([cita requerida]) »; « bittersweet » « .^([definición imprecisa]) »; « favor » (1868) « Hacer un favor [sentido del sustantivo] para »; « hardcore » (5085) « Hardcore (definiciones [1,2]); Hardcore (definiciones [4,5]) »; « leaven » « Este lema en este idioma es ampliable. Retira este aviso… » | Fix (D2) |
| 2b | An optional word in brackets | 19 / 5 rows | « full-time » (220) « [a] tiempo completo »; « pram » « Cochecito [de bebé] »; « aria » « [el] aria » | Leave: Spanish, part of the gloss |
| 3 | An English label, a respelling or a sense number in a translation | 78 / 18 rows, 39 / 9 of them a word labelled disused; 42 expressions; in the edition's own senses 1 / 1 | « orchestra » (4805) « Orquesta, orquestra (disused) »; « hall » (1121) « Pasillo, hall (hol), jol »; « app » (1134) « Apli, app (ap) »; « insider » « Insider (insáider) »; « backwater » « villorrio (despective) »; « onside » « [4] a favor »; « to be honest » « Para ser honesto [with le and a, or with con] »; the edition's: « south » (318) « (region) Sur » | Fix the tables' (D7); « (region) » left (D8) |
| 4 | « etc » without its period | 47 / 34 rows, 1 expression; es-en 35 rows, es-fr 28, en-fr 57 | « oh » (292) « …desaprobación, etc »; « personal » (517); « fair » (888) « Moderadamente bueno, grande, amplio, etc » | Leave here: `reduce_common.clean_gloss`, every pair (D8) |
| 5 | A usage note in a sense | after the meaning: 55 / 33 rows write a sense in two sentences, 9 / 8 of them a usage note; before it: 146 / 92 rows open a sense on a parenthesis, nearly all a context (« (de él) Su, sus »); a function word's description as a sense: 24 / 19 rows | after: « a » (5) « Un, una. A veces se omite en la traducción » (its row), « wow » (939) « Guau. Empleada para expresar sorpresa… », « degree » (971) « Grado. A no confundir con Licenciatura… »; before: « get » (54) « (Seguido de un participio pasado) Ser », « go » (89) « Se emplea para iniciar un juego o competencia. ¡Ya!… », « have » (17) « De un recurso o sustancia, generalmente alimento: Consumir, usar »; a description: « be » (15) « Se usa en be to (no existe en español) », « will » « Úsase para construir el futuro » | After: fix (D2); the rest change 24's 4.2 (Q5) |
| 6 | A sense shown without its label | obsolete or outdated 176 / 107 rows (237 senses, 40 / 18 opening the row); regional 168 / 105 (225 senses, 66 / 35 opening it); register 366 / 240 (556 senses, 133 / 66); rare 19 / 15 | « do » (45) « Timar, estafar; Follar » (colloquial, slang); « mouse » (3010) « Timorato, apocado, flojo » (colloquial); « or » (25) « Oro » (a heraldry topic) | The obsolete ones after the current (D5); the labels for the owner (Q1) |
| 7 | A determiner headed « adjetivo » | possessive or demonstrative 6 / 6 rows; the quantifiers the edition calls indeterminate 16 / 14 | « her » (51) « [adjetivo] Su (de ella) », « my », « its », « their », « that » « [adjetivo] Ese; Aquel », « such »; « all », « no », « any », « much », « other » | Fix the six (D4); the quantifiers for the owner (Q3) |
| 8 | A row opening on an unexpected sense | on an obsolete or outdated sense: 40 / 18 rows; on the page's first part of speech: 3,689 / 2,226 rows have two runs or more, 50 / 46 put an open-class run before a function word's | « go » (89) « Andar, marchar, caminar » (obsolete) before « Ir »; « will » (37) « Deseo, inclinación, disposición » (outdated); « up » (41) « Construido »; « like » (39) « Gustar »; « lead » (513) « Plomo »; « by » (21) « Junto a »; « about » (40) « En círculo alrededor… »; « well » (90) « Competentemente », its adverb's « Bien » taken by the adjective's | Obsolete first: fix (D5); the rest for the owner (Q2) |
| 9 | One word repeated across runs | inverted 58 / 10 rows (52 / 9 repeat a whole sense); direct 236 / 53 | « lengthy » (6076) « Largo; Largo; Largo; Largo »; « grey » (2240) « Gris, plomo; Gris, plomo; Plomo; Agrisar »; « ah » « Ay; Ay »; direct: « israeli » « Israelí; Israelí » | Fix the inverted table's (D7); the direct table's are the English word's own parts of speech (D8) |
| 10 | The inverted table's regional or other-sense words | 504 / 73 rows come from the inverted table; about a dozen of the 73 read wrong or regional | « second-hand » (303) « Chivera »; « la » (762) « La, hombrecillo »; « ave » (4549) « AVE »; « metro » (3704) « Subte »; « pickup » (6006) « Troca »; « sup » (9301) « KLK »; « ul » « IU » | For the owner (Q4); a frequency floor measured and rejected (D8) |
| 11 | A common word read as another entry | 1 / 0 | « billiard » (23,796) « Billardo », a numeral | Leave: the English Wiktionary translates only its numeral (D8) |
| 12 | Upstream wording | « de el » for « del » 27 / 25 rows; the rest one row each | « it's » (48) « Contracción de el pronombre it »; « huh » (3189) « !Um¡, !uf¡ »; « timber » « !Árbol abajo! »; « lean » (3945) « Incllinarse »; « read » (223) « Consistir de un cierto texto »; « one » (35) « Uno, i, I o 1 »; « silver » « Que esta hecho de plata »; « batty » « cu lo » | Leave (D8) |
| 13 | One form named two ways | 1 / 1 | « be » (15) « Estar (be + participio presente) », the card's line « forma en -ing de run » (M10) | Fix (D6) |
| 14 | Typography | straight double quotes 32 / 20 rows, 12 expressions; `...` 4 / 3 rows, 7 expressions; single quotes 27 / 21 rows | « it's » (48) « ("ello") »; « i'm » (65) « "yo soy" o "yo estoy" »; « on » (12) « Relacionado con el tema ..., sobre el tema »; « neither » (1358) « (neither ... nor) Ni »; « who'd » « ¿Quién tenía / tuvo / había / hubo....? »; « that's » « Contracción de 'that is' » | The double quotes and the ellipsis: fix (D6); single quotes kept |
| 15 | The owner's calls of change 24 (4.2) | « Sólo » 5 / 5 rows; « Úsase » 8 / 5; « Hasta cerrar » 1 / 1 | « only » (70) « Sólo, solamente, únicamente »; « will »; « to » (2) « [adverbio] Hasta cerrar » | Change 24's 4.2 (Q5) |

The rules D2–D7 together, on the real reduction: **295 rows change, 149 of the top 10,000** —
159 / 121 glossed by the Spanish Wiktionary, 78 / 18 by the direct table, 58 / 10 by the inverted one;
the first sense of 140 / 63, the runs alone of 6 / 6. **One lemma loses its gloss and none gains
one**: « malign » (33,911), whose one translation, « malignar (desus.) », is labelled disused. **64
expressions change**, 2 of them losing their gloss for the same reason (« deep end » « Fondón
(disused) », « with both hands » « A manteniente (disused) »), none gaining one. Rule by rule, alone:
D2 15 / 12 rows (6 / 4 notes, 9 / 8 usage notes) and 3 expressions; D3 61 / 53 rows; D4 6 / 6 (the
runs alone); D5 41 / 27 rows, 20 / 12 opening differently; D6 39 / 26 rows and 19 expressions; D7
136 / 28 rows (39 / 9 a disused word left out, 68 / 16 a note left out, 58 / 10 the inverted table's
repeated words) and 42 expressions. Coverage cannot move: no lemma of the top 20,000 gains or loses a
gloss, so it stays 93.0 / 85.0 / 71.7 %, and the translation-table share stays 26.0 % (2,135 direct,
73 inverted). No row of the card (`rowGloss`) ends on an opening mark. These are a prototype's
figures: the pull request measures again on its own code, and the sample (D10) is drawn from it.

## Decisions

### D1 — en-es's rules, in the Spanish edition and in en-es's reducer

Every rule runs on the entries and the tables before the shared rules read them, as es-en's do
(`refine-lingua-es-en-glosses` D1), and each lives where its subject does:

- **The Spanish Wiktionary's writing** — its notes to its readers (D2), the order it gives its
  obsolete senses (D5), its typography (D6) — in `reduce_edition_es.py`: `_NOTES` gains D2's notes,
  and one pre-pass, `spanish.read_as_meanings(src, dst)`, writes D5 and D6; D6's typography is one
  function, `spanish.typography(text)`, which the pair's reducer also applies to the tables' words.
- **Which senses gloss an English word, and the translation tables** — the name notes (D3), the
  determiners (D4), M10's « forma en -ing » (D6), the tables' notes and repeated words (D7) — in
  `reduce-en-es.py`: one pre-pass, `english_entries(src, dst)`, over the Spanish Wiktionary's English
  entries, and `read_translated` over the two tables.

`reduce-en-es.py` runs `without_letters`, then `spanish.read_as_meanings`, then `english_entries`,
then `native_side`. A pass writes a line it cannot read as it is, as the other passes do. Both files
are en-es's alone today — no other pin names them — so the edits re-pin en-es and no other pair.
The edition's rules are the Spanish Wiktionary's whatever section it writes: fr-es (change 49) will
load them for the French section and measure them there; nothing English-specific goes into the
edition (« participio presente » names a French participle in a French entry).

Why not `reduce_common.py`: it is every pair's, so editing it re-pins en-fr, es-fr and es-en — and the
French and English editions write none of these notes. Why not a field of `Edition`: the class is
`reduce_common.py`'s, the same re-pin. Why pre-passes and not a pass over the reduced glosses: the
shared cleaning would undo some of it (it strips a sense's final periods, so « on »'s « Posicionado sobre
la superficie superior de ... » loses its ellipsis unless it is already « … »), a note must be gone before the cut (« river pumpkin »'s
expression was cut inside « .^([de »), and the senses must be meanings, in their order, before the
round-robin picks eight. The expressions read the same entries and tables, so they get the same
rules.

Where a shared fix is better, it is named and left out (D8): « etc »'s period, a card that reads a
token's capital, the repeated words es-fr's and es-en's translation glosses hold.

### D2 — The edition's notes to its readers, and a usage note after the meaning

`_NOTES` gains, each taken out wherever it sits in a sense:

- **a maintenance template** as the dump renders it, « ^([…]) »: « ^([cita requerida]) » (10 senses of
  the section), « ^([definición imprecisa]) » (5);
- **a disambiguation note**, « [sentido de(l) …] »: « Hacer un favor [sentido del sustantivo] para »;
- **a reference to numbered senses**, a parenthesis naming « definición », « definiciones »,
  « acepción » or « acepciones » before bracketed numbers: « Hardcore (definiciones [1,2]) » reads
  « Hardcore », and the noun's and the adjective's identical senses are then picked once;
- **the expansion notice**, « Este lema en este idioma es ampliable. » to the sense's end: « leaven »'s
  verb, which said nothing else, goes;
- **a usage note after the meaning**: a sentence opening, after the meaning's period, on one of a closed
  list — « A veces », « Precediendo », « Usado », « Usada », « Usados », « Usadas », « Utilizado »,
  « Utilizada », « Utilizados », « Utilizadas », « Empleado », « Empleada », « Se usa », « Se dice »,
  « Se emplea », « Se utiliza », « A no confundir », « Compárese » — goes with the rest of the sense:
  « a » « Un, una; Un tal, una tal », « however » « En cualquier grado o extensión », « wow » « Guau »,
  « degree » « Grado », « cheese » « Patata, whisky », « would've » « Contracción de would y have
  («habría sido») », « badge », « maria », « shaka ».

A second sentence that carries the meaning stays: « isn't » « …not. Traducida como «no es» o «no
está» », « we're » « Contracción de we are. Nosotros somos o nosotros estamos », « pilot » « Piloto.
Persona que dirige un avión » — « Traducida », « Nosotros » and « Persona » are no openers. A note
written before the meaning (« get », « go »'s interjection, « have ») and a function word's
description (« be »'s « Se usa en be to (no existe en español) ») are change 24's 4.2 (Q5). Optional
words in brackets (« [a] tiempo completo », « Cochecito [de bebé] ») are Spanish and stay.

### D3 — A name does not gloss the common word spelled like it

The Spanish Wiktionary writes a surname or a given name as a proper noun under its capitalised
headword, and its senses only say so: « Apellido », « Nombre de pila de varón, equivalente del español
Juan », « Nombre personal femenino », « Hipocorístico de William ». The shared rules read that entry
for the lower-case lemma like any other, so « will » ends on « Apellido; Hipocorístico de William »,
« smith » opens on « Apellido », and the two crowd « Voluntad, albedrío » and « Legar » out of
« will »'s eight.

`english_entries` leaves out such a sense — of an entry whose part of speech is `name`, whose headword
opens on a capital, glossed « Apellido… », « Nombre de pila… », « Nombre personal… » or
« Hipocorístico… » — when the word has an entry written in lower case that is not a proper noun's and
holds a meaning: the condition *An acronym does not gloss the word it is spelled like* sets for an
acronym's entry. An entry left with no sense goes. 61 / 53 rows: « smith » « Herrero », « mike »
« Micro, micrófono », « ward », « hood », « sparrow », « amber », « daisy », « ruby » no longer open on a
name; « will », « small », « white », « king », « queen », « low », « brown », « john » no longer end on
one. None loses its gloss.

Why only the notes and not the proper noun's entry, as an acronym's: its other senses are the token's
reading when capitalised — « south »'s « (region) Sur », « don »'s « El río Don », « Turkey »'s
« Turquía » — and es-en measured what writing names last costs (`refine-lingua-es-en-glosses` D4). Why
not every row that holds a name note: measured, it moves 97 / 70 more rows, all of them names' own
rows, where it reads worse — « donald » « Nombre de varios lugares », « howard », « arnold » « Nombre
de varios lugares en Estados Unidos » — and 7 lemmas lose their gloss (« annie », « hal »,
« kathy »…). A name's own row keeps its notes: « wayne » « Apellido » tells the reader of « Wayne »
what it is, and the 366 rows of the top 10,000 glossed by name notes alone hold 3.7 points of its
coverage (85.0 % without them would be 81.3 %, under the proposed floor of 83.2 %). Which reading a
capitalised token gets is the case-aware card's (D8).

### D4 — Possessives and demonstratives are determiners

The Spanish Wiktionary heads `my`, `her`, `its`, `their` « adjetivo posesivo » and `that`, `such`
« adjetivo demostrativo »; kaikki files them as `adj` with the entry tags `possessive` and
`demonstrative`, so the card heads « Su (de ella) » « [adjetivo] ». en-fr's card heads the same senses
« déterminant », and the RAE's grammar calls Spanish « su », « ese » determinantes. `english_entries`
reads an `adj` entry tagged `possessive` or `demonstrative` as `det`: 6 / 6 rows change their runs
alone (`ADJ` → `DET`), no sense moves. « such » is the one row whose heading departs from en-fr's
(« adjectif »): it follows its tag, and the sample names it. The quantifiers the edition calls
indeterminate (« all », « no », « any », « much », « other »: 16 / 14 rows) stay adjectives, as en-fr
heads them (Q3).

### D5 — Current senses first

The Spanish Wiktionary often opens an entry on its oldest sense: « go »'s first is « Andar, marchar,
caminar », tagged `obsolete` and `outdated`, before « Ir »; « will »'s noun opens on three outdated or
obsolete senses before « Voluntad, albedrío ». The round-robin takes each entry's first senses, so the
row opens on what the edition itself marks unused.

`spanish.read_as_meanings` writes the senses tagged `obsolete` or `outdated`, or raw-tagged
« Arcaico », « arcaico », « obsoleta », « Obsoleto » or « obsoleto », after the other senses of their
entry, in their order; an entry whose every sense is so marked keeps its order (« thou », « thy »).
Nothing is left out. 41 / 27 rows, the first sense of 20 / 12: « go » « Ir; Marchar; Irse, marcharse,
partir; Repartir », « will » (with D3) « Voluntad, albedrío; Decisión, intención; Testamento; Deseo,
inclinación, disposición; Testar; Úsase para construir el futuro; Legar; Querer, desear », « gay » « Gozoso,
festivo, alegre; Gay, homosexual… », « crap » « Mierda… », « mere » « Mero, simple… ». The rule takes
the edition's label as written: « font » then opens on « Pila bautismal », its obsolete « Fuente »
last, and the sample names it. Whether the labels themselves show, or a labelled sense is left out, is
the owner's (Q1); rare senses (19 / 15) are not moved.

### D6 — One Spanish typography, and M10's name for the -ing form

`spanish.typography` writes, in each sense of the edition and each word of the translation tables:

- **One ellipsis**: three dots or more become « … »; between two words it is spaced on both sides
  (« Relacionado con el tema …, sobre el tema … », « (neither … nor) Ni », « Invitarlo a salir de cita a
  … con »); anywhere else the spacing written stays (« ¿Quién tenía / tuvo / había / hubo…? »,
  « ¿sabías que…? »). A « … » is no period, so the shared cleaning keeps a sense's last one: « Posicionado
  sobre la superficie superior de … »; a table's word is not cleaned, so « Me llamo …, mi nombre es … »
  keeps both.
- **Angular quotes**: a sense with an even number of `"` has them paired « » in order, with no space
  inside them (« Contracción de el pronombre it («ello») y el verbo is («es») »); an odd number stays as
  written (an expression cut inside its quote). The RAE advises the angular quotes first; the edition
  writes straight ones (137 senses of the section), « » five times, “ ” never. Single quotes stay: ’ and
  ' are also the apostrophes of the English words the senses name (« Contracción de 'that is' »).

`english_entries` writes « participio presente » as « forma en -ing » in the English entries' senses:
« be » reads « Estar (be + forma en -ing) », the card's name for the form on the line above it (M10,
settled; change 24's *Known data defects*). It is the one a gloss holds — the section's 20 others are
senses of -ing forms and of the suffix « -ing » (« Yendo, participio presente del verbo go »), which
gloss no lemma — and it stays out of the edition: in a French entry « participio presente » names the
French participle.

None of them moves a word otherwise. 39 / 26 rows (the first sense of 29 / 19, nearly all
contractions) and 19 expressions. Measured on the changed rows, no row of the card ends on an opening
mark — « is in the cut's trailing set already (change 24's D4); `row-gloss-tables.spec.ts` stays the
gate.

### D7 — The translation tables' words, without their translators' notes

The English Wiktionary's translators write a note inside the Spanish word they list, and the derived
file keeps it there: a label (« orquestra (disused) », « villorrio (despective) », « dentística
(Americanism) »), a loanword's respelling (« hall (hol) », « app (ap) », « playlist (pléilist) »), a
sense number (« [4] a favor »), an English usage note (« Para ser honesto [with le and a, or with
con] »). `read_translated` reads the direct table's words as:

- **A disused word left out**: a word labelled « (disused) » or « (desus.) » is not listed — « orchestra »
  « Orquesta », « blacksmith » « Herrero, herrera, herrador », « annals » « Anales, fastos, crónica » (the
  next word takes its place). 39 / 9 rows. « malign » (33,911) had no other word and loses its gloss,
  as « deep end » and « with both hands » lose theirs: no gloss is better than a word the table says is
  no longer used, and none is in the top 20,000.
- **A note left out, the word kept**: a note in parentheses or square brackets goes when it follows a
  loanword — the text before it is the English headword or one of its words (« hall (hol) », « app
  (ap) », « insider (insáider) », « trifecta (Arg.) », « Daisy chain (deisi chein) »); or holds no
  Spanish word, every word of it under 1.0 on wordfreq's Spanish Zipf scale (« (despective) »,
  « (Americanism) », « (foundationless) », « (pléilist) »); or is only a number (« [4] »); or opens on
  « with ». A note holding a Spanish word stays (« dimitir (de) », « guardería (infantil) », « (Lat.
  Am.) »). A word the note's removal makes one already listed is listed once, as `read_translations`
  lists each (« septum (séptum) » after « septum »). 68 / 16 rows; with the disused words, 78 / 18
  rows and 42 expressions. Measured: one Spanish qualifier goes with them, « (articulario) » in the
  expression « range of motion »; two rows and five expressions keep an English note that opens on a
  word Spanish texts also use (« mat » « posavasos (coaster) », « sketchy » « (to be sketchy) »,
  « over there » « (further than «allá») »): listed in the pull request, not chased.

And it reads the inverted table's:

- **A Spanish word listed once**: a Spanish word several of whose parts of speech list the English word
  is listed once, under the first of them the English word's readings name (`tables/en/grammar.tsv`),
  else under the first listed; a part of speech left with no word goes. « lengthy » reads « Largo »
  (adjective), « grey » « Gris, plomo; Agrisar », « seaman », « mariner », « lavatory » and « gaol » are
  nouns, as their readings say, not the adjective the Spanish « marinero », « inodoro », « penal » are
  first. 58 / 10 rows, no word lost.

The direct table's parts of speech are the English word's own, so a word it lists under two of them
stays under both (« israeli » « Israelí; Israelí », 236 / 53 rows): each run says something true of the
English word. `grammar.tsv` is one of the six studied tables the pin's `studied` record already holds
and `pack_version`'s studied digest covers; when en-fr's reduction moves it, en-es is reduced again on
the same branch already (`tables/en-es/README.md`, *When English's tables move*).

### D8 — Left, for the owner or for a shared fix

- **« etc » and abbreviations at a sense's end** lose their period in every pair: `clean_gloss`'s
  `.rstrip(".:")` (en-es 47 rows, es-en 35, es-fr 28, en-fr 57). Keeping a closing abbreviation's period
  is one rule for every pair — the follow-up es-en's change names, which re-pins all four.
- **A name's own row**: 603 / 366 rows are glossed by name notes alone (D3).
- **« (region) Sur »** on « south »: the edition's region label as kaikki renders it, the section's one;
  upstream.
- **Labels** (Q1), **the part of speech a row opens on and the page's sense order** (Q2), **the
  quantifiers' heading** (Q3), **the inverted table's regional words** (Q4): measured above. For Q4, a
  floor on the Spanish word's frequency was measured: under 2.5 Zipf it would leave out 8 of the 73
  inverted rows of the top 10,000 — « Cohibición », « Indisponible », « Inspiracional » and « Indeseado »
  among them, which read right — and keep « La, hombrecillo » and « AVE ».
- **Repeated words**: the direct table's across the English word's parts of speech stay (D7); es-fr's
  inverted table repeats them in 254 glosses and es-en's in 9 (`tables/en-es/README.md`): theirs, through
  `reduce_common.translation_gloss`, a shared follow-up.
- **« billiard »** « Billardo » (NUM): the English Wiktionary translates only its numeral sense, a
  million milliards; rank 23,796.
- **A note before the meaning, a function word's description as a sense, « Sólo », « Úsase », « Hasta
  cerrar »**: change 24's 4.2 (Q5).
- **Upstream wording** — « !Um¡, !uf¡ », « !Árbol abajo! », « Incllinarse », « Consistir de », « Uno, i,
  I o 1 », « Que esta hecho », « cu lo », and « de el » for « del » in 27 / 25 rows: corrected on the
  Spanish Wiktionary, they arrive with en-es's next update. A table of corrections would be glosses
  written here, not by the source; « de el » → « del » as a rule would break « de El Salvador ».

### D9 — en-es re-pinned alone, at its snapshot

`build.sh --reduce en-es` from `lingua-pack-sources-en-es-2026.10.08`: the three derived files,
nothing fetched beyond the release's assets. The pin keeps its `snapshot`, its `studied` record (en-fr,
the six tables' sha256) and its `sources` byte for byte — `kaikki-es` (the Spanish dump's record and its
two derived files), `kaikki-en` (the English extract's record and its derived file), `wordfreq`; its
`reducer` digest moves (`reduce-en-es.py`, `reduce_edition_es.py`; `files` unchanged), and with it
`pack_version` (`2026.10.08+<digest[:7]>.e1915ca`, the studied digest unchanged), the pack's sha256 and
size. `gloss_coverage.py --pair en-es` holds `FLOORS["en-es"]`; `tables/en-es/README.md` (21,964
lemmas: 10,247 / 11,213 / 504; 17,094 expressions; the share; the pack's size; *Known noise*) and
`SOURCES.md` follow. en-fr's, es-fr's and es-en's tables and pins are byte for byte unchanged — their
rule digests name neither file — and the reduce job reproduces every committed byte.

### D10 — Every moved line read, and a sample for the owner

`LINGUA_BLESS=1 cargo test -p lingua-wasm --test en_es_baseline` re-blesses the golden, and `yarn
vitest run test/word-card-en-es.spec.ts -u` (the flag after the file) the Spanish card's snapshot.
`EXPEDITION_FIRST_PAGE` stays « Expedición »; `the_golden_is_the_english_one_on_the_studied_side` passes
unchanged — only the native side moves. The prototype moves the glosses of `a`, `be`, `go`, `will`
and the runs of `that`, `her`, `its` among the probes (twelve of the snapshot's: `went`, `gone`,
`goes`, `go`, `are`, `was`, `won't`, `will`, `a`, `that`, `her`, `its`), and the corpus tokens'
glosses of `on`, `small`, `smith`, `low` (« lowers »), `street`, `speed`, `pale`, `sailor`, `grease`,
`however`, `wow`, `it's`, `i'm`, `he's`, `gonna`; `my`'s runs move in no line, the corpus showing its
gloss alone. `yarn vitest run test/row-gloss-tables.spec.ts` runs without `-u`: it is the gate, its
French snapshot (`test/baseline/selection-rows-fr.txt`) must pass as committed — re-blessing it would
hide a French row that moved — and no row of any pair may end on an opening mark.

The pull request lists every changed line of the golden and of the snapshot, with the rule that moved
it, and a before/after sample: every changed row of the top 10,000 (about 149) with its first
differing sense, the changed expressions by rule, and 30 rows drawn from the rest — what the owner
reviews (M9). The sample names the rows whose opening D5 moves to a sense a reader may meet less
(« font », « disco », « dot », « hence »), the rows that opened on a name note (« smith », « mike »,
« ward », « hood », « sparrow »), each note D7 leaves out with the clause that removed it (with
« (articulario) »), « such »'s heading, and the lemma and the two expressions that lose their gloss.

### D11 — With the open decisions, and after change 38

- **Change 22's floor (task 5.1)**: `FLOORS["en-es"]` is the one place the value lives; the re-pin is
  checked against whatever it holds when this change lands. No lemma of the top 20,000 moves, so
  coverage stays 93.0 / 85.0 / 71.7 % and holds against any floor at or below it, whichever lands first.
  Change 22's sample of 100 glosses moves in one row, « stage », past its shown cut, and the share not
  at all: the owner's 5.1 review reads the same. `tables/en-es/README.md`'s *Known noise, for the
  owner's 5.1* is rewritten: what this change fixed (the tables' notes, the inverted repeated words, the
  name notes on common words), what is left (Q1–Q4).
- **Change 38's per-pair update rule**: change 38 re-pinned no pair and keeps en-es's records as its
  first update wrote them; this change re-pins en-es from those same derived files, `sources` byte for
  byte, and dispatches no update. en-es moves to the editions' dumps at its next update, the owner's
  dispatch, whose report names the 186 glosses and 55 expressions whose words the dump orders
  otherwise. D7 acts on each translation word, so it applies to the dump's tables as to the
  extract's; which words come first and which third word is kept stay the update's, and the owner
  judges them there. While kaikki still serves the English dump of 2026-10-03, the implementation
  derives the direct table from it and reduces en-es under these rules (`pack_report.py
  --identical`), recorded in `SOURCES.md` beside 38's measurement; otherwise `SOURCES.md` says so and
  the next update names any difference.
- **M15 and M16**: untouched. M15 opens translation per pair — routes, models and marks; en-es is in
  `MARKED_PAIRS` already — and M16 sets the App Store locales. Change 37's listings say en-es's glosses
  are written by people, from Wiktionary, never machine-translated: still true — these rules leave
  out, reorder and retype what the editions wrote, and write only marks and M10's « forma en -ing »;
  its screenshots are captured at change 35's release (its 3.1), after this change, so they show these
  glosses.
- **M20** is the English edition's (es-en); the Spanish edition's long-parenthesis bound stays 0.
- **Change 24's 4.2**: an answer that is a rule of the Spanish edition or of en-es's reducer
  (« Hasta cerrar » left out, « Sólo » respelled, « Úsase » reworded, a note before a meaning moved)
  joins this change before it merges, en-es re-pinned again; any other is a follow-up named in the pull
  request.
- **Change 35** lists this change among its prerequisites.

## For the owner

- **Q1 — Labels.** (a) Prefix a sense with its label in Spanish — « (coloquial) Timar, estafar »,
  « (Reino Unido) … », a closed table of the edition's tags written in Spanish: register 366 / 240 rows
  (556 senses), regional 168 / 105 (225), rare 19 / 15, obsolete or outdated 176 / 107 (237); (b) leave
  out the vulgar and slang senses of a word that has others; (c) neither, as today with D5's order. (a)
  and (b) combine.
- **Q2 — The part of speech a row opens on**: accepted as the page's order until a studied-side
  frequency exists, as es-en's Q4? 3,689 / 2,226 rows have two runs or more; 50 / 46 open on an
  open-class run before a function word's (« up » « Construido », « like » « Gustar », « only »
  « Único »), and « lead » opens on « Plomo », « by » on « Junto a », « about » on « En círculo
  alrededor… », « well » on « Competentemente ».
- **Q3 — Quantifiers headed « adjetivo »** (16 / 14 rows: « all », « no », « any », « much », « other »):
  kept as en-fr heads them, or read as determiners? Universal Dependencies tags « no », « any »,
  « some », « all », « another », « either », « both » as determiners and « many », « much », « other »,
  « same », « certain », « enough » as adjectives, so a rule would need a list of words.
- **Q4 — The inverted table's regional or other-sense words**: 73 of the top 10,000 are glossed by it,
  about a dozen wrongly or regionally (« second-hand » « Chivera », « la » « La, hombrecillo », « ave »
  « AVE », « sup » « KLK », « metro » « Subte », « pickup » « Troca »). Accepted, or a rule the owner
  names (D8 measured a frequency floor and rejects it)?
- **Q5 — Change 24's 4.2**, answered there (D11).

An answer that is a rule of the Spanish edition or of en-es's reducer joins this change before it
merges (en-es re-pinned again); any other is named as a follow-up in the pull request.

## Risks / Trade-offs

- **A meaning read as a note** (a second sentence that carries the meaning) → D2's openers are a closed
  list measured on the section; « Traducida », « Nosotros », « Persona » are not in it, and « isn't »,
  « we're », « pilot » are tested.
- **A name's note that was the token's reading** (« Will » a character's name) → the capitalised token
  gets the common word's senses, as the acronym rule already does; a name's own row keeps its notes,
  and the case-aware card is the shared fix (D8).
- **The edition's label on a sense the reader meets most** (« font »'s « Fuente », obsolete, last) → the
  label is the edition's; the sample shows every row whose opening D5 moves.
- **A Spanish note read as a label** (« (articulario) ») → the clauses are measured one by one and every
  removed note is in the pull request; a note holding a Spanish word stays.
- **A word lost with its label** (« malign ») → one lemma beyond the top 20,000 and two expressions,
  named; no gloss is better than a word the table says is no longer used.
- **A heading that departs from en-fr's** (« such » « determinante ») → named in the sample.
- **A golden that moves** → re-blessed in the pull request, each line read (D10); the French rows'
  snapshot is run, never re-blessed.

## Migration Plan

No release: en-es's tables, its pin and the tests that pin it, all inert until change 35 lists en-es.
