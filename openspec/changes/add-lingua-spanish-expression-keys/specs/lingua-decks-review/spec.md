## ADDED Requirements

### Requirement: Review finds a Spanish expression card by its name
When review shows the pack's gloss for a card studying Spanish whose lemma holds a space and whose gloss is in another language than the engine's native language (*Review shows a gloss the reader can read*), it SHALL read the pack's expression table at the card's lemma and, when that lemma is no key, at the key the lemma — the expression's name — reads as, written as *A Spanish pack keys its expressions as Spanish is read* writes a key. Every other card SHALL be looked up as before, and the card's stored gloss and label SHALL NOT be rewritten.

#### Scenario: A named Spanish expression card glossed in another language
- **WHEN** a French-native engine holding es-fr reviews the Spanish card `tener en cuenta`, whose gloss is in English
- **THEN** the view's gloss is es-fr's expression gloss for the key `tener en contar`, and the card keeps its English text and label

#### Scenario: A card made before names
- **WHEN** a French-native engine holding es-fr reviews the Spanish card `tener en contar`, whose gloss is in English
- **THEN** the view's gloss is es-fr's expression gloss for that key, as before

#### Scenario: French expression cards do not move
- **WHEN** an English-native engine holding the fr-en fixture reviews the French card `au revoir`, whose gloss is in Spanish
- **THEN** the view's gloss is the fixture's gloss for the key `à le revoir`, as before
