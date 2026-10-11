## ADDED Requirements

### Requirement: An expression's card shows its whole gloss, paged
The card answering an expression SHALL show the expression's gloss as the pack holds it, never cut, paged as a word's long gloss is — pages of whole senses within 160 characters, at least one sense a page however long —, and a deck card created from it SHALL store its first page, as a word's paged card does.
A gloss that fits one page shows no paging control. A word-by-word row of the selection card still
shows an expression's first sense within 80 characters, cut after a whole word with « … », as it shows
a word's: the row is a hint, the expression's own card its answer.

#### Scenario: An expression longer than the old cut
- **WHEN** a reader of Spanish selects « sin embargo » with es-en
- **THEN** the card reads "however, notwithstanding, nevertheless, in spite of, nonetheless" on one page, with no paging control

#### Scenario: An expression longer than a page
- **WHEN** the reader opens the card of an expression whose three senses hold 300 characters together
- **THEN** the card shows its first senses, says it is on page 1 of its pages and offers the next one, each page holding whole senses

#### Scenario: Adding a paged expression to the deck
- **WHEN** the reader presses "+ Deck" on a paged expression card
- **THEN** the deck card stores the expression's first page as its gloss

#### Scenario: An expression's row
- **WHEN** a selected phrase holds an expression whose first sense runs 95 characters
- **THEN** its word-by-word row shows that sense cut after a whole word within 80 characters, ended with « … »
