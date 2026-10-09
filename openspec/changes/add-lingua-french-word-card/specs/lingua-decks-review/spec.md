## ADDED Requirements

### Requirement: Review finds a French expression card named without a space
When review shows the pack's gloss for a card studying French whose gloss is in another language than the engine's native language (*Review shows a gloss the reader can read*), and whose lemma holds no space but reads, through French's reading, as two tokens or more — an expression named as the dictionary writes it, such as `d'abord` —, it SHALL read the pack's expression table at the key that lemma reads as, as *Review finds a French expression card by its name* reads a name holding a space. A lemma read as one token SHALL be looked up as a word, as before, and the card's stored gloss and label SHALL NOT be rewritten.

#### Scenario: An expression named without a space
- **WHEN** an English-native engine holding a French pack whose table holds `d'abord` reviews the French card `d'abord`, whose gloss is in Spanish
- **THEN** the view's gloss is the pack's gloss for the key `de abord`, and the card keeps its Spanish text and label

#### Scenario: A word written with an apostrophe
- **WHEN** the same engine reviews the French card `aujourd'hui`, whose gloss is in Spanish
- **THEN** the view's gloss is the pack's word gloss for `aujourd'hui`, as before
