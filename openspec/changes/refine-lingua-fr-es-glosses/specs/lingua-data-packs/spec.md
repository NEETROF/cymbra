## ADDED Requirements

### Requirement: fr-es's glosses show a sense's register, age and place in the Spanish Wiktionary's words
The fr-es reduction SHALL open a word's sense with the labels the Spanish Wiktionary's French section gives it of three kinds — its register, its age and its place —, in parentheses, in the edition's own Spanish words as its categories file the sense, register first, then age, then place; and SHALL NOT move a sense in its row because it is labelled.
The words come from closed tables: register « coloquial », « malsonante », « jergal », « despectivo »,
« literario », « vulgar », « eufemístico », « formal », « infantil », « irónico », « jocoso »; age
« anticuado », « obsoleto »; place the regions and countries the section names (« Quebec »,
« Canadá », « Francia », « Bélgica », « Suiza »…), a place left out when a place inside it is shown. A
category whose template named another language than French is read alike. A figurative or a rare
sense shows no label, and neither does a sense that only points at another word. Each labelled sense
carries its own labels. The senses the edition marks outdated or obsolete stay after the current ones
of their entry, as the Spanish edition's pre-pass writes them. An expression's senses show no labels.
These are rules of fr-es's reducer alone: they re-pin fr-es and no other pair, and no lemma gains or
loses a gloss through them.

#### Scenario: A vulgar sense and an outdated one
- **WHEN** the section defines `baiser` as a verb whose first sense « Coger (sexualmente). » is filed under « Términos malsonantes », and whose sense « Besar. » is filed under « Términos anticuados », « Canadá » and « Bélgica », and as a noun « Beso, besuqueo u ósculo. »
- **THEN** the gloss of `baiser` reads « (malsonante) Coger (sexualmente) » first, « (anticuado, Canadá, Bélgica) Besar » after the verb's other senses, and « Beso, besuqueo u ósculo » unlabelled

#### Scenario: A place inside another
- **WHEN** a sense of `oui` is filed under « América » and « Quebec »
- **THEN** it opens on « (Quebec) »

#### Scenario: A label the English tags miss
- **WHEN** a sense of `avec` is filed under « Quebec » and « Bélgica » and kaikki tags it `Canada` and `Quebec` alone
- **THEN** it opens on « (Quebec, Bélgica) »

#### Scenario: No reordering for a label
- **WHEN** the section's `cul` opens its noun on « Culo. », filed under « Términos malsonantes », before « Fundo (de un objeto). »
- **THEN** the gloss of `cul` still opens on « (malsonante) Culo »

#### Scenario: A figurative sense
- **WHEN** a sense of a word is tagged figurative and filed under « Términos en sentido figurado » alone
- **THEN** it shows no label

### Requirement: fr-es's definitions read as a French word's meanings
The fr-es reduction SHALL read a noun entry of the section whose every sense repeats, word for word, a sense of the same word's verb entries, when French's readings name the word a verb, after those verb entries; SHALL read a contraction the section defines as « Contracción de la preposición … » as a preposition; SHALL take out a parenthesis after a sense's meaning that names the part of speech again (« (Pronombre nominativo.) »), with the period before it; and SHALL write « etc » not followed by a period as « etc. ».
Any other note after a meaning stays (« (Plural exclusivo.) », « (Literalmente: …) »), as does a note
before the meaning (« (être + participio) Haber »). A noun entry that only shares some senses with the
verb, or a word French's readings do not name a verb, keeps the section's order. These are rules of
fr-es's reducer alone: they move rows' sense runs and words, and no lemma gains or loses a gloss.

#### Scenario: An infinitive's noun after its verb
- **WHEN** the section has `être` as a noun « Ser. » before its verb « Ser. », « Estar. » and its auxiliary « (être + participio) Haber. », and French's readings name `être` a verb
- **THEN** the gloss of `être` opens on « Ser » as a verb, and every run of it is a verb's

#### Scenario: A contraction of a preposition
- **WHEN** the section has `des` as an article « Algunos, algunas, unos o unas. » and as a contraction « Contracción de la preposición de y el artículo les; de las o de los. »
- **THEN** the gloss of `des` holds a determiner's run and a preposition's run, and no run without a part of speech

#### Scenario: The part of speech named again
- **WHEN** the section defines `qui` « Quién. (Pronombre nominativo.) » and « Que. (Pronombre nominativo.) »
- **THEN** the gloss of `qui` is « Quién; Que »

#### Scenario: Etc with its period
- **WHEN** the section defines `cochon` « Cerdo, marrano, guarro, cochino, etc. »
- **THEN** the gloss of `cochon` ends on « etc. »

