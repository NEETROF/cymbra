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

On the 2026-10-08 tables (pinned snapshot `2026.10.08`, `pack_version` `2026.10.08+65915c5.e1915ca`: the
snapshot, the rules' digest, and the digest of the studied tables `pin.json` records):

- **Spanish glosses for 21,966 lemmas** of English's 40,685: 10,247 from the
  Spanish Wiktionary's English entries (its definitions, by the Spanish edition's rules), 11,214
  from the Spanish translations the English Wiktionary lists (the direct table, in its order) and
  505 from the English translations the Spanish Wiktionary's Spanish entries list,
  read backwards (the inverted table, the commonest Spanish word first); **17,096
  expressions**. Up to eight senses grouped by part of speech, as every pair's, opening on a
  capital as the Spanish Wiktionary writes them. A letter glosses no word, in either direction.
  No pivot through a third language, no machine translation.
- The share of the commonest lemmas glossed, which the `reduce` job holds to a floor the owner sets
  on the pull request (`gloss_coverage.py --pair en-es --floor …`, the same value in `FLOORS`):

  | Lemmas | Spanish Wiktionary | with the translations | the floor (proposed) | the study |
  |---|---|---|---|---|
  | top 5,000 | 80.4 % | 93.0 % | 91.4 % | 93.4 % |
  | top 10,000 | 62.9 % | 85.0 % | 83.2 % | 85.2 % |
  | top 20,000 | 42.5 % | 71.7 % | 69.9 % | 71.9 % |
  | all 40,685 | 25.2 % | 54.0 % | — | — |

  The floor is proposed at the study's figures less two points (the programme's risk 5, M6's
  rule); the owner settles it (task 5.1), and the value is written into the requirement and into
  the reduce job before merge.
- **The translation-table share** (D4): of the 8,496 glossed lemmas among the 10,000
  commonest, **26.0 %** come from a translation table rather than from a definition — 2,135
  from the direct table, 74 from the inverted one (`measures.json`, which the reducer
  writes beside its tables and no pack stores; `pack_report.py --measures` shows it in the update's
  summary).
- **Dictionary words are en-fr's**: 2,501 lemmas en-es glosses are no dictionary word
  of English, and 5,334 dictionary words have no Spanish gloss, so the pack
  carries a lexical section, and the vocabulary estimate counts the same 24,799 words for
  both pairs.
- **The pack is 1,691,073 B**, 32.3 % of the 5 MiB budget.

## A sample of 100 glosses

For the owner's review (D5): the glossed lemmas among the 10,000 commonest, in rank order, every
⌊n/100⌋-th from the first (every 84th, from n = 8,496) — a systematic sample, so that
every frequency band is represented and the list is reproducible from the tables and
`measures.json`. The source of each gloss is marked: a *definition* of the Spanish Wiktionary, the
English Wiktionary's Spanish translations (*direct*), or the Spanish Wiktionary's English
translations read backwards (*inverted*). The Spanish edition's settings stay as change 6 set them
unless the sample says otherwise.

