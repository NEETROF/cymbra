## ADDED Requirements

### Requirement: The Spanish pack's sense runs carry a noun's gender
The es-fr pack's sense runs SHALL carry, for a noun's senses, the gender the noun's readings give it, so that the card's heading names it. A noun of both genders SHALL keep a run without a gender.

#### Scenario: A feminine noun
- **WHEN** the tables are reduced
- **THEN** the noun run of `casa` reads `NOUN|Gender=Fem`, and the card headed `casa` shows its gloss under « nom féminin »

#### Scenario: A noun of both genders
- **WHEN** the tables are reduced and `estudiante` is masculine or feminine by the person
- **THEN** its noun run carries no gender
