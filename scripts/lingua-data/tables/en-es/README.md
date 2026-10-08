# The en→es dictionary tables

The reduced tables Cymbra Lingua's English→Spanish data pack is built from: English glossed in
Spanish, for Spanish speakers studying English — the first pair glossed in a native language no
shipped pack speaks (add-lingua-pack-en-es, change 22 of `docs/lingua/language-matrix-programme.md`).
They are committed so that every build makes the same pack with no download, and so that a change
to the dictionary is a pull request whose diff shows it. No extension package carries the pack:
`apps/lingua-extension/packs.json` does not list en-es until the interface speaks Spanish and the
pair is enabled (change 35).

This folder holds what belongs to en-es alone: its Spanish glosses, its expressions, the parts of
speech of their senses, its notice, manifest and pin. English's own tables — its forms, ranks,
levels, readings, tag pool and dictionary words — are kept once, in `../en/`, and written by en-fr's
reduction alone (`../en/studied.json` names en-fr, English's reference pair). en-es's reduction reads
them as committed and computes nothing of them: its lemmas and their ranks are `../en/forms.tsv`,
`../en/freq.tsv` and the level lists' words of `../en/level.tsv`, its readings, levels and
dictionary words en-fr's.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → Spanish gloss | the Spanish Wiktionary's English entries; else the Spanish translations the English Wiktionary's English entries list; else the English translations the Spanish Wiktionary's Spanish entries list, read backwards (all CC BY-SA 4.0 + GFDL, through kaikki) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | the same |
| `mwe.tsv` | expression → Spanish gloss | the same sources, for multi-word headwords |
| `NOTICE` | the attribution stack, embedded in the pack: both sides' sources — the studied side as en-fr's notice credits it | — |
| `manifest.json` | the pack's metadata: English glossed in Spanish, English's analyser version, and `pack_version` (en-es's snapshot, the rules that reduced it, and a digest of the studied tables it was built on) | — |
| `pin.json` | en-es's raw sources — the three files it derives from the Spanish Wiktionary's dump and the English Wiktionary's English extract, no extract of its own — the pack these tables build with `../en/`, the rules that reduced them, and the sha256 of each of `../en/`'s six tables they were built on (`studied`) | — |

## What is in them

On the 2026-10-08 tables (pinned snapshot `2026.10.08`, `pack_version` `2026.10.08+8ba4a79.e1915ca`: the
snapshot, the rules' digest, and the digest of the studied tables `pin.json` records), reduced again
from the same sources with the rules that read the glosses as meanings (below,
refine-lingua-en-es-glosses):

- **Spanish glosses for 21,964 lemmas** of English's 40,685: 10,247 from the
  Spanish Wiktionary's English entries (its definitions, by the Spanish edition's rules), 11,213
  from the Spanish translations the English Wiktionary lists (the direct table, in its order) and
  504 from the English translations the Spanish Wiktionary's Spanish entries list,
  read backwards (the inverted table, the commonest Spanish word first); **17,094
  expressions**. Up to eight senses grouped by part of speech, as every pair's, opening on a
  capital as the Spanish Wiktionary writes them. A letter glosses no word, in either direction:
  a single letter is glossed only by a sense that is neither the letter nor a name borrowed
  through it — `a` « Un, una… », `i` « Yo » and the vocative `o` « Oh, oy » are words; « i latina »
  names the letter, and so does the Spanish Wiktionary's note `do` « C », which glossed `c`
  (rank 376) « Do » on the first reduction (`reduce-en-es.py read_translated`: read backwards, a
  one-letter English word is left out whatever lists it). No pivot through a third language, no
  machine translation.
- The share of the commonest lemmas glossed, which the `reduce` job holds to a floor the owner sets
  on the pull request (`gloss_coverage.py --pair en-es`, against `FLOORS["en-es"]` — the one place
  the value lives; the job passes no `--floor`):

  | Lemmas | Spanish Wiktionary | with the translations | the floor (proposed) | the study |
  |---|---|---|---|---|
  | top 5,000 | 80.4 % | 93.0 % | 91.4 % | 93.4 % |
  | top 10,000 | 62.9 % | 85.0 % | 83.2 % | 85.2 % |
  | top 20,000 | 42.5 % | 71.7 % | 69.9 % | 71.9 % |
  | all 40,685 | 25.2 % | 54.0 % | — | — |

  The floor is proposed at the study's figures less two points (the programme's risk 5, M6's
  rule); the owner settles it (task 5.1), and the value is written into `FLOORS` and into the
  requirement before merge.
- **The translation-table share** (D4): of the 8,495 glossed lemmas among the 10,000
  commonest, **26.0 %** come from a translation table rather than from a definition — 2,135
  from the direct table, 73 from the inverted one (`measures.json`, which the reducer
  writes beside its tables and no pack stores; `pack_report.py --measures` shows it in the update's
  summary).
- **Dictionary words are en-fr's**: 2,501 lemmas en-es glosses are no dictionary word
  of English, and 5,336 dictionary words have no Spanish gloss, so the pack
  carries a lexical section, and the vocabulary estimate counts the same 24,799 words for
  both pairs.
- **The pack is 1,688,931 B**, 32.2 % of the 5 MiB budget.

## A sample of 100 glosses

For the owner's review (D5): the glossed lemmas among the 10,000 commonest, in rank order, every
⌊n/100⌋-th from the first (every 84th, from n = 8,495) — a systematic sample, so that
every frequency band is represented and the list is reproducible from the tables and
`measures.json`. The source of each gloss is marked: a *definition* of the Spanish Wiktionary, the
English Wiktionary's Spanish translations (*direct*), or the Spanish Wiktionary's English
translations read backwards (*inverted*): 75 definitions, 25 direct, none inverted. Seven
glosses longer than 160 characters are shortened here with « … »; the tables hold them whole. The
Spanish edition's settings stay as change 6 set them unless the sample says otherwise. One row
moved with refine-lingua-en-es-glosses, past its shown cut: « stage » (row 9) no longer reads « Identificar la
fase de un proceso.^([cita requerida]) », the edition's maintenance template taken out; no other row
of the sample moved, and none moved in or out of it.

| # | Rank | Lemma | Source | Gloss |
|---|---|---|---|---|
| 1 | 1 | the | definition | El, la, los, las |
| 2 | 84 | back | definition | Espalda; Lomo; Parte trasera; Trasero; De vuelta; Atrás; Ir hacia atrás; Mover hacia atrás |
| 3 | 166 | against | definition | Contra |
| 4 | 248 | try | definition | Enjuiciar, juzgar; Probar, intentar, tratar; Tentativa |
| 5 | 333 | anyone | definition | Cualquiera (en frase afirmativa), nadie (en frase negativa), alguien (en frase interrogativa), alguno |
| 6 | 420 | plan | definition | Plan; Plano; Planear |
| 7 | 500 | especially | definition | De manera especial, especialmente; Particularmente, en mayor medida de lo normal; Se utiliza para poner mayor énfasis en alguien o algo |
| 8 | 587 | exactly | definition | Exactamente |
| 9 | 674 | stage | definition | Fase, etapa; Escenario, escena; Platina de un microscopio; Área de descanso, área de servicio; Escenificar, poner en escena, representar; Identificar la fase… |
| 10 | 760 | below | definition | Abajo; Debajo, por debajo de, debajo de |
| 11 | 847 | normal | definition | Normal |
| 12 | 933 | knowledge | definition | Conocimiento, entendimiento, comprensión, inteligencia, razón |
| 13 | 1016 | multiple | direct | Múltiple; Múltiplo |
| 14 | 1100 | otherwise | definition | Por lo demás, por otra parte, de lo contrario, otramente; Si no; Contrario, de otra forma |
| 15 | 1186 | everybody | definition | Hablando de personas, todos, todo el mundo |
| 16 | 1270 | silver | definition | Plata; Moneda hecha de este metal; Artículos de mesa hechos de este metal; Que esta hecho de plata o que es similar a la plata; Propio o relacionado con la p… |
| 17 | 1357 | lucky | definition | Suertudo; Afortunado |
| 18 | 1441 | bay | definition | Bahía; Muelle; Plataforma de carga; Laurel (llamado también por este nombre en inglés) |
| 19 | 1523 | path | definition | Camino, sendero, vereda; Trayectoria; Recorrido; Curva; Secuencia de estados o pasos de un proceso informático |
| 20 | 1610 | entry | definition | Entrada; Acceso |
| 21 | 1698 | distribution | definition | Distribución |
| 22 | 1782 | capable | definition | Capaz; Hábil |
| 23 | 1867 | definition | definition | Definición |
| 24 | 1953 | frequently | definition | Frecuentemente |
| 25 | 2044 | quit | definition | Dejar; Renunciar; Abandonar; Saldar o pagar una deuda |
| 26 | 2133 | comedy | direct | Comedia |
| 27 | 2222 | olympic | direct | Olímpico |
| 28 | 2311 | bedroom | definition | Dormitorio, cuarto, recámara (México) |
| 29 | 2396 | unusual | definition | Inusual |
| 30 | 2481 | designer | definition | Diseñador; De diseño o diseñador |
| 31 | 2571 | crystal | definition | Cristal; Vidrio; Hecho de cristal; Cristalino |
| 32 | 2656 | visible | direct | Visible |
| 33 | 2749 | wayne | definition | Apellido |
| 34 | 2843 | asshole | definition | Ano; Gilipollas (España), boludo (Argentina), pendejo (México), culero (México) |
| 35 | 2934 | complaint | definition | Queja, reclamación |
| 36 | 3025 | symbol | definition | Símbolo |
| 37 | 3116 | remarkable | definition | Notable |
| 38 | 3207 | tune | definition | Melodía, canción, tonada; Afinar, ajustar |
| 39 | 3297 | rude | definition | Grosero |
| 40 | 3390 | dominant | definition | Que gobierna, prevaleciente; Predominante, común, prevalente, de la mayor importancia; (de una parte del cuerpo) Preferida y usada con mayor destreza que la… |
| 41 | 3481 | slight | definition | Leve, pequeño, suave, o débil, despreciable, poco importante, insignificante; Liviano, esbelto; Quitarle importancia a algo, ignorar, obviar; Actuar negligen… |
| 42 | 3573 | generous | definition | Generoso |
| 43 | 3670 | accessible | definition | Accesible |
| 44 | 3760 | hiv | direct | VIH |
| 45 | 3855 | vancouver | definition | Vancouver |
| 46 | 3945 | lean | definition | Incllinarse; apoyarse en algo, recargar en o sobre algo; Delgado, flaco; Magro; Malo, escaso, pobre; Inclinación; Parte sin grasa o delgada de algo; Parte de… |
| 47 | 4037 | separation | definition | Separación |
| 48 | 4130 | keyboard | definition | Teclado; Órgano electrónico |
| 49 | 4223 | diary | definition | Diario o publicación diaria; Diario o libro donde se registran eventos día por día; Libro con apartados para hacer anotaciones día por día; Diario |
| 50 | 4319 | predict | definition | Predecir |
| 51 | 4413 | ballot | definition | Boleta o papeleta para votar; El proceso de votación secreta; Votación; El derecho al voto |
| 52 | 4507 | milan | definition | Milán |
| 53 | 4607 | thanksgiving | definition | Día de Acción de Gracias, festividad nacional de Canadá y EEUU en la que se agradece a Dios por los favores recibidos, las buenas cosechas, etc. Se celebra a… |
| 54 | 4700 | complexity | definition | Complejidad, complicación |
| 55 | 4806 | oz | definition | Abreviatura de ounce ('onza') |
| 56 | 4904 | vaccine | direct | Vacuna, linfa |
| 57 | 4999 | defendant | definition | Demandado, acusado |
| 58 | 5092 | jackie | definition | Hipocorístico de Jack o John; Hipocorístico de Jacqueline o Jacquelyn |
| 59 | 5190 | socially | definition | Socialmente |
| 60 | 5286 | sovereign | definition | Soberano; Una nación que gobierna un territorio más allá de sus fronteras; Una moneda de oro que se empleaba en la Gran Bretaña |
| 61 | 5386 | cuban | definition | Originario, relativo a, o propio de Cubano |
| 62 | 5491 | joshua | definition | Nombre de pila de varón, equivalente del español Josué |
| 63 | 5588 | plaza | direct | Plaza, azogue, zócalo |
| 64 | 5694 | spotlight | direct | Foco, proyector, rodal alumbrado; Enfocar |
| 65 | 5802 | coconut | direct | Coco, laña |
| 66 | 5918 | prospective | definition | Futuro; Eventual, posible |
| 67 | 6018 | sunlight | definition | Sol, luz del sol |
| 68 | 6126 | creep | definition | Arrastrarse; Reptar; Moverse sigilosamente; Trepar; Avanzar o moverse lentamente; Adular a alguien; Persona desagradable, repugnante |
| 69 | 6241 | lego | direct | Lego |
| 70 | 6336 | methodology | direct | Metodología |
| 71 | 6430 | nate | definition | Hipocorístico de Nathan o Nathanael o Nathaniel |
| 72 | 6533 | psychologist | definition | Psicólogo, sicólogo |
| 73 | 6639 | stall | definition | Puesto, tenderete; Una acción destinada a causar, o que en realidad causa, retraso; Platea, luneta; Pérdida de sustentación debido a que se excede el ángulo… |
| 74 | 6743 | uganda | definition | Uganda |
| 75 | 6856 | viewer | definition | Espectador; Visor |
| 76 | 6964 | chandler | definition | Nombre de pila de varón |
| 77 | 7070 | challenger | direct | Desafiador, retante |
| 78 | 7186 | contention | direct | Contienda, contención, contencioso |
| 79 | 7300 | deluxe | direct | De lujo |
| 80 | 7403 | boulder | definition | Peña, roca, bloque de roca |
| 81 | 7515 | adore | definition | Adorar |
| 82 | 7628 | validity | direct | Validez |
| 83 | 7744 | willow | definition | Sauce |
| 84 | 7848 | caffeine | definition | Cafeína |
| 85 | 7953 | transmit | direct | Transmitir |
| 86 | 8065 | suppress | direct | Suprimir |
| 87 | 8187 | slash | direct | Barra oblicua, barra, slash; Tajear |
| 88 | 8306 | sewer | definition | Alcantarillado, alcantarilla, cloaca; Costurero, costurera, cosedor, cosedora |
| 89 | 8423 | shareholder | direct | Accionista, accionario |
| 90 | 8530 | rapist | direct | Violador, violadora |
| 91 | 8644 | perpetual | definition | Perpetuo |
| 92 | 8748 | evidently | definition | Evidentemente |
| 93 | 8862 | duplicate | direct | Duplicado; Duplicar, repetir; Duplicado |
| 94 | 8972 | citrus | direct | Cítrico |
| 95 | 9091 | combustion | definition | Combustión |
| 96 | 9204 | asteroid | definition | Asteroide |
| 97 | 9308 | upstream | direct | Aguas arriba, a contracorriente, río arriba; A contracorriente, río arriba |
| 98 | 9427 | rag | definition | Harapo, andrajo, jirón; Trapo |
| 99 | 9555 | nausea | direct | Náusea, repulsión, asco |
| 100 | 9673 | magician | direct | Mago, fenómeno |

## Meanings and the translators' words

The Spanish Wiktionary writes notes to its readers into its senses, opens an entry on its oldest
sense, writes a surname's note under the capitalised headword spelled like a common word, heads a
possessive « adjetivo », and has its own typography; the English Wiktionary's translators write
their notes inside the Spanish words they list; read backwards, a Spanish word is listed once per
part of speech of its own. Read as written, en-es glossed « will » (rank 37) « Deseo, inclinación,
disposición; …; Apellido; Hipocorístico de William », « smith » « Apellido; Herrero », « a » « Un,
una. A veces se omite en la traducción », « orchestra » « Orquesta, orquestra (disused) »,
« lengthy » « Largo; Largo; Largo; Largo ». Rules of the Spanish edition and of en-es's reducer read
the entries and the tables before the shared rules do (refine-lingua-en-es-glosses):

- **The edition's notes to its readers** (`reduce_edition_es.py`, its notes): a maintenance template
  (« ^([cita requerida]) »), a disambiguation note (« [sentido del sustantivo] »), a reference to
  numbered senses (« (definiciones [1,2]) »), the expansion notice, and a usage note after the
  meaning, a sentence opening on a closed list (« A veces », « Usado », « Se dice », « A no
  confundir »…). A second sentence that carries the meaning stays (« isn't » « …not. Traducida como
  «no es» o «no está» »), and so do optional words in brackets (« [a] tiempo completo »).
- **Current senses first, one Spanish typography** (`reduce_edition_es.read_as_meanings`): the
  senses the edition marks obsolete or outdated go after the other senses of their entry, nothing
  left out (« go » opens on « Ir », « will » on « Voluntad, albedrío »); one ellipsis « … », spaced
  between two words; straight double quotes paired « » (« it's » « («ello») »), single quotes kept.
- **Which senses gloss an English word** (`reduce-en-es.py english_entries`): a capitalised proper
  noun's sense that only says the word is a surname or a given name glosses no word that has an
  entry of its own in lower case holding a meaning (« smith » « Herrero », « mike » « Micro,
  micrófono »); a name's own row keeps its notes (« wayne » « Apellido »), and a proper noun's other
  senses stay (« south » « …; (region) Sur »); a possessive or demonstrative adjective section is a
  determiner (« her », « my », « its », « their », « that », « such »); « participio presente » reads
  « forma en -ing », as the card names the form (« be »).
- **The translation tables' words** (`reduce-en-es.py read_translated`): the direct table leaves out
  a word labelled « (disused) » or « (desus.) » (« orchestra » « Orquesta »), and the note its
  translator wrote inside a word when it follows a loanword (« hall (hol) »), holds no Spanish word
  (« (despective) »), is only a number (« [4] a favor ») or opens on « with »; a note holding a
  Spanish word stays (« guardería (infantil) »). The inverted table lists a Spanish word once per
  English word, under the first of its parts of speech the English word's readings name
  (`../en/grammar.tsv`): « lengthy » « Largo », « grey » « Gris, plomo; Agrisar », « seaman » a noun.

Against the tables before them: **295 rows change, 149 of the top 10,000** — 159 / 121 glossed by
the Spanish Wiktionary, 78 / 18 by the direct table, 58 / 10 by the inverted one —, the first sense
of 140 / 63; **64 expressions change**. One lemma loses its gloss, « malign » (rank 33,911), whose
one translation is labelled disused, and two expressions for the same reason (« deep end », « with
both hands »); none gains one. Rule by rule: the notes 15 / 12 rows and 3 expressions, the names
61 / 53, the determiners 6 / 6 (their runs alone), the order 41 / 27 (20 / 12 opening otherwise),
the typography and the -ing form 39 / 26 and 19 expressions, the translation tables 136 / 28 and 42
expressions. The coverage and the share do not move: no lemma of the top 20,000 gains or loses a
gloss.

## Known noise, for the owner's 5.1

What the sample and a pass over the tables show, with the module a fix would live in. Fixed by
refine-lingua-en-es-glosses (above): the English Wiktionary's editor notes in the direct table (39
glosses held a disused word, 0 now), the inverted table's repeated words (52 glosses repeated a
whole sense, 0 now) and the name notes on common words (61 rows, 53 of the top 10,000). Left,
measured in its design (*Measured*, *For the owner*):

