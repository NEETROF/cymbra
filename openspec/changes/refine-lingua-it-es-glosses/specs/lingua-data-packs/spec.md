# Spec Delta

## ADDED Requirements

### Requirement: it-es's glosses open on the Spanish word its sources agree on, in the order Italian uses the word
The it-es reduction SHALL let a row whose first word its own source alone gives, or that fewer of its sources open on than another word, open on the word its sources agree on; SHALL open a function word's row on the part of speech UD Italian-MarkIT reads the word as; and SHALL open a row a reviewed table names on the sense the table gives it, whatever the rules read.
A Spanish word's support for a lemma is the number of it-es's sources whose gloss of the lemma holds it —
the gloss's items split at a comma and at « o » or « u » between short items, their parentheses out,
their case folded —, each of the pivot's translation tables counting as a source. A row yields when the
word most sources open on is opened on by at least two of them, by more than open on the row's first word,
and is held by more sources than that word; or when the row's first word is held by its own source alone
and another word, one some source gives in the part of speech the row opens on, by two or more. Ties go to
the earlier source in the order of *Italian is glossed in Spanish…*, then to the word's place there. The sense holding the winner opens the row, the winner first in it; a winner the
row lacks brings the sense of the earliest source holding it, and the row's first sense goes only when
none of its words another source gives — but the row's first word stays, as a sense of its own after the
winner's, when it is the cognate of a lemma of a learned suffix (`-zione`, `-sione`, `-enza`, `-anza`,
`-ità`, `-mente`, `-abile`, `-ibile`: the word ends as Spanish writes the suffix, accent included, and is at
most 0.2 of its length in edit distance from the lemma written so, accents out) and its Spanish Zipf
frequency is at least 3; a row keeps at most eight senses. A row Wikidata's lexemes of one
item gloss, whose first word no other source gives, takes the gloss of the next source whose first word
two sources give.
The function words' parts of speech the treebank may move first are `ADP`, `DET`, `PRON`, `CCONJ`,
`SCONJ`, `PART` and `ADV`, at least ten times and twice as often as the part of speech the row opens on,
never a proper noun's, no noun, verb or adjective moved ahead of another; it-es's pin records MarkIT's
training and development sections at the commit and sha256s it-fr's pin records, and reads them for these
counts alone. A row holding only nouns, verbs or adjectives, for a word MarkIT reads at least ten times and
twice as often as an adverb or a preposition, gains that part of speech's first sense from the next source
that gives one, when two sources give its first word, ahead of the row's own senses.
The reviewed table, `FIRST_RUN`, holds a row only for a word the rules open on a wrong word — of the 300
commonest, or settled by the owner —, each with its reason: the source, the sense the row opens on — a sense that source gives the
word, never a word written for it — and whether the row's other senses stay; the reduction fails on a row
whose sense the source no longer writes. A row `FUNCTION_WORDS`, `FIRST_RUN`, the `di` family's table or
`CORRECTIONS` writes is never moved by agreement, by the treebank or by the rare-word rule. These are
rules of it-es's own modules: they re-pin it-es and no other pair, Italian's studied tables do not move,
its coverage stays at least its floor, and the it-es baseline is re-blessed once, in the pull request that
commits them.

#### Scenario: A lone first word yields
- **WHEN** the Spanish tables read backwards alone give `caso` « Azar », and Wikidata's links, its items and several of the pivot's tables give it « Caso »
- **THEN** the gloss of `caso` opens on « Caso »

#### Scenario: Most sources open on another word
- **WHEN** the section glosses `andare` « Andar », and the Italian Wiktionary's translations, the Spanish tables read backwards, the Spanish section and the pivot open on « Ir »
- **THEN** the gloss of `andare` opens on « Ir, andar »

#### Scenario: The winner keeps the row's part of speech
- **WHEN** the Spanish tables read backwards gloss `insieme` with the adverb « Juntamente », and two sources give it the noun « Conjunto »
- **THEN** the gloss of `insieme` does not open on « Conjunto »

#### Scenario: A reviewed row stays
- **WHEN** `FUNCTION_WORDS` writes `ne`'s adverb « De allí, de ahí », which no other source gives, and the pivot's tables give it other words
- **THEN** the gloss of `ne` keeps the reviewed senses

#### Scenario: Wikidata's lone item
- **WHEN** Wikidata's lexemes of one item alone gloss `don` « D. », and the pivot's tables give it « Don »
- **THEN** the gloss of `don` opens on « Don »

