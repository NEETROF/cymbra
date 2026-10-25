# The it→es dictionary tables

The reduced tables Cymbra Lingua's Italian→Spanish data pack is built from: Italian glossed in
Spanish, for Spanish speakers studying Italian — Italian's thin reader pair (add-lingua-pack-it-es,
change 27 of `docs/lingua/italian-programme.md`). They are committed so that every build makes the same
pack with no download, and so that a change to the dictionary is a pull request whose diff shows it.
No extension package carries the pack: `apps/lingua-extension/packs.json` does not list it-es until
Italian is enabled for Spanish speakers (change 30). Its glosses are refined by change 28
(`refine-lingua-it-es-glosses`): they open on the Spanish word their sources agree on, in the order
Italian uses the word, in Spanish words read as their sources mean them (*How the sources are read*).

This folder holds what belongs to it-es alone: its Spanish glosses, its expressions, the parts of
speech of their senses, its notice, manifest and pin. Italian's own tables — its forms, ranks, levels,
readings, tag pool, dictionary words and acronym rows — are kept once, in `../it/`, and written by
it-fr's reduction alone (`../it/studied.json` names it-fr, Italian's reference pair). it-es's reduction
reads them as committed and computes nothing of them.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → Spanish gloss | in this order, each for what the ones before leave out: the Spanish Wiktionary's Italian entries; the Spanish translations the Italian Wiktionary's Italian entries list; the Spanish Wiktionary's Spanish entries listing the Italian word, read backwards; Wikidata's lexemes; the Italian Wiktionary's Spanish entries, read backwards; the pivot of seven Wiktionaries' translation tables (kaikki, CC BY-SA 4.0 + GFDL; Wikidata, CC0 1.0) — then each row read again by change 28's rules and reviewed tables |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | the same; a noun's gender is not here, the builder reads it from `../it/grammar.tsv`'s readings |
| `mwe.tsv` | expression → Spanish gloss | the section and the two Wiktionaries' tables, for headwords with a space and the words Italian's pre-pass splits (`d'accordo`); never the pivot |
| `NOTICE` | the attribution stack, embedded in the pack: both sides' sources, Wikidata's CC0 among them | — |
| `manifest.json` | the pack's metadata: Italian glossed in Spanish, the Italian analyser's version, `levels_estimated`, every source's licence (`Cc0` for Wikidata), and `pack_version` | — |
| `pin.json` | it-es's raw sources, the pack these tables build with `../it/`, its base and layer, the rules that reduced them, and the sha256 of each of `../it/`'s six tables they were built on (`studied`) | — |

## What is in them

<!-- Measured on the tables change 28 reduces again at it-es's pin (the release
`lingua-pack-sources-it-es-2026.10.25` and MarkIT at `e210079d`), the implementation's local forecast,
which the dispatched reduction is to match byte for byte. -->

- **Spanish glosses for 24,705 lemmas** of Italian's 60,000 (41.2 %) — 3,967 from the Spanish
  Wiktionary's Italian section, 3,575 from the Italian Wiktionary's translations, 2,998 from the Spanish
  tables read backwards, 754 from Wikidata (14 by its translation links and glosses, 740 by its items),
  1,150 from the Italian Wiktionary's Spanish section read backwards, 12,260 from the pivot, each the
  source that first glosses the word before change 28's row rules read it again — and `USA`,
  glossed through its synonyms (`America`, `Stati Uniti`); and **1,165 expressions**: 198 headwords with
  a space from the section, 963 from the two tables, 4 words Italian's pre-pass splits.
- The share of the commonest lemmas glossed after each source, which the `reduce` job holds to the
  floor the owner settled before the first committed measurement (`gloss_coverage.py --pair it-es`,
  `FLOORS["it-es"]`, the study's figures on change 9's committed ranks less two points):

  | Lemmas | definitions | + Italian Wiktionary | + Spanish tables backwards | + Wikidata | + Spanish section | **+ the pivot** | the floor |
  |---|---|---|---|---|---|---|---|
  | top 5,000 | 31.6 % | 47.7 % | 59.1 % | 61.3 % | 66.7 % | **85.2 %** | 83.2 % |
  | top 10,000 | 22.5 % | 36.4 % | 47.1 % | 49.4 % | 53.8 % | **76.1 %** | 74.0 % |
  | top 20,000 | 15.1 % | 26.4 % | 35.1 % | 37.1 % | 40.7 % | **65.1 %** | 63.1 % |

  Of the 7,606 lemmas glossed among the 10,000 commonest, 70.4 % come from a source
  after the definitions. The pivot's lemmas by edition — each counted under every edition a table of
  which gives a Spanish word its gloss shows: English 8,518, German 4,738, French 4,060, Russian 3,403,
  Polish 3,362, Portuguese 1,595, Dutch 649.
