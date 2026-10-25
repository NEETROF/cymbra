# The it→es dictionary tables

The reduced tables Cymbra Lingua's Italian→Spanish data pack is built from: Italian glossed in
Spanish, for Spanish speakers studying Italian — Italian's thin reader pair (add-lingua-pack-it-es,
change 27 of `docs/lingua/italian-programme.md`). They are committed so that every build makes the same
pack with no download, and so that a change to the dictionary is a pull request whose diff shows it.
No extension package carries the pack: `apps/lingua-extension/packs.json` does not list it-es until
Italian is enabled for Spanish speakers (change 30), after its refinement (change 28,
`refine-lingua-it-es-glosses`).

This folder holds what belongs to it-es alone: its Spanish glosses, its expressions, the parts of
speech of their senses, its notice, manifest and pin. Italian's own tables — its forms, ranks, levels,
readings, tag pool, dictionary words and acronym rows — are kept once, in `../it/`, and written by
it-fr's reduction alone (`../it/studied.json` names it-fr, Italian's reference pair). it-es's reduction
reads them as committed and computes nothing of them.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → Spanish gloss | in this order, each for what the ones before leave out: the Spanish Wiktionary's Italian entries; the Spanish translations the Italian Wiktionary's Italian entries list; the Spanish Wiktionary's Spanish entries listing the Italian word, read backwards; Wikidata's lexemes; the Italian Wiktionary's Spanish entries, read backwards; the pivot of seven Wiktionaries' translation tables (kaikki, CC BY-SA 4.0 + GFDL; Wikidata, CC0 1.0) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | the same; a noun's gender is not here, the builder reads it from `../it/grammar.tsv`'s readings |
| `mwe.tsv` | expression → Spanish gloss | the section and the two Wiktionaries' tables, for headwords with a space and the words Italian's pre-pass splits (`d'accordo`); never the pivot |
| `NOTICE` | the attribution stack, embedded in the pack: both sides' sources, Wikidata's CC0 among them | — |
| `manifest.json` | the pack's metadata: Italian glossed in Spanish, the Italian analyser's version, `levels_estimated`, every source's licence (`Cc0` for Wikidata), and `pack_version` | — |
| `pin.json` | it-es's raw sources, the pack these tables build with `../it/`, its base and layer, the rules that reduced them, and the sha256 of each of `../it/`'s six tables they were built on (`studied`) | — |

## What is in them

<!-- Measured on the tables of the update of 2026-10-25 (`lingua-pack-update` run 40210737711, the
release `lingua-pack-sources-it-es-2026.10.25`), byte for byte the implementation's forecast. -->

- **Spanish glosses for 24,705 lemmas** of Italian's 60,000 (41.2 %) — 3,973 from the Spanish
  Wiktionary's Italian section, 3,582 from the Italian Wiktionary's translations, 2,996 from the Spanish
  tables read backwards, 752 from Wikidata (14 by its translation links and glosses, 738 by its items),
  1,148 from the Italian Wiktionary's Spanish section read backwards, 12,253 from the pivot — and `USA`,
  glossed through its synonyms (`America`, `Stati Uniti`); and **1,165 expressions**: 198 headwords with
  a space from the section, 963 from the two tables, 4 words Italian's pre-pass splits.
- The share of the commonest lemmas glossed after each source, which the `reduce` job holds to the
  floor the owner settled before the first committed measurement (`gloss_coverage.py --pair it-es`,
  `FLOORS["it-es"]`, the study's figures on change 9's committed ranks less two points):

  | Lemmas | definitions | + Italian Wiktionary | + Spanish tables backwards | + Wikidata | + Spanish section | **+ the pivot** | the floor |
  |---|---|---|---|---|---|---|---|
  | top 5,000 | 31.7 % | 47.8 % | 59.2 % | 61.3 % | 66.8 % | **85.2 %** | 83.2 % |
  | top 10,000 | 22.6 % | 36.5 % | 47.2 % | 49.4 % | 53.8 % | **76.1 %** | 74.0 % |
  | top 20,000 | 15.1 % | 26.4 % | 35.2 % | 37.1 % | 40.7 % | **65.1 %** | 63.1 % |

  Of the 7,606 lemmas glossed among the 10,000 commonest, 70.3 % come from a source
  after the definitions. The pivot's lemmas by edition — each counted under every edition a table of
  which gives a Spanish word its gloss shows: English 8,515, German 4,738, French 4,058, Russian 3,402,
  Polish 3,360, Portuguese 1,595, Dutch 649.
- **Dictionary words are it-fr's**: 3,289 lemmas it-es glosses are no dictionary word of Italian, and
  2,851 dictionary words have no Spanish gloss, so the pack carries a lexical section, and the vocabulary
  estimate counts the same words as it-fr's.
- **The pack is 1,894,196 B**, 36.1 % of the 5 MiB budget: Italian's base, 1,551,196 B, byte for byte
  it-fr's, and it-es's layer, 343,240 B.

