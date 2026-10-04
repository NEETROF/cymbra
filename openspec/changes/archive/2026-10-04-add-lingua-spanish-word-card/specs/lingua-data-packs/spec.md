## ADDED Requirements

### Requirement: The Spanish pack's sense runs carry a noun's gender
The es-fr pack's sense runs SHALL carry, for a noun's senses, the gender the noun's readings give it, so that the card's heading names it. A noun of both genders SHALL keep a run without a gender.

#### Scenario: A feminine noun
- **WHEN** the tables are reduced
- **THEN** the noun run of `casa` reads `NOUN|Gender=Fem`, and the card headed `casa` shows its gloss under « nom féminin »

#### Scenario: A noun of both genders
- **WHEN** the tables are reduced and `estudiante` is masculine or feminine by the person
- **THEN** its noun run carries no gender

### Requirement: A letter's name gives no reading of its plural
The es-fr pack SHALL read no form as an inflection of a letter's name. A noun that only names a letter SHALL keep the reading of its own form, and a noun that is also another word SHALL keep that word's inflections.

#### Scenario: The card of a form that is also a letter's plural
- **WHEN** the reader opens the card of `Es`, the present of *ser*
- **THEN** it names no other dictionary form: `es` is not read as the plural of the letter E

#### Scenario: A noun that is also another word
- **WHEN** the tables are reduced and `be` names the letter B and a sheep's bleat
- **THEN** `bes` stays the bleat's plural
