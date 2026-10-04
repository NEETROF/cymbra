## ADDED Requirements

### Requirement: The word card says how common its word is
The word card SHALL say how common its dictionary form is, from the pack's frequency rank, in one of five plain-language bands: among the 100, the 1,000 or the 5,000 most frequent words, beyond the 5,000, or beyond the 20,000, which includes a word the pack does not rank. The line SHALL NOT depend on the level or the calibration the reader chose. A word in the reader's deck SHALL keep saying it is being learnt, and an expression SHALL keep its own line. The rank SHALL arrive with the word's grammar, under the same bound. A card that has not received it SHALL show no frequency line.

#### Scenario: A common word for a beginner
- **WHEN** a reader at « Débutant » opens the card of `Es`, whose dictionary form *ser* ranks among the 100 commonest Spanish words
- **THEN** the card says « Très courant — parmi les 100 mots les plus fréquents. »

#### Scenario: A word the pack does not rank
- **WHEN** the reader opens the card of a word whose dictionary form the pack does not rank
- **THEN** the card says « Rare — au-delà des 20 000 mots les plus fréquents. »

#### Scenario: A word in the deck
- **WHEN** the reader opens the card of a word they are learning
- **THEN** the card says « Dans ton deck — en cours d'apprentissage. », whatever its rank

#### Scenario: No answer in time
- **WHEN** the card completes because the bound passed without the engine's answer
- **THEN** it shows no frequency line
