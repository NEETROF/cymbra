## MODIFIED Requirements

### Requirement: Catalan and Galician are not read as Spanish
The core SHALL refuse as Spanish a block the detector reads as Spanish whose Catalan function words, or whose Galician function words, outnumber its Spanish ones; Galician's `da` and `das`, which Spanish writes too (*da*, « gives »; *das*, « you give »), SHALL count for Galician only in a block that holds another Galician function word. A refused block SHALL be excluded from a Spanish analysis and SHALL NOT vote for Spanish when a document's language is chosen. A tie, or a block with no such function word counted, SHALL stay Spanish unless *Occitan is not read as Spanish* refuses it. The guard SHALL leave English's detection unchanged, and adding it, or changing what it counts, SHALL bump Spanish's analyser version and no other. A short Catalan or Galician line with no function word of either kind is not refused, and still reads as Spanish; nor is a Galician line whose only Galician function words are `da` and `das`.

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

#### Scenario: Spanish's « gives » is not Galician
- **WHEN** "¿Cuánto se da de propina en España?" and "¿Te das cuenta de la hora que es?" are offered as Spanish
- **THEN** both are Spanish, although `da` and `das` are Galician function words, and a Spanish analysis counts them

#### Scenario: `da` beside another Galician function word
- **WHEN** a block read as Spanish holds `da` beside another Galician function word, as "O profesor Smith é recoñecido por ser un dos máis grandes eruditos da filoloxía inglesa." does beside `máis`
- **THEN** `da` counts for Galician, outnumbering `por`, and the block is not Spanish

#### Scenario: Catalan and Occitan are refused as before
- **WHEN** a block that the Catalan comparison, or *Occitan is not read as Spanish*, refuses is offered as Spanish
- **THEN** it is still refused

#### Scenario: A Galician line whose only function word is `da`
- **WHEN** "A esperanza é a razón da vida." is offered as Spanish
- **THEN** it is not refused, as this requirement states
