# Spec Delta

## MODIFIED Requirements

### Requirement: The Italian home page describes Lingua for the readers it serves
The Italian home page SHALL describe Cymbra Lingua from the pairs it ships. Once a pair glossed in Italian ships, it SHALL name the languages read with an Italian gloss and link the Italian Lingua page. Until then, it SHALL name the languages Lingua reads and the speakers it is made for, and link the English Lingua page; it SHALL NOT say that Lingua explains words in Italian.

#### Scenario: Before any pair glossed in Italian
- **WHEN** no shipped pair is glossed in Italian
- **THEN** the Italian home page's Lingua card names the languages read and the speakers of each shipped native language as the readers it is made for, and links `/en/lingua`

#### Scenario: Italian speakers learning English
- **WHEN** en-it ships, and no other pair glossed in Italian
- **THEN** the card names English as the language read and no audience, and links `/it/lingua`

#### Scenario: Italian speakers learning French
- **WHEN** fr-it ships beside en-it, and no other pair glossed in Italian
- **THEN** the card names English and French as the languages read and no audience, and links `/it/lingua`

#### Scenario: Italian speakers learning Spanish
- **WHEN** es-it ships beside en-it, and no other pair glossed in Italian
- **THEN** the card names English and Spanish as the languages read and no audience, and links `/it/lingua`

#### Scenario: Italian speakers learning French and Spanish
- **WHEN** fr-it and es-it ship beside en-it, and no other pair glossed in Italian
- **THEN** the card names English, French and Spanish as the languages read and no audience, and links `/it/lingua`
