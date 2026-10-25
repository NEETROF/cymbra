# Design — refine-lingua-it-es-glosses

## Context

See proposal.md (Why). What exists, on `main` at `16da30a05` (change 27, #1068; its tables committed
by the owner's update of 2026.10.25, run 40210737711):

| Where | What |
|---|---|
| `reduce-it-es.py` | it-es's reducer: the native side alone, against `tables/it/` as committed — the Spanish Wiktionary's Italian section (`straight_headwords` → `italian.function_words` → `common.without_letter_senses` → `spanish.read_as_meanings` → `italian.read_section`), then `native_side`: the section's definitions, then each table-shaped source in D2's order (`common.fallback_glosses`), the first that glosses a lemma glossing it whole; the split words, the acronym rows |
| `reduce_italian_es.py` | it-es's own pre-pass (change 27's D4): names and acronyms, pointers, carried meanings, notes, `FUNCTION_WORDS` (11 rows, `ne`'s adverb « De allí, de ahí » the one a person wrote), `ITALIAN_ALIKE`, the Spanish section's notes, marked words, split words |
| `reduce_spanish_tables.py` | fr-es's table rules copied, every fr-es global a parameter, imported by it-es alone: not edited here |
| `reduce_common.py`, `reduce_edition_es.py`, `reduce_italian_acronyms.py` | The shared rules, the Spanish edition's (en-es's, fr-es's and it-es's digests) and Italian's acronym rows (it-fr's, it-en's and it-es's): not edited here |
| `reduce_french_treebank.py`, `reduce_italian_treebank.py` | fr-en's order rule (`commonest_first`) and MarkIT's counts (`markit_pos_counts`), which it-fr imports (change 17's D2) and change 23 imports for it-en: imported here unchanged |
| `tables/it-es/` | 24,705 glossed lemmas (definitions 3,973; the Italian Wiktionary's translations 3,582; the Spanish tables read backwards 2,996; Wikidata's links 14, its items 738; the Spanish section 1,148; the pivot 12,253; `USA`); 1,165 expressions; coverage **85.2 / 76.1 / 65.1 %** against `FLOORS["it-es"]` 83.2 / 74.0 / 63.1; the pack 1,894,196 B, its layer 343,240 B |
| `tables/it/` | Italian's studied side, it-fr's, read as committed; no rule here writes it |
| The requirement this change touches | Change 27's *Italian is glossed in Spanish from the Spanish Wiktionary's Italian section, the two Wiktionaries' tables, Wikidata and a pivot*, open, held by `add-lingua-pack-it-es` alone |
| The lists for this change | Change 27's *Known data defects* and *Left for change 28* (its design), its README's *Known defects, for change 28*; the specialist review of the committed tables (2026-10-25: 11 systematic issues, the wrong rows of the 300 commonest, error rates by band and by source, with `sources.json` and `allsources.json`, which this design reproduced byte for byte) |

## Goals / Non-Goals

**Goals:**
- it-es's glosses open on the Spanish word an Italian word's commonest use calls for, before change 30
  lists it-es — by rules of it-es's own modules, each measured alone on the committed tables and kept
  only where it reads better (the criteria below), and a short reviewed table for what no rule reaches.
- Every item of the lists above decided: a rule, a reviewed row, left with its reason, or put to the
  owner in plain words with examples (tasks group 0).

**Non-Goals:**
- Any byte of `tables/it/` or of the eight other pairs (en-fr, es-fr, es-en, en-es, fr-en, fr-es,
  it-fr, it-en): no rule of `reduce_common.py`, `reduce_edition_es.py`, `reduce_italian_acronyms.py`,
  `reduce_spanish_tables.py` or the treebank modules is edited (D1).
- A new source beyond MarkIT's two sections, which it-fr already pins (D1).
- The coverage floor: `FLOORS["it-es"]` stays 83.2 / 74.0 / 63.1.
- The Spanish edition's own gaps, which fr-es and en-es share (« Contracción de… », « _(n) »): a
  follow-up of that edition (change 27's *Known data defects*).

## How this was measured

**Criteria, written on 2026-10-25 before any figure of this change's rules was read.** Change 27's own
figures (its design, its README) and the specialist review of its tables (2026-10-25) are this
change's inputs, read before: they are the list, not a measurement of a rule.

- *The base.* it-es's committed tables on `main` at `16da30a05` (change 27, #1068: `gloss.tsv`
  `0ab46596…`, `senses.tsv` `53e6e980…`), reproduced byte for byte by `reduce-it-es.py` over the pinned
  release `lingua-pack-sources-it-es-2026.10.25`, fetched by `pack_sources.py fetch-pinned` into a
  scratch folder and read as data, with Python 3.12 and `requirements-reduce.txt` (wordfreq 3.1.1).
- *The bands.* A row is counted by its lemma's rank in `tables/it/freq.tsv`: 1–1,000, 1,001–5,000,
  5,001–20,000, past 20,000; the 300 commonest are the review's band.
- *Each rule alone, then together.* Each rule is the committed reducer with that rule alone added, in a
  scratch copy; then every rule kept, together. Each is compared row by row with the committed
  `gloss.tsv`: rows changed by band, rows whose first sense changes, lemmas gained and lost, expressions
  changed, and the coverage of the 5,000 / 10,000 / 20,000 commonest.
- *What counts as better.* A row reads **better** when its first sense moves to the Spanish word a
  bilingual dictionary gives first for the word's commonest use — the review's « right gloss » where it
  names one —, or when it loses a sense that is wrong (not Spanish, another word's, a pointer's or a
  description's wording, a name's note on a common word, a repeat) and keeps a right one; **worse** when
  its first sense moves to a rarer, regional, wrong or non-Spanish word, or when it loses its only right
  sense or a common sense no other sense says; **no worse** otherwise (a reordering of right senses, a
  synonym for a synonym).
- *Who reads.* Every changed row among the 1,000 commonest, and, in each of 1,001–5,000 and
  5,001–20,000, every changed row when a rule changes at most 60 there, else 60 drawn by the seed
  20261025. The reader is an agent, not a native speaker; the owner's specialist reads the gloss diff of
  the implementation (tasks).
- *What keeps a rule.* A rule is kept when, in every band read, its better rows outnumber its worse
  rows at least four to one, and no row of the 300 commonest reads worse unless a reviewed row (below)
  restores it; it is narrowed — and the narrowing recorded, with the rows it cost — when it fails that
  and a narrower wording passes; dropped otherwise. A variant of a rule (a threshold, a tie-break, a
  part of speech's handling) is measured the same way, and the variant kept is the one with the most
  better rows among those that pass.
- *What bars a rule, whatever it fixes.*
  - It takes the only gloss of a ranked lemma of the 20,000 commonest, unless that gloss is pure noise
    (a name's note on a common word, a description, a word that is not Spanish, another word's meaning).
  - It puts the coverage of the 5,000 / 10,000 / 20,000 commonest under the floor, **83.2 / 74.0 /
    63.1** (`FLOORS["it-es"]`, settled by the owner, never lowered).
  - It needs a module another pair reads to change: `reduce_common.py`, `reduce_edition_es.py`,
    `reduce_italian_acronyms.py`, the treebank modules, `reduce-fr-es.py`, `reduce-en-es.py`. Such a fix
    is reported, not made. `reduce_spanish_tables.py` may change only while it-es alone imports it,
    shown by every other pair's pin and by fr-es's own reduction, byte for byte.
  - It reads a source it-es's pin does not record, or another pair's committed tables (whose next
    reduction would then move it-es without its pin saying so).
  - It reads UD Italian-PUD or MarkIT's test section.
- *A reviewed row* (`FIRST_RUN`, `CORRECTIONS`) is written only for a clear error of the 300 commonest,
  or of an item change 27 handed to this change, that no kept rule reaches; each with its reason, keyed
  so that it fails or names itself when the source no longer writes what it corrects. Its words come
  from a source where one gives them; a gloss no source gives is **written by a person** and marked so,
  in the code and in the pull request, as change 27's `ne` « De allí, de ahí » and it-fr's `dati`
  « Données ». A matter of taste goes to the owner, with examples.
- *The precedents already settled apply*: a treebank reorders a function word's row, never a content
  word ahead of another (fr-en's Q2, it-fr's and it-en's D2a); a female equivalent is no word's meaning
  (fr-en's Q3); a name opening a row is accepted (fr-en's Q5); a name alone keeps its note (it-en's D6).
- *The target.* On the review's judged rows (its tally, the same rows read again on the new tables),
  the 300 commonest read wrong in at most **3 %** of their rows — the level of the review's bands
  past 300 (4.7 % for 301–5,000, 2 % for 5,001–20,000) and of it-en's 1,000 commonest before change 23
  (about 29 of 1,000) —, against 8.6 % now, with the three unglossed articulated prepositions glossed;
  and the rows read « partly right » fewer than now (11 %). The 301–5,000 and 5,001–20,000 bands judged
  wrong by the review read no worse. A target missed is reported with the rows that miss it, not met by
  lowering a criterion.

**The data.** it-es's pinned release, `lingua-pack-sources-it-es-2026.10.25`, fetched by
`pack_sources.py fetch-pinned` into a scratch folder (every asset checked by the sha256 its pin
records), and MarkIT's training and development sections at `e210079d` (`4f6fc342…`, `633b6186…`,
it-fr's pin); read as data, by scripts outside the repository, with Python 3.12 and wordfreq 3.1.1
(`requirements-reduce.txt`), whose bundled lists — Spanish, and English, French, Portuguese, Italian
and German for D5 — are the pinned package. `reduce-it-es.py` as committed, run over those bytes, gives
`gloss.tsv`, `senses.tsv`, `mwe.tsv` and the `NOTICE` byte for byte, and each source's gloss of every
lemma exactly as the review's `allsources.json`.

**The prototype.** A scratch copy of the reducer with each rule as a switch: the section's rules after
`italian.read_section`, the table rules on the six table-shaped sources before `native_side`, the row
rules after it, in this order — D4, D2's Wikidata items, D9's taxa, D6, D2, D3, D7's cap, D9's repeats,
then the reviewed tables (D11, D12). Every figure below is re-measured by the real reduction at
implementation (task 3.2).

**Reading.** Every changed row of the 1,000 commonest, and in each of 1,001–5,000 and 5,001–20,000
every changed row or 60 drawn by the seed 20261025, as the criteria say; a rule's variants read the
same way. The reader is an agent (Claude), not a native speaker: its readings decide which variant is
kept, and the owner's specialist reads the gloss diff of the implementation (task 4.1).

## Measured

**The classes, on the committed tables**, and what this design does:

| # | Class (the review's issue) | Ranked rows | Examples | Decision |
|---|---|---|---|---|
| 1 | The first source wins against agreement (1) | 1,485 rows its rule reorders (105 / 312 / 597 / 471) | `rumore` « Rumor » (four sources say « Ruido »), `accordo` « Acorde », `caso` « Azar », `andare` « Andar », `corso` « Cambio », `periodo` « Menstruación », `unico` « Simple » | D2 |
| 2 | A rare part of speech first, or the common one missing (2) | 9 rows MarkIT reorders, 2 it fills | `ancora` « Ancla; Todavía », `secondo`, `perché`; `spesso` « Gordo » without « a menudo », `verso` « Sentido » without « hacia » | D3 |
| 3 | A feminine pointer lends the masculine (3) | 5 rows by the rule, 4 by reviewed rows | `ragazza` « Chico, muchacho, cabro; Novio, pololo », `nonna` « Abuelo », `zia` « Tío… », `attrice` « Actor », `porca` « (Sus scrofa domesticus) cerdo… », `fanciulla`, `alunna` « Alumno » | D4, D2, D12 |
| 4 | Words that are not Spanish; place names read as words (4) | 168 distinct words Spanish rarely writes (303 table items) in 40 rows; 17 place labels in 3 rows | `evidente` « Evident », `obbedire` « Obéir », `battere` « …, battre, taper », `transitorio` « Temporaneo, effimero »; `ladro` « Ladrón, chorizo, Spagna » | D5 |
| 5 | Rare, heraldic or regional first words (5) | 837 rows a commoner word opens; 8 heraldic senses in 5 rows | `rapidamente` « Aquejadamente », `pulizia` « Limpiadura », `panchina` « Cadiera », `cellulare` « Movíl », `caos` « Sindiós », `controllo` « Aprobanza »; `nudo` « …; Dicho de un mueble heráldico… » | D6 |
| 6 | False friends (6) | — (each a case of class 1 or 5) | `andare`, `rumore`, `caldo` « Cálido », `colpo` « Brazada », `cancellare`, `nominare` fixed by D2; `subito` « Súbito », `stipendio` « Estipendio », `tasso` « Tejón, tejo », `costume`, `bravo` « Bravo, gallardo… » by D12 | D2, D12 |
| 7 | Definitions and dumps instead of translations; given names (7) | 11 list senses capped; 127 given names' senses in 104 rows | `inoltre` (166 characters), `talvolta`, `gratis`, `ergo`; `paolo` « Nombre de pila de varón, equivalente del español Pablo » | D7 |
| 8 | A name's note alone on a common word (8) | 7 rows; 128 rows of the 20,000 hold names' notes alone (change 27 counted 136) | `senatore` « Apellido », `rocca`, `genovese`, `signorile` | D8, D7 |
| 9 | Duplicate senses; a taxon before the meaning (9) | 42 rows, 1 expression; 23 taxa | `persona` « Persona (ser humano); Persona », `islandese` « Islandés, Islandés », `d'accordo` « De acuerdo; De acuerdo, vale »; `cavallo` « (Equus caballus) Caballo » | D9 |
| 10 | Verb forms inside other rows (10) | 10 rows | `faccia` « Cara, haga, hagas », `dati` « Datos, facto; Sea », `salvo` « …; Salvarse » | D10 |
| 11 | The `di` family (11) | 6 rows, 3 unglossed | `del` « Algo, un tanto, algún tanto; Para; Del », `della` « Contracción de la preposición di y el artículo la »; `dei`, `degli`, `delle` none | D11 |
| 12 | Wikidata's lone items (change 27's list) | 99 rows | `don` « D. », `domestico` « Gato », `carcere` « Penal », `registro` « Idioma » | D2f |
| 13 | What no rule reaches | 39 reviewed rows (17 of the 300 commonest, 10 the owner settled, 12 added after the review) | `a` « En (cuando…) », `ci`, `te` « La, lo, te », `cui` « Quien », `posto` « En », `nulla` « Ninguna cosa, ni flores, naranjas », `subito`, `solo` « Único », `bene` « Aristocrático », `the` « Té » | D12 |

**Each rule alone**, against the committed rows (rows changed among the ranks 1–1,000 / 1,001–5,000 /
5,001–20,000 / beyond, rows whose first sense changes, lemmas gained or lost; the coverage moves only
where named); *rejected* variants in italics, each read as the criteria say:

| Rule | Rows changed | First sense | Gained / lost | Read (better / worse) | Examples |
|---|---|---|---|---|---|
| D2, agreement, the pivot's tables counted, Wikidata's lone items aside | 1,485 (105 / 312 / 597 / 471) | 1,485 | — | top 1,000: about 70 better, 5 worse — `voglia` « Lunar, deseo… », `ognuno` « Cada, todo, cada una », `entrambi` « Ambas », `bravo` « Bravo, gallardo… » (a false friend), `presa` « Enchufe… » (loses « captura ») —, none of the 300, each held by a reviewed row (D12) but `ognuno` (task 0.4); samples of 60: 40 / 1 (`riportare` « Ir por… »), 40 / 2 (`pompino`, `esponente` « Superíndice, exponente ») | `caso` « Caso », `accordo` « Acuerdo; Acorde », `andare` « Ir, andar », `rumore` « Ruido; Rumor », `corso` « Curso, transcurso, materia », `periodo`, `unico`, `paese` « País, estado, nación; Pueblo », `ormai` « Ya, todavía; … », `controllo` « Control », `europeo` « Europeo », `commissione` « Comisión… », `spiegare` « Explicar » |
| *D2 counting a source's parts of speech apart* | 690 (73 / 175 / 264 / 178) | 690 | — | 3 of the 300 worse | `male` « Malamente, mal », `piano` « Grave, paroxítono, llano; … », `sotto` « Abajo; Bajo » |
| *D2 with the winner of any part of speech* | 624 (70 / 156 / 229 / 169) | 624 | — | 1 of the 300 worse | `insieme` « Conjunto » (« together » read as « set ») |
| *D2 counting a word's -o/-a and -os/-as forms as one* | 77 rows differ from D2 (9 / 12 / 32 / 24) | 74 | — | fixes `entrambi` « Ambos », but `ragazza`, `amica`, `nonna`, `pratica` go back to their masculine, `prospettiva` « Perspectivo », `avocado` « Palto », `alimenti` « Alimento, comer… » | rejected |
| *D2 without the pivot's tables counted* | 602 (68 / 145 / 226 / 163) | 602 | — | passes, 926 fewer rows, most of them better with the tables counted | leaves `corso`, `periodo`, `unico`, `commissione`, `europeo`, `colpo` |
| D2f, Wikidata's lone items, alone | 99 (1 / 19 / 45 / 34) | 99 | — | 1,001–5,000: 19 read, 0 worse; 5,001–20,000: 45 read, 0 worse | `don` « Don, padre, doña », `domestico` « Doméstico… », `carcere` « Cárcel, prisión, presidio », `registro` « Registro… » |
| *D2f yielding to any next source* | 138 (2 / 22 / 64 / 50) | 138 | — | `web` « Www », `nottata` « Pernoctación », `suonatore` « Trompeta » | — |
| D3a, MarkIT's order for a function word's row | 9 (9 / 0 / 0 / 0) | 9 | — | 8 better or no worse; `uno` opens on a form note (D12 removes the note) | `ancora` « Todavía; Ancla », `perché` « Porque; … », `quando`, `secondo` « Según; Segundo », `su` « Encima de; Sobre… », `proprio` « Muy, realmente; … » |
| D3b, a missing adverb or preposition filled | 2 (2 / 0 / 0 / 0) | 2 | — | both better | `spesso` « A menudo, frecuentemente, con frecuencia; Gordo », `verso` « Hacia; Sentido » |
| *D3b for every missing function part of speech* | 17 (17 / 0 / 0 / 0) | 16 | — | 6 better, 5 worse | `più` « Ya; Más », `dopo` « Detrás de… », `però` « No obstante, ahora… », `loro` « Su, sus, suyo; … » |
| D4, a feminine's own word for its lent masculine | 5 (2 / 2 / 1 / 0) | 5 | — | 5 better | `ragazza` « Chica, muchacha, niña », `pratica` « Práctica », `nonna` « Abuela », `zia` « Tía », `diplomatica` |
| *D4, the pointer lending nothing* | 12 (3 / 4 / 5 / 0) | 12 | `comunicativa` lost | 2 worse in 1,001–20,000 | `porca` « Caballón, camellón », `fanciulla` « Guaina; Jovencito » |
| D5a, words that are not Spanish (168 distinct, 303 items) | 40 (0 / 3 / 22 / 15) | 37 | — | none worse | `evidente` « Evidente », `obbedire` « Acatar », `specializzazione` « Especialización », `emblematico` « Emblemático », `transitorio`, `manubrio` |
| *D5a with the other languages' threshold at 2.5, or Spanish's at 1.5* | 56 / 75 | 52 / 69 | 3 / 6 lost | barred: `ancorato`, `congeniale`, `friulano` lose their only gloss | — |
| D5b, place names as labels | 3 (0 / 2 / 1 / 0) | 3 | — | better | `ladro` « Ladrón, chorizo, chorro », `assunzione` « Empleo » |
| *D5b on a list's first word too* | 12 | 12 | 8 lost | barred | `gesù` « Jesús », `giacomo` « Santiago » |
| D6a, a rare first word, Spanish Zipf under 2.0 | 837 (3 / 45 / 246 / 543) | 837 | — | top 1,000: 3 better; 1,001–5,000: none worse; 5,001–20,000: 3 worse in 60 (`turbare` « Preocupar… », `fulcro` « Toque », `espellere` « Desterrar ») | `rapidamente` « Rápidamente, deprisa, velozmente », `pulizia` « Limpieza… », `cellulare` « Celular », `panchina` « Banco », `soffrire` « Sufrir, padecer, soportar », `tranquillamente` |
| *D6a at 2.5* | 1,365 (6 / 64 / 425 / 870) | 1,365 | — | the review of this proposal read 5,001–20,000: 8 better, 4 equal, 3 worse — under 4:1, so 2.5 fails and 2.0 is the highest threshold that passes | — |
| *D6a at 1.5 (the review's) / 1.0* | 457 / 256 | 457 / 256 | — | pass, fewer better rows | — |
| D6b, the section's heraldic senses | 5 (2 / 3 / 0 / 0) | 0 | — | no worse | `nudo` « Desnudo », `torta` « Pastel (o similar) » |
| D7a, a list of eight items or more cut at three | 11 (4 / 3 / 4 / 0) | 5 | — | no worse | `inoltre` « Además, adicionalmente, allende », `talvolta`, `ergo`, `gratis` |
| *D7a at five items, in the section* | 69 (17 / 20 / 21 / 11) | 40 | — | `sì` loses « sí », `rete` cut inside a description | — |
| D7b, a given name read as its Spanish name | 104 (9 / 36 / 40 / 19) | 103 | — | better | `paolo` « Pablo (nombre de pila) », `maria` « María (nombre de pila) », `giuseppe` « José (nombre de pila) » (« Nombre de varón, equivalente del español José ») |
| D8, a surname's note off a common word another source glosses | 7 (0 / 4 / 1 / 2) | 6 | — | better | `senatore` « Senador », `genovese` « Genovés… », `signorile` « Elegante », `rocca` « Rueca » |
| *D8 for given names, or where nothing else glosses* | 26 / 14 | 26 / 13 | 14 / 7 lost | barred | `francesco`, `luigi`; `esposito`, `sorrentino` |
| D9a, a taxon's name before the meaning | 23 (0 / 8 / 8 / 7) | 22 | — | better | `cavallo` « Caballo », `gatto` « Gato » |
| D9b, repeats | 42 (13 / 15 / 11 / 3) | 3 | — | no worse; 1 expression | `persona` « Persona (ser humano) », `islandese` « Islandés », `grazie` « ¡Gracias! », `salute` « Salud », `ecco` « He aquí », `d'accordo` « De acuerdo, vale » |
| *D9b merging two senses equal once their parentheses are out* | 36 | 2 | — | worse | `firma` loses « Firma (empresa) », `su` « Sobre (un sujeto o tópico) »; `vicolo cieco` loses « Callejón sin salida » |
| D10, verb forms | 10 (2 / 3 / 3 / 2) | 4 | — | no worse | `faccia` « Cara », `dati` « Datos, facto », `salvo` without « Salvarse », `schiacciata` « Mate, machaque, pan árabe » |
| *D10 without its guards* | 54 | 30 | 14 lost | barred | `studio` « Atelier », `tuo` « Tu », `generalizzato`, `ire` lose their gloss |
| D11, the `di` family | 5 (5 / 0 / 0 / 0) | 5 | `dei`, `degli`, `delle` gained | better | `del` « Del », `della` « De la » |
| D12, `FIRST_RUN` (20 rows) and `CORRECTIONS` (19) | 37 (22 / 11 / 3 / 1) | 34 | `the`, `tag`, `trad` lost | better | `bene` « Bien (excelentemente); … », `posto` « Lugar, sitio, puesto », `cui` « Que; Cuyo » |
| *A function word's pivot words of one table out* | 8 | 8 | 5 lost | barred; `ci`, `te`, `ti` untouched | `alquanto`, `noialtri` lose their gloss |

**Together**: **2,085 rows change** (149 / 394 / 780 / 762), **2,031 in their first sense** (137 / 377 /
763 / 754); `dei`, `degli` and `delle` gain a gloss; `the`, `tag` and `trad` lose a wrong one (« Té »,
« Chapa », « Maletero »: task 0.2); 2 expressions change (`d'accordo` « De acuerdo, vale », `cumino
tedesco` « Alcaravea »). Coverage **85.2 / 76.1 / 65.1 %**, as now to one decimal, against the floor
**83.2 / 74.0 / 63.1**. Prototype tables: `gloss.tsv` `82922ca8…`, `senses.tsv` `0fb7c20c…`, `mwe.tsv`
`f0965ef2…` (re-measured at implementation).

**The target, on the review's judged rows** (its tally, the same rows read again on the prototype's
tables, by this design's reader):

| Band | Wrong before | Wrong after | Partly right before | Partly right after |
|---|---|---|---|---|
| The 300 commonest (292 glossed before, 294 after) | 25, **8.6 %** | 0, **0 %** | 33, 11.3 % | 10, **3.4 %** (`da`, `vi`, `oltre`, `pure`, `seguito`, `piano`, `strada`, `conto`, `tratta`, `avanti`) |
| 301–5,000 (the review's sample) | 7 | 1 (`mark` « Mark »); `tag` unglossed | 13 | no worse |
| 5,001–20,000 (the review's sample) | 2 | 1 (`frodo` « Fraude »); `trad` unglossed | 10 | no worse |

Of the 25 wrong rows of the 300, rules fix 13 — `ancora` (D3a), `caso`, `qualche`, `andare`,
`accordo`, `corso`, `periodo`, `unico`, `ormai` (D2), `verso`, `spesso` (D3b), `qualsiasi` (D6a),
`ragazza` (D4, its « Novia » added by a reviewed row) —; the reviewed tables the other twelve — `del`,
`della` (D11), `a`, `ci`, `solo`, `cui`, `bene`, `te`, `contro`, `posto`, `nulla`, `subito` (D12) — and
D11 glosses the three unglossed `di` forms. The target (at most 3 %) is met on these rows as this
reader reads them; the specialist's reading of the gloss diff confirms it or names what misses (task
4.1).

## Decisions

### D1 — Every rule is it-es's, in its two modules; MarkIT joins it-es's pin

The rules extend `reduce_italian_es.py` (the section's rules, the table rules, the row rules and the
reviewed tables) and `reduce-it-es.py` (the order of the passes, each source's gloss of every lemma kept
for D2, the expressions' repeats). D3 calls `reduce_french_treebank.commonest_first` over
`reduce_italian_treebank.markit_pos_counts`, imported as written, as it-fr and change 23's it-en import
them: both join it-es's rule digest, and editing either would re-pin fr-en, fr-es, it-fr, it-en and
it-es together. `reduce_common.py`, `reduce_edition_es.py`, `reduce_italian_acronyms.py` and
`reduce_spanish_tables.py` are not edited, so en-es, fr-es, es-en, fr-en, it-fr and it-en keep their
digests, tables and packs (`test_reduce_editions.py`).

**One file outside it-es's two modules is edited**: `pack_sources.py`, whose `PINNED["it-es"]` gains
`ITALIAN_MARKIT`, the two sections it-fr's pin names — as change 23 adds them to `PINNED["it-en"]` and
fr-es's refinement added UD French-GSD to `PINNED["fr-es"]`. It is in no pair's rule digest. **it-es's
`pin.json` gains the two `sources` entries by hand in the same commit**, it-fr's url and sha256
(`4f6fc342…`, `633b6186…`): `fetch_pinned` raises `PinError` on a `PINNED` source the pin does not
record. The checks that name it-es's sources follow in that commit: `test_pack_sources.py` and
`committed_tables.rs`'s list of it-es's source keys. wordfreq's English, French, Portuguese, Italian and
German lists (D5) are part of the pinned wordfreq 3.1.1, already in it-es's pin: no source is added for
them.

### D2 — A lone first word yields to the word its sources agree on

it-es reads seven sources; `native_side` lets the first that glosses a lemma gloss it whole, so a word
one table gives (`caso` « Azar », the Spanish tables read backwards) hides the word four others give.

- **a. Support.** A Spanish word's support for a lemma is the number of D2's sources whose gloss of the
  lemma holds it — items split at « , » and at « o »/« u » between short items, parentheses out, case
  folded —, **each of the pivot's tables counting as a source** (the pivot already orders its words so,
  change 27's D3). *Rejected — the pivot as one source*: 926 rows fewer, most of them better with the
  tables counted (`corso`, `periodo`, `unico`, `commissione`, `europeo`, `colpo` « Golpe »).
- **b. When a row yields.** When the word most sources open on is opened on by at least two sources,
  by more than open on the row's first word, and is given by more sources than it (`andare`: four open
  on « Ir », one on « Andar »); or when the row's first word is given by its own source alone and
  another word by two or more (`qualche` « Poca » → « Algún »). Ties go to the earlier source in D2's
  order, then to the word's place there.
- **c. Its part of speech.** The second trigger's winner must be a word some source gives in the part
  of speech the row opens on, its support counted across every part of speech (the sources label one
  word differently: the section's possessives are adjectives); the first trigger's is a word two sources
  open on, which the measurement read as it is (*Implementation*: requiring the part of speech of it too
  moves 25 rows, `amica` back to « Amigo »). *Rejected*: support counted by part of speech (`male`
  « Malamente, mal », `piano` « Grave, paroxítono… », `sotto` « Abajo »); a winner of any part of speech
  (`insieme` « Conjunto »).
- **d. Where it goes.** The sense holding the winner opens the row, the winner first in it; a winner the
  row lacks brings the sense of the earliest source holding it, and the row's first sense goes only when
  none of its words another source gives; within eight senses.
- **e. Reviewed rows stay.** A row `FUNCTION_WORDS`, `FIRST_RUN`, `CORRECTIONS` or the `di` family
  writes is never moved by D2, D3 or D6 (`ne`'s « De allí, de ahí »).
- **f. Wikidata's lone items.** A row Wikidata's lexemes of one item gloss (P5137, 20 % wrong in the
  review), whose first word no other source gives, takes the gloss of the next source whose first word
  two sources give (`don` « D. » → « Don », `domestico` « Gato » → « Doméstico »). *Rejected — the
  next source whatever it says*: `web` « Www », `nottata` « Pernoctación ».

**1,485 rows** (105 / 312 / 597 / 471) and 99 by D2f. The worse rows read — in the 1,000 commonest
`voglia` « Lunar, deseo, anhelo » (« voglia » is also a birthmark), `ognuno` « Cada, todo, cada una »,
and, read again on the review of this proposal, `entrambi` « Ambas » (the feminine first), `bravo`
« Bravo, gallardo, valeroso, valiente » (an obsolete sense, a false friend) and `presa` « Enchufe,
tomacorriente, toma » (« captura » lost); past them `riportare` « Ir por… », `pompino`, `esponente`
« Superíndice, exponente » — are within the criteria's ratio, none of the 300. The four of the 1,000
that a source's words restore are reviewed rows (D12: `entrambi` « Ambos », `bravo` « Bueno, bravo,
hábil; … », `presa` « Toma, captura », `voglia` « Gana »); `ognuno` needs « Cada uno », which no source
gives (task 0.4). *Rejected — a word's -o/-a and -os/-as forms counted as one word*: it fixes `entrambi`
but sends `ragazza`, `amica`, `nonna` and `pratica` back to the masculine their pointer lends, and opens
`prospettiva` on « Perspectivo », `avocado` on « Palto » (77 rows).

### D3 — Parts of speech: MarkIT's order, and the missing adverb or preposition

- **a. MarkIT's order**, it-fr's D2 and change 23's D2a as written: fr-en's rule (`ADP`, `DET`, `PRON`,
  `CCONJ`, `SCONJ`, `PART`, `ADV`; at least 10 and twice the row's first; never a proper noun first; no
  content word moved ahead of another), a reviewed row held (D2e: `ne`). **9 rows, all of the 1,000**:
  `ancora` « Todavía; Ancla », `perché` « Porque; Para que; Por qué », `quando` « Cuando; Cuándo »,
  `come`, `su` « Encima de; Sobre… », `secondo` « Según; Segundo », `proprio` « Muy, realmente; … »,
  `quale`, and `uno`, which opens on « Forma de un antes de «s impura»… », a note the reviewed table
  removes (D12).
- **b. A missing adverb or preposition.** A row holding only content parts of speech, for a word
  MarkIT reads at least ten times and twice as often as an adverb or a preposition, gains the first
  sense of that part of speech from the next source, when two sources give its first word, ahead of
  the row's own senses: `spesso` « A menudo, frecuentemente, con frecuencia; Gordo », `verso` « Hacia;
  Sentido ». **2 rows.** *Rejected — any missing function part of speech*: 17 rows, 5 worse, where a
  source tags the word otherwise than UD does (`più` « Ya; Más », `dopo` « Detrás de, en pos de; … »,
  `però` « No obstante, ahora… », `loro` « Su, sus, suyo; … », against it-fr's and it-en's row);
  determiners the Spanish edition writes as adjectives (`suo`, `ogni`) excluded.

### D4 — A feminine's own word for its lent masculine

The Spanish section heads many feminine nouns « Forma del femenino singular de X », which the shared
rules follow to X's gloss: `ragazza` « Chico, muchacho, cabro; Novio, pololo ». For a word Italian's
readings make a noun, whose section entries hold no meaning of their own, the lent gloss gives way to
the next source's, **when its first word is the lent word's feminine** (« chico » → « chica »,
« abuelo » → « abuela », « tío » → « tía ») and two sources give it. **5 rows**: `ragazza` « Chica,
muchacha, niña », `pratica` « Práctica », `nonna` « Abuela », `zia` « Tía », `diplomatica`. `amica`
« Amiga; Amigo » and `attrice` « Actriz; Actor » come from D2. *Rejected — the pointer lending nothing*:
`porca` « Caballón, camellón », `fanciulla` « Guaina; Jovencito », and `comunicativa` loses its gloss.
**With every rule on, D2 still drops `porca`'s lent sense**: its first word, « cerdo », is the
section's alone, so the row opens on the direct table's « Caballón, camellón » — a worse row, *porca*
being mostly the sow, « cerda », as in *porca miseria*. The pivot gives « cerda », so a reviewed row
restores it from a source's words (D12: « Cerda; Caballón, camellón »). `fanciulla` (« Moza », the
pivot's) and `alunna` (« Alumna, alumno, escolar », the pivot's), whose lent masculine no rule replaces,
are reviewed rows too; `comunicativa`, which no other source glosses, keeps « Comunicativo ».
fr-en's owner settled that a female equivalent is no word's meaning (its Q3): here the lent word is
replaced by the feminine a source writes, never by a word written for it.

### D5 — Words that are not Spanish, and place names read as words

- **a.** A one-word item of a table-shaped source whose Spanish Zipf frequency is under 1 while its
  English, French, Portuguese, Italian or German one is at least 3 is left out, unless `ITALIAN_ALIKE`
  names it: **168 distinct words, 303 items, 40 rows**, none worse — `evidente` « Evidente », `obbedire` « Acatar »,
  `specializzazione` « Especialización », `battere` without « battre », `emblematico`, `transitorio`,
  `manubrio`, `trascurabile` without « negligible ». *Rejected*: the other languages' threshold at 2.5,
  or Spanish's at 1.5 — `ancorato`, `sessa`, `congeniale`, `friulano` lose their only gloss.
- **b.** A capitalised word that follows another in the Italian Wiktionary's or the Spanish tables'
  list, and that Italian's tables keep as a name (a lemma with no reading), is a label, not a word:
  `ladro` and `rapinatore` « Ladrón, chorizo, chorro » (« Spagna » out), `assunzione` « Empleo ».
  **17 words, 3 rows.** *Rejected — on a list's first word too*: `gesù` « Jesús », `giacomo`
  « Santiago » lose their gloss.
- The section's definitions are not read so: `svezia` « Suécia » is left (D13).

### D6 — Rare and heraldic first words

- **a.** A row whose first word's Spanish Zipf frequency is under **2.0** opens on the first commoner
  word of its first sense, or else on the first sense of its part of speech of the next source whose
  first word is commoner: **837 rows** (3 / 45 / 246 / 543) — `rapidamente` « Rápidamente, deprisa,
  velozmente », `pulizia` « Limpieza, pulcritud, higiene », `cellulare` « Celular », `panchina`
  « Banco », `controllo` « Control », `qualsiasi` « Cualquier, posible, cualquiera », `soffrire`
  « Sufrir, padecer, soportar », `autista` « Conductor, autista, chófer ». 2.0 kept over the review's
  1.5 (457 rows) and 1.0 (256), each passing, and over 2.5 (1,365 rows), which fails the 4:1 ratio in 5,001–20,000 (8 better, 4 equal, 3 worse, the review's reading), as the criteria keep the passing variant with the most
  better rows; its three worse rows read (`turbare` « Preocupar, conmocionar, confundir », `fulcro`
  « Toque », `espellere` « Desterrar ») are past rank 5,000.
- **b.** A sense the section files under heraldry goes: 8 senses, 5 rows, no first sense moved
  (`nudo` « Desnudo », `torta` « Pastel (o similar) »). Heraldic words a table gives (`volto`
  « Contornado », which two sources give) are a reviewed row (D12, task 0.2).
- Regional words (`cabro`, `pololo`) are not a class a rule reads: they go with the rows that carry them
  (`ragazza`, D4).

### D7 — Long lists and given names

- **a.** A sense of at least eight items, three quarters of them of at most four words and none ending a
  sentence, keeps its first three: **11 rows** — `inoltre` « Además, adicionalmente, allende »,
  `talvolta`, `gratis`, `ergo`, `dunque`, `abilità`. *Rejected — five items, in the section*: `sì`
  loses « sí », `rete` is cut inside a description. Wikidata's Spanish glosses of Italian senses, which
  change 27 handed over for a shorter cap, hold two senses over 150 characters among the 20,000
  (`nobel`, `nebulosa`): left, the card pages them (D13).
- **b.** A given name's note « Nombre de pila de varón, equivalente del español Pablo » — or its
  variants « Nombre de varón, equivalente del español José » and « Nombre de pila de varón equivalente
  a Hugo » — reads « Pablo (nombre de pila) »: **127 senses, 104 rows** (`maria`, `francesco`,
  `giovanni`, `paolo`, `giuseppe` « José (nombre de pila) »). fr-es shows the full note; the short form
  here is the owner's choice (task 0.3).

### D8 — A surname's note off a common word

A proper noun's « Apellido… » keeps off a word Italian's readings give a common reading, **where
another source glosses the word**, which then glosses it: **7 rows** — `senatore` « Senador », `rocca`
« Rueca », `genovese` « Genovés, genovese; Genovesa », `signorile` « Elegante », `giordano`. The
review's wording — « when it-en or it-fr has a common sense » — reads another pair's committed tables,
which the criteria bar; Italian's readings are it-es's own input and say the same of these words.
*Rejected*: given names too (`francesco`, `luigi` lose their gloss); a surname with nothing else to say
the word (`esposito`, `sorrentino` would lose their only gloss — it-en's D6 keeps a row of names).
**Change 27's names-only rows**: 128 rows of the 20,000 hold only names' notes on the committed tables
(change 27 counted 136 its own way); after D7b and D8, 48 do — 36 surnames alone (`berlusconi`,
`ferrari`), 12 given names the section gives no equivalent (`riccardo`, `enzo`, `italo`).

### D9 — Taxa and repeats

- **a.** A taxon's name opening a sense goes (« (Equus caballus) Caballo » → « Caballo »): 23 rows.
- **b.** Within a part of speech, a sense equal to another once their parentheses are out goes when it
  has none (`persona` « Persona (ser humano) », `inferno` « Infierno »), and so does a sense with no
  parenthesis whose every item another sense says (`capo` without the second « Jefe »); an item
  repeated in a sense goes (`islandese` « Islandés »); a sense with no parenthesis equal to an earlier
  sense of any part of speech, once « ¡ », « ! », « ¿ » and « ? » are out too, goes (`grazie` « ¡Gracias! »,
  `salute` « Salud », `ecco` « He aquí »); an expression's sense that only opens another goes (`d'accordo`
  « De acuerdo, vale »). **42 rows, 1 expression.** *Left*: a sense contained in a sense of another part
  of speech (`quale` « Cuál; Cuál, qué; Qué; Como », `tenente` « Teniente, tenante; Tenante »), which the
  sources label differently and a rule cannot merge without merging real distinctions. *Rejected*: merging two senses that
  differ by their parentheses (`firma` « Firma (leyenda); Firma (empresa) », `su`), and an expression's
  sense contained in another (`vicolo cieco` would keep only its figurative sense).

### D10 — Verb forms inside other rows

- **a.** A translation the Italian Wiktionary files under a verb form's sense (« 1ª persona singolare
  del congiuntivo presente di fare »), which no other source gives the word, is left out: `faccia`
  « Cara ». 85 translations.
- **b.** The pivot's verb sense for a word Italian reads only as another verb's form and as no verb of
  its own, unless it is the row's only part of speech: `dati` « Datos, facto », `salvo` without
  « Salvarse ». **10 rows together**, none worse. *Rejected*: (a) without its guard (`studio` « Atelier »,
  `tuo` « Tu », `vostro` « Tu; Vuestro »: a possessive's « seconda persona singolare » and a verb form's
  « estudio » the noun shares); (b) for every word with no verb reading (`sodo` loses « Duro »,
  `fantoccio`; two better for one worse in 5,001–20,000) or from every table (`generalizzato`,
  `unificato` lose their gloss). `efficiente` « …; Efecto, producir » (a Polish translation split into
  its words) is left (D13).

### D11 — The `di` family, a reviewed table

`DI_FAMILY`, keyed by headword, glosses the contractions of `di` and the article whatever the sources
give, each row with its reason; the reduction fails on a row whose headword is no ranked lemma:

| Row | Gloss | Words from |
|---|---|---|
| `del` | « Del » | the pivot (`del`'s third sense) |
| `dello` | « Del » | the pivot, as committed: kept so the family reads alike |
| `della` | « De la » | **written by a person**: the section describes it (« Contracción de la preposición di y el artículo la »), no source glosses it |
| `dei` | « De los » | **written by a person**: unglossed since change 27's split-translation filter |
| `degli` | « De los » | **written by a person**: idem |
| `delle` | « De las » | **written by a person**: idem |

As it-fr's `DI_FAMILY` reads the family through its link, and change 27's `ne` « De allí, de ahí »:
each row a person wrote is marked so in the code, and approved by the owner (task 0.1).

### D12 — `FIRST_RUN` and `CORRECTIONS`

**`FIRST_RUN`** (it-es's shape): for a row the rules open on a wrong word, the source and the sense the
row opens on — a sense of a source D2 reads for the word, never a word written for it —, and whether
the row's other senses stay; the reduction fails on a row whose sense the source no longer writes.
**20 rows**: 11 of the 300 commonest; 4 the owner settled (task 0.2); 5 *added after the review of this
proposal, same class as 0.2* (a word a source already gives).

| Row | Opens on | Source | Others | Reason |
|---|---|---|---|---|
| `bene` | « Bien (excelentemente) » | the section | kept | the adjective « Aristocrático, noble » first; MarkIT reads too few adverbs (8) |
| `solo` | « Sólo, solamente, únicamente » | the pivot | kept (« Único ») | the adverb's agreed word is the pivot's alone |
| `contro` | « Contra » | the pivot | dropped | « Versus, en vez de, en contra; …; Por, contrario, conta » |
| `posto` | « Lugar, sitio, puesto » | the pivot | dropped | « En », a slip two sources share (the Italian Wiktionary's translations and its Spanish section) |
| `nulla` | « Nada, vacío, nihilidad » | the pivot | dropped | « Ninguna cosa, ni flores, naranjas », a translation split into its words |
| `subito` | « Inmediatamente, enseguida, ahora mismo » | the pivot | dropped | « Súbito », a false friend (*subito* is also *subire*'s participle) |
| `ci` | « Nos, nosotros, ahí » | the pivot | dropped | « …; De esto, aquí, por allí; Ce; No, hay » |
| `che` | « Que » | the section | kept | « Qué » first; the relative and the conjunction are the common uses |
| `sotto` | « Bajo, debajo de, so » | the pivot | kept | D2 opens it on the adverb « Abajo »; the preposition is MarkIT's (10 against 1) but no row sense of it |
| `ora` | « Ahora » | the Spanish tables read backwards | kept (« Hora ») | the noun « Hora » alone; MarkIT reads 6 adverbs, too few |
| `ma` | « Pero, mas, sino » | the pivot | dropped | « Mas o pero; Indicador emocional o enfático » |
| `stipendio` | « Sueldo » | the Italian Wiktionary's translations | dropped | « Estipendio », a false friend — 0.2, the owner's |
| `volto` | « Rostro, faz, cara » | the pivot | dropped | « Contornado », a heraldic term two sources give — 0.2 |
| `decina` | « Decena » | the Spanish section | dropped | « Década » (Wikidata's item and the pivot) — 0.2 |
| `aderenza` | « Adhesión, adherencia, viscosidad » | the pivot | dropped | « Tractor », a Wikidata item two sources share — 0.2 |
| `entrambi` | « Ambos » | the Spanish tables read backwards | dropped | D2 opens it on the pivot's feminine « Ambas » — *added after the review* |
| `bravo` | « Bueno, bravo, hábil » | the pivot | kept | D2 opens it on the obsolete « Bravo, gallardo, valeroso, valiente », a false friend — *added after the review* |
| `voglia` | « Gana » | the Spanish tables read backwards | kept | D2 opens it on « Lunar », the birthmark — *added after the review* |
| `costume` | « Costumbre, traje, disfraz » | the pivot | dropped | « Costumbre » alone, the review's false friend — *added after the review* |
| `alunna` | « Alumna, alumno, escolar » | the pivot | dropped | the lent masculine « Alumno » (D4 needs two sources) — *added after the review* |

**`CORRECTIONS`** (it-fr's and change 23's shape: the source, the headword, its part of speech, the
source's text, the corrected text or none, the reason, the page reported upstream; a correction firing
on nothing is named by the update's summary line). **19 rows**: 6 of the 300 commonest, 6 the owner
settled (task 0.2), 7 *added after the review of this proposal, same class as 0.2*; every word a
source's but the three marked written by a person:

| Row | Source text | Corrected | Reason |
|---|---|---|---|
| `a` | the section's « En (cuando se trata de un concepto más que un lugar físico). » | « A; En (…) » | **« A » written by a person**: no source glosses a one-letter word |
| `te` | the pivot's « La, lo, te » | « Ti, te » | **« ti » written by a person**: the stressed pronoun after a preposition (« per te ») |
| `ti` | the pivot's « Te, ti, la » | « Te, ti » | « la », a clitic table's word |
| `cui` | the Spanish tables' « Quien » | « Que; Cuyo » | the relative's words, the Spanish section's and the pivot's |
| `uno` | the section's « Forma de un antes de «s impura»… » | none | a note on the form, which D3a would open the row on |
| `the` | the pivot's « Té » (Russian « чай ») | none | the English article on Italian pages; it-en leaves `the` unglossed too (change 23's D9) |
| `tasso` | the direct table's « Tejón, tejo » | « Tasa; Tejón, tejo » | the rate first, the pivot's « tasa » — 0.2 |
| `svezia` | the section's « Suécia » | « Suecia » | **written by a person**: the Portuguese spelling — 0.2 |
| `salsa` | the pivot's « Salsa, greve, gravy » | « Salsa » | « greve », « gravy », not Spanish — 0.2 |
| `efficiente` | the pivot's « Efecto, producir » | none | a Polish translation split into its words — 0.2 |
| `tag` | the pivot's « Chapa » | none (unglossed) | no source gives « etiqueta » — 0.2 |
| `trad` | the Spanish tables' « Maletero » | none (unglossed) | another word's translation — 0.2 |
| `porca` | the section's lent « (Sus scrofa domesticus) cerdo… » | « Cerda; Caballón, camellón » | D2 drops the lent sense for « Caballón »; the pivot's « cerda » — *added after the review* |
| `qualunque` | the pivot's « Quequier » | none | not a Spanish word — *added after the review* |
| `appunto` | the pivot's « …; Exactamente, exacto, école » | « Exactamente, exacto; Broma, reto, chiste » | « école », French; the adverb first (MarkIT: 8 adverbs) — *added after the review* |
| `stop` | Wikidata's « Fin » | « Alto, pare » | the pivot's words, its « pelota corta » out — *added after the review* |
| `fanciulla` | the section's lent « Chico, muchacho… » | « Moza » | the pivot's feminine — *added after the review* |
| `ragazza` | — | « Chica, muchacha, niña; Novia » | D4's row with the pivot's « novia » (English and German tables) — *added after the review* |
| `presa` | the pivot's « Enchufe, tomacorriente, toma » | « Toma, captura » | D2 drops the section's « Captura » — *added after the review* |

Hand-written glosses, all of this change: `a`'s « A », `te`'s « ti », `svezia`'s « Suecia », and D11's
four rows — seven, the owner's (tasks 0.1, 0.2). Three wrong rows needed a word no source gives and went
back to the owner, who approved them (task 0.4, 2026-10-25): three more `CORRECTIONS` rows, each written
by a person — `tessera` « Tesela » → « Tarjeta, carné » (the card), `cancellato` « Celosiado » →
« Cancelado, borrado », `ognuno` « Cada, todo, cada una » → « Cada uno »: ten in all, 22 rows of
`CORRECTIONS`.

### D13 — Measured and left

- **The partly right rows of the 300** (10): `da` « De, a, cerca; … », `vi` « Os, los, las; Aquí, por
  allí, para allá », `oltre` « Además, más », `pure` « Hasta », `seguito`, `piano` « Llano, plano; … »
  (it-en's row opens it on the floor), `strada` « Camino », `conto` « Cálculo », `tratta`, `avanti`: no
  rule reaches them without moving right rows, and each opens on a right word or a near one.
- **Wrong rows a person's word fixes**: `tessera`, `cancellato`, `ognuno` — task 0.4, approved:
  `CORRECTIONS` (D12).
- **Left wrong past the 300**: `mark` « Mark », `frodo` « Fraude » (*frodo* is mostly « di frodo »,
  contraband): no source gives a better word.
- **A function word's pivot words given by one table**: measured as a rule and rejected (`alquanto`,
  `noialtri`, `siffatto` lose their gloss; `ci`, `te`, `ti` untouched, their wrong words coming from many
  tables).
- **Repeats across parts of speech** (`quale`, `tenente`): D9's *Left*.
- **Wikidata's long glosses** (`nobel` 276 characters, `nebulosa` 151): 2 rows of the 20,000.
- **The pivot's homographs, participles and last words** (`sparato` « Bragueta », `restringere`
  « Restringir, reducir, eliminar », `soddisfatto` « …, qué bueno »): a rule cannot tell them from a
  right word; `sino` and `braccia` are no ranked lemmas on the committed tables.
- **The Spanish edition's gaps**, fr-es's and en-es's too: a follow-up of that edition.

### D14 — it-es re-reduced at its pin; what moves and what cannot

One dispatch (`lingua-pack-update`, mode `reduce`, it-es), started by the programme's coordinator on
the implementation branch, from the pinned release `lingua-pack-sources-it-es-2026.10.25` and MarkIT's
two sections, which it-es's pin records from the commit that adds them (D1). The pin keeps its
snapshot and its sources byte for byte; its rule digest moves (`reduce-it-es.py`, `reduce_italian_es.py`,
and the two treebank modules joining it), with `pack_version`, the pack and the layer; the base is
it-fr's. `gloss_coverage.py --pair it-es` passes the floor: predicted 85.2 / 76.1 / 65.1.

**What may move, and nothing else**: `tables/it-es/gloss.tsv`, `senses.tsv`, `mwe.tsv`,
`manifest.json`, `pin.json`, `README.md`; `SOURCES.md`; `it-es.golden` and `word-card-it-es.txt` (D15).
**What does not**: `tables/it/`, the eight other pairs' tables, pins, packs, goldens and snapshots; the
shared modules; the extension's source; Italian's analyser version.

### D15 — The golden and the card, read probe by probe

The update workflow re-blesses `it-es.golden` and `word-card-it-es.txt` itself in its bless step
(`lingua-pack-update.yml`: `LINGUA_BLESS=1 cargo test … --test it_es_baseline` and the cards'
`yarn vitest run … test/word-card-it-es.spec.ts … -u`), so nothing is blessed a second time: the pull
request lists every moved probe and line with its rule, for the owner's approval. The studied side
still equals it-fr's golden (`the_golden_is_the_italian_one_on_the_studied_side`); `cross_native`
passes unblessed.

### D16 — Order

| Change | Relation |
|---|---|
| 27 `add-lingua-pack-it-es` | before: its first requirement is MODIFIED here, so this change archives after it |
| 23 `refine-lingua-it-en-glosses` | either side: both add MarkIT to a pair's `PINNED` entry and edit `test_pack_sources.py`; whichever merges second rebases those lines |
| 29 `measure-lingua-it-es-translation-marks` | either side: its tier reads the engine's column alone |
| 30 `enable-lingua-italian-for-spanish-speakers` | after: it lists it-es once this change has merged; its `archiveAfter` already names this change |

## Settled by the owner (2026-10-25)

Answered by Guillaume on 2026-10-25, each as recommended (tasks 0.1–0.4):
- **Q1, six glosses a person writes** (D11, D12): (a), all six — `della` « De la », `dei` and `degli`
  « De los », `delle` « De las », `a` « A » before the section's sense, `te` « Ti, te » —, each marked
  written by a person and reported to the Wiktionary.
- **Q2, the isolated errors past the 300 commonest** (D12): (a), the ten rows the question listed (it
  said eleven) — `stipendio`
  « Sueldo », `tasso` « Tasa; … », `volto` « Rostro… », `decina` « Decena », `aderenza` « Adhesión… »
  opening on a source's word; `salsa` « Salsa », `efficiente` « Eficiente, eficaz, efectivo », `tag` and
  `trad` without their wrong word; `svezia` « Suecia », written by a person. The review of this
  proposal then found twelve rows of the same class — a source's word, or a wrong word removed —, added
  under that answer and marked so (D12: `entrambi`, `bravo`, `voglia`, `costume`, `alunna`, `porca`,
  `qualunque`, `appunto`, `stop`, `fanciulla`, `ragazza`'s « Novia », `presa`), read by the owner in the
  gloss diff.
- **Q3, given names** (D7b): the short form « Pablo (nombre de pila) » is kept, although fr-es shows the
  full note (« Nombre de pila de varón, equivalente del español Adolfo »): the difference between the
  two pairs is the owner's choice; the variants without « de pila » read the same way.

- **Q4, three words no source gives** (D12, task 0.4): (a) — `tessera` « Tarjeta, carné »,
  `cancellato` « Cancelado, borrado », `ognuno` « Cada uno », three `CORRECTIONS` rows, each written by a
  person and marked so, as the seven above.

Decided here, so not asked: every rule and its variant (the criteria), the precedents (fr-en's Q2, Q3,
Q5; it-en's D6 on names), the 15 reviewed rows of the 300 whose words a source writes (all of D12's
rows of the 300 but `a` and `te`).

## Risks / Trade-offs

- **[Agreement moves many rows]** → 1,485 rows, read as the criteria say; the specialist reads every
  changed row of the 5,000 in the gloss diff before the owner approves it (task 4.1).
- **[The pivot's tables counted as sources]** → they are the pivot's own order (change 27's D3); a word
  many tables give for a function word can still be wrong (`ci`, `te`): those rows are reviewed.
- **[wordfreq's thresholds]** → measured in variants; a word left out by D5 or moved by D6 is listed in
  the pull request among the changed rows.
- **[Hand-written glosses]** → ten, marked in the code and the pull request, approved by the owner
  (tasks 0.1, 0.2, 0.4).
- **[A pin gains a source]** → MarkIT's two files are it-fr's, at the same commit and sha256s, read for
  counts alone.

## Migration Plan

No reader holds an it-es pack before change 30: nothing to migrate. The tables arrive by the dispatch
(D14); rollback is a revert of the tables, the pin and the golden.

## Effort

3.75–5.25 ideal days, against the programme's 2–3.5 (49b): the rules D2–D10 in `reduce_italian_es.py`
with their tests 1.5–2 (agreement and its pivot counts the largest); `FIRST_RUN` (20 rows), `CORRECTIONS`
(19), `DI_FAMILY`, MarkIT's pin entries and the checks naming it-es's sources 0.75–1; the dispatch, the
README, `SOURCES.md` and the measurements 0.75–1; the golden's and card's listing and the specialist's
review 0.5–1; the owner's reading 0.25. Eleven classes of the review, against change 23's seven, and the
agreement across seven sources account for the difference.

## Implementation

Measured on 2026-10-25 by the reduction itself — `build.sh --reduce it-es` over it-es's pinned sources
(the release `lingua-pack-sources-it-es-2026.10.25`, fetched again into a scratch folder, and MarkIT at
`e210079d`), Python 3.12 and `requirements-reduce.txt` —, whose tables the dispatch of task 3.1 is to
write byte for byte:

- **Together**: the figures of *Measured*, byte for byte the prototype's tables (`gloss.tsv` `82922ca8…`,
  `senses.tsv` `0fb7c20c…`, `mwe.tsv` `f0965ef2…`) but for the three rows of task 0.4 — 2,085 rows (149 /
  394 / 780 / 762), 2,031 in their first sense — and those three on top: **2,087 rows** (149 / 396 / 780 /
  762), **2,033 in their first sense** (137 / 379 / 763 / 754); `dei`, `degli` and `delle` gained, `the`,
  `tag` and `trad` lost; 2 expressions (`d'accordo` « De acuerdo, vale », `cumino tedesco` « Alcaravea »);
  coverage **85.2 / 76.1 / 65.1 %**, as before, against the floor 83.2 / 74.0 / 63.1. `gloss.tsv`
  `69824f66…`, `senses.tsv` `0fb7c20c…`, `mwe.tsv` `f0965ef2…`. The 545 changed rows of the 5,000 commonest
  are listed in the pull request, each with the rules that change it alone.
- **Each rule alone** (the pre-pass and that rule, `RULES`), as *Measured* gives it: D4 5 rows (2 / 2 / 1 /
  0), D2f 99 (1 / 19 / 45 / 34), D9a 23, D6a 837 (3 / 45 / 246 / 543), D6b 5 (no first sense), D3a 9, D3b
  2, D7a 11, D7b 104 (103 first senses), D8 7, D9b 42 and `d'accordo`, D5a 40 (37) and `cumino tedesco`,
  D5b 3, D10 10 (4). **D2 alone moves 1,483 rows (104 / 311 / 597 / 471), not 1,485**: the prototype's
  run of D2 alone predates its last reading of « ¡ » and « ! », and moved `grazie` « Gracias; ¡Gracias! »
  and `buongiorno`, which D9b's repeats are; together the count is the prototype's. The reviewed tables
  alone (D11, D12): 45 rows (28 / 13 / 3 / 1), 42 in their first sense — *Measured*'s 42 and task 0.4's
  three (`ognuno`, rank 847; `cancellato`, 2,744; `tessera`, 4,591).
- **D2c, as implemented**: the part of speech binds the second trigger's winner (a first word its own
  source alone gives), as the prototype measured it; the first trigger's winner is a word two sources open
  the row on. Requiring the part of speech of it too moves 25 rows (6 / 9 / 7 / 3), most of them worse —
  `amica` back to « Amigo », `bambino` « Infantil (conducta); Niño », `bilancia` « Libra; … », `bagno`
  « Cuarto de baño » —: kept as measured, the requirement's wording made exact.
- **The review's judged rows**: read again on these tables, the rows the review judged are the
  prototype's, word for word — task 0.4's three are none of them: of the 300 commonest, **0 wrong** (from
  25, 8.6 %) and **10 partly right, 3.4 %** (from 33, 11.3 %), as this design's reader reads them;
  `mark` and `frodo` left wrong past the 300, `tag` and `trad` unglossed.
- **The reduction's report**: 8 heraldic senses, 7 surnames' notes, 127 given names' notes read, 85 verb
  forms' translations, 53 of the pivot's verb words, 303 words that are not Spanish and 17 places' labels
  out; rows moved, in order, 5 by D4, 99 by D2f, 23 by D9a, 809 by D6a, 1,392 by D2, 2 by D3b, 7 by D3a,
  11 by D7a, 66 by D9b (a row counted under each rule that moves it); 20 of 20 rows of `FIRST_RUN`, 22 of
  22 of `CORRECTIONS`, 6 of 6 of `DI_FAMILY` fired; 11 of 11 function words.
- **The pack**: 1,897,366 B (3,170 B more than change 27's), its base it-fr's, byte for byte (1,551,196 B),
  its layer 346,410 B, at `pack_version` `2026.10.25+3c957aa.442fd70`; only it-es's tables, pin and
  manifest move. The reduction of every pair from its pin, as the extension check's `reduce` job runs it,
  moves it-es's alone.
- **The golden and the card**, as the update workflow's bless step re-blesses them (D15), listed in the
  pull request for the owner (task 4.1): `it-es.golden` moves in 75 of its 239 probes (its `pack` line and
  the glosses of 82 lemmas: `andare` « Ir, andar », `ancora` « Todavía; De nuevo; Ancla », `del` « Del »,
  `a` « A; En (…) », `subito` « Inmediatamente… », `the` unglossed, …), `word-card-it-es.txt` in 101 lines;
  no other baseline moves.
- **How it is built**: the three passes are `reduce_italian_es.refine_section` (after `read_section`),
  `refine_tables` with `without_verb_forms_translations` (the Italian Wiktionary's translations read again
  after D10a) and `refine_rows` (after `native_side`, on each source's gloss of a lemma,
  `source_glosses`), each rule named in `RULES` so the reduction can run any of them alone;
  `FIRST_RUN`, `CORRECTIONS`, `DI_FAMILY` and `BY_A_PERSON` live in `reduce_italian_es.py`. A
  `CORRECTIONS` row is keyed by its source's sense, as the pinned file writes it, and writes the row whole
  (the « corrected » text of D12 is the row as the correction leaves it); one whose source no longer
  writes its sense is named on the report line, and so is a `FIRST_RUN` row the reduction never reaches.
  The update's sample names a row of the `di` family's table `DI_FAMILY`. `committed_tables.rs` reads the
  pin's sources sorted (`serde_json`), so it holds their names, and their order from the pin's text.