## Its sources

All derived at an update from the editions' dumps (`../../SOURCES.md`, *IT → ES*), pinned by it-es's own
fetch and published under its own release (`lingua-pack-sources-it-es-<snapshot>`): the Spanish
Wiktionary's Italian section and Spanish entries' Italian translations; the Italian Wiktionary's Spanish
translations and Spanish section; Wikidata's Italian–Spanish lexeme pairs; and the Italian and Spanish
words the English, French, German, Portuguese, Russian, Polish and Dutch Wiktionaries' tables list
under one sense. **The pivot is the exception the programme's I11 makes for it-es**: each word is one a
person wrote, their pairing inferred from a shared sense, read last and marked by its source; no
machine translation.

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

## A sample of 100 glosses, marked by source

The reduction draws them from the glossed lemmas of the 20,000 commonest with `random.Random(20261024)`
(`measures.json`, which the update's report shows). « definitions » is the Spanish Wiktionary's Italian
section, « direct » the Italian Wiktionary's translations, « inverted » the Spanish tables read
backwards, « spanish section » the Italian Wiktionary's Spanish section read backwards; a pivot row names
the edition, the entry, its part of speech and the sense of the table that first gives its first word.
Long glosses are cut here.

| Rank | Lemma | Source | Gloss |
|---:|---|---|---|
| 50 | `ne` | definitions | De allí, de ahí; De él, de ella, de ellos, de ellas |
| 217 | `genere` | definitions | Género |
| 316 | `dietro` | direct | Detrás, detrás de |
| 563 | `lista` | inverted | Lista |
| 620 | `finito` | inverted | Acabado |
| 738 | `insomma` | pivot (de: also (adv), « ein eine Schlussfolgerung einleitendes Wort, folglich ») | Así, con eso, por fin; Así así |
| 820 | `attacco` | direct | Ataque |
| 943 | `collaborazione` | definitions | Colaboración |
| 966 | `attesa` | pivot (en: wait (noun), « a delay ») | Espera, esperanza, retraso |
| 976 | `fiume` | definitions | Río |
| 1038 | `praticamente` | definitions | Prácticamente (en práctica); Prácticamente (casi enteramente); Prácticamente (en una manera práctica); Casi |
| 1056 | `giudice` | definitions | Juez |
| 1279 | `ipotesi` | definitions | Hipótesis |
| 1321 | `venti` | definitions | Veinte |
| 1329 | `chiusura` | spanish section | Clausura |
| 1398 | `proprietario` | direct | Poseedor |
| 1503 | `sanità` | pivot (en: health (noun), « state of being free of physical or psychological disease, illness, or malfunction ») | Sanidad, salud, atención médica |
| 1511 | `alternativa` | direct | Alternativa |
| 1917 | `spiegazione` | inverted | Explicación |
| 2077 | `st` | pivot (de: pst (intj), « häufig imperativisch: Aufforderung, still oder leise zu sein ») | Pss, chito |
| 2384 | `tuttora` | pivot (en: still (adv), « up to a time, as in the preceding time ») | Aún, todavía, seguir |
| 2541 | `spagnola` | spanish section | Española |
| 2903 | `significativo` | spanish section | Considerable, significativo |
| 2957 | `soddisfatto` | pivot (en: satisfied (adj), « in a state of satisfaction ») | Satisfecho, contento, qué bueno |
| 3122 | `aquila` | definitions | Águila |
| 3188 | `estero` | pivot (fr: étranger (adj), « Qui est d'une autre nation, qui vient d’un autre pays ») | Extranjero, forastero, de ultramar; El extranjero |
| 3210 | `sopravvivenza` | direct | Supervivencia, pervivencia |
| 3216 | `turco` | definitions | Turco |
| 3254 | `merce` | pivot (en: ware (noun), « product ») | Mercancía, bien, género |
| 3439 | `shock` | direct | Shock |
| 3563 | `monitoraggio` | pivot (de: Monitoring (noun), « systematisches Beobachten, Kontrollieren von Prozessen ») | Vigilancia, monitoreo, monitorización |
| 3627 | `pubblicamente` | pivot (en: publicly (adv), « in an open and public manner — see also in public ») | Públicamente, en público |
| 4572 | `obbiettivo` | pivot (en: objective (noun), « goal ») | Objetivo |
| 4664 | `spostamento` | wikidata items | Desplazamiento |
| 4876 | `bastone` | definitions | Palo |
| 5084 | `sbagliare` | definitions | Equivocar, errar |
| 5153 | `scoppio` | direct | Estampido, estallido |
| 5208 | `integrato` | pivot (en: built-in (adj), « constructed as a non-detachable part ») | Integrado, empotrado, encastrado; Circuito integrado |
| 5573 | `reclusione` | pivot (de: Haft (noun), « Zustand des Verhaftetseins ») | Arresto, privación de libertad, encarcelamiento |
| 5624 | `fallire` | inverted | Errar |
| 6094 | `infermiere` | wikidata items | Enfermero, enfermera |
| 6143 | `telecomunicazioni` | pivot (pl: telekomunikacja (noun), « sense 1.2 ») | Telecomunicaciones, telecomunicación |
| 6160 | `avorio` | direct | Marfil |
| 6287 | `femminismo` | definitions | Feminismo |
| 6739 | `pronunciare` | pivot (en: pronounce (verb), « to sound out (a word or phrase); to articulate ») | Pronunciar, decir, hablar |
| 6814 | `ingannare` | definitions | Engañar |
| 7011 | `delinquente` | inverted | Delincuente |
| 7147 | `pellegrinaggio` | direct | Peregrinación |
| 7516 | `statistico` | wikidata items | Estadígrafo, estadístico |
| 7541 | `asino` | direct | Asno, burro |
| 8268 | `rovescio` | pivot (en: downpour (noun), « heavy fall of rain ») | Chubasco, chaparrón, aguacero |
| 8379 | `pericolosità` | pivot (en: dangerousness (noun), « state or quality of being dangerous ») | Peligrosidad |
| 8510 | `oggettivamente` | pivot (en: objectively (adv), « in an impartial, objective manner ») | Objetivamente |
| 8555 | `uscente` | direct | Saliendo |
| 8809 | `vodka` | definitions | Vodka, vodca |
| 9219 | `tic` | pivot (en: jerk (noun), « sudden, uncontrolled movement ») | Tic, crispación, espasmo |
| 9832 | `impaziente` | direct | Impaciente |
| 10083 | `doge` | pivot (en: doge (noun), « chief magistrate in the republics of Venice and Genoa ») | Dux, dogo |
| 10226 | `carreggiata` | wikidata items | Hormigón |
| 10591 | `mancino` | inverted | Huele, zurdo |
| 10717 | `grazioso` | definitions | Bello, guapo (adorable); Grácil; Delicioso (deleitable, deleitoso); Gracioso |
| 10811 | `ammasso` | spanish section | Piña |
| 10911 | `periferica` | wikidata items | Periférico |
| 11012 | `cazzone` | pivot (pt: otário (noun), « pessoa que é facil de se enganar ») | Gilipollas, desastre, pendejo |
| 11017 | `congiura` | spanish section | Conjura |
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
| 13187 | `rosetta` | inverted | Roseta |
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
| 16810 | `manubrio` | direct | Sterzo |
| 16888 | `schiavone` | definitions | Apellido |
| 17581 | `spaccata` | pivot (en: split (noun), « manoeuvre, acrobatic feat ("splits") ») | Split, spagat, espagar |
| 17620 | `agorà` | pivot (de: Agora (noun), « griechischer Markt, griechische Volksversammlung, Marktplatz in der Antike ») | Ágora |
| 17697 | `foulard` | pivot (en: neckerchief (noun), « type of scarf ») | Fular, pañuelo, pañuelo de cabeza |
| 17704 | `gioioso` | pivot (en: cheerful (adj), « happy ») | Alegre, gozoso, jovial |
| 18263 | `riconquista` | pivot (en: reconquest (noun), « act or process of conquering again ») | Reconquista |
| 18751 | `quinquennio` | wikidata items | Lustro, quinquenio |
| 19057 | `taco` | wikidata items | Taco |
| 19271 | `provocatore` | inverted | Provocador |
| 19610 | `accomodare` | definitions | Acomodar |

## Known defects, for change 28

`refine-lingua-it-es-glosses`, after these tables: function words the pivot glosses from many forms at
once (`del` « Algo, un tanto, algún tanto; Para; Del », `ci`, `ti`, `te`, `vi`, `contro`); the tables'
wrong rows at the top (`posto` « En », `caso` « Azar », `solo` « Único », `cui` « Quien », `verso`
« Sentido », `corso` « Cambio »); the pivot's homographs and participles and its last words (`sparato`,
`restringere` « …, eliminar »); Wikidata's odd items (`aderenza` « Tractor », `domestico` « Gato »);
names' notes alone (136 rows of the 20,000); `della`'s « Contracción de la preposición… », the Spanish
edition's gap; an expression glossed twice by two sources (`d'accordo` « De acuerdo; De acuerdo, vale »).

## How it changes

An update (`lingua-pack-update`, mode `update`, pair `it-es`) reads today's dumps — about 5.3 GB — and
proposes the folder again on its own branch; a change to it-es's rules or to the Spanish edition's
re-reduces it from its pin (mode `reduce`), as does a change to Italian's studied tables, which it-fr's
update brings along. Every pull request rebuilds the pack from these tables and checks it against
`pin.json`, and the `reduce` job reduces it again from its pinned sources and holds it to its floor.
