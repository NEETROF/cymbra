## ADDED Requirements

### Requirement: An estimated ladder shows English's typical vocabularies
When the ladder shows a language whose pack's levels are estimated, each level's typical vocabulary SHALL be the one English's CEFR lists give that level, whenever the English pack is loaded, and the ladder SHALL say that the figures are English's. A ladder of CEFR-list levels SHALL show its own figures, as before.

#### Scenario: The Spanish ladder
- **WHEN** the statistics open for Spanish, the English pack being loaded
- **THEN** each level's « estimés » equals English's at that level, and the legend says they are taken from English

#### Scenario: The English ladder
- **WHEN** the statistics open for English
- **THEN** the ladder shows the figures its own lists give, with its legend as before
