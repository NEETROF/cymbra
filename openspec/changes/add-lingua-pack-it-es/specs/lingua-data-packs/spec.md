# Spec Delta

## ADDED Requirements

### Requirement: Italian is glossed in Spanish from the Spanish Wiktionary's Italian section, the two Wiktionaries' tables, Wikidata and a pivot
it-es's glosses, sense runs and expressions SHALL be reduced by a reducer of its own, reading Italian's studied tables as committed and writing none of it, with the Spanish Wiktionary's rules, from these sources in this order: the Spanish Wiktionary's Italian section's definitions; the Italian Wiktionary's Spanish translations; the Spanish Wiktionary's Italian translations read backwards; Wikidata's lexeme translation links and Spanish glosses of Italian senses, then its lexemes of one item; the Italian Wiktionary's Spanish section read backwards; then the pivot of seven Wiktionaries' translation tables — at most three words per part of speech from each table-shaped source, a pivot's Spanish words ordered by the number of tables and editions giving them, then by frequency, then alphabetically. The pivot SHALL read a table's group named by a sense or a sense number, or the whole entry of a one-sense entry, never a « to sort » table; SHALL leave out entries tagged name, abbreviation, symbol, character, letter, affix or punctuation, one-letter headwords, capitalised headwords but German nouns, capitalised Italian or Spanish words but acronyms of at most six letters, and Spanish words with a grave accent, « ° » or more than sixty characters; and SHALL supply no expression. Before the shared rules, it-es's own pre-pass SHALL keep an acronym's sense and a name's note off a word with a lowercase entry, and a name's note off a word the Italian or the Spanish Wiktionary's translation table glosses; SHALL let a pointer that names a word lend that word's gloss only for a headword with no meaning and no pointer the edition tags of its own, and read a meaning a pointer carries after a colon only for a headword with no meaning of its own; SHALL take out sense links, superscripts and subscripts, and open a gloss with a capital letter; SHALL read the commonest function words from a reviewed list; and, for every table-shaped source, SHALL take no letter, no Spanish word listed under two parts of speech of one Italian word — listed once, under the first of them Italian's readings name, or else the first listed —, no word with a grave accent or « ° », and no Italian word given as Spanish unless a reviewed list names it a Spanish word too. it-es's expressions SHALL be the section's glossed headwords with a space, then those the two Wiktionaries' tables gloss, then the words without a space that Italian's pre-pass splits, each keyed at build as *An Italian pack keys its expressions as Italian is read* requires; the acronym rows SHALL be glossed as *Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms* requires. Every gloss SHALL be Spanish written by a person.

#### Scenario: A direct source first
- **WHEN** the Spanish Wiktionary's Italian section defines `casa` and the pivot also gives it Spanish words
- **THEN** `casa` is glossed by the section's definition first, the pivot's words only where no earlier source gives any

#### Scenario: A pivot ordered by support
- **WHEN** an Italian lemma no earlier source glosses has, in one part of speech of the pivot, a Spanish word five tables give and another one table gives
- **THEN** in that part of speech, the word the five tables give comes first

#### Scenario: A name kept off
- **WHEN** a pivot table's entry is a name, or its Spanish word is capitalised and not an acronym of at most six letters
- **THEN** it glosses nothing

#### Scenario: A pointer lends its word's gloss
- **WHEN** the section's `aver` has only the sense « Variante de avere » and no meaning or tagged pointer of its own
- **THEN** `aver` is glossed by `avere`'s gloss, while `sono`, whose senses the edition tags as forms of `essere`, does not borrow `suono`'s « Sonido » through its noun's « Variante de suono »

#### Scenario: A function word's description replaced
- **WHEN** the section describes `tu` as « Pronombre personal nominativo … »
- **THEN** its gloss is the reviewed list's « Tú »

#### Scenario: Nothing else moves
- **WHEN** it-es's tables are committed
- **THEN** every other pair's tables, pins, packs and goldens are unchanged

### Requirement: it-es is committed beside Italian's studied tables, held to a coverage floor, and not shipped
it-es's native side SHALL be reduced from its own pin and committed in `tables/it-es/`, reading `tables/it/` as committed and writing none of it; its pin SHALL record the studied tables it read, and its notice and manifest SHALL credit every source of both sides, Wikidata's CC0 among them. Its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, and SHALL stay under the size budget. Its coverage of the 5,000, 10,000 and 20,000 commonest Italian lemmas SHALL be measured at each reduction, by source, and SHALL NOT fall under `FLOORS["it-es"]`, fixed before the first committed measurement at the study's figures on the committed ranks less two points, settled by the owner, and never lowered after it: a first measurement under the floor SHALL leave the tables uncommitted, and a later reduction under it SHALL fail the checks. An it-es invariance baseline SHALL run over the Italian corpus wherever the Italian ones run, the it-fr, it-en and it-es packs reading every Italian word alike. No package SHALL list it-es before Italian is enabled for Spanish speakers.