- **Change 28 reads 2,092 rows otherwise** than change 27 wrote them — 154 of the 1,000 commonest,
  396 of the next 4,000, 780 of the next 15,000, 762 past them —, **2,033 in their first sense**
  (191 of round 1's rows read otherwise after the specialist's review, round 2);
  `dei`, `degli` and `delle` gain a gloss, `the`, `tag` and `trad` lose a wrong one; two expressions
  change (`d'accordo` « De acuerdo, vale », `cumino tedesco` « Alcaravea »). The coverage does not move.
- **Dictionary words are it-fr's**: 3,286 lemmas it-es glosses are no dictionary word of Italian, and
  2,848 dictionary words have no Spanish gloss, so the pack carries a lexical section, and the vocabulary
  estimate counts the same words as it-fr's.
- **The pack is 1,896,498 B**, 36.2 % of the 5 MiB budget: Italian's base, 1,551,196 B, byte for byte
  it-fr's, and it-es's layer, 345,542 B.

## Its sources

All derived at an update from the editions' dumps (`../../SOURCES.md`, *IT → ES*), pinned by it-es's own
fetch and published under its own release (`lingua-pack-sources-it-es-<snapshot>`): the Spanish
Wiktionary's Italian section and Spanish entries' Italian translations; the Italian Wiktionary's Spanish
translations and Spanish section; Wikidata's Italian–Spanish lexeme pairs; and the Italian and Spanish
words the English, French, German, Portuguese, Russian, Polish and Dutch Wiktionaries' tables list
under one sense. **The pivot is the exception the programme's I11 makes for it-es**: each word is one a
person wrote, their pairing inferred from a shared sense, read last and marked by its source; no
machine translation. Since change 28, **UD Italian-MarkIT**'s training and development sections too,
pinned at it-fr's commit and sha256s and read for a row's parts of speech alone (CC BY 4.0, which the
notice credits for the studied side already).

## How the sources are read

- **The section**, through the Spanish edition's pre-passes and it-es's own
  (`../../reduce_italian_es.py`): a reviewed list of function words the section describes (`tu` « Tú »,
  `lo` « El; Lo », `essere` without « Verbo auxiliar »; `ne`'s adverb « De allí, de ahí », the one row a
  person wrote); names' notes and acronyms off common words, and a name's note off a word a table glosses
  (`secondo` « Segundo; Según »); a worded pointer lending its word to a headword with no meaning, and a
  pointer's meaning after a colon read for such a headword; sense links, superscripts and stray
  subscripts out, a gloss opening on a capital.
- **Every table-shaped source**, through fr-es's table rules copied into `../../reduce_spanish_tables.py`
  and it-es's own: no letter; no Spanish word with a grave accent or « ° »; no Italian word given as
  Spanish but those Spanish writes alike (`credo`, `friulano`, `nutrimento`, `visivo`); each Spanish
  word listed once across the Italian word's parts of speech; at most three per part of speech.
- **The Italian Wiktionary's Spanish section, read backwards**: the first gloss of a sense, its notes
  to its readers out (« vedi italiano »), none when it opens on a parenthesis, which narrows the sense.
- **The pivot**: a table named by a sense or a sense number, or a one-sense entry whole, never a table
  left to sort; no name, abbreviation, symbol, letter or affix; no capitalised word but an acronym of at
  most six letters; no translation its extract split into its words (the Polish and Russian ones); its
  Spanish words read as the tables write them, ordered by the tables and the editions giving them, then
  by Spanish frequency.

**Change 28's rules** (`refine-lingua-it-es-glosses`, it-es's own, in `../../reduce_italian_es.py`, each
named in its `RULES` and measured alone on change 27's tables — rows changed of the 1,000 commonest, of
the next 4,000, of the next 15,000 and past them):