#### Scenario: A function word's commonest part of speech
- **WHEN** the section gives `ancora` the noun « Ancla » before the adverb « Todavía », and MarkIT reads `ancora` 26 times as an adverb and never as a noun
- **THEN** the gloss of `ancora` opens on « Todavía »

#### Scenario: A missing adverb
- **WHEN** the section glosses `spesso` with the adjective « Gordo » alone, MarkIT reads it 27 times as an adverb, and the Spanish section and the pivot give the adverb « A menudo »
- **THEN** the gloss of `spesso` opens on « A menudo, frecuentemente, con frecuencia », then « Gordo »

#### Scenario: A row the reviewed table opens
- **WHEN** `FIRST_RUN` opens `posto` on the pivot's « Lugar, sitio, puesto », its other senses dropped
- **THEN** the gloss of `posto` is « Lugar, sitio, puesto », not « En »

#### Scenario: A sense the table no longer finds
- **WHEN** an update's source no longer writes the sense a row of `FIRST_RUN` names
- **THEN** the reduction fails, naming the row

#### Scenario: Nothing else moves
- **WHEN** it-es is reduced with these rules
- **THEN** Italian's studied tables and the eight other pairs' tables, pins, packs and goldens are unchanged

### Requirement: it-es's glosses hold Spanish words, read as their sources mean them
The it-es reduction SHALL leave out of its table-shaped sources a word that is not Spanish, a place name written as another word's label and a translation filed under another word's verb form; SHALL open a row whose first word is rare in Spanish on a commoner word its sources give; SHALL read a feminine noun's lent masculine as the feminine its sources write; and SHALL leave out the senses the section files under heraldry.
A word is not Spanish when it is one word whose Spanish Zipf frequency (wordfreq) is under 1 while its
English, French, Portuguese, Italian or German one is at least 3, unless the reviewed list of Spanish
words Italian writes alike names it. A place label is a capitalised word after another in a list of the
Italian Wiktionary's Spanish translations or of the Spanish tables read backwards that Italian's tables
keep as a name — a lemma with no reading —; a list's first word is never one. A row whose first word's
Spanish Zipf frequency is under 2 opens on the first commoner word of its first sense, or else on the first
sense of its part of speech of the next source whose first word is commoner. A translation the Italian
Wiktionary files under a verb form's sense, which no other source gives the word, is left out; so is the
pivot's verb sense for a word Italian reads only as another verb's form and as no verb of its own, unless it
is the row's only part of speech. Once a row's sources agree, an item of a sense the row rules brought into
the row — no sense of the row as the sources first glossed it — goes when Spanish does not write it (a Zipf
frequency of 0) or writes it under 1.5 while another of those languages writes it at least 3, unless the
reviewed list names it; a sense's last item stays. A feminine noun the section points at its masculine, with no meaning of
its own, takes the next source's gloss when that gloss opens on the lent word's feminine, which two
sources give.

#### Scenario: A displaced cognate
- **WHEN** agreement opens `insufficienza` on the pivot's « Falta, carencia », and the section's first word « Insuficiencia » is the lemma's cognate, which Spanish writes at a Zipf frequency of 3.6
- **THEN** the gloss of `insufficienza` keeps « Insuficiencia » as a sense after « Falta, carencia »

#### Scenario: A false friend displaced
- **WHEN** agreement opens `mirare` on « Apuntar », and the row's first word « Mirar » is a verb's cognate, which no learned suffix forms
- **THEN** « Mirar » is not kept

#### Scenario: A word that is not Spanish
- **WHEN** the Italian Wiktionary's translations give `evidente` « Evident » and the Spanish tables read backwards « Evidente »
- **THEN** the gloss of `evidente` is « Evidente »

#### Scenario: A foreign item agreement brings
- **WHEN** agreement opens `marmo` on the pivot's « Mármol, sepulcro, alioj », and Spanish does not write « alioj »
- **THEN** the gloss of `marmo` is « Mármol, sepulcro »

#### Scenario: A word Spanish writes alike
- **WHEN** a table gives `friulano` « Friulano », which the reviewed list names a Spanish word too
- **THEN** « Friulano » is kept

#### Scenario: A place label
- **WHEN** the Italian Wiktionary lists `ladro`'s Spanish translations « ladrón, chorizo, Spagna, chorro »
- **THEN** the gloss of `ladro` is « Ladrón, chorizo, chorro »

#### Scenario: A rare first word
- **WHEN** the Spanish section alone opens `rapidamente` on « Aquejadamente », which Spanish rarely writes, and the pivot gives « Rápidamente »
- **THEN** the gloss of `rapidamente` opens on « Rápidamente »

