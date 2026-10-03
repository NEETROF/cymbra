## ADDED Requirements

### Requirement: Estimated levels are labelled as such
Wherever the extension shows or asks a level of a language whose pack's levels are estimated, it SHALL say so. The level titles SHALL read « estimé ». Réglages, the statistics and onboarding SHALL say that the levels are estimated from word frequency. The ladder SHALL NOT call its words taught. A language whose levels come from a CEFR list SHALL read as before.

#### Scenario: Réglages for Spanish
- **WHEN** the reader opens Réglages with Spanish accepted
- **THEN** the Spanish level block is titled « Niveau d'espagnol estimé » and says that the levels are estimated from word frequency

#### Scenario: The ladder for Spanish
- **WHEN** the statistics open for Spanish
- **THEN** the ladder is titled « Mon niveau d'espagnol estimé », and its column of words up to each level reads « courants », not « enseignés »

#### Scenario: Onboarding for Spanish
- **WHEN** the reader chooses B1 for Spanish during onboarding
- **THEN** the confirmation reads « Niveau enregistré : B1 (estimé). »

#### Scenario: English reads as before
- **WHEN** the reader opens Réglages or the statistics for English
- **THEN** they read « Niveau d'anglais » and « Mon niveau d'anglais », with no estimate note, and the ladder keeps « enseignés »