- **Labels** the packs do not carry (Q1): a sense obsolete or outdated (now after the others),
  regional, of a register (colloquial, slang, vulgar) or rare is shown without its label —
  « do » « Timar, estafar; Follar », « mouse » « Timorato, apocado, flojo ». Where: a rule of the
  Spanish edition (`reduce_edition_es.py`), en-es alone.
- **The part of speech a row opens on** (Q2): the page's order — « up » « Construido », « like »
  « Gustar », « lead » « Plomo », « by » « Junto a ». Where: a studied-side frequency of a part of
  speech, which no table holds.
- **The quantifiers' heading** (Q3): « all », « no », « any », « much », « other » stay « adjetivo »,
  as en-fr heads them. Where: `english_entries`, with a list of words.
- **The inverted table's regional or other-sense words** (Q4): 504 lemmas, 73 of the top 10,000,
  about a dozen of those reading wrong or regionally (`second-hand` « Chivera », `sup` « KLK », `la`
  « La, hombrecillo », `metro` « Subte »); a floor on the Spanish word's frequency was measured and
  rejected. Where: `read_translated` (`reduce-en-es.py`), en-es alone.
- **A name's own row**: 603 rows (366 of the top 10,000) are glossed by name notes alone
  (`david`, `wayne` « Apellido »; in the sample, `chandler`, `joshua`) — they tell the reader of the
  capitalised word what it is, and hold 3.7 points of the top 10,000's coverage. Where: a card that
  reads a token's capital, shared.