### Requirement: fr-es's translation-table glosses list each Spanish word once, and never the French word
The fr-es reduction SHALL list a Spanish word of the French Wiktionary's translation table once across the French word's parts of speech — a run whose shown words another run of the word shows going, of two equal runs the later, and a word still shown by two runs staying in the first —; and SHALL NOT gloss a French word with a Spanish word of that table spelled as the French word, up to case, that wordfreq rates at least a hundred times commoner in French than in Spanish, unless it is a loanword Spanish writes alike, from a reviewed list.
A run left with no word goes, and the run's next words come up. A word whose only translations are the
French word itself takes no gloss from the table, and the next source glosses it if it can. The list
of loanwords is closed, each entry reviewed by the owner (« diaporama », « raï »); a candidate a later
update brings is left out until it is listed. The Spanish Wiktionary's own definitions are not read by
this rule.

#### Scenario: A word under two parts of speech
- **WHEN** the French Wiktionary lists « partido » for `parti` as an adjective and as a noun
- **THEN** the gloss of `parti` is « Partido », once

#### Scenario: A run inside another
- **WHEN** the French Wiktionary lists « ruso » for `russe` as an adjective, and « ruso », « rusa » as a noun
- **THEN** the gloss of `russe` is « Ruso, rusa », once

#### Scenario: A word shown twice in different runs
- **WHEN** the French Wiktionary lists « claro », « luminoso », « límpido » for `clair` as an adjective and « claro », « claramente » as an adverb
- **THEN** the gloss of `clair` is « Claro, luminoso, límpido; Claramente »

#### Scenario: The French word given as Spanish
- **WHEN** the French Wiktionary lists « arnaque » as the Spanish for `arnaque`, and « retraite », « jubilación », « retiro », « pensión » for `retraite`
- **THEN** `arnaque` takes no gloss from the table, and the gloss of `retraite` is « Jubilación, retiro, pensión »

#### Scenario: A loanword Spanish writes alike
- **WHEN** the French Wiktionary lists « diaporama » as the Spanish for `diaporama`, and the list names it
- **THEN** the gloss of `diaporama` is « Diaporama »

#### Scenario: A cognate
- **WHEN** the French Wiktionary lists « club » for `club`, a word wordfreq rates about as common in Spanish as in French
- **THEN** the gloss of `club` is « Club », as before

### Requirement: fr-es's inverted table glosses no French word through a name, an acronym or another sense
Reading the Spanish Wiktionary's French translations backwards, the fr-es reduction SHALL NOT read the translation template's language code « fr » as a French word; SHALL let a French word written as an acronym — two letters or more, all capitals — gloss its lemma only through a Spanish word with the same letters; SHALL NOT let a Spanish saint's name (« San », « Santa », « Santo ») gloss a one-word French name; SHALL NOT read a French word holding an elided article read into it (« lOrient ») as a word; and SHALL leave out the translations a reviewed list names, each a French word and a Spanish word with its reason.
The letters are compared without dots, spaces or case (« AEC » « A. e. c. »). A name translated by a
name stays (« Pâques » « Pascua »), and an expression's words are read as before. The list holds the
other-sense words of the 10,000 commonest lemmas that no measured rule tells from a translation, the
French Wiktionary's direct table included (« ds » « Tiburón »); an entry fires on nothing once the
source changes. A lemma left with no word takes no gloss.

#### Scenario: An acronym's other word
- **WHEN** the Spanish Wiktionary's `EEUU` lists « US » and « USA » among its French translations, and its `ONU` lists « ONU »
- **THEN** `us` and `usa` take no gloss from it, and the gloss of `onu` is « ONU »

#### Scenario: The language code
- **WHEN** the Spanish Wiktionary's `calabaza` lists « fr » beside « citrouille »
- **THEN** `fr` takes no gloss from it, and `citrouille` is read as before

#### Scenario: A saint for a first name
- **WHEN** the Spanish Wiktionary's phrase `San Lucas` lists « Luc », and its `Pascua` lists « Pâques »
- **THEN** `luc` takes no gloss from it, and the gloss of `pâques` is « Pascua »

#### Scenario: A word the list names
- **WHEN** the Spanish Wiktionary's `secuestro` lists « rap » among its French translations, and the list names `rap` with `secuestro`
- **THEN** `rap` takes no gloss from it

#### Scenario: Nothing else moves, and the floor holds
- **WHEN** fr-es is reduced again from its pinned sources with these rules
- **THEN** en-fr's, es-fr's, es-en's, en-es's and fr-en's tables and pins, and every table of `tables/fr/`, are byte for byte unchanged, fr-es's pin keeps its snapshot, its studied record and its sources, and fr-es's coverage of the 5,000, 10,000 and 20,000 commonest lemmas is at least its floor
