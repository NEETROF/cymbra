# Spec Delta

## ADDED Requirements

### Requirement: Spanish is glossed in Italian from the Italian Wiktionary's Spanish section, the two Wiktionaries' tables, Wikidata and a pivot
es-it's glosses, sense runs and expressions SHALL be reduced by a reducer of its own, reading Spanish's studied tables as committed and writing none of it, with the Italian edition's rules and without the Spanish edition's, from these sources in this order: the Italian Wiktionary's Spanish section's definitions; the Spanish Wiktionary's Italian translations; the Italian Wiktionary's Spanish translations read backwards; Wikidata's lexeme translation links, then its lexemes of one item; the Spanish Wiktionary's Italian section read backwards; then the pivot of seven Wiktionaries' translation tables — at most three words per part of speech from each table-shaped source, a pivot's Italian words ordered by the number of tables and editions giving them, then by frequency, then alphabetically. The pivot SHALL read a table's group named by a sense or a sense number, or the whole entry of a one-sense entry, never a « to sort » table; SHALL leave out entries tagged name, abbreviation, symbol, character, letter, affix or punctuation, one-letter headwords, capitalised headwords but German nouns, capitalised Spanish or Italian words but acronyms of at most six letters, and Italian words holding a letter Italian does not write (ñ, á, í, ú, ü, ¿, ¡), « ° » or more than sixty characters; and SHALL supply no expression. Wikidata's lexemes of category letter, abbreviation or affix SHALL gloss nothing, and a one-letter Spanish lemma SHALL take nothing from Wikidata or the pivot but through the reviewed list. es-it SHALL read the commonest function words from a reviewed list first: each entry SHALL name a Spanish lemma and Italian words that one of es-it's sources pairs with it, written by a person — the section, either table, Wikidata, the Spanish Wiktionary's Italian section read backwards or the pivot —, with that source; the list SHALL only choose and order them, and SHALL compose none. In both tables a letter's entry, a one-letter word's noun entry and an entry whose every translation is its own letter SHALL gloss nothing, and, read backwards, no one-letter Spanish word SHALL be glossed; but a one-letter conjunction or preposition whose translation is the same letter (`o` « o ») SHALL be kept. For every table-shaped source, es-it SHALL take no Italian word listed under two parts of speech of one Italian entry — listed once, under the first of them Spanish's readings name, or else the first listed —, no translation under a sense the Italian Wiktionary marks heraldic (« araldica »), and every word in the Italian edition's typography. It SHALL take no Spanish key from the pointer wordings of the Spanish Wiktionary's Italian section read backwards (« Forma del plural de », « Variante de », « Grafía alternativa de »). A definition SHALL NOT give way to a table because it repeats the Spanish headword: Spanish and Italian share the word. es-it's expressions SHALL be the section's glossed headwords with a space, then those the two Wiktionaries' tables gloss, each keyed at build as *A Spanish pack keys its expressions as Spanish is read* requires. Every gloss SHALL be written by a person.

#### Scenario: A direct source first
- **WHEN** the Italian Wiktionary's Spanish section defines `casa` and the pivot also gives it Italian words
- **THEN** `casa` is glossed by the section's definition first, the pivot's words only where no earlier source gives any

#### Scenario: A pivot ordered by support
- **WHEN** a Spanish lemma no earlier source glosses has, in one part of speech of the pivot, an Italian word five tables give and another one table gives
- **THEN** in that part of speech, the word the five tables give comes first

#### Scenario: A name kept off
- **WHEN** a pivot table's entry is a name, or its Italian word is capitalised and not an acronym of at most six letters
- **THEN** it glosses nothing

#### Scenario: A letter, and a one-letter word
- **WHEN** the reviewed list names neither `a` nor `o`, Wikidata or the pivot pairs the Spanish `a` with « ampere », and the Spanish Wiktionary's table translates the conjunction `o` « o »
- **THEN** `a` takes nothing from Wikidata or the pivot, and `o` is glossed « o »

#### Scenario: A one-letter word kept by the direct table
- **WHEN** the Spanish Wiktionary's table translates the conjunction `y` « e, ed »
- **THEN** `y` is glossed « e, ed », its translation being no letter of its own

#### Scenario: Wikidata's letters left out
- **WHEN** a Wikidata lexeme of category letter pairs a Spanish and an Italian word
- **THEN** it glosses nothing

#### Scenario: The reviewed list first
- **WHEN** the reviewed list gives `durante` « durante », a word the Spanish Wiktionary's table pairs with it, and the pivot gives it « autunno »
- **THEN** `durante` is glossed « durante » first, and « autunno » never stands before it

#### Scenario: The reviewed list composes nothing
- **WHEN** a reviewed entry names an Italian word that none of es-it's sources pairs with its Spanish lemma
- **THEN** es-it's tests fail, naming the entry