- **The section**: heraldic senses out (D6b: 8 senses, 5 rows, `nudo` « Desnudo »); a surname's note off
  a word Italian's readings make common, where a table glosses it (D8: 7 rows, `senatore` « Senador »); a
  given name's note read as its Spanish name (D7b: 104 rows, `paolo` « Pablo (nombre de pila) », the
  owner's short form).
- **The tables**: the Italian Wiktionary's translations under a verb form's sense that no other source
  gives (`faccia` « Cara », not « haga, hagas »), and the pivot's verb words for another verb's form
  (`dati` « Datos, facto »): D10, 10 rows; words that are not Spanish, by wordfreq across five languages
  (D5a: 303 items, 40 rows, `evidente` « Evidente », `obbedire` « Acatar »); places' labels (D5b: 17
  items, 3 rows, `ladro` « Ladrón, chorizo, chorro »).
- **The rows**, in this order: a feminine's own word for its lent masculine (D4: 5 rows, `ragazza`
  « Chica, muchacha, niña »); Wikidata's lone items (D2f: 99 rows, `don` « Don, padre, doña »); taxa (D9a:
  23 rows, `cavallo` « Caballo »); a rare first word, Spanish Zipf under 2.0 (D6a: 837 rows, `rapidamente`
  « Rápidamente, deprisa, velozmente »); **agreement** — a row whose first word its own source alone
  gives, or that fewer sources open on than another word, opens on the word its sources agree on, each
  of the pivot's tables a source (D2: 1,483 rows, 104 / 311 / 597 / 471; `caso` « Caso », `andare` « Ir,
  andar », `rumore` « Ruido; Rumor », `corso` « Curso… ») —; the missing adverb (D3b: 2 rows, `spesso`
  « A menudo…; Gordo ») and MarkIT's order for a function word's row (D3a: 9 rows, `ancora` « Todavía; … »,
  `perché` « Porque; … »), fr-en's `commonest_first` over MarkIT's counts, imported as it-fr and it-en
  import them; long lists cut at three (D7a: 11 rows), their items first ordered by the sources giving
  them, then by Spanish frequency (D7c, round 2: `talvolta` « A veces, de vez en cuando, a la de veces »,
  `gratis` without « de bóbilis »); repeats (D9b: 42 rows and `d'accordo`, `persona` « Persona (ser
  humano) »).
- **Round 2**, after the specialist's review of the gloss diff (the owner's, 2026-10-25): a first word
  agreement displaces stays after the lead when it is the cognate of a lemma of a learned suffix
  (`-zione`, `-enza`, `-ità`, `-mente`, `-abile`, …) and common in Spanish (D2g: 9 rows, `insufficienza`
  « …; Insuficiencia », `prestazione` « …; Prestación »); an item that is no Spanish word goes from a sense
  a rule brought into the row, never its last (D5c: 154 rows, `marmo` « Mármol, sepulcro », not « …,
  alioj », `afferrare` without « America ispanica »).
- **The reviewed tables**, which no rule moves: `DI_FAMILY` (D11: `del` « Del », `della` « De la »,
  `dei` and `degli` « De los », `delle` « De las »), `FIRST_RUN` (D12: 20 rows open on a sense a source
  gives, `posto` « Lugar, sitio, puesto », `subito` « Inmediatamente… », `stipendio` « Sueldo ») and
  `CORRECTIONS` (D12: 44 rows, `a` « A; En (…) », `cui` « Que; Cuyo », `porca` « Cerda; Caballón,
  camellón », `the`, `tag` and `trad` unglossed; round 2's 22, every word a source's: `cattura`
  « Captura, detención, arresto », `autista` « Conductor, chófer », `strada` « Calle, carretera, camino »,
  `loro` « Ellos, ellas; Ustedes; Su, sus », …). **Ten glosses are written
  by a person**, marked so in the code (`BY_A_PERSON`) and approved by the owner on 2026-10-25: `della`,
  `dei`, `degli`, `delle`, `a`'s « A », `te`'s « ti », `svezia` « Suecia » (tasks 0.1, 0.2), and
  `tessera` « Tarjeta, carné », `cancellato` « Cancelado, borrado », `ognuno` « Cada uno » (task 0.4);
  with change 27's `ne` « De allí, de ahí », eleven in the pack.

