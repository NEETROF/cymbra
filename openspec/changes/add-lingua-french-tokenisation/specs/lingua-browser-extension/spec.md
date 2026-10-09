## ADDED Requirements

### Requirement: A click opens the piece of a split word under the pointer
Clicking a highlighted word whose pieces the studied language's pre-pass gave spans of their own SHALL open the card of the piece under the pointer, and a click on the boundary between two pieces SHALL open the piece that starts there. A word whose pieces share one span (`don't`, `del`, `au`) SHALL open on its first piece, and a click at the end of a word followed by a space SHALL open that word, as before.

#### Scenario: The noun after an elided article
- **WHEN** the reader clicks the left half of the first letter of « homme » in « l’homme », whose pieces `le` and `homme` have their own spans
- **THEN** the card of `homme` opens, not the card of `le`

#### Scenario: The elided article
- **WHEN** the reader clicks « l’ » in « l’homme »
- **THEN** the card of `le` opens

#### Scenario: Pieces sharing one span
- **WHEN** the reader clicks « don't », whose pieces `do` and `not` share its span
- **THEN** the card of `do` opens, as before

#### Scenario: The end of a word
- **WHEN** the caret of a click lands right after the last letter of a highlighted word followed by a space
- **THEN** that word's card opens, as before
