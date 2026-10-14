## ADDED Requirements

### Requirement: The French home page describes Lingua for the readers it serves
The French home page SHALL describe Cymbra Lingua from the pairs it ships: its Lingua card SHALL name every language read with a French gloss, in the order the packages list them, and SHALL link the French Lingua page.

#### Scenario: Today's pairs
- **WHEN** the packages ship en-fr and es-fr among their pairs
- **THEN** the French home page's Lingua card reads « Lisez le web en anglais ou en espagnol… » and links `/lingua`

#### Scenario: English alone
- **WHEN** the packages ship en-fr and no other pair glossed in French
- **THEN** the card reads « Lisez le web en anglais… »