- **The direct table's repeated word** across the English word's own parts of speech (`israeli`
  « Israelí; Israelí », 237 glosses, 53 of the top 10,000): each run says something true of the
  English word, so it stays. es-fr's inverted table repeats words in 254 glosses (`abridor`
  « Greffoir; Greffoir ») and es-en's in 9, through `reduce_common.translation_gloss`: shared.
- **English notes a Spanish text also writes** (`mat` « posavasos (coaster) », `sketchy` « (to be
  sketchy) », the expression « over there » « (further than «allá») »), kept by the rule that a note
  holding a word wordfreq knows in Spanish stays; and one Spanish qualifier the rule takes out
  (« (articulario) » in « range of motion »).
- **« etc » and an abbreviation lose a sense's last period** in every pair (`oh`, `fair`: 47 glosses
  here): `reduce_common.clean_gloss`, shared — re-pins every pair.
- **Upstream wording** — « !Um¡, !uf¡ » (`huh`), « Incllinarse » (`lean`), « de el » for « del » (27
  glosses) — corrected on the Spanish Wiktionary, it arrives with en-es's next update.

## Its sources

en-es pins no extract of its own: everything it reads is derived from whole Wiktionary dumps and
extracts at an update, in one pass each, and the dumps are not kept (`pack_sources.py DUMPS`,
D2). The derived files are the assets of en-es's own release, `lingua-pack-sources-en-es-<snapshot>`
(today `lingua-pack-sources-en-es-2026.10.08`):