| # | Rank | Lemma | Source | Gloss |
|---|---|---|---|---|
| 1 | 1 | the | definition | El, la, los, las |
| 2 | 84 | back | definition | Espalda; Lomo; Parte trasera; Trasero; De vuelta; Atrás; Ir hacia atrás; Mover hacia atrás |
| 3 | 166 | against | definition | Contra |
| 4 | 248 | try | definition | Enjuiciar, juzgar; Probar, intentar, tratar; Tentativa |
| 5 | 333 | anyone | definition | Cualquiera (en frase afirmativa), nadie (en frase negativa), alguien (en frase interrogativa), alguno |
| 6 | 418 | near | definition | Cercano, próximo; Aproximado; Cerca; Casi, aproximadamente; Acercar, aproximar |
| 7 | 498 | account | definition | Cuenta; Relato; Estado de cuenta; Argumento, causa, motivo; Relatar; Dar explicaciones; Reportar sobre el dinero recibido y gastado; Considerar |
| 8 | 586 | director | definition | Director |
| 9 | 673 | serious | definition | Serio |
| 10 | 759 | base | definition | Base, pie, apoyo, sustento, soporte; Base, fundamento, esencia, raíz; Base; Base, base de Lewis, hidróxido, álcali; Base, basa, pedestal, pie, peana, soporte… |
| 11 | 846 | island | definition | Isla; Formar una isla; Poner en una isla |
| 12 | 932 | heavy | definition | Pesado |
| 13 | 1015 | lack | definition | Falta, carencia; Escasez; Faltar, carecer |
| 14 | 1099 | nation | definition | Nación |
| 15 | 1185 | engine | definition | Motor; Locomotora |
| 16 | 1269 | rise | definition | Subida; Elevación; Aumento; Ascenso; Surgir, salir; Subir; Elevar; Aumentar |
| 17 | 1356 | length | definition | Longitud |
| 18 | 1440 | audience | definition | Audiencia; Público |
| 19 | 1522 | museum | definition | Museo |
| 20 | 1609 | drama | direct | Drama, obra teatral |
| 21 | 1697 | debate | definition | Participar en un debate; Un argumento o discusión, usualmente en un marco ordenado o formal, frecuentemente con más de dos personas, que generalmente termina… |
| 22 | 1781 | bright | definition | Brillante; Iluminado, con luz; Animado, radiante; Claro, diáfano |
| 23 | 1866 | brazil | definition | Brasil |
| 24 | 1952 | expert | definition | Persona con amplio conocimiento o habilidad en un tema determinado, experto; Un jugador clasificado justo por debajo de maestro; Extraordinariamente capaz o… |
| 25 | 2043 | print | definition | Imprenta; Letra de imprenta; Letra; Impresión; Copia; Imprimir, escribir; Estampar; Impreso, disponible para su venta |
| 26 | 2132 | clinical | definition | Clínico; Desapasionado, frío |
| 27 | 2221 | monster | definition | Monstruo; Monstruoso |
| 28 | 2310 | awful | definition | Horrible; Extremadamente |
| 29 | 2395 | uncle | definition | Tío |
| 30 | 2480 | dallas | definition | Dallas |
| 31 | 2570 | crap | definition | Afrecho, barcia, salvado; Tirada perdedora (2, 3 ó 12) en el juego de dados craps; Mierda; Basura; Estupideces, pendejadas (México), boludeces (Argentina); D… |
| 32 | 2655 | translation | definition | Traducción; Desplazamiento |
| 33 | 2748 | ward | definition | Apellido; Guarda, escolta, vigilante, centinela; Custodia; Tutelaje; Sala, ala; Distrito electoral |
| 34 | 2842 | agricultural | definition | Agrícola |
| 35 | 2933 | communist | definition | Comunista |
| 36 | 3024 | steady | definition | Firme; Estable; Estabilizar |
| 37 | 3115 | rely | direct | Atenerse, fiarse |
| 38 | 3206 | tokyo | definition | Tokio, Tokío |
| 39 | 3296 | roy | definition | Nombre de pila de varón |
| 40 | 3388 | discipline | definition | Un comportamiento controlado, autocontrol; Una rama específica del conocimiento, el aprendizaje o la práctica: disciplina |
| 41 | 3480 | safely | definition | Con seguridad, de modo seguro, seguramente |
| 42 | 3572 | forty | definition | Cuarenta |
| 43 | 3669 | xbox | direct | Xbox |
| 44 | 3759 | grammar | definition | Gramática |
| 45 | 3853 | tier | direct | Piso, rango, nivel |
| 46 | 3944 | kyle | definition | Nombre personal masculino |
| 47 | 4036 | sandwich | definition | Sándwich, emparedado; Bocadillo |
| 48 | 4129 | hybrid | definition | Híbrido |
| 49 | 4222 | darling | definition | Persona muy estimada por otra; Querido, tratamiento afectuoso; La persona o cosa que es la favorita; Muy estimado; Favorito; Encantador |
| 50 | 4318 | poison | definition | Veneno; Envenenar |
| 51 | 4411 | assure | definition | Asegurar; Convencer |
| 52 | 4506 | medieval | definition | Medieval |
| 53 | 4606 | taiwan | definition | Taiwan |
| 54 | 4699 | carol | definition | Nombre de pila de varón; Nombre de pila de mujer |
| 55 | 4805 | orchestra | direct | Orquesta, orquestra (disused) |
| 56 | 4903 | telegraph | direct | Telégrafo; Telegrafiar |
| 57 | 4998 | dale | definition | Valle |
| 58 | 5091 | inclusive | direct | Inclusivo, inclusive |
| 59 | 5189 | skirt | definition | Falda; Bordear, rodear |
| 60 | 5285 | sixteen | definition | Dieciséis |
| 61 | 5385 | costly | definition | Costoso |
| 62 | 5490 | joey | definition | Nombre de pila de varón |
| 63 | 5587 | pioneer | definition | Pionero; Promover; Ser pionero |
| 64 | 5693 | snack | definition | Snack, colación, refrigerio, refacción, piscolabis, chuchería, mecato; Merendar, comer algo ligero; Comer entre comidas |
| 65 | 5801 | clutch | definition | Aferrar; Agarrar; Embrague, cloch; Abrazo fuerte; Apuro, crisis, aprieto; Apretón; Desempeñarse o tender a desempeñarse bien en situaciones difíciles y de al… |
| 66 | 5916 | processor | direct | Procesador |
| 67 | 6017 | stunt | definition | Escena peligrosa; Truco, ardid, treta |
| 68 | 6124 | cardiac | direct | Cardiaco |
| 69 | 6240 | katherine | definition | Nombre de pila de mujer, equivalente del español Catalina. Variante común de Catherine |
| 70 | 6335 | metallic | definition | Metálico |
| 71 | 6429 | narrator | definition | Narrador |
| 72 | 6532 | psychic | inverted | Psíquico |
| 73 | 6638 | springfield | definition | Apellido |
| 74 | 6742 | turnover | direct | Cifra de negocios, empanada, movimiento de mercancías |
| 75 | 6855 | velvet | definition | Terciopelo |
| 76 | 6963 | c'mon | definition | Contracción de 'come on' |
| 77 | 7069 | cereal | direct | Cereal, herbal, cereales |
| 78 | 7185 | coil | direct | Espiral, hélice, bobina; Enrollar |
| 79 | 7297 | contributor | definition | Contribuidor |
| 80 | 7401 | boiler | direct | Caldera, calentador, bóiler |
| 81 | 7514 | wildly | definition | Alocadamente, locamente; Insensatamente, irreflexivamente, a tontas y a locas; Ferozmente, furiosamente, violentamente; Desordenadamente, sin disciplina |
| 82 | 7626 | uruguay | definition | Uruguay |
| 83 | 7743 | willis | definition | Apellido |
| 84 | 7847 | bunker | definition | Refugio subterraneo; Búnker; Depósito de combustible; Combustible; Carbonera; Caer en el búnker; Dificultar; Cargar combustible |
| 85 | 7952 | translator | definition | Traductor |
| 86 | 8064 | stupidity | definition | Tontería |
| 87 | 8186 | seafood | definition | Animales o plantas comestibles procedentes del mar |
| 88 | 8304 | scarlet | definition | Nombre de pila de mujer; Escarlata, grana |
| 89 | 8422 | seldom | definition | Casi nunca, rara vez |
| 90 | 8529 | presume | direct | Presumir |
| 91 | 8643 | payday | direct | Día de pago, recompensa |
| 92 | 8747 | evelyn | definition | Nombre de pila de mujer; Apellido matronímico; Nombre de pila de varón |
| 93 | 8860 | disrupt | definition | Interrumpir; Desbaratar; Perturbar |
| 94 | 8971 | cindy | definition | Nombre de pila de mujer |
| 95 | 9090 | colt | direct | Potranco, potro |
| 96 | 9203 | armenia | definition | Armenia |
| 97 | 9307 | truman | definition | Apellido |
| 98 | 9426 | pun | definition | Juego de palabras, uso de palabras con dobles sentidos o de modo equívoco, ya sea por igualdad o similitud sonora u ortográfica de las mismas |
| 99 | 9550 | mitigate | definition | Reducir o decrementar, hacer menos severo o más fácil de soportar; Minimizar, quitar hierro |
| 100 | 9672 | madden | direct | Enloquecer |

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
  words per part of speech, in the table's order. The programme's risk 6 names the raw English
  dump; the extract keeps the update within the job's reach today, and change 38 switches the
  address if kaikki stops serving it (`../../SOURCES.md`).

## Licences

The repository is Apache-2.0; **these files are not**. They are derived from the sources above and
carry their licences: `gloss.tsv`, `senses.tsv` and `mwe.tsv`, CC BY-SA 4.0 and the GFDL (kaikki).
English's tables in `../en/` carry theirs (`../en-fr/README.md`). `NOTICE` gives the full
attribution. See `../../SOURCES.md`.

## Changing them

Never by hand.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=en-es` and `mode=update`.
  It reads today's Spanish Wiktionary dump and English Wiktionary extract, keeps the three derived
  files as the release `lingua-pack-sources-en-es-<snapshot>`, reduces, and pushes the branch
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
