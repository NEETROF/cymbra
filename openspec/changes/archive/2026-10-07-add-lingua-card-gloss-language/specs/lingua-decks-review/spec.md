## ADDED Requirements

### Requirement: A card says the language of its gloss
A card SHALL carry the language its gloss is written in, `fr` by default; a card created on an engine — added from a page, or seeded from the pack — SHALL take the engine's native language; the local export of card operations SHALL emit it when it is not `fr`, and the apply SHALL read it, an absent or empty value meaning `fr`, so that the exported operations of a French-glossed deck keep their bytes. The backup SHALL leave the label out when it is `fr`, so that a backup written before this requirement, and every backup whose glosses are French, keep their bytes; a card labelled other than `fr` is another language in the records, so a backup holding one SHALL be written with schema version 2 and a version 1 backup SHALL never carry a label; the newest schema version SHALL NOT change for it, and a build released before this requirement SHALL read a backup that carries it.

#### Scenario: Every card today
- **WHEN** a reader of French with 20 French-glossed cards is backed up
- **THEN** the backup is byte for byte what the previous build wrote, and every card restores with the gloss language `fr`

#### Scenario: A card created on an engine glossed in English
- **WHEN** a card is created on an engine whose packs are glossed in English
- **THEN** its gloss language is `en`, the backup carries it, and the previous build still reads the backup

#### Scenario: A card applied from a card operation
- **WHEN** a card operation with the gloss language `en` is applied, and another with none
- **THEN** the first card is labelled `en` and the second `fr`, and the export emits `en` on the first and no label on the second

### Requirement: Review shows a gloss the reader can read
When the gloss of the card under review is in a language other than the engine's native language, review SHALL show the gloss of the pack held for the card's studied language — a word's gloss, or an expression's from the pack's expression table — and SHALL show the card's own text when that pack has none or none is held. A card whose gloss is in the engine's native language SHALL be shown as before. The card's stored gloss and label SHALL NOT be rewritten by review, and the substitution SHALL NOT add or rename a key of the view.

#### Scenario: The English baseline
- **WHEN** the English baseline runs
- **THEN** every review view is byte for byte as pinned

#### Scenario: A word glossed in another native language
- **WHEN** a French-native engine reviews a Spanish word card whose gloss is in English
- **THEN** the view's gloss is es-fr's gloss for that lemma, and the card keeps its English text and label

#### Scenario: An expression glossed in another native language
- **WHEN** a French-native engine reviews a Spanish expression card whose gloss is in English
- **THEN** the view's gloss is es-fr's expression gloss for that key

#### Scenario: A gloss the pack has not
- **WHEN** the card's lemma is not glossed by the current pack
- **THEN** the view shows the card's own text, in its language