#### Scenario: A feminine's own word
- **WHEN** the section's `ragazza` only points at `ragazzo`, whose gloss « Chico, muchacho » it lends, and two sources give `ragazza` « Chica »
- **THEN** the gloss of `ragazza` is « Chica, muchacha, niña »

#### Scenario: A lent masculine no other source glosses
- **WHEN** the section's `comunicativa`, a noun in Italian's readings, only points at `comunicativo`, whose « Comunicativo » it lends, and no other source glosses `comunicativa`
- **THEN** the gloss of `comunicativa` keeps « Comunicativo »

#### Scenario: A verb form inside a noun's row
- **WHEN** the Italian Wiktionary lists among `faccia`'s translations « haga » and « hagas » under the senses « persona singolare del congiuntivo presente di fare », which no other source gives `faccia`
- **THEN** the gloss of `faccia` is « Cara »

#### Scenario: A heraldic sense
- **WHEN** the section gives `nudo` « Desnudo » and a sense filed under heraldry
- **THEN** the gloss of `nudo` is « Desnudo »

### Requirement: it-es's glosses hold no name's note, list or repeat a common word does not need
The it-es reduction SHALL keep a surname's note off a word Italian's readings give a common reading when another source glosses the word; SHALL read a given name's note as the Spanish name it names; SHALL cut a list of eight items or more to its first three; and SHALL leave out a taxon's name before a meaning, a sense repeated within a part of speech, and an expression's sense that only opens another.
A surname's note is a proper noun's sense opening on « Apellido »; a word whose only gloss it is keeps it.
A given name's note « Nombre de pila de varón, equivalente del español Pablo », or its wording without
« de pila », reads « Pablo (nombre de pila) ». A list is cut when three quarters of its items hold at most four words and none ends a sentence,
its items first ordered by the number of sources giving each — each of the pivot's tables a source —, then
by their Spanish Zipf frequency, the list's order breaking ties.
A sense equal to another of its part of speech once their parentheses are out goes when it has none, and
so does a sense with no parenthesis whose every item another sense of its part of speech says; two senses
that differ by their parentheses both stay. A sense with no parenthesis equal to an earlier sense of any
part of speech, once « ¡ », « ! », « ¿ » and « ? » are out too, goes.

#### Scenario: A surname's note off a common word
- **WHEN** the section's only sense of `senatore` is the proper noun's « Apellido », Italian's readings make `senatore` a noun, and Wikidata gives it « Senador »
- **THEN** the gloss of `senatore` is « Senador »

#### Scenario: A surname alone
- **WHEN** the section's only sense of `esposito` is « Apellido italiano » and no other source glosses it
- **THEN** the gloss of `esposito` keeps « Apellido italiano »

#### Scenario: A given name
- **WHEN** the section glosses `paolo` « Nombre de pila de varón, equivalente del español Pablo », and `giuseppe` « Nombre de varón, equivalente del español José »
- **THEN** the gloss of `paolo` is « Pablo (nombre de pila) », and that of `giuseppe` « José (nombre de pila) »

#### Scenario: A list cut
- **WHEN** the section glosses `inoltre` with a sense of thirteen items, « Además, adicionalmente, allende, a mayor abundamiento, … »
- **THEN** the sense reads « Además, adicionalmente, allende »

#### Scenario: A list cut by use
- **WHEN** the section glosses `talvolta` with « A la de veces, a las de veces, a las veces, a tiempos, a veces, … », and the pivot's tables and the Spanish tables read backwards give « a veces » and « de vez en cuando »
- **THEN** the sense reads « A veces, de vez en cuando, a la de veces »

#### Scenario: A taxon before the meaning
- **WHEN** the section glosses `cavallo` « (Equus caballus) Caballo »
- **THEN** the gloss of `cavallo` is « Caballo »

#### Scenario: A repeated sense
- **WHEN** the section glosses `persona` « Persona (ser humano); Persona », and `firma` « Firma (leyenda); Firma (empresa) »
- **THEN** the gloss of `persona` is « Persona (ser humano) », and `firma` keeps both senses

#### Scenario: An expression glossed twice
- **WHEN** the section glosses `d'accordo` « De acuerdo » and a table « De acuerdo, vale »
- **THEN** the expression's gloss is « De acuerdo, vale »

