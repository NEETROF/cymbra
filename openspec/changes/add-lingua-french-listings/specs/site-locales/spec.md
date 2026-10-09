## ADDED Requirements

### Requirement: The English home page describes Lingua for the readers it serves
The English home page SHALL describe Cymbra Lingua from the pairs it ships. Once a pair glossed in English ships, its Lingua card SHALL name the languages read with an English gloss. Until then, its Lingua card SHALL read as before this requirement.

#### Scenario: Before any pair glossed in English
- **WHEN** every shipped pair is glossed in French
- **THEN** the English home page's Lingua card is byte for byte what it was

#### Scenario: English speakers learning Spanish
- **WHEN** es-en ships
- **THEN** the card names Spanish as the language read, and links `/en/lingua`

#### Scenario: English speakers learning Spanish and French
- **WHEN** es-en and fr-en ship
- **THEN** the card names Spanish and French as the languages read
