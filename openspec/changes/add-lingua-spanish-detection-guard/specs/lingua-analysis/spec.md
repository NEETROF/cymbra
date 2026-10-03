## ADDED Requirements

### Requirement: Catalan and Galician are not read as Spanish
The core SHALL refuse as Spanish a block the detector reads as Spanish whose Catalan function words, or whose Galician function words, outnumber its Spanish ones. A refused block SHALL be excluded from a Spanish analysis and SHALL NOT vote for Spanish when a document's language is chosen. A tie, or a block with no such function word, SHALL stay Spanish. The guard SHALL leave English's detection unchanged, and Spanish's analyser version SHALL be `1.1.0`. A short Catalan or Galician line with no function word of either kind is not refused, and still reads as Spanish.

#### Scenario: A Catalan paragraph
- **WHEN** "El govern ha aprovat el pressupost amb el suport dels grups, però també amb crítiques." is offered as Spanish
- **THEN** it is not Spanish, and a Spanish analysis excludes it

#### Scenario: A Galician paragraph
- **WHEN** "Non hai unha solución sinxela, pero o concello xa traballa nela." is offered as Spanish
- **THEN** it is not Spanish

#### Scenario: Spanish prose is kept
- **WHEN** "El gobierno aprobó el presupuesto con el apoyo de los grupos, pero también hubo críticas." is offered as Spanish
- **THEN** it is Spanish

#### Scenario: A Catalan page for a reader of English and Spanish
- **WHEN** a page whose blocks are Catalan is offered with English and Spanish as candidates
- **THEN** its blocks give Spanish no vote

#### Scenario: The leak the guard leaves
- **WHEN** a short Catalan caption with no function word of either kind is offered as Spanish
- **THEN** the guard does not refuse it, as this requirement states