#### Scenario: The floor holds
- **WHEN** it-es is reduced and its coverage is at or above `FLOORS["it-es"]`
- **THEN** the tables are committed and the coverage, by source, is shown in their README

#### Scenario: Under the floor
- **WHEN** a pull request re-reduces it-es and its committed tables measure under `FLOORS["it-es"]`
- **THEN** the checks fail — the gloss-coverage floor step of the extension check's `reduce` job —, naming the pair, the top and the figure

#### Scenario: Not lowered after measuring
- **WHEN** a pull request lowers `FLOORS["it-es"]` after the first committed measurement
- **THEN** the test holding the floor to the figures the requirement states fails

#### Scenario: The sample marks its sources
- **WHEN** the README shows its sample of glosses
- **THEN** every row names its source, and a pivot row its edition, entry and sense

#### Scenario: The credits
- **WHEN** it-es's tables are committed
- **THEN** its notice names the Spanish and Italian Wiktionaries, the seven Wiktionaries of the pivot, Wikidata under CC0, and every source of Italian's studied tables

#### Scenario: One studied side, three natives
- **WHEN** the Italian corpus is read with the it-fr, it-en and it-es packs
- **THEN** every token, lemma, reading and level is the same, and only the glosses differ

#### Scenario: Not shipped
- **WHEN** it-es's tables are committed and the list of shipped pairs does not name it-es
- **THEN** no package carries it-es, and no reader's package or engine moves

## MODIFIED Requirements

### Requirement: A gloss is in the reader's language, written by a person
A pack's glosses SHALL be written by a person in the pack's native language: never in the studied language, never in a third language, never machine-translated, and never pivoted through a third language but as it-es's exception below. A word a person wrote into a Wiktionary translation table SHALL qualify when the table pairs the studied and the native language directly: the native words a studied entry lists, or the native entry whose table lists the studied word. For it-es alone (the Italian programme's I11), an Italian and a Spanish word a person wrote into senses of the same Wikidata item SHALL qualify after the Spanish Wiktionary's Italian section and both Wiktionaries' Italian–Spanish translation tables, and an Italian and a Spanish word a person wrote under the same sense of another Wiktionary's translation table — or the same entry of one sense when the table names none — after every other source; such a gloss SHALL be marked by its source wherever the pack's glosses are shown by source.

#### Scenario: An English gloss is not a French one
- **WHEN** neither the French Wiktionary nor a French–Spanish translation table glosses a Spanish lemma that the English Wiktionary's Spanish section glosses in English
- **THEN** the es-fr pack gives that lemma no gloss

#### Scenario: No pivot through English
- **WHEN** one sense of the English Wiktionary's `house` lists French *maison* and Spanish *casa* among its translations
- **THEN** neither glosses the other in any pack

#### Scenario: A translation table qualifies
- **WHEN** a sense of the English Wiktionary's English entry `house` lists Spanish *casa* among its translations
- **THEN** a pack studying English and glossed in Spanish may gloss `house` « casa »

#### Scenario: Nothing written, no gloss
- **WHEN** no person-written source glosses a lemma in the pack's native language
- **THEN** the lemma has no gloss, and no translation engine fills it

#### Scenario: A pivot for it-es
- **WHEN** no source pairing Italian and Spanish directly glosses `acquirente`, and the English Wiktionary's `buyer`, under its sense « person who makes purchases », lists Italian *acquirente* and Spanish *comprador*
- **THEN** the it-es pack may gloss `acquirente` « Comprador », and no other pack glosses through that table

### Requirement: French is glossed in Spanish from the Spanish Wiktionary's French section and the French Wiktionary's translation tables
fr-es's glosses, sense runs and expressions SHALL be reduced from the Spanish Wiktionary's French section with the Spanish edition's rules, then, for what that section leaves out, from the Spanish translations the French Wiktionary's French entries list, in the table's order, then from the Spanish entries of the Spanish Wiktionary whose translations list the French word, the commonest Spanish word first — words people wrote in Spanish, with no pivot and no machine translation —, and fr-es SHALL read French's committed studied tables as a pair that is not its studied language's reference does, loading no other pair's reducer.
Before the shared rules read the section, its letters are left out; its senses are read as the
Spanish edition reads en-es's — the senses it marks obsolete or outdated after the others of their
entry, one typography —; the sense of a capitalised proper noun that only says a word is a surname or
a given name glosses no word that has an entry of its own in lower case, other than a proper noun's,
holding a meaning; and an adjective section the edition tags possessive or demonstrative, or whose
every sense is a form of such a word, is read as a determiner. A typographic apostrophe in a French
headword, or in a French word of either table, is read as `'`. A definition whose every sense is the
French headword itself, as written up to case, SHALL give way to the French Wiktionary's Spanish
translations of the word when they hold no word spelled as the headword. In both tables a letter
SHALL gloss no word, and a Spanish word that several parts of speech of one Spanish entry list for a
French word SHALL be listed once, under the first of them French's readings name, or else the first
listed. A pack built from fr-es's tables carries French's dictionary words, fr-en's glossed lemmas:
a lemma fr-es alone glosses is none. The share of the glossed lemmas a definition glosses — of all
of them, and of the 10,000 commonest — SHALL be measured and shown with the tables, and stored in no
pack. The Spanish edition's rules SHALL be read by en-es, fr-es and it-es and by no other committed pair.

