# Spec Delta

## ADDED Requirements

### Requirement: Catalan, Galician, Latin and the regional languages of Italy are not read as Italian
The core SHALL refuse as Italian a block the detector reads as Italian whose function words of Catalan, Galician, Latin and the regional languages of Italy the guard knows — Sicilian, Neapolitan, Venetian, Sardinian, Friulian and Corsican, each kept by measurement —, counted together, outnumber its Italian ones, a neighbour's function word counting only where it is written in lowercase, a Neapolitan elided article counting with its apostrophe. A refused block SHALL be excluded from an Italian analysis and SHALL NOT vote for Italian when a document's language is chosen. A tie, or a block with no such function word, SHALL stay Italian. The guard SHALL run for Italian alone, so that English's, Spanish's and French's detection and analyser versions do not move. Italian's analyser version SHALL be bumped with the guard.

#### Scenario: A Catalan paragraph
- **WHEN** "Les tradicions de la ciutat són molt antigues i el carrer principal és ple de gent." is offered as Italian
- **THEN** it is not Italian, and an Italian analysis excludes it

#### Scenario: A Galician paragraph
- **WHEN** "Os veciños da vila non están de acordo coas novas normas, xa que non hai tempo." is offered as Italian
- **THEN** it is not Italian

#### Scenario: A Latin paragraph
- **WHEN** "Gallia est omnis divisa in partes tres, quarum unam incolunt Belgae." is offered as Italian
- **THEN** it is not Italian

#### Scenario: A Sicilian paragraph
- **WHEN** "Lu patri nun vulìa ca li figghi jissiru a la scola ogni jornu." is offered as Italian
- **THEN** it is not Italian

#### Scenario: A Neapolitan paragraph
- **WHEN** "'O sole mio sta 'nfronte a te, e nun ce sta niente 'e cchiù bello." is offered as Italian
- **THEN** it is not Italian, its elided articles counting with their apostrophe

#### Scenario: A Venetian paragraph
- **WHEN** "El canal xe pieno de barche e la zente ghe va par vardar." is offered as Italian
- **THEN** it is not Italian

#### Scenario: Italian naming Catalan, Sicilian and Latin places and works is kept
- **WHEN** "Abbiamo visitato Sant Joan de les Abadesses, poi Lo Capo di Siracusa, e letto il De bello Gallico." is offered as Italian
- **THEN** it is Italian

#### Scenario: Regional Italian is kept
- **WHEN** "Ma va', non ci credo mica che sei andato giù in piazza con quel caldo." and "Dai, vieni su che ti faccio vedere una cosa." are offered as Italian
- **THEN** both are Italian

#### Scenario: A capital makes a name
- **WHEN** a block read as Italian writes a neighbour's function word with a capital, as « El Niño » or « Los Angeles » do
- **THEN** that word does not count against Italian

#### Scenario: A Catalan page for a reader of French and Italian
- **WHEN** a page whose blocks are Catalan and Galician, read as Italian by the detector, is offered with French and Italian as candidates
- **THEN** its blocks give Italian no vote

#### Scenario: English, Spanish and French do not move
- **WHEN** the six invariance baselines run after the guard is added
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and English's, Spanish's and French's analyser versions are unchanged

#### Scenario: The leak the guard leaves
- **WHEN** a short neighbour line with no function word of either kind, such as "Bona nit." or "Arrivederci Roma.", is offered as Italian
- **THEN** the guard does not refuse it, as this requirement states

#### Scenario: What the Italian baseline shows
- **WHEN** the Italian baseline analyses its `misto` page after this change
- **THEN** its Catalan, Galician and Latin blocks are excluded, its Italian blocks counted, and the analysis reports a version above the one change 7 gave
