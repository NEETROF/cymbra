## ADDED Requirements

### Requirement: Review reads a French expression card's gloss under its own headword
When review reads the pack's gloss of a French card at the key its lemma reads as (*Review finds a French expression card by its name*, and a lemma without a space that reads as an expression), and several headwords reach that key (*A French pack keeps every headword of a key several reach*), it SHALL show the gloss of the headword whose name is the card's lemma, and the key's own gloss where none is. Every other card SHALL be looked up as before, and the card's stored gloss and label SHALL NOT be rewritten.

#### Scenario: A card on the headword that does not win the key
- **WHEN** an English-native engine holding a French pack whose key `ça aller` holds `ça ira`, which wins it, and `ça va` reviews the French card `ça va`, whose gloss is in Spanish
- **THEN** the view's gloss is the pack's gloss for `ça va`, and the card keeps its Spanish text and label

#### Scenario: A card named without a space
- **WHEN** the same engine, whose pack's key `aller y` holds `allez-y`, `allons-y` and `vas-y`, reviews the French card `allons-y`, whose gloss is in Spanish
- **THEN** the view's gloss is the pack's gloss for `allons-y`

#### Scenario: A card named by no headword of the key
- **WHEN** the same engine reviews a French card whose lemma reads as `ça aller` and is neither `ça ira` nor `ça va`
- **THEN** the view's gloss is the key's own gloss, `ça ira`'s
