# Spec Delta

## MODIFIED Requirements

### Requirement: The Spanish home page describes Lingua for the readers it serves
The Spanish home page SHALL describe Cymbra Lingua from the pairs it ships. Once a pair glossed in Spanish ships, it SHALL name the languages read with a Spanish gloss and link the Spanish Lingua page. Until then, it SHALL name the languages Lingua reads and the speakers it is made for, and link the English Lingua page; it SHALL NOT say that Lingua explains words in Spanish.

#### Scenario: Before any pair glossed in Spanish
- **WHEN** every shipped pair is glossed in French or English
- **THEN** the Spanish home page's Lingua card names the languages read and the speakers of each native language as the readers it is made for, and links `/en/lingua`

#### Scenario: Spanish speakers learning English
- **WHEN** en-es ships, and no other pair glossed in Spanish
- **THEN** the card names English as the language read and no audience, and links `/es/lingua`

#### Scenario: Spanish speakers learning Italian
- **WHEN** it-es ships beside en-es and fr-es, and no other pair glossed in Spanish
- **THEN** the card reads "Lee la web en inglés, en francés o en italiano…", names no audience, and links `/es/lingua`
