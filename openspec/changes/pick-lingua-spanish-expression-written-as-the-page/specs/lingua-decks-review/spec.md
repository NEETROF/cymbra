## ADDED Requirements

### Requirement: Review reads a Spanish expression card's gloss under its own headword
When review reads the pack's gloss of a Spanish card at the key its lemma reads as (*Review finds a Spanish expression card by its name*), and several headwords reach that key (*A Spanish pack keeps every headword of a key several reach*), it SHALL show the gloss of the headword whose name is the card's lemma, and the key's own gloss where none is. A card whose lemma is itself a key SHALL be read at that key, as before. Every other card SHALL be looked up as before, and the card's stored gloss and label SHALL NOT be rewritten. For a Spanish card, this rule SHALL take the place of the sentence of *Review reads a French expression card's gloss under its own headword* by which every other card is looked up as before, every other rule of that requirement still applying.

#### Scenario: A card on the headword that does not win the key
- **WHEN** a French-native engine holding es-fr, whose key `hacer tiempo` holds `hacer tiempo`, which wins it, and `hace tiempo`, reviews the Spanish card `hace tiempo`, whose gloss is in English
- **THEN** the view's gloss is es-fr's gloss for `hace tiempo`, and the card keeps its English text and label

#### Scenario: A card on the headword that wins the key
- **WHEN** the same engine reviews the Spanish card `hacer tiempo`, whose gloss is in English
- **THEN** the view's gloss is es-fr's gloss for `hacer tiempo`, as before

#### Scenario: A card named by no headword of the key
- **WHEN** the same engine, whose key `bueno día` holds `buen día`, which wins it, and `buenos días`, reviews the Spanish card `bueno día` made before expressions were named, whose gloss is in English
- **THEN** the view's gloss is the key's own gloss, `buen día`'s, as before
