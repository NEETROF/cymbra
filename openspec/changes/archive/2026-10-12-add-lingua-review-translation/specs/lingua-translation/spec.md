## ADDED Requirements

### Requirement: The review translates a card the pack cannot answer
When a card without a gloss comes up in a review — an expression, or a single word the pack does not gloss — and this device has a model ready for the card's language, the review SHALL ask the engine for the card's sentence translated with the word's place marked, as soon as the card comes up, and SHALL show that translation with the answer once revealed, labelled as a machine translation (French UI copy: « Dans votre phrase — traduction automatique ») and rendered as text. An expression whose sentence does not hold it SHALL be translated alone, under a label saying it is a machine translation of the expression (French UI copy: « Traduction automatique »); a single word SHALL never be translated alone. A card with a gloss SHALL show its gloss alone, and the engine SHALL NOT be asked for it. The translation SHALL NOT show before the reveal, SHALL NOT hold the reveal back, and SHALL be discarded with the card, never stored.

#### Scenario: An expression in review
- **WHEN** the card of « such as those made », which has no gloss, comes up and a model is ready for English
- **THEN** once the answer is revealed, it shows the card's sentence translated with the expression's place marked, under the machine-translation label, and nothing of it shows before the reveal

#### Scenario: Ready at the reveal
- **WHEN** the reader reveals the answer after the translation has arrived
- **THEN** the translation is part of the answer from that moment, with nothing added afterwards

#### Scenario: Still on its way
- **WHEN** the reader reveals the answer before the translation has arrived
- **THEN** the answer says that a translation is coming, the translation replaces that line when it lands, and if none comes the answer keeps saying the card has no translation

#### Scenario: A word the pack glosses
- **WHEN** a card with a gloss comes up
- **THEN** the engine is not asked, and the answer is the gloss alone

#### Scenario: No model
- **WHEN** extended translation is off on this device, or the models of the card's language are not on it
- **THEN** the review is as it was before the engine existed, with no line about a translation

#### Scenario: A sentence that does not hold the expression
- **WHEN** an expression's card holds a sentence the expression is not in
- **THEN** the expression is translated alone, under the label of a machine translation of the expression

#### Scenario: Another card
- **WHEN** a translation lands after the reader has answered its card
- **THEN** it is shown nowhere

#### Scenario: Never stored
- **WHEN** the reader backs the deck up, or the deck synchronises, after such a review
- **THEN** no machine translation is in the backup or in what is sent