## A sample of 100 glosses, marked by source

The reduction draws them from the glossed lemmas of the 20,000 commonest with `random.Random(20261024)`
(`measures.json`, which the update's report shows). « definitions » is the Spanish Wiktionary's Italian
section, « direct » the Italian Wiktionary's translations, « inverted » the Spanish tables read
backwards, « spanish section » the Italian Wiktionary's Spanish section read backwards; a pivot row names
the edition, the entry, its part of speech and the sense of the table that first gives its first word.
The source is the one that first glossed the word; change 28's rows may open on another source's words
(`bravo`, a `FIRST_RUN` row); a row of the `di` family's table no source glosses reads `DI_FAMILY`.
Long glosses are cut here.

| Rank | Lemma | Source | Gloss |
|---:|---|---|---|
| 47 | `degli` | DI_FAMILY | De los |
| 215 | `trovare` | definitions | Encontrar |
| 314 | `sopra` | pivot (en: above (prep), « in or to a higher place ») | Sobre, encima de, por encima; Súper, pero que muy, sumamente; Sobre algo, alto |
| 561 | `discorso` | direct | Discurso, mensaje, oración |
| 618 | `dunque` | definitions | Entonces (después); Consecuentemente (así); Por consiguiente, por lo tanto, pues |
| 735 | `est` | inverted | Este |
| 818 | `rispondere` | definitions | Responder |
| 941 | `bravo` | definitions | Bueno, bravo, hábil; De buen conducto. (Usado antes de sustantivos.); Ducho, experto, hábil, habilidoso; Obediente; Bravo, gallardo, valeroso, valiente; Fier… |
| 963 | `zero` | direct | Cero |
| 974 | `documento` | direct | Documento |
| 1036 | `perdita` | pivot (en: loss (noun), « the result of no longer possessing something ») | Pérdida, perjuicio, daño |
| 1054 | `freddo` | definitions | Frío |
| 1277 | `gioia` | definitions | Alegría |
| 1319 | `tradizionale` | pivot (en: traditional (adj), « of or relating to tradition ») | Tradicional, popular, normal |
| 1327 | `braccio` | definitions | Brazo |
| 1396 | `pietra` | definitions | Piedra |
| 1501 | `penale` | inverted | Penal |
| 1508 | `turismo` | definitions | Turismo |
| 1915 | `rom` | inverted | Romaní |
| 2074 | `sinceramente` | pivot (en: sincerely (adv), « In a sincere or earnest manner; honestly ») | Sinceramente, francamente |
| 2381 | `suonare` | spanish section | Tocar |
| 2539 | `sms` | pivot (en: text (noun), « a brief written message transmitted between mobile phones ») | Texto |
| 2901 | `sedia` | definitions | Silla |
| 2955 | `simpatico` | spanish section | Simpático |
| 3118 | `uni` | pivot (en: uni (noun), « university ») | Uni |
| 3186 | `dintorni` | direct | Alrededores, cercanías, aledaños |
| 3208 | `sfruttamento` | definitions | Explotación |
| 3215 | `torneo` | inverted | Torneo |
| 3253 | `margherita` | direct | Margarita silvestre |
| 3438 | `sapienza` | definitions | Sapiencia; Sabiduría |
| 3562 | `miele` | definitions | Miel |
| 3626 | `progettazione` | pivot (ru: дизайн (noun), « its one sense ») | Diseño, designio, deseño |
| 4570 | `massacro` | direct | Masacre |
| 4663 | `sposo` | definitions | Esposo, marido, cónyuge |
| 4874 | `arancione` | definitions | Naranja |
| 5083 | `ritirata` | pivot (en: retreat (noun), « act of pulling back or withdrawing ») | Retirada, baño, repliegue |
| 5151 | `salvaguardia` | spanish section | Salvaguardia |
| 5205 | `fuso` | direct | Huso |
| 5572 | `recitare` | definitions | Recitar |
| 5623 | `evo` | pivot (en: age (noun), « particular period of time in history ») | Era, edad, época |
| 6093 | `incrocio` | pivot (en: intersection (noun), « junction of two or more paths, etc ») | Cruce, intersección, crucero |
| 6142 | `strategico` | pivot (en: strategic (adj), « of or pertaining to strategy ») | Estratégico |
| 6159 | `attestato` | direct | Certificación, apuntado |
| 6287 | `femminismo` | definitions | Feminismo |
| 6739 | `pronunciare` | pivot (en: pronounce (verb), « to sound out (a word or phrase); to articulate ») | Pronunciar, decir, hablar |
| 6814 | `ingannare` | definitions | Engañar |
| 7011 | `delinquente` | inverted | Delincuente |
| 7147 | `pellegrinaggio` | direct | Peregrinación |
| 7516 | `statistico` | wikidata items | Estadístico, estadígrafo |
| 7541 | `asino` | direct | Asno, burro |
| 8268 | `rovescio` | pivot (en: downpour (noun), « heavy fall of rain ») | Chubasco, chaparrón, aguacero |
| 8379 | `pericolosità` | pivot (en: dangerousness (noun), « state or quality of being dangerous ») | Peligrosidad |
| 8510 | `oggettivamente` | pivot (en: objectively (adv), « in an impartial, objective manner ») | Objetivamente |
| 8555 | `uscente` | direct | Saliendo |
| 8809 | `vodka` | definitions | Vodka, vodca |
| 9219 | `tic` | pivot (en: jerk (noun), « sudden, uncontrolled movement ») | Tic, crispación, espasmo |
| 9832 | `impaziente` | direct | Impaciente |
| 10083 | `doge` | pivot (en: doge (noun), « chief magistrate in the republics of Venice and Genoa ») | Dux, dogo |
| 10226 | `carreggiata` | wikidata items | Calzada, carril, banda; Plataforma de la vía |
| 10591 | `mancino` | inverted | Zurdo, huele |
| 10717 | `grazioso` | definitions | Lindo, bonito, gracioso; Grácil; Delicioso (deleitable, deleitoso) |
| 10811 | `ammasso` | spanish section | Montón, conjunto, reservas |
| 10911 | `periferica` | wikidata items | Periférico |
| 11012 | `cazzone` | pivot (pt: otário (noun), « pessoa que é facil de se enganar ») | Gilipollas, desastre, pendejo |
| 11017 | `congiura` | spanish section | Complot, urdimbre |
| 11027 | `epocale` | pivot (en: significant (adj), « having noticeable effect ») | Importante, considerable, significativo |
| 11290 | `acconto` | pivot (en: down payment (noun), « payment representing a fraction of the price ») | Entrada, arras, pie |
| 11571 | `provvidenza` | pivot (en: providence (noun), « prudent care and management of resources; thriftiness ») | Providencia |
| 11575 | `resina` | inverted | Resina |
| 11626 | `assolvere` | direct | Absolver |
| 11654 | `ceco` | direct | Checo |
| 11917 | `prurito` | direct | Picazón |
| 12489 | `approdare` | inverted | Abordar |
| 12593 | `malore` | pivot (de: Unwohlsein (noun), « gefühlte körperliche oder psychische Unstimmigkeit ») | Malestar, indisposición |
| 12654 | `suggestivo` | pivot (en: fascinating (adj), « having interesting qualities; captivating; attractive ») | Fascinante, pintoresco, pintoresca |
| 12703 | `celibato` | spanish section | Celibato |
| 13187 | `rosetta` | inverted | Arandela, bollo, bolillo |
| 13315 | `integro` | pivot (ru: целый (adj), « единый ») | Íntegro, intacto, entero |
| 13486 | `cupido` | pivot (de: rangig (adj), « veraltet: stark am materiellen Besitz interessiert; habgierig ») | Codicioso, ansioso, ávido |
| 14192 | `amnesia` | inverted | Amnesia |
| 14384 | `annientare` | pivot (en: annihilate (verb), « to reduce to nothing, to destroy, to eradicate ») | Aniquilar, exterminar, eliminar |
| 14725 | `puntino` | direct | Punto, mota |
| 14931 | `saggistica` | pivot (en: nonfiction (noun), « written works intended to give facts ») | No ficción, no-ficción |
| 15302 | `menare` | pivot (fr: mener (verb), « Conduire (sens général) ») | Llevar, entrar, conducir |
| 15356 | `sigaro` | wikidata items | Cigarro |
| 15447 | `domenicano` | direct | Dominico, dominicano |
| 15660 | `distorto` | pivot (en: skewed (adj), « twisted at an angle ») | Chueco, enviajado |
| 16170 | `pallino` | pivot (en: jack-ball (noun), « ball used in boules ») | Boliche, perdigón, afición |
| 16572 | `incoerenza` | pivot (en: incoherence (noun), « quality of being incoherent ») | Incoherencia |
| 16810 | `manubrio` | pivot (en: dumbbell (noun), « a weight with two disks attached to a short bar ») | Mancuerna, manillar, manubrio |
| 16888 | `schiavone` | definitions | Apellido |
| 17581 | `spaccata` | pivot (en: split (noun), « manoeuvre, acrobatic feat ("splits") ») | Split, spagat, espagar |
| 17620 | `agorà` | pivot (de: Agora (noun), « griechischer Markt, griechische Volksversammlung, Marktplatz in der Antike ») | Ágora |
| 17697 | `foulard` | pivot (en: neckerchief (noun), « type of scarf ») | Pañuelo, fular, pañuelo de cabeza |
| 17704 | `gioioso` | pivot (en: cheerful (adj), « happy ») | Alegre, gozoso, jovial |
| 18263 | `riconquista` | pivot (en: reconquest (noun), « act or process of conquering again ») | Reconquista |
| 18751 | `quinquennio` | wikidata items | Lustro, quinquenio |
| 19057 | `taco` | wikidata items | Taco |
| 19271 | `provocatore` | inverted | Provocador |
| 19610 | `accomodare` | definitions | Sentar, hacer obras, arreglar |

## Known defects

Change 28 (`refine-lingua-it-es-glosses`) answered change 27's list and the specialist review of its
tables (2026-10-25): on the review's judged rows, the 300 commonest read wrong in **0 %** of them, from
8.6 % (25 rows), and partly right in 3.4 %, from 11 % — as the design's reader reads them, for the
specialist to confirm on the gloss diff. What it left, and why:

- **Partly right rows of the 300**: `da` « De, a, cerca; … », `vi` « Os, los, las; Aquí, por allí, para
  allá », `oltre` « Además, más », `pure` « Hasta », `seguito`, `piano` « Llano, plano; … », `strada`
  « Camino », `conto` « Cálculo », `tratta`, `avanti`: no rule reaches them without moving right rows,
  and each opens on a right word or a near one.
- **Left wrong past the 300**: `mark` « Mark », `frodo` « Fraude » (*frodo* is mostly « di frodo »):
  no source gives a better word.
- **The pivot's homographs, participles and last words** (`sparato` « Bragueta », `restringere`
  « Restringir, reducir, eliminar », `soddisfatto` « …, qué bueno »): a rule cannot tell them from a right
  word.