- **The Spanish Wiktionary's English section** (`kaikki-es-English.jsonl`, 36,426,539 B):
  its English entries as the dump writes them, from kaikki's dump of the whole edition — es-fr's
  and es-en's address, on en-es's own snapshot of it (kaikki's regeneration of 2026-10-02).
  Their definitions gloss first, cleaned by the Spanish Wiktionary's rules (`reduce_edition_es.py`).
- **The Spanish Wiktionary's English translations** (`kaikki-es-traductions-en.jsonl`,
  2,200,504 B): the English words its Spanish entries list, derived from the same dump in
  the same pass — es-en's derivation, run again on en-es's snapshot, so that en-es's pin is
  self-contained and an es-en re-snapshot moves no en-es byte. Read backwards, they gloss what
  nothing else does, at most three Spanish words per part of speech, the commonest first.
- **The English Wiktionary's Spanish translations** (`kaikki-en-traductions-es.jsonl`,
  13,757,388 B): the Spanish words its English entries list, under their senses, derived from
  kaikki's extract of the English Wiktionary's English section
  (`kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl`), served uncompressed —
  3,335,546,346 B as served at the first update (kaikki's regeneration of 2026-10-03), read as a dump is, told apart
  by the gzip magic, and not kept. They gloss what the Spanish Wiktionary leaves out, at most three
  words per part of speech, in the table's order. Change 38 (`migrate-lingua-pack-sources-to-raw-
  dumps`) switched the address, as the programme's risk 6 asks: an update derives this file from
  the English Wiktionary's dump (`kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz`,
  2,981,058,381 B gzipped), the pin keeping the extract's derivation until en-es's next update.
  The dump keeps each table where the page writes it, in the page's order, where the extract moves
  it under the sense it translates, and 60 entries repeat 77 translations they already list (the
  extract lists each once): 186 glosses and 55 expressions take their words in another order or
  another third word, no row added or removed (`../../SOURCES.md`, *Extract and dump are measured
  against each other*) — the same 186 and 55 under the rules above, which act on each word the
  file lists whatever its order (measured again from the dump of 2026-10-03 08:24, still served on
  2026-10-08). No reducer fix — the derived file keeps no sense glosses to order the
  words by —: en-es's next update carries them, named in its report, and the owner judges the 186
  glosses there.

## Licences

The repository is Apache-2.0; **these files are not**. They are derived from the sources above and
carry their licences: `gloss.tsv`, `senses.tsv` and `mwe.tsv`, CC BY-SA 4.0 and the GFDL (kaikki).
English's tables in `../en/` carry theirs (`../en-fr/README.md`). `NOTICE` gives the full
attribution. See `../../SOURCES.md`.

## Changing them

Never by hand.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=en-es` and `mode=update`.
  It reads today's Spanish and English Wiktionaries' dumps, keeps the three derived files as the
  release `lingua-pack-sources-en-es-<snapshot>`, whose notes name the dumps, reduces, and pushes
  the branch
  `lingua-pack/en-es/<snapshot>`. The update's summary shows the coverage and the share beside what
  the tables change.
- **When English's tables move**: en-fr's update or re-reduction writes `../en/`, and brings en-es
  along on the same branch, reduced again from its own pinned sources; en-es's `pack_version` moves
  with the studied tables. A pull request that moves a table of `../en/` without recording en-es
  again fails, naming en-es and the table (`pack_sources.py check-reducer`, `pack_report.py`).
- **After editing the reduction rules** — `reduce-en-es.py`, `reduce_common.py` (every pair's), or
  `reduce_edition_es.py` (the Spanish Wiktionary's, which no other committed pair loads), and
  en-fr's rules too, whose reduction writes `../en/`: the check lane fails until the tables are
  reduced again from the pinned sources. Run `scripts/lingua-data/build.sh --reduce en-es <out>`
  (Python 3.12, `requirements-reduce.txt`), after en-fr's when both are due, or `lingua-pack-update`
  with `mode=reduce`.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.

The monthly dry run of the update checks this pair as it checks en-fr, es-fr and es-en.