#### Scenario: A heraldic translation left out
- **WHEN** the Italian Wiktionary's `piegato`, under a sense marked « araldica », lists Spanish `curva`, and the reviewed list does not name `curva`
- **THEN** `curva` is not glossed « piegato »

#### Scenario: A pointer wording is no key
- **WHEN** the Spanish Wiktionary's Italian section defines an Italian form « Forma del plural de libro »
- **THEN** es-it takes no Spanish key « forma del plural de » from it

#### Scenario: A cognate defines its word
- **WHEN** the section defines `alcaloide` « alcaloide » and the Spanish Wiktionary's table lists « alcaloidi »
- **THEN** the gloss of `alcaloide` is the section's « alcaloide »

#### Scenario: Nothing else moves
- **WHEN** es-it's tables are first committed, by its own reducer, with no shared or edition module edited
- **THEN** every other pair's tables, pins, packs and goldens are unchanged

### Requirement: es-it is committed beside Spanish's studied tables, held to a coverage floor, and not shipped
es-it's native side SHALL be reduced from its own pin and committed in `tables/es-it/`, reading `tables/es/` as committed and writing none of it; its pin SHALL record the studied tables it read, and its notice and manifest SHALL credit every source of both sides, Wikidata's CC0 among them. Its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, and SHALL stay under the size budget. Its coverage of the 5,000, 10,000 and 20,000 commonest Spanish lemmas SHALL be measured at each reduction and SHALL NOT fall under `FLOORS["es-it"]`, fixed before the first committed measurement at the measured figures on the committed ranks less two points, settled by the owner, and never lowered after it: a first measurement under the floor SHALL leave the tables uncommitted, and a later reduction under it SHALL fail the checks. An es-it invariance baseline SHALL run over the Spanish corpus with the committed es-it pack wherever the Spanish ones run, and the extension's tests SHALL render es-it's card in Italian into a committed snapshot re-blessed with its golden. No package SHALL list es-it before Spanish is enabled for Italian speakers.

#### Scenario: The floor holds
- **WHEN** es-it is reduced and its coverage is at or above `FLOORS["es-it"]`
- **THEN** the tables are committed and the coverage, by source, is shown in their README

#### Scenario: Under the floor
- **WHEN** a pull request re-reduces es-it and its committed tables measure under `FLOORS["es-it"]`
- **THEN** the checks fail — the gloss-coverage floor step of the extension check's `reduce` job —, naming the pair, the top and the figure

#### Scenario: Not lowered after measuring
- **WHEN** a pull request lowers `FLOORS["es-it"]` after the first committed measurement
- **THEN** the test holding the floor to the figures the requirement states fails

#### Scenario: The sample marks its sources
- **WHEN** the README shows its sample of glosses
- **THEN** every row names its source, and a pivot row its edition, entry and sense

#### Scenario: The credits
- **WHEN** es-it's tables are committed
- **THEN** its notice names the Italian and Spanish Wiktionaries, the seven Wiktionaries of the pivot, Wikidata under CC0, and every source of Spanish's studied tables

#### Scenario: One studied side, three natives
- **WHEN** the Spanish corpus is read with the es-fr, es-en and es-it packs
- **THEN** every token, lemma, reading and level is the same, and only the glosses differ

#### Scenario: Not shipped
- **WHEN** es-it's tables are committed and the list of shipped pairs does not name es-it
- **THEN** no package carries es-it, and no reader's package or engine moves

## MODIFIED Requirements

### Requirement: English is glossed in Italian from the Italian Wiktionary's English section and the two Wiktionaries' translation tables
en-it's glosses SHALL be reduced from the Italian Wiktionary's English section with the Italian edition's rules, then, for what that section leaves out, from the English Wiktionary's Italian translation tables, then from the Italian Wiktionary's English translations read backwards — words people wrote in Italian, at most three per part of speech from each table, with no pivot and no machine translation —, reading the committed English studied tables as a reader pair does and loading no other pair's reducer. No translation table SHALL gloss a letter: a letter's entry, a one-letter word's noun entry and an entry whose every translation is its own letter are left out of both tables, and, read backwards, every one-letter English word. In the direct table a word labelled disused SHALL NOT be listed, and a translator's note inside a word — a sense number, a loanword's respelling, a label holding no word wordfreq's Italian list rates at 1.0 Zipf or more, a usage note opening on « with » — SHALL be taken out and the word kept. Read backwards, an Italian word SHALL be listed once per English word, under the first of its parts of speech English's readings name, or else the first listed, the words ordered by Italian frequency. Every table word SHALL be written in the Italian edition's typography, for words and expressions alike. Its expressions SHALL be keyed as en-es's are. The share of glossed lemmas among the 10,000 commonest whose gloss came from a translation table SHALL be measured and shown. The Italian edition's rules SHALL be read by en-it, fr-it and es-it and by no other committed pair.

