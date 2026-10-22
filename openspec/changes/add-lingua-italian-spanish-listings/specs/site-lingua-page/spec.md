# Spec Delta

## MODIFIED Requirements

### Requirement: The coverage table reads at a phone's width
While every shipped pair is glossed in one language, the Lingua page's coverage table SHALL keep one column per pair. Once pairs of two or more native languages are listed, it SHALL have one row per pair, named "<studied> → <native>", the page's readers' pairs first, with one column per size of the commonest-words list; and the page SHALL NOT scroll sideways at a phone's width, the table scrolling within its own box where it is wider than the text.

#### Scenario: Today's pairs
- **WHEN** Cymbra Lingua ships en-fr and es-fr, and no other pair
- **THEN** the table has one column per pair, as before

#### Scenario: Six pairs on a phone
- **WHEN** the six pairs ship, and no other pair, and the English page is shown 375 pixels wide
- **THEN** the table has six rows, Spanish → English and French → English first, and the page does not scroll sideways

#### Scenario: Seven pairs on a phone
- **WHEN** it-fr ships beside the six pairs, and no other pair, and the French page is shown 375 pixels wide
- **THEN** the table has seven rows, English → French, Spanish → French and Italian → French first, and the page does not scroll sideways

#### Scenario: Eight pairs on a phone
- **WHEN** it-en ships beside the seven pairs, and no other pair, and the English page is shown 375 pixels wide
- **THEN** the table has eight rows, Spanish → English, French → English and Italian → English first, and the page does not scroll sideways

#### Scenario: Nine pairs on a phone
- **WHEN** en-it ships beside the eight pairs, and no other pair, and the Italian page is shown 375 pixels wide
- **THEN** the table has nine rows, English → Italian first, and the page does not scroll sideways

#### Scenario: Nine pairs on a phone, it-es the ninth
- **WHEN** it-es ships beside the eight pairs, and no other pair, and the Spanish page is shown 375 pixels wide
- **THEN** the table has nine rows, English → Spanish, French → Spanish and Italian → Spanish first, and the page does not scroll sideways

#### Scenario: Ten pairs on a phone, en-it and it-es
- **WHEN** en-it and it-es ship beside the eight pairs, and no other pair, and the Spanish page is shown 375 pixels wide
- **THEN** the table has ten rows, English → Spanish, French → Spanish and Italian → Spanish first, and the page does not scroll sideways
