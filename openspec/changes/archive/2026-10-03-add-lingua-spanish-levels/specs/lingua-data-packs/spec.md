## ADDED Requirements

### Requirement: The Spanish pack's estimated levels
The es-fr pack SHALL carry a level table derived from word frequency: in rank order, the commonest lemmas whose French gloss is not only a proper noun's SHALL take the sizes of English's CEFR levels — 1,020 at A1, 1,158 at A2, 2,015 at B1, 2,347 at B2, 886 at C1 and 876 at C2. A lemma without such a gloss SHALL take no level.

#### Scenario: The commonest words are A1
- **WHEN** the tables are reduced
- **THEN** `de` and `que` are A1, and `madrid`, glossed only as a place, has no level

#### Scenario: Six levels of English's sizes
- **WHEN** the tables are reduced
- **THEN** the level table holds 8,302 lemmas, as many at each level as English's

### Requirement: A pack says when its levels are estimated
A pack's metadata SHALL say when its level table is estimated rather than taken from a CEFR list. A pack whose metadata says nothing SHALL read as not estimated, and saying nothing SHALL leave the pack's bytes as they were.

#### Scenario: The Spanish pack's levels are estimated
- **WHEN** the es-fr pack is loaded
- **THEN** the engine reports its levels as estimated

#### Scenario: The English pack is unchanged
- **WHEN** the en-fr pack is built from its committed tables
- **THEN** its bytes match its pin, and the engine reports its levels as not estimated
