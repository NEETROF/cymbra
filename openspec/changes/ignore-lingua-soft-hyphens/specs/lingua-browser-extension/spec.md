## ADDED Requirements

### Requirement: A soft hyphen does not cut the reader's word
The extension SHALL hand the core each block of a page or a book as written, soft hyphens (U+00AD) included, and SHALL highlight and hit-test a word through the span the core returns in that text. It SHALL extend a selection across a soft hyphen as across a letter, and SHALL read without soft hyphens every text it takes from the page itself: the selection's text, the word as written under the pointer, and the sentence with the selection's place in it. A card is headed and keyed by that text, the deck stores and syncs it, translation is sent it and read-aloud speaks it. A page or a book without soft hyphens SHALL be read, highlighted and captured exactly as before. In the scenarios, ‧ stands for a soft hyphen.

#### Scenario: A hyphenated word on a page
- **WHEN** a reader of Spanish opens a page or a book that writes « vi‧da »
- **THEN** a highlight of that word covers the whole word, and a click on any part of it opens the card of `vida`

#### Scenario: A selection that stops inside the word
- **WHEN** the reader selects « vi » in « vi‧da »
- **THEN** the selection captured is `vida`, as for any word the selection stops inside

#### Scenario: What a card keeps
- **WHEN** the reader adds to the deck a word whose sentence, in a book, holds soft hyphens
- **THEN** the card's word, form and sentence hold none, and the sentence's translation is asked of the sentence without them, the word still marked in it

#### Scenario: Nothing moves without soft hyphens
- **WHEN** a page holds no soft hyphen
- **THEN** the blocks handed to the core, the highlights and every capture are the ones made before this change
