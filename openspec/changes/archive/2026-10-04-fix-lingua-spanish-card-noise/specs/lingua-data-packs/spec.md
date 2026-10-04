## ADDED Requirements

### Requirement: A Spanish form reads as the common word before a proper name
When a form of the es-fr pack is both a proper name and a form of another word, the pack SHALL read it as the commoner of the two by frequency, the form's own frequency standing for the name: the treebank's counts SHALL NOT give the form to a name on their own.

#### Scenario: A verb form spelled like a surname
- **WHEN** the reader opens the card of `miró`, which the treebank only meets as the surname Miró
- **THEN** the card is keyed by *mirar*, as its preterite

#### Scenario: A plural spelled like a given name
- **WHEN** the reader opens the card of `dolores`
- **THEN** the card is keyed by *dolor*

#### Scenario: A country commoner than its adjective
- **WHEN** the reader opens the card of `Argentina`
- **THEN** the card is keyed by *argentina*, the country, not by the adjective *argentino*

### Requirement: A letter is no French gloss of a Spanish word
The es-fr pack's French glosses SHALL leave out the senses that name a letter of the alphabet, and the entries that describe a letter.

#### Scenario: The commonest preposition
- **WHEN** the reader opens the card of `a`
- **THEN** its gloss opens on « À », with no sense naming the letter A

#### Scenario: A gloss that only mentions the word « lettre »
- **WHEN** the reader opens the card of `carta de amor`
- **THEN** it is glossed « Lettre d'amour »
