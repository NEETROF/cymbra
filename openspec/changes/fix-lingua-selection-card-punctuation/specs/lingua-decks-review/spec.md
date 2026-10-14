## MODIFIED Requirements

### Requirement: Review shows a gloss the reader can read
When the card under review holds no gloss, or holds one in a language other than the engine's native language, review SHALL show the gloss of the pack held for the card's studied language — a word's gloss, or an expression's from the pack's expression table — and SHALL show the card's own text, or no gloss when it holds none, when that pack has none or none is held. The pack SHALL be read at the card's lemma without the characters that are neither letters, combining marks nor digits at its two ends, and that trimmed lemma SHALL decide whether the card is read as a word or as an expression. A card whose gloss is in the engine's native language SHALL be shown as before. The card's stored lemma, gloss and label SHALL NOT be rewritten by review, and the substitution SHALL NOT add or rename a key of the view.

#### Scenario: The English baseline
- **WHEN** the English baseline runs
- **THEN** every review view is byte for byte as pinned, except a card pinned without a gloss whose lemma the pack glosses, which now shows that gloss (the probe `review-current second` of each pair's baseline)

#### Scenario: A word glossed in another native language
- **WHEN** a French-native engine reviews a Spanish word card whose gloss is in English
- **THEN** the view's gloss is es-fr's gloss for that lemma, and the card keeps its English text and label

#### Scenario: An expression glossed in another native language
- **WHEN** a French-native engine reviews a Spanish expression card whose gloss is in English
- **THEN** the view's gloss is es-fr's expression gloss for that key

#### Scenario: A gloss the pack has not
- **WHEN** the card's lemma is not glossed by the current pack
- **THEN** the view shows the card's own text, in its language

#### Scenario: An expression card captured with its comma
- **WHEN** a French-native engine whose en-fr pack holds `four-poster bed → Lit à baldaquin` reviews the English card `four-poster bed,`, captured without a gloss
- **THEN** the view's gloss is « Lit à baldaquin », and the card keeps its lemma `four-poster bed,`, no gloss and its label

#### Scenario: A card without a gloss the pack has learnt since
- **WHEN** an English expression card captured without a gloss is reviewed on a pack whose expression table now holds its lemma
- **THEN** the view's gloss is the table's

#### Scenario: A card without a gloss the pack does not know
- **WHEN** a card captured without a gloss is reviewed and the pack holds nothing at its trimmed lemma
- **THEN** the view has no gloss, as before