### Requirement: it-es glosses the `di` family and corrects, by name, what its sources write
The it-es reduction SHALL gloss `del`, `dello`, `della`, `dei`, `degli` and `delle` from a reviewed table, and SHALL correct, by a reviewed list, what a source writes wrongly, with a source's words where one gives them; a gloss no source writes SHALL be marked in its table as written by a person, with its reason, and approved by the owner.
The `di` family's table is keyed by headword and replaces whatever the sources give the six words; the
reduction fails on a row whose headword is no ranked lemma. `CORRECTIONS` is it-fr's shape: each entry the
source, the headword, its part of speech, the source's text as the pinned file writes it, the corrected
text or none, the reason and the page reported upstream; the update's summary line names a correction that
finds nothing.

#### Scenario: An article read through the table
- **WHEN** the section describes `della` « Contracción de la preposición di y el artículo la »
- **THEN** the gloss of `della` is « De la », marked in the table as written by a person

#### Scenario: A form no source glosses
- **WHEN** no source glosses `dei`
- **THEN** the gloss of `dei` is « De los », marked as written by a person

#### Scenario: A lent sense a source's word restores
- **WHEN** agreement opens `porca` on the direct table's « Caballón, camellón », dropping the lent « cerdo », and the pivot gives `porca` « cerda »
- **THEN** the gloss of `porca` is « Cerda; Caballón, camellón », the correction's words the sources'

#### Scenario: A word taken for another language's
- **WHEN** the pivot glosses `the`, the English article on Italian pages, « Té »
- **THEN** `the` has no gloss

#### Scenario: A correction that finds nothing
- **WHEN** an update's source no longer writes the text a correction names
- **THEN** the update's summary line names the correction, to be removed

## MODIFIED Requirements

### Requirement: Italian is glossed in Spanish from the Spanish Wiktionary's Italian section, the two Wiktionaries' tables, Wikidata and a pivot
it-es's glosses, sense runs and expressions SHALL be reduced by a reducer of its own, reading Italian's studied tables as committed and writing none of it, with the Spanish Wiktionary's rules, from these sources in this order — the first that glosses a lemma glossing it, but where *it-es's glosses open on the Spanish word its sources agree on, in the order Italian uses the word* moves its row —: the Spanish Wiktionary's Italian section's definitions; the Italian Wiktionary's Spanish translations; the Spanish Wiktionary's Italian translations read backwards; Wikidata's lexeme translation links and Spanish glosses of Italian senses, then its lexemes of one item; the Italian Wiktionary's Spanish section read backwards; then the pivot of seven Wiktionaries' translation tables — at most three words per part of speech from each table-shaped source, a pivot's Spanish words ordered by the number of tables and editions giving them, then by frequency, then alphabetically. The pivot SHALL read a table's group named by a sense or a sense number, or the whole entry of a one-sense entry, never a « to sort » table; SHALL leave out entries tagged name, abbreviation, symbol, character, letter, affix or punctuation, one-letter headwords, capitalised headwords but German nouns, capitalised Italian or Spanish words but acronyms of at most six letters, Spanish words with a grave accent, « ° » or more than sixty characters, and a group of a content word's entry whose Italian or Spanish words hold an article, a preposition or a conjunction — a translation its extract split into its words; and SHALL supply no expression. Before the shared rules, it-es's own pre-pass SHALL keep an acronym's sense and a name's note off a word with a lowercase entry, and a name's note off a word the Italian or the Spanish Wiktionary's translation table glosses; SHALL let a pointer that names a word lend that word's gloss only for a headword with no meaning and no pointer the edition tags of its own, and read a meaning a pointer carries after a colon only for a headword with no meaning of its own; SHALL take out sense links, superscripts and subscripts, and open a gloss with a capital letter; SHALL read the commonest function words from a reviewed list; SHALL take no Italian key from the Italian Wiktionary's notes to its readers in its Spanish section read backwards; and, for every table-shaped source, SHALL take no letter, no Spanish word listed under two parts of speech of one Italian word — listed once, under the first of them Italian's readings name, or else the first listed —, no word with a grave accent or « ° », and no Italian word given as Spanish unless a reviewed list names it a Spanish word too. it-es's expressions SHALL be the section's glossed headwords with a space, then those the two Wiktionaries' tables gloss, then the words without a space that Italian's pre-pass splits, each keyed at build as *An Italian pack keys its expressions as Italian is read* requires; the acronym rows SHALL be glossed as *Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms* requires. Every gloss SHALL be Spanish written by a person; a gloss no source writes, which a reviewed table of it-es gives, SHALL be marked there as written by a person, as *it-es glosses the `di` family and corrects, by name, what its sources write* requires.

#### Scenario: A direct source first
- **WHEN** the Spanish Wiktionary's Italian section defines `casa` « Casa », the word the other sources open on too, and the pivot also gives it Spanish words
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
