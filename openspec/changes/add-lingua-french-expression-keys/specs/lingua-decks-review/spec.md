## ADDED Requirements

### Requirement: Review finds a French expression card by its name
When review shows the pack's gloss for a card studying French whose lemma holds a space and whose gloss is in another language than the engine's native language (*Review shows a gloss the reader can read*), it SHALL read the pack's expression table at the key that lemma — the expression's name — reads as, written as *A French pack keys its expressions as French is read* writes a key. Every other card SHALL be looked up as before, and the card's stored gloss and label SHALL NOT be rewritten.

#### Scenario: A French expression card glossed in another language
- **WHEN** an English-native engine holding a French pack whose table holds `au revoir` reviews the French card `au revoir`, whose gloss is in Spanish
- **THEN** the view's gloss is the pack's gloss for the key `à le revoir`, and the card keeps its Spanish text and label

#### Scenario: Spanish expression cards do not move
- **WHEN** a French-native engine reviews a Spanish expression card whose gloss is in English
- **THEN** the view's gloss is es-fr's expression gloss for the card's lemma, as before