- **Repeats across parts of speech** (`quale` « Cuál; Cuál, qué; Qué; Como »): the sources label one word
  differently, and a rule cannot merge them without merging real distinctions.
- **Names' notes alone**: 47 rows of the 20,000, from 129 — 34 surnames alone (`berlusconi`,
  `esposito`), 13 given names the section gives no Spanish equivalent (`enzo`, `italo`) —; a name alone
  keeps its note (it-en's D6).
- **Wikidata's long glosses** (`nobel`, `nebulosa`): the card pages them.
- **The Spanish edition's gaps** (« Contracción de… », « _(n) »), which fr-es and en-es share: a
  follow-up of that edition, not of it-es's rules.
- **Three words no source gives**, which the specialist asked for: `riportare`'s « traer de vuelta »,
  `richiamare`'s « volver a llamar » and « atraer ». The rows hold the sources' « Devolver, traer;
  Referir » and « Llamar nuevamente; Reprender, amonestar »; the words are the owner's to write or not.
- **A function word's pivot words a table alone gives** (`ci`, `te`, `ti` from many tables): measured as
  a rule and rejected; the commonest are reviewed rows.

## How it changes

An update (`lingua-pack-update`, mode `update`, pair `it-es`) reads today's dumps — about 5.3 GB — and
proposes the folder again on its own branch; a change to it-es's rules or to the Spanish edition's
re-reduces it from its pin (mode `reduce`), as does a change to Italian's studied tables, which it-fr's
update brings along. Every pull request rebuilds the pack from these tables and checks it against
`pin.json`, and the `reduce` job reduces it again from its pinned sources and holds it to its floor.
