# The fr→es dictionary tables

The reduced tables Cymbra Lingua's French→Spanish data pack is built from: French glossed in
Spanish, for Spanish speakers studying French (add-lingua-pack-fr-es, change 49 of
`docs/lingua/language-matrix-programme.md`, decision M6). They are committed so that every build
makes the same pack with no download, and so that a change to the dictionary is a pull request whose
diff shows it. No extension package carries the pack: `apps/lingua-extension/packs.json` does not
list fr-es, and change 52 lists it only because these tables stand at or above the floor below.

This folder holds what belongs to fr-es alone: its Spanish glosses, its expressions, the parts of
speech of their senses, its notice, manifest and pin. French's own tables — its forms, ranks,
readings, levels, tag pool and dictionary words — are kept once, in `../fr/`, and written by fr-en's
reduction alone (`../fr/studied.json` names fr-en, French's reference pair). fr-es's reduction reads
them as committed and computes nothing of them: its lemmas and their ranks are `../fr/forms.tsv` and
`../fr/freq.tsv` (all 60,000), its readings, levels and dictionary words fr-en's.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → Spanish gloss | the Spanish Wiktionary's French section (its definitions); else the Spanish translations the French Wiktionary's French entries list; else the French translations the Spanish Wiktionary's Spanish entries list, read backwards (all CC BY-SA 4.0 + GFDL, through kaikki) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | the same |
| `mwe.tsv` | expression → Spanish gloss | the same sources, for multi-word headwords |
| `NOTICE` | the attribution stack, embedded in the pack: both sides' sources — the studied side as fr-en's notice credits it | — |
| `manifest.json` | the pack's metadata: French glossed in Spanish, French's analyser version, `levels_estimated`, and `pack_version` (fr-es's snapshot, the rules that reduced it, and a digest of the studied tables it was built on) | — |
| `pin.json` | fr-es's raw sources — the three files it derives from the Spanish and French Wiktionaries' dumps, no extract and no English dump — the pack these tables build with `../fr/`, the rules that reduced them, and the sha256 of each of `../fr/`'s six tables they were built on (`studied`) | — |

## The floor, fixed before the measurement (M6)

The programme measured fr-es as the thinnest pair of the matrix (its risk 5). The owner settled M6 on
2026-10-09, before the update whose tables are committed here: fr-es is held to **81.4 / 68.8 /
54.5 %** of the 5,000 / 10,000 / 20,000 commonest lemmas glossed — the study's 83.4 / 70.8 / 56.5 %
less two points, en-es's and fr-en's rule — kept in one place, `gloss_coverage.py`'s
`FLOORS["fr-es"]`, which the `reduce` job reads (`gloss_coverage.py --pair fr-es`, no `--floor`) and
the tests hold to the requirement's value.

**Below it.** Had the first update measured under the floor at any top, these tables would not have
been committed, no package would list fr-es — French would ship for English speakers alone — and
fr-es would be dispatched again at a later regeneration of the dumps, against the same floor. Now
that they are committed, any pull request that reduces fr-es again under the floor — its sources
updated, its rules or the Spanish edition's changed, French's studied tables moved — fails the
`reduce` job, naming the pair, the top and the figure, and readers keep the last committed fr-es.
The floor is never lowered after a measurement: a lower one is the owner's decision, recorded before
the measurement it applies to.

**The committed measurement.** The first update, run
[38041319531](https://github.com/NEETROF/cymbra/actions/runs/38041319531) of `lingua-pack-update`
(2026-10-10, 5 min 22 s), read the dumps kaikki served that day — the Spanish Wiktionary's
regenerated 2026-10-02 12:12 and the French one's 2026-10-02 00:10, the regenerations the design's
prototype read — and measured:

| Lemmas | Spanish Wiktionary alone | the three sources | the floor | margin | the study |
|---|---|---|---|---|---|
| top 5,000 | 40.0 % | **83.2 %** (4,161) | 81.4 % | +1.8 | 83.4 % |
| top 10,000 | 28.5 % | **70.8 %** (7,082) | 68.8 % | +2.0 | 70.8 % |
| top 20,000 | 18.3 % | **56.8 %** (11,359) | 54.5 % | +2.3 | 56.5 % |
| all 60,000 | 7.6 % | 31.8 % (19,050) | — | — | — |

At or above the floor at every top: the tables are committed, and fr-es ships with change 52, its
coverage published beside the other pairs' as es-fr's is (87.6 / 77.2 / 63.7 %). Until then it is
shown here and in the pull request alone.

## What is in them

On the 2026-10-10 tables (pinned snapshot `2026.10.10`, `pack_version` `2026.10.10+3f293e8.e18e7e8`:
the snapshot, the rules' digest, and the digest of the studied tables `pin.json` records),
reproduced byte for byte, twice, by `build.sh --reduce fr-es` from the pinned release:

- **Spanish glosses for 19,050 lemmas** of French's 60,000: **4,560 from the Spanish Wiktionary's
  French section** (its definitions, by the Spanish edition's rules), **13,195 from the Spanish
  translations the French Wiktionary lists** (the direct table, in its order) and **1,295 from the
  French translations the Spanish Wiktionary's Spanish entries list, read backwards** (the inverted
  table, the commonest Spanish word first); **12,177 expressions** — 587 from the section, 10,686
  from the direct table, 904 from the inverted one. Up to eight senses grouped by part of speech, as
  every pair's; a table's gloss at most three Spanish words per part of speech, as one sense, opening
  on a capital as the edition's senses do. No pivot through a third language, no machine translation.
- **The share of definitions** (D9): **23.9 %** of all glossed lemmas come from a definition (4,560
  of 19,050 — the programme's « 24 % »); among the glossed lemmas of the 10,000 commonest, **40.2 %**
  (2,850 of 7,082) and **59.8 % from a translation table** (3,967 direct, 265 inverted:
  `measures.json`, which the reducer writes beside its tables and no pack stores; `pack_report.py
  --measures` shows it in the update's summary); 48.1 % of the top 5,000's (2,000 of 4,161), 32.1 %
  of the top 20,000's (3,648 of 11,359). en-es's share from a table is 26.0 % of its glossed top
  10,000. A translation is a word a person chose for the French word, not a meaning (« répondre »
  « Contestar, responder »), grouped by part of speech like any gloss.
- **Dictionary words are fr-en's**: 544 lemmas fr-es glosses are no dictionary word of French (12 /
  40 / 126 of the top 5,000 / 10,000 / 20,000: `quant`, `onu`, `vih`, but also `el`, `for`, `last`,
  `okay`), and 11,549 of fr-en's 30,055 glossed lemmas have no Spanish gloss, so the pack carries a
  lexical section and the vocabulary estimate counts the same words for both pairs.
- **The pack is 1,973,407 B**, 37.6 % of the 5 MiB budget: gloss 178,589 B, senses 57,156 B,
  expressions 268,946 B (133,168 + 104,581 + their names 31,197), the lexical table 7,500 B, and
  French's studied sections — forms, lemmas, ranks, levels, paradigms and the tag pool — byte for
  byte fr-en's (`cross_native.rs`). The design's prototype weighed 1,911,078 B on changes 45's and
  46's prototype tables; the committed readings and levels are heavier, as fr-en's pack (2,527,222
  B, 2,422,318 B in the same prototype) shows.

What none of the sources glosses: 839 of the top 5,000 — English words and names in French text
(`the`, `of`, `york`, `david`), initialisms (`km`, `tv`), unaccented spellings (`etat`, `meme`), and
common French words neither edition translates into Spanish (`part`, `lors`, `afin`, `taux`,
`retrouver`, `tandis`, `classement`, `réellement`). fr-en glosses 529 of them.

## The rules, and what each moves

The Spanish Wiktionary's French section reaches the shared rules through the Spanish edition's notes
and pre-pass, as en-es's English section does (refine-lingua-en-es-glosses), then fr-es's own pass;
the tables through fr-es's `read_translated`. Measured on the committed sources, each rule against the
three sources with the section's letters left out (19,061 lemmas, 83.4 / 70.9 / 56.9 %); rows / of the
top 10,000:

| Rule | Where | Rows | First sense | Expressions | Examples |
|---|---|---|---|---|---|
| A headword's typographic apostrophe read as `'` (D2) | `straight_apostrophes`, `translation_words` | 4 lemmas gained, 900 expressions | — | +900 | `main-d'œuvre` « Mano de obra » (rank 3,053), `chef-d'œuvre`, `inch'allah`, `qu'en-dira-t-on`; « aller de l'avant », « coup d'état » |
| The edition's notes (D4), against the edition before 24b | `reduce_edition_es.ES` | 3 / 3 | 2 / 2 | 0 | « madame » « Señora (vocativo) », no longer « … Se utiliza en presencia de… »; `mademoiselle`, `de` |
| Senses marked obsolete or outdated after the others, one typography (D4) | `reduce_edition_es.read_as_meanings` | 19 / 7 | 10 / 2 | 3 | « chapelet » « Rosario; Guirnalda »; « baiser » opens on « Coger (sexualmente) », its « Besar » labelled outdated; « trêve de plaisanterie » « Bromas aparte… » |
| A name's note does not gloss the common word (D5) | `french_entries` | 17 / 12 | 8 / 6 | 0 | « pierre » « Piedra », « jean » « Color de mezclilla; Mezclilla; Tejanos », « romain » « Romano » |
| Possessives, demonstratives and their forms are determiners (D5) | `french_entries` | 7 / 7, runs alone | 0 | 0 | `ce`, `mon`, `ma`, `ton`, `ta`, `notre`; `mes` « Mi », borrowed from `mon` as a determiner |
| A Spanish word read backwards listed once (D6) | `listed_once` | 93 / 30 | 6 / 2 | 0 | « cet » « Este », « condom » « Condón, preservativo, profiláctico » (a noun, as French’s readings say, not « Preservativo, profiláctico; Condón ») |
| A letter glosses no word (D6) | `translation_words`, `read_translated` | 11 / 11 lose a gloss | — | 0 | `h` « H », `x` « X », `r`, `b`, `o`, `g`, `k`, `w`, `z`, `q`, `i` « I latina, i »; `à` and `y` keep their definitions, `ô` the direct table's « Oh » |
| The studied word is no definition (D7) | `no_self_definition` | 9 / 4 | 9 / 4 | 0 | « et » (rank 3) « Et » → « Y, e »; `élite` « Elite », `troll` « Trol », `slip` « Calzoncillos, braguitas », `clochard`, `gourmet`, `azimut`, `yucca`, `octogonal` |
| **Together** | | **149 / 64**, and 7 / 7 runs alone | **33 / 14** | **4** | coverage 83.4 / 70.9 / 56.9 → 83.2 / 70.8 / 56.8 %; 11 letters lose their gloss, none gained |

The direct table's words are in the edition's typography too (« Bromas aparte… », the fourth
expression). The translators' notes and disused words en-es leaves out of its direct table occur in
no word a fr-es gloss keeps: not ported. A cognate the French Wiktionary also translates as itself
keeps its definition (« venir » « Venir », « entre » « Entre », « normal » « Normal »).

Every figure is the design's (*Measured*) but two: the glossed lemmas of the top 10,000 are 7,082,
2,850 of them definitions, where the design wrote 7,081 and 2,849 — its 4,161 of the top 5,000
already counts `main-d'œuvre` (rank 3,053), which the top 10,000 holds too; the percentages are
unchanged —, and the Spanish word read backwards listed once moves 93 / 30 rows alone where the
prototype's 94 / 31 read change 45's prototype readings, the committed ones here (together, 149 / 64
as designed).

## A sample of 100 glosses

For the owner's review (task 6.1): the glossed lemmas among the 10,000 commonest, in rank order, every
⌊n/100⌋-th from the first (every 70th, from n = 7,082) — a systematic sample, so that every frequency
band is represented and the list is reproducible from the tables and `measures.json`. The source of
each gloss is marked: a *definition* of the Spanish Wiktionary, the French Wiktionary's Spanish
translations (*direct*), or the Spanish Wiktionary's French translations read backwards (*inverted*):
46 definitions, 51 direct, 3 inverted (the design's sample, every 70.8th, drew 40 / 55 / 5). One gloss
longer than 160 characters is shortened here with « … »; the tables hold it whole.

| # | Rank | Lemma | Source | Gloss |
|---|---|---|---|---|
| 1 | 1 | de | definition | De (indica posesiva); De (indica asociación o propiedad); De (indica origen, se usa sin artículo antes de sustantivos proprios); De (indica una cantidad); De (… |
| 2 | 71 | bon | definition | Bueno |
| 3 | 143 | politique | definition | Político; Política |
| 4 | 216 | gouvernement | definition | Gobierno; Mando; Administración |
| 5 | 294 | moyen | definition | Medio |
| 6 | 365 | vivre | definition | Vivir |
| 7 | 437 | esprit | definition | Espíritu; Mente |
| 8 | 511 | activité | definition | Actividad |
| 9 | 585 | domaine | definition | Dominio |
| 10 | 660 | modèle | direct | Plantilla, modelo |
| 11 | 732 | analyse | direct | Análisis |
| 12 | 809 | russie | definition | Rusia |
| 13 | 886 | jacques | definition | Nombre de pila de varón, equivalente del español Jaime, Diego, Santiago o Jacobo |
| 14 | 958 | mari | definition | Marido |
| 15 | 1033 | el | definition | Ella, ello o él |
| 16 | 1105 | triste | definition | Triste |
| 17 | 1178 | avantage | direct | Ventaja, provecho, ganancia |
| 18 | 1250 | amie | definition | Amia o lamia |
| 19 | 1328 | secours | direct | Socorro |
| 20 | 1404 | sale | definition | Sucio |
| 21 | 1481 | métro | definition | Metro (medio de locomoción) |
| 22 | 1556 | chapitre | direct | Capítulo |
| 23 | 1638 | étudiant | direct | Estudiante, alumno |
| 24 | 1719 | partenaire | inverted | Socio |
| 25 | 1799 | choc | direct | Choque, golpe, colisión |
| 26 | 1879 | récit | direct | Relato |
| 27 | 1962 | inscription | inverted | Inscripción |
| 28 | 2040 | montage | direct | Montaje |
| 29 | 2120 | prêtre | definition | Sacerdote |
| 30 | 2200 | plaque | direct | Placa |
| 31 | 2283 | paysage | direct | Paisaje |
| 32 | 2369 | registre | direct | Registro |
| 33 | 2453 | thierry | definition | Apellido; Nombre de pila de varón, equivalente del español Teodorico |
| 34 | 2540 | racisme | direct | Racismo |
| 35 | 2631 | chaise | direct | Silla |
| 36 | 2720 | parvenir | definition | Llegar a un término que se propuso (y llegar ahí ni sin esfuerzo ni dificultad); Acceder a lo que uno ansiaba; Llegar a un destino; Elevarse en dignidad |
| 37 | 2805 | catastrophe | definition | Catástrofe |
| 38 | 2887 | écologique | definition | Ecológico, ecológica |
| 39 | 2973 | hall | direct | Explanada |
| 40 | 3062 | automatiquement | definition | Automáticamente |
| 41 | 3149 | réparer | definition | Arreglar, reparar o restaurar |
| 42 | 3244 | largeur | direct | Anchura |
| 43 | 3331 | valoir | definition | Valer |
| 44 | 3428 | extra | direct | Extra; Extra |
| 45 | 3517 | révélation | direct | Revelación |
| 46 | 3607 | fidélité | definition | Fidelidad |
| 47 | 3699 | renaissance | direct | Renacimiento; Renacimiento |
| 48 | 3790 | constant | inverted | Constante |
| 49 | 3882 | soif | definition | Sed |
| 50 | 3977 | nier | definition | Negar |
| 51 | 4077 | cuba | definition | Cuba |
| 52 | 4175 | victoria | definition | Victoria |
| 53 | 4271 | bouffer | direct | Comer |
| 54 | 4364 | palmarès | direct | Palmarés |
| 55 | 4459 | témoigner | definition | Atestiguar, testimoniar, testificar |
| 56 | 4547 | ds | direct | Tiburón |
| 57 | 4649 | pakistan | definition | Pakistán |
| 58 | 4752 | chaussure | definition | Zapato |
| 59 | 4845 | distributeur | direct | Distribuidor |
| 60 | 4954 | perdant | definition | Perdedor; Fracasado, perdedor, pobre diablo |
| 61 | 5059 | expulsion | definition | Expulsión |
| 62 | 5173 | subvention | direct | Subvención |
| 63 | 5287 | bêtise | definition | Tontería |
| 64 | 5385 | coutume | direct | Costumbre, uso, hábito |
| 65 | 5495 | honnêteté | definition | Honestidad |
| 66 | 5602 | mêler | definition | Mezclar |
| 67 | 5703 | versement | direct | Pago |
| 68 | 5814 | cire | direct | Cera |
| 69 | 5914 | atomique | direct | Atómico |
| 70 | 6025 | détection | direct | Detección |
| 71 | 6138 | refroidissement | direct | Enfriamiento |
| 72 | 6250 | annuaire | direct | Anuario |
| 73 | 6357 | artisan | definition | Artesano |
| 74 | 6467 | approfondir | direct | Ahondar, profundizar |
| 75 | 6577 | circonstance | definition | Circunstancia |
| 76 | 6696 | lieutenant-colonel | direct | Teniente coronel |
| 77 | 6823 | tunisien | direct | Tunecino, tunecina; Tunecino |
| 78 | 6944 | autrichien | direct | Austriaco; Austríaco |
| 79 | 7061 | buisson | direct | Arbusto, mata, matorral |
| 80 | 7180 | dépendre | direct | Desahorcar, depender |
| 81 | 7279 | cascade | direct | Cascada, catarata, acrobacia |
| 82 | 7400 | chariot | direct | Carro, silla de ruedas, carrito |
| 83 | 7513 | cruauté | definition | Crueldad |
| 84 | 7626 | convenable | direct | Acomodado, adecuado, útil |
| 85 | 7748 | imam | direct | Imán |
| 86 | 7893 | porcelaine | direct | Porcelana, planta madre perla, planta fantasma |
| 87 | 8004 | lèvre | definition | Labio |
| 88 | 8125 | institutionnel | direct | Institucional |
| 89 | 8250 | lacroix | definition | Apellido |
| 90 | 8375 | glissement | direct | Desliz |
| 91 | 8513 | inaccessible | direct | Inaccesible |
| 92 | 8643 | loir | direct | Lirón |
| 93 | 8791 | planifier | direct | Planificar |
| 94 | 8897 | géologie | definition | Geología |
| 95 | 9035 | intimidation | direct | Intimidación |
| 96 | 9165 | importer | definition | Importar |
| 97 | 9318 | légalisation | direct | Legalización |
| 98 | 9428 | démolir | direct | Demoler, derribar, derrumbar |
| 99 | 9539 | alliage | definition | Aleación |
| 100 | 9678 | asthme | direct | Asma |

What reads wrong in it: « amie » « Amia o lamia » (the section's `amie` is the fish, not « amiga »),
« el » « Ella, ello o él » (no dictionary word of French), « ds » « Tiburón » (the inverted table's
other-sense word), « hall » « Explanada », « porcelaine » « Porcelana, planta madre perla, planta
fantasma », and « de »'s descriptions « De (indica posesiva); … »; « extra », « renaissance »,
« tunisien » and « autrichien » repeat a word across the French word's parts of speech (Open
Question 3).

## Known data defects

What still reads wrong in fr-es, measured on the committed tables (rows / of the top 10,000), for the
owner's review and a refinement before change 52 if one is wanted:

| Class | Rows | Examples | Where its fix belongs |
|---|---|---|---|
| A note before the meaning | 46 / 23 | « être » « (être + participio) Haber », « chien » « (Canis lupus familiaris) Perro », « faire » « (faire chaud, faire froid) hacer » | change 24's Q5, the Spanish edition |
| Labels not shown | register, obsolete or outdated, regional and rare senses, as the design measured them | « mec » « …; Proxeneta », « baiser » | 24b's Q1, the Spanish edition |
| A pointer the edition's wording misses | 13 / 11 | `mme` « Abreviatura de madame », `al` « Abreviatura de année-lumière », `du` « Contracción de la preposición de y el articulo le, del » | the Spanish edition's `_FORM_OF` (re-pins en-es and fr-es) |
| A function word's description | 2 / 2 | `il` « …; Pronombre sujeto expletivo impersonal. (No tiene traducción al español. No existe en español.) », `de` | change 24's Q5 |
| A possessive headed as a pronoun | 2 / 2 | `son` « Su; Sonido; Salvado, moyuelo » (its first run PRON), `leur` « Suyo, suya » | upstream, or a rule naming them |
| The direct table's word under two parts of speech | 421 / 129 | « parti » « Partido; Partido », « européen », « extra » « Extra; Extra » | Open Question 3 |
| The inverted table's other-sense words | about a dozen of the 265 rows of the top 10,000 it glosses | « us » « EEUU », « fr » « Imperial, calabaza », « rap » « Secuestro », « ds » « Tiburón » | 24b's Q4, Open Question 4 |
| A translation written in French, the studied language (M5) | 8 / 2, as the design measured them | « arnaque » « Arnaque », « gâche » « Gâche »; loanwords Spanish spells alike among them (« diaporama », « raï ») | upstream, Open Question 8 |
| Upstream wording | single rows | « siège » « …; Local, stilla », « pieux » « …, pièsa », « amie » « Amia o lamia » | upstream |
| « etc » without its period | 5 / 4 | « avec », « adresse », « cochon » | shared, every pair (24b D8) |
| A proper noun's run in the row | 596 / 353 rows hold one, 590 / 347 open on it | « france » « Francia; Nombre de pila de mujer », « terre » | the case-aware card, shared |
| Rows glossed by a name's notes alone | 313 / 167 | « françois », « jacques », « thierry » « Apellido; Nombre de pila de varón… » | kept: they say what the capitalised token is |
| Lemmas outside French's dictionary words glossed | 544 / 40 | `el` « Ella, ello o él », `for` « Fuero », `last` « Lastre », `okay` « Oquey, oqué » | no dictionary word, no vocabulary count |

## Its sources

fr-es pins no extract of its own: everything it reads is derived from whole Wiktionary dumps at an
update, in one pass each, and the dumps are not kept (`pack_sources.py DUMPS["fr-es"]`, D3) — the
catalogue's three files, no new derivation, and no English dump: French's side is `../fr/` as
committed, recorded by sha256 in the pin's `studied` record, not as a source. The derived files are
the assets of fr-es's own release, `lingua-pack-sources-fr-es-<snapshot>` (today
`lingua-pack-sources-fr-es-2026.10.10`, 1.6 MiB of zstd assets):

- **The Spanish Wiktionary's French section** (`kaikki-es-Frances.jsonl`, 8,683 entries,
  7,438,610 B, sha256 `2e724e39…`), from the Spanish edition's dump regenerated 2026-10-02 12:12
  (103,226,106 B gzipped, 1,233,016,167 B decompressed, sha256 `46e1f04f…`). Its definitions gloss
  first, cleaned by the Spanish Wiktionary's rules (`reduce_edition_es.py`).
- **The Spanish Wiktionary's French translations** (`kaikki-es-traductions.jsonl`, 18,449 entries,
  1,537,580 B, `bf44ebb4…`), from the same dump in the same pass. Read backwards, they gloss what
  nothing else does, the commonest Spanish word first.
- **The French Wiktionary's Spanish translations** (`kaikki-fr-traductions.jsonl`, 56,466 entries,
  6,191,621 B, `bd14a5c4…`), from the French edition's dump regenerated 2026-10-02 00:10
  (736,590,407 B gzipped, 6,865,136,428 B decompressed, `968f7df0…`). They gloss what the section
  leaves out, in the table's order.

The two translation files are byte for byte the ones es-fr pins, read the other way round: fr-es
records them under its own release all the same (add-lingua-pack-es-en D2), and the `reduce` job's
asset cache fetches each once.

## Licences

The repository is Apache-2.0; **these files are not**. They are derived from the sources above and
carry their licences: `gloss.tsv`, `senses.tsv` and `mwe.tsv`, CC BY-SA 4.0 and the GFDL (kaikki).
French's tables in `../fr/` carry theirs (`../fr-en/README.md`). `NOTICE` gives the full attribution.
See `../../SOURCES.md`.

## Changing them

Never by hand, and never under the floor.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=fr-es` and `mode=update`. It
  reads today's Spanish and French Wiktionaries' dumps, keeps the three derived files as the release
  `lingua-pack-sources-fr-es-<snapshot>`, reduces, and pushes the branch
  `lingua-pack/fr-es/<snapshot>`; its summary shows the coverage and the share beside what the tables
  change. The pull request's `reduce` job holds it to `FLOORS["fr-es"]`.
- **When French's tables move**: fr-en's update or re-reduction writes `../fr/`, and brings fr-es
  along on the same branch, reduced again from its own pinned sources; fr-es's `pack_version` moves
  with the studied tables, and its coverage is measured against the floor again — a move of French's
  tables that takes it under fails the pull request. A pull request that moves a table of `../fr/`
  without recording fr-es again fails, naming fr-es and the table (`pack_sources.py check-reducer`,
  `pack_report.py`). Changes 41 and 42, which bump French's analyser version, re-reduce fr-es too.
- **After editing the reduction rules** — `reduce-fr-es.py`, `reduce_common.py` (every pair's), or
  `reduce_edition_es.py` (the Spanish Wiktionary's, which en-es and fr-es load): the check lane fails
  until the tables are reduced again from the pinned sources. Run
  `scripts/lingua-data/build.sh --reduce fr-es <out>` (Python 3.12, `requirements-reduce.txt`), after
  fr-en's when both are due, or `lingua-pack-update` with `mode=reduce`.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.

The monthly dry run of the update checks this pair as it checks the others.
