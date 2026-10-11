## MODIFIED Requirements

### Requirement: An estimated ladder shows English's typical vocabularies
When the ladder shows a language whose pack's levels are estimated, each level's typical vocabulary SHALL be the one English's CEFR lists give that level as the core records it, whatever packs the engine holds — 0 at A1, 1,213 at A2, 3,074 at B1, 7,155 at B2, 14,433 at C1 and 18,123 at C2 — and the ladder SHALL say that the figures are English's. A ladder of CEFR-list levels SHALL show its own figures, as before.
The recorded figures are those English's own ladder gave when English's dictionary words stopped
holding names, so that, until an English dictionary update moves English's own figures, a reader who
studies English and another language reads the same figure for a level on both ladders.

#### Scenario: The Spanish ladder
- **WHEN** the statistics open for Spanish
- **THEN** each level's « estimés » equals English's at that level, and the legend says they are taken from English

#### Scenario: No English pack
- **WHEN** the statistics open for Spanish on an engine that holds no English pack
- **THEN** the figures are the same, still taken from English

#### Scenario: An English dictionary update
- **WHEN** the en-fr pack is rebuilt from new tables
- **THEN** the Spanish ladder's figures do not change

#### Scenario: The English ladder
- **WHEN** the statistics open for English
- **THEN** the ladder shows the figures its own lists give, with its legend as before
