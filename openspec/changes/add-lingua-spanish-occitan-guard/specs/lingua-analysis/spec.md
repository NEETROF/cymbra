## ADDED Requirements

### Requirement: Occitan is not read as Spanish
The core SHALL refuse as Spanish a block the detector reads as Spanish whose Occitan function words outnumber its Spanish ones that Occitan does not write — every Spanish function word of the guard but `lo`, `los`, `las` and `sus`, which Occitan writes too — an Occitan function word, its elided `qu'`, `m'` and `t'` among them, counting only where it is written in lowercase. A refused block SHALL be excluded from a Spanish analysis and SHALL NOT vote for Spanish when a document's language is chosen. A tie, or a block with no Occitan function word, SHALL stay Spanish unless the Catalan or Galician comparison refuses it, and every block that comparison refuses SHALL stay refused. The comparison SHALL leave English's and French's detection, and the en-fr and en-es output, byte for byte unchanged; adding it SHALL bump Spanish's analyser version and no other. A short Occitan line with no Occitan function word is not refused and still reads as Spanish, and a page some of whose Occitan blocks are not refused can still be chosen as Spanish.

#### Scenario: An Occitan sentence whose articles are Spanish's
- **WHEN** "Los dròlles son totjorn dins lo jardin." is offered as Spanish
- **THEN** it is not Spanish, although `los` and `lo` are Spanish function words, and a Spanish analysis excludes it

#### Scenario: An Aranese sentence
- **WHEN** "Era hemna qu'ei arribada damb eth tren de Tolosa." is offered as Spanish
- **THEN** it is not Spanish

#### Scenario: Regional Spanish is kept
- **WHEN** "Vos sos de Buenos Aires, ¿no? Che, ¿querés tomar unos mates?" and "Los chiquillos se fueron a la playa en la guagua y comieron papas arrugadas con mojo." are offered as Spanish
- **THEN** both are Spanish

#### Scenario: Spanish naming Occitan and Catalan places is kept
- **WHEN** "El ayuntamiento de Vielha e Mijaran aprobó el presupuesto de las pistas de Baqueira." and "La candidatura de Junts pel Sí ganó las elecciones." are offered as Spanish
- **THEN** both are Spanish

#### Scenario: A capital makes a name
- **WHEN** a block read as Spanish writes an Occitan function word with a capital, as "Los comercios de Pas de la Casa abren los domingos." does
- **THEN** that word does not count against Spanish, and the block is Spanish

#### Scenario: Catalan and Galician are refused as before
- **WHEN** a block that the Catalan or Galician comparison refuses is offered as Spanish
- **THEN** it is still refused

#### Scenario: An Occitan page for a reader of English and Spanish
- **WHEN** a page whose blocks are Occitan, read as Spanish by the detector, is offered with English and Spanish as candidates
- **THEN** its blocks give Spanish no vote

#### Scenario: English and French do not move
- **WHEN** the English, en-es and French invariance baselines run after the comparison is added
- **THEN** the en-fr and en-es probes are byte for byte the output recorded before, without re-blessing, no French analysis moves, and English's and French's analyser versions are unchanged

#### Scenario: The leak the comparison leaves
- **WHEN** a short Occitan line with no Occitan function word, such as "Lo libre es sus la taula.", is offered as Spanish
- **THEN** it is not refused, as this requirement states
