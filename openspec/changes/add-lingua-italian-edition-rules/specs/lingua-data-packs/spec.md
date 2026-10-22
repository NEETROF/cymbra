## ADDED Requirements

### Requirement: The Italian Wiktionary's senses are read as meanings
The Italian Wiktionary's rules SHALL read a sense kaikki does not tag as a form as a pointer when it is one of the edition's pointer wordings followed by « di » or one of its articled forms and a target of at most three words that ends the sense, or « vedi » and a word alone, or one of its English bot wordings; the pointer SHALL lend the gloss of the word its wording names. Such a sense with text after its target SHALL stay a meaning; « contrazione di » SHALL NOT make a sense a pointer, and « abbreviazione di » SHALL make one only when the word it names lends a gloss. A sense kaikki tags as a form that is not worded as one SHALL be a meaning only when its word has no other meaning, and a gloss nested under a pointer SHALL be read by its own text. The edition's notes to its readers — its placeholder for a missing definition, « vedi italiano » after a meaning, « (inv) », and template residue — SHALL be taken out before a sense is read; « vedi italiano » alone, or followed by a qualifier, SHALL be read as the headword itself; a letter's name SHALL gloss nothing. Obsolete, archaic and heraldic senses SHALL be written after the others in their entry, and a word whose every sense with a gloss is heraldic SHALL take no gloss from its definitions. Gender and number marks SHALL be taken out, never from the word a pointer names; straight quotes SHALL be written « »; and a gloss of translation-table words SHALL keep its words' case.

#### Scenario: A pointer lends the word it names
- **WHEN** the English section glosses `labor` only as a variant of `labour`, which it glosses « lavoro »
- **THEN** `labor` takes `labour`'s gloss

#### Scenario: A pointer carrying a meaning
- **WHEN** an untagged sense reads « plurale di that: quelli, quelle »
- **THEN** it stays a meaning

#### Scenario: A contraction and an abbreviation
- **WHEN** a sense reads « contrazione di is not », and another « abbreviazione di » an English expansion no entry glosses
- **THEN** both stay meanings, and `isn't` is glossed by the first

#### Scenario: A meaning kaikki tags as a form
- **WHEN** every sense of `hear` is tagged as a form and none is worded as one
- **THEN** `hear` is glossed « sentire, udire; ascoltare…; capire »

#### Scenario: The Italian word is the same
- **WHEN** the English section glosses `panda` only « vedi italiano »
- **THEN** `panda` is glossed « panda »

#### Scenario: An obsolete sense last
- **WHEN** an entry's first sense is tagged obsolete and its second is not
- **THEN** the second sense comes first in the gloss, and the obsolete one after it

#### Scenario: A word glossed by heraldry alone
- **WHEN** every sense the French section gives `le` is heraldic
- **THEN** `le` takes no gloss from its definitions, and the translation tables gloss it

#### Scenario: A letter's name
- **WHEN** both senses the Spanish section gives `zeta` name the letter (« La lettera Zeta »)
- **THEN** `zeta` takes no gloss from its definitions

#### Scenario: A gender mark, and a pointer's word
- **WHEN** one sense reads « avvenimento m., evento m., fatto m » and another « participio presente di sing »
- **THEN** the first reads « avvenimento, evento, fatto », and the second still names `sing`

#### Scenario: A table gloss keeps its case
- **WHEN** en-it glosses `up` from a translation table that gives « su »
- **THEN** the gloss reads « su »

## MODIFIED Requirements

### Requirement: A gloss is cleaned by the rules of the Wiktionary edition that wrote it
The reducer SHALL clean each gloss by the rules of the Wiktionary edition it comes from — French, English, Spanish or Italian: which senses only name the word they are a form of, which notes and placeholders the edition writes for its own readers, and which coordinators a sense can be left hanging on. One edition's rules SHALL NOT change a gloss taken from another, the French Wiktionary's rules SHALL clean as they did before, and a pair SHALL load the rules of the one edition whose senses are its definitions, and no other edition's.

#### Scenario: The French Wiktionary's glosses do not move
- **WHEN** the en-fr and es-fr tables are reduced again from their pinned sources with every edition's rules in place
- **THEN** their seven data tables and NOTICE are byte for byte as committed

#### Scenario: An English form-of sense
- **WHEN** the English Wiktionary glosses the noun `casas` only « plural of casa »
- **THEN** that sense is read as a form of `casa`, not as a meaning

#### Scenario: A Spanish form-of sense
- **WHEN** the Spanish Wiktionary glosses `chips` « Forma del plural de chip. » and `read` « Participio pasado del verbo (to) read. »
- **THEN** each sense is read as a form of `chip` and of `read`, not as a meaning

#### Scenario: An English sense with no gloss
- **WHEN** the English Wiktionary marks a sense as having no gloss
- **THEN** the sense is left out, and a word left with no sense has no gloss

#### Scenario: The Italian edition's rules edited
- **WHEN** a rule of the Italian edition changes
- **THEN** only the rule digests of the pairs whose definitions come from the Italian Wiktionary move, and no pair whose definitions come from another edition loads the Italian edition's module
