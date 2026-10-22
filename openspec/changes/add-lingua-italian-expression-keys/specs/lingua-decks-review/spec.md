# Spec Delta

## ADDED Requirements

### Requirement: Review finds an Italian expression card by its name
When review shows the pack's gloss for a card studying Italian whose lemma holds a space and whose gloss is in another language than the engine's native language (*Review shows a gloss the reader can read*), it SHALL read the pack's expression table at the card's lemma and, when that lemma is no key, at the key the lemma — the expression's name — reads as, written as *An Italian pack keys its expressions as Italian is read* writes a key; where several headwords reach that key, it SHALL show the gloss of the headword whose name is the card's lemma, and the key's own gloss where none is. Every other card SHALL be looked up as before, and the card's stored gloss and label SHALL NOT be rewritten. For an Italian card, this rule SHALL take the place of the sentences of *Review finds a French expression card by its name*, *Review finds a Spanish expression card by its name*, *Review reads a French expression card's gloss under its own headword* and *Review reads a Spanish expression card's gloss under its own headword* that look every other card up as before, every other rule of those requirements still applying.

#### Scenario: A named Italian expression card glossed in another language
- **WHEN** a card `al dente` made by an engine glossing in French is reviewed by an engine glossing in English, with an it-en pack keying `al dente` as `a il dente`
- **THEN** review shows it-en's gloss of `al dente`, read at `a il dente`

#### Scenario: A card on the headword that does not win the key
- **WHEN** a card `vale la pena` is reviewed in another language, the key `valere la pena` being won by `valere la pena`
- **THEN** review shows `vale la pena`'s own gloss

#### Scenario: Every other card
- **WHEN** an English, Spanish or French card, or an Italian card on a single word, is reviewed
- **THEN** it is looked up as before