#### Scenario: A definition first
- **WHEN** the Italian Wiktionary's English section defines `summer` and a translation table also gives it Italian words
- **THEN** `summer` is glossed by the section's definition

#### Scenario: A word only a table glosses
- **WHEN** no sense of the section glosses an English lemma and the English Wiktionary's Italian table translates it
- **THEN** it is glossed by the table's Italian words, at most three per part of speech

#### Scenario: Listed once
- **WHEN** the Italian Wiktionary's tables, read backwards, give `left-hand` « sinistro » under three parts of speech
- **THEN** « sinistro » is listed once

#### Scenario: A translator's note
- **WHEN** the English Wiktionary's Italian table gives `enrichment` « arricchimento (1,2) »
- **THEN** en-it's gloss reads « arricchimento »

#### Scenario: A rule of the Italian edition
- **WHEN** a rule of the Italian edition changes
- **THEN** en-it's, fr-it's and es-it's rule digests move, and no other committed pair's does

#### Scenario: Nothing else moves
- **WHEN** en-it's tables are committed by its own reducer, with no shared or edition module edited
- **THEN** every other pair's tables, pins, packs and goldens are unchanged, but for the line of fr-it's golden naming the en-it pack beside it

### Requirement: A gloss is in the reader's language, written by a person
A pack's glosses SHALL be written by a person in the pack's native language: never in the studied language, never in a third language, never machine-translated, and never pivoted through a third language but as it-es's and es-it's exception below. A word a person wrote into a Wiktionary translation table SHALL qualify when the table pairs the studied and the native language directly: the native words a studied entry lists, or the native entry whose table lists the studied word. For it-es and es-it alone (the Italian programme's I11), an Italian and a Spanish word a person wrote into senses of the same Wikidata item, or linked by a lexeme translation link, SHALL qualify after the pair's native Wiktionary's section on its studied language and both Wiktionaries' Italian–Spanish translation tables, and an Italian and a Spanish word a person wrote under the same sense of another Wiktionary's translation table — or the same entry of one sense when the table names none — after every other source; such a gloss SHALL be marked by its source wherever the pack's glosses are shown by source.

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
- **THEN** the it-es pack may gloss `acquirente` « Comprador », and no pack but it-es and es-it glosses through that table

#### Scenario: A pivot for es-it
- **WHEN** no source pairing Spanish and Italian directly glosses `comprador`, and the English Wiktionary's `buyer`, under its sense « person who makes purchases », lists Spanish *comprador* and Italian *acquirente*
- **THEN** the es-it pack may gloss `comprador` « acquirente », marked as the pivot's

### Requirement: it-es's further sources are derived and pinned like every source
The pack update SHALL register and derive every file it-es and es-it read and pin each by its sha256 in the pin of each pair that reads it, as it pins every source: the Spanish Wiktionary's Italian section and Italian translations and the Italian Wiktionary's Spanish translations; from the English, French, German, Portuguese, Russian, Polish and Dutch Wiktionaries' dumps, for each translation table, the Italian and the Spanish words it lists under one sense — or under the whole entry when the table names none, marked so —, kept together; the Italian Wiktionary's Spanish section; and, from a dated Wikidata Lexemes dump read as an edition of its own, the pairs of an Italian and a Spanish sense linked to the same item or by a translation link. Each Italian and Spanish word SHALL be one a person wrote; their pairing SHALL be inferred from a shared sense, entry or item, never machine-translated. Only it-es's and es-it's pins SHALL name the pivot and lexeme files; no pack SHALL read them before it-es or es-it is reduced, and no other pair's sources, pin or tables SHALL move.

#### Scenario: A table's sense keeps its words together
- **WHEN** an English entry's translation table lists, under one sense, the Italian `finestra` and the Spanish `ventana`, and under another sense other words
- **THEN** the derived file holds `finestra` and `ventana` together under the first sense, and the other sense's words apart

#### Scenario: A table numbering its senses only
- **WHEN** a Polish entry's translations are keyed by sense numbers alone
- **THEN** each number's Italian and Spanish words are kept apart from the other numbers'

#### Scenario: A table naming no sense
- **WHEN** a French entry's translation table names no sense
- **THEN** its Italian and Spanish words are kept under the whole entry, marked as naming none

#### Scenario: A table without both languages
- **WHEN** a translation table lists an Italian word but no Spanish one under a sense
- **THEN** nothing is derived from that sense

#### Scenario: The lexemes on a re-reduction
- **WHEN** it-es or es-it is reduced again from its pin
- **THEN** the lexeme pairs are read from that pair's sources release, checked against their sha256, and no Lexemes dump is fetched

#### Scenario: Nothing else moves
- **WHEN** it-es's or es-it's sources are derived
- **THEN** every other pair's sources, pins and tables are byte for byte what they were
