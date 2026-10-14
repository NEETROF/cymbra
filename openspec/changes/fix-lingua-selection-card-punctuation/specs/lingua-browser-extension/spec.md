## ADDED Requirements

### Requirement: A selection is read without its edge punctuation
Before a captured selection opens a card, the extension SHALL trim, at each of its two ends, every character that is neither a letter, a combining mark nor a digit — punctuation, quotes, brackets, dashes, apostrophes, hyphens, symbols and whitespace. The trimmed text SHALL be what the card is headed and keyed by, the form it stores as met, what the pack and the translation are asked, and what decides between a word and an expression; the selection's place in its sentence SHALL shrink by the same characters. Characters inside the selection SHALL NOT be trimmed, so that an elision, a hyphenated compound or an inner apostrophe is read as written. A selection made of such characters only SHALL open no card. The page's own selection SHALL be left as the reader made it.

#### Scenario: An expression selected with its comma
- **WHEN** the reader selects « four-poster bed, » in « He slept in a four-poster bed, alone. » and adds it to the deck
- **THEN** the card is headed `four-poster bed`, its form met is `four-poster bed`, and its place in the sentence ends before the comma

#### Scenario: An expression the pack does not know
- **WHEN** the reader selects « stood in silence. », which the pack holds no expression for
- **THEN** the whole-selection card is headed `stood in silence`, without the full stop, and "+ Deck" stores that headword

#### Scenario: Quotes around a selection
- **WHEN** the reader selects « ‘four-poster bed,’ » in a book quoting with single marks
- **THEN** the card is headed `four-poster bed`

#### Scenario: One word with a comma
- **WHEN** the reader selects « harbour, »
- **THEN** the popup of the word `harbour` opens, as for a selection of « harbour »

#### Scenario: Apostrophes and hyphens inside are kept
- **WHEN** the reader selects « d’abord, » or « dit-il. » or « rock 'n' roll »
- **THEN** the selection is read `d’abord`, `dit-il` and `rock 'n' roll`

#### Scenario: Punctuation alone
- **WHEN** the reader selects « — » between two words
- **THEN** no card opens

### Requirement: The review's front reads a card's word without its edge punctuation
The review's front SHALL read a card's headword and form met without the characters *A selection is read without its edge punctuation* trims from the ends of a selection: it SHALL mark that trimmed form in the card's sentence, head a card whose sentence does not hold it by the trimmed headword, offer the dictionary form only when it differs from the marked form once both are trimmed, and read the trimmed form aloud. The card SHALL NOT be rewritten.

#### Scenario: A card captured with its comma
- **WHEN** the card `four-poster bed,` comes up with the sentence « He slept in a four-poster bed, alone. »
- **THEN** `four-poster bed` is marked without the comma, no dictionary form is offered beside it, and the card keeps its stored headword

#### Scenario: A card without punctuation
- **WHEN** a card whose headword holds no edge punctuation comes up
- **THEN** the front is as before
