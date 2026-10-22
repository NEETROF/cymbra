## ADDED Requirements

### Requirement: The Italian Wiktionary's senses are read as meanings
The Italian Wiktionary's rules SHALL read a sense as a pointer when it is one of the edition's pointer wordings followed by « di » (or « of » in its English wordings) and a target of at most three words that ends the sense, the pointer lending the gloss of the word its wording names; a sense with text after its target, and « contrazione di », SHALL stay meanings. A sense kaikki tags as a form that is not worded as one SHALL be a meaning only when its word has no other meaning, and a nested gloss SHALL be read by its own text. The edition's notes to its readers — its placeholder for a missing definition, « vedi italiano » after a meaning, « (inv) », and template residue — SHALL be taken out; « vedi italiano » alone SHALL be read as the headword itself; a letter's name SHALL gloss nothing. Obsolete, archaic and heraldic senses SHALL be written after the others in their entry. Gender and number marks SHALL be taken out, straight quotes written « », and a gloss of translation-table words SHALL keep its words' case.

#### Scenario: A pointer lends the word it names
- **WHEN** the Italian Wiktionary glosses `labor` only as a variant of `labour`, which it glosses « lavoro »
- **THEN** `labor` takes `labour`'s gloss, not the gloss of the last word its sense links

#### Scenario: A pointer carrying a meaning
- **WHEN** a sense reads « plurale di that: quelli, quelle »
- **THEN** it stays a meaning

#### Scenario: A contraction
- **WHEN** a sense reads « contrazione di is not »
- **THEN** it stays a meaning, and `isn't` is glossed by it

#### Scenario: The Italian word is the same
- **WHEN** the English section glosses `panda` only « vedi italiano »
- **THEN** `panda` is glossed « panda »

#### Scenario: An obsolete sense last
- **WHEN** an entry's first sense is tagged obsolete and its second is not
- **THEN** the second sense comes first in the gloss, and the obsolete one after it

#### Scenario: A gender mark
- **WHEN** a sense reads « avvenimento m., evento m., fatto m »
- **THEN** its gloss reads « avvenimento, evento, fatto »

## MODIFIED Requirements

### Requirement: A gloss is cleaned by the rules of the Wiktionary edition that wrote it
The reducer SHALL clean each gloss by the rules of the Wiktionary edition it comes from — French, English, Spanish or Italian: which senses only name the word they are a form of, which notes and placeholders the edition writes for its own readers, and which coordinators a sense can be left hanging on. One edition's rules SHALL NOT change a gloss taken from another, the French Wiktionary's rules SHALL clean as they did before, and an edition's rules SHALL be read only by the pairs glossed from it.

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

#### Scenario: An Italian form-of sense
- **WHEN** the Italian Wiktionary glosses an English plural only « plurale di » followed by its singular
- **THEN** that sense is read as a form of the singular, not as a meaning

#### Scenario: The Italian edition moves no committed pair
- **WHEN** every committed pair is reduced again from its pin with the Italian edition's rules in place
- **THEN** its tables, pin and pack are byte for byte as committed, and no committed reducer imports the Italian edition's module
