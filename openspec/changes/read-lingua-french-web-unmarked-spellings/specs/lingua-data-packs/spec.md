## ADDED Requirements

### Requirement: The web's French spellings without their marks read as the words a reviewed list names
fr-en's reducer SHALL read a word the web writes without its accents, cedilla or diaeresis as another French word only when the owner's reviewed list names that spelling, row by row, beside the four rows change 43b's table holds; no rule SHALL read a spelling by its marks alone. A row SHALL name a spelling to which the English Wiktionary's French section gives no meaning of its own — no sense that is neither a form nor a spelling of another word, as a headword, as a form of such a word or as an inflection such a word's entry lists — and which it does not give only as a spelling of its accented word, and the reduction SHALL fail otherwise, so that a word of its own or a spelling the dictionary gives (the dictionary's own unmarked spellings) is never a row of this list. The list SHALL hold the spellings the owner approved — among them `tres` → *très*, `deja` → *déjà*, `meme` → *même*, `apres` → *après*, `francais` and `francaise` → *français*, `facon` → *façon*, `garcon` → *garçon*, `recu` → *recevoir*, `ecole` → *école*, `etais` and `ete` → *être*, `tete` → *tête* — and SHALL never hold `ou`, `a`, `la`, `sur`, `du`, `des`, `cote`, `tache`, `pale`, `foret` or `aine`, nor `video`, `hotel` or `these` unless the owner names them. A spelling the list names SHALL be a form of the named word alone, as change 43b's rows are: no ranked lemma, French dictionary word, levelled word or word a pair glosses. Every spelling of the table, change 43b's rows included, SHALL carry the readings of its word's forms it writes without their marks, each kept as its own tag, and none when those forms have none. Whatever its case, a spelling SHALL read as its word, as French's analysis looks every token up lowercased. fr-es SHALL read the spellings through the `tables/fr/` fr-en writes, French's levels SHALL not move, and en-fr's, es-fr's, es-en's and en-es's tables, pins, packs and invariance baselines SHALL not move.

#### Scenario: « tres » reads as « très »
- **WHEN** fr-en is reduced from its pinned sources with the reviewed list
- **THEN** `tables/fr/forms.tsv` maps `tres` to *très*, `tres` is in none of `freq.tsv`, `lexical.tsv`, `level.tsv` and fr-en's `gloss.tsv`, and *très* keeps its rank and its level

#### Scenario: The owner's sentence
- **WHEN** a reader studying French through fr-en selects « c'etait deja tres bien »
- **THEN** `deja` reads as *déjà* « already » and `tres` as *très*, the selection meets the expression `très bien`, and `etait` reads as the tables read it before this change

#### Scenario: A spelling carries its word's readings
- **WHEN** the card asks the grammar of `arrete` as *arrêter* and of `recu` as *recevoir*
- **THEN** `arrete` answers the readings of `arrête` and of `arrêté` together — the past participle, the present indicative and subjunctive of the first and third persons singular, and the imperative of the second person singular —, and `recu` the masculine singular past participle

#### Scenario: Change 43b's rows read alike
- **WHEN** the card asks the grammar of `ages` as *âge*
- **THEN** it answers the masculine plural noun that `âges` carries

#### Scenario: A capital without its accent
- **WHEN** a French page writes « L'Ecole est fermée. »
- **THEN** `Ecole` reads as *école* « school », not as a word of its own and not as a name

#### Scenario: The dictionary names it, so no row does
- **WHEN** a row of the list names a spelling the French section holds as a headword or lists as an inflection — `etait`, « obsolete spelling of était », or `cote`, « rating »
- **THEN** the reduction fails, naming the row

#### Scenario: A spelling the dictionary names only as another spelling's form
- **WHEN** the section lists `tete` only as an inflection of `teter` « alternative form of téter », and the list holds `tete` → *tête*
- **THEN** the reduction passes, `tete` maps to *tête* and carries its feminine singular noun reading, and is no form of `teter`

#### Scenario: A word the list does not name
- **WHEN** fr-en is reduced
- **THEN** `ou` maps to *ou*, `a` to *avoir*, `la` to *le*, `sur` to *sur* and `cote` to *cote*, and `video`, `hotel` and `these` read as they did before this change

#### Scenario: The levels do not move
- **WHEN** the change's tables are committed
- **THEN** `tables/fr/level.tsv` is byte for byte as before, since no spelling of the list was a levelled word

#### Scenario: fr-es follows French's tables
- **WHEN** fr-es is reduced again on the `tables/fr/` fr-en writes
- **THEN** no spelling of the list is a lemma fr-es glosses, a lemma entering the cut may gain its gloss, every other gloss is byte for byte as before, and its pin records the new studied tables and its pack

#### Scenario: Nothing else moves
- **WHEN** the change's tables are committed
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, and their invariance baselines pass without re-blessing
