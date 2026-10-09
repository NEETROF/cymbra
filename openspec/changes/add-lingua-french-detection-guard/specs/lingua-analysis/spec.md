## ADDED Requirements

### Requirement: Catalan, Occitan and Romanian are not read as French
The core SHALL refuse as French a block the detector reads as French whose Catalan, Occitan or Romanian function words outnumber its French ones, a neighbour's function word counting only where it is written in lowercase. A refused block SHALL be excluded from a French analysis and SHALL NOT vote for French when a document's language is chosen. A tie, or a block with no such function word, SHALL stay French. The guard SHALL run for French alone, so that English's and Spanish's detection, and the en-fr, es-fr, es-en and en-es output, stay byte for byte unchanged; adding it SHALL bump French's analyser version and no other. A short Catalan, Occitan or Romanian line with no function word of either kind is not refused and still reads as French, and Franco-Provençal, Picard and Walloon, which share French's own function words, are not refused either.

#### Scenario: A Catalan paragraph
- **WHEN** "Les tradicions de la ciutat són molt antigues i el carrer principal és ple de gent." is offered as French
- **THEN** it is not French, and a French analysis excludes it

#### Scenario: An Occitan paragraph
- **WHEN** "Dins la vila, los carrièrs son estrechs e las ostals son vièlhas." is offered as French
- **THEN** it is not French

#### Scenario: A Romanian paragraph
- **WHEN** "Guvernul a aprobat bugetul cu sprijinul grupurilor, dar și cu critici." is offered as French
- **THEN** it is not French

#### Scenario: French naming Catalan and Occitan places is kept
- **WHEN** "Nous avons visité Sant Joan de les Abadesses, puis Vilafranca del Penedès et Lo Pont de Montvert." is offered as French
- **THEN** it is French

#### Scenario: Regional French is kept
- **WHEN** "Pis là, i m'a dit qu'y avait pus de place pantoute." and "Septante personnes sont venues, savez-vous, une fois." are offered as French
- **THEN** both are French

#### Scenario: A capital makes a name
- **WHEN** a block read as French writes a neighbour's function word with a capital, as « El Niño » or « Los Angeles » do
- **THEN** that word does not count against French

#### Scenario: A Catalan page for a reader of English and French
- **WHEN** a page whose blocks are Catalan and Occitan, read as French by the detector, is offered with English and French as candidates
- **THEN** its blocks give French no vote

#### Scenario: English and Spanish do not move
- **WHEN** the English, Spanish, es-en and en-es invariance baselines run after the guard is added
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and English's and Spanish's analyser versions are unchanged

#### Scenario: The leak the guard leaves
- **WHEN** a short Occitan line with no function word of either kind, such as "Soi content de te veire.", is offered as French
- **THEN** the guard does not refuse it, as this requirement states