#### Scenario: A definition first
- **WHEN** the Spanish Wiktionary's French section defines `maison` « Casa. »
- **THEN** the gloss of `maison` is « Casa », as a noun

#### Scenario: A direct-table gloss
- **WHEN** the section has no entry for `intérêt`, and the French Wiktionary's `intérêt` lists « interés » among its Spanish translations
- **THEN** the gloss of `intérêt` is « Interés », at most three Spanish words per part of speech

#### Scenario: An inverted-table gloss
- **WHEN** neither the section nor the French Wiktionary's table glosses `travers`, and the Spanish Wiktionary's `través` lists `travers` as its French translation
- **THEN** the gloss of `travers` is « Través »

#### Scenario: The studied word is no gloss
- **WHEN** the section defines the conjunction `et` « Et. » and the French Wiktionary translates `et` « y » and « e »
- **THEN** the gloss of `et` is « Y, e »

#### Scenario: A cognate keeps its definition
- **WHEN** the section defines `venir` « Venir. » and the French Wiktionary's table lists « venir »
- **THEN** the gloss of `venir` is « Venir »

#### Scenario: A name on a common word's card
- **WHEN** the section has `pierre`, a noun defined « Piedra. », and `Pierre`, a proper noun defined « Nombre de pila de varón, equivalente del español Pedro. »
- **THEN** the gloss of `pierre` is « Piedra »

#### Scenario: A name's own row
- **WHEN** the section has `François`, a proper noun defined « Nombre de pila de varón, equivalente del español Francisco », and no `françois` in lower case
- **THEN** the gloss of `françois` is that note, as before

#### Scenario: Possessives and their forms
- **WHEN** the section has `mon`, an adjective section tagged possessive defined « Mi. », and `mes`, an adjective section whose every sense is a form of `mon`
- **THEN** `mon` and `mes` are glossed « Mi » as determiners

#### Scenario: A letter glosses no word
- **WHEN** French's tables rank `h` and `x` among the 500 commonest lemmas, and the only Spanish word a translation table gives each is the letter itself
- **THEN** `h` and `x` have no gloss in fr-es, and `à` and `y` keep the section's definitions

#### Scenario: A Spanish word listed once
- **WHEN** the Spanish Wiktionary's `este` lists `cet` among its French translations as an adjective and as a pronoun
- **THEN** the gloss of `cet` is « Este », once

#### Scenario: A typographic apostrophe
- **WHEN** the French Wiktionary's `main-d’œuvre` lists « mano de obra » among its Spanish translations
- **THEN** the committed lemma `main-d'œuvre` is glossed « Mano de obra »

#### Scenario: French's dictionary words
- **WHEN** fr-es's pack is built from `tables/fr/` and `tables/fr-es/`
- **THEN** it carries a lexical table listing exactly the lemmas `tables/fr/lexical.tsv` lists, and a lemma fr-es glosses and fr-en does not (`quant` in the prototype) is no dictionary word

#### Scenario: The pack is built, not shipped
- **WHEN** a pull request runs the extension's checks while the list of shipped pairs does not name fr-es
- **THEN** fr-es's pack is built from `tables/fr/` and `tables/fr-es/`, has the sha256 its pin records and weighs less than 5 MiB, and no package carries it

#### Scenario: A rule of the Spanish edition
- **WHEN** `reduce_edition_es.py` changes
- **THEN** en-es's, fr-es's and it-es's rule digests move, and en-fr's, es-fr's, es-en's, fr-en's, it-fr's and it-en's do not

#### Scenario: Nothing else moves
- **WHEN** fr-es's native side is committed
- **THEN** en-fr's, es-fr's, es-en's, en-es's and fr-en's tables, pins and packs and every table of `tables/fr/` are byte for byte as before, and the invariance baselines and the extension's snapshots pass without re-blessing
