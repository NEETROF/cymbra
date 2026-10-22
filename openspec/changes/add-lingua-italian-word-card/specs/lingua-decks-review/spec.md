# Spec Delta

## ADDED Requirements

### Requirement: Review finds an Italian expression card named without a space
When review shows the pack's gloss for a card studying Italian whose lemma holds no space and whose gloss is in another language than the engine's native language (*Review shows a gloss the reader can read*), it SHALL read the pack's expression table at the key the lemma reads as through Italian's reading when that reading is two tokens or more, each a lemma — the gloss of the headword the card is named by, else the key's own —, and SHALL read it as a word where the table has nothing there; a lemma Italian reads as one token SHALL be read as a word, as before. For an Italian card, this rule SHALL take the place of the sentence of *Review finds a French expression card named without a space* that reads every other card as before, every other rule of that requirement still applying.

#### Scenario: d'accordo
- **WHEN** a card `d'accordo` made in it-fr is reviewed by an English-native engine over an it-en pack keying `d'accordo` as `di accordo`
- **THEN** review shows it-en's gloss of `d'accordo`

#### Scenario: A one-token lemma
- **WHEN** a card `perché` is reviewed in another language
- **THEN** it is read as a word, as before
