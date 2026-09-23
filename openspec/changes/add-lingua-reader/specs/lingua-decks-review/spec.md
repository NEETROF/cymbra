## MODIFIED Requirements

### Requirement: Card schema with provenance
A card SHALL carry at minimum: the lemma, the encountered form, the originating context
sentence, the source of the encounter (a page URL, a book's title and section, or an agent
session identifier, with a timestamp), the gloss, and an optional media slot (`media`, with
`source: capture|stock|generated` and `sync_policy`) — not populated in this change but
present in the schema. Multi-word expressions SHALL be cards in their own right.

#### Scenario: Card created from the popup
- **WHEN** the user adds `conundrum` to the deck from a web page
- **THEN** the card holds the lemma, the complete originating sentence and the page URL

#### Scenario: Card created from a book
- **WHEN** the user adds a word to the deck from a section of a book in the extension's reader
- **THEN** the card holds the lemma, the complete originating sentence, and the book's title and section title as its source

#### Scenario: Expression card
- **WHEN** the user captures the selection "compelling starting point" with the keyboard shortcut
- **THEN** an expression card is created with the originating sentence
