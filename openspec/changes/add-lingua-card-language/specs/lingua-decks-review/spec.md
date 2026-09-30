# lingua-decks-review — a card is keyed by its studied language

## ADDED Requirements

### Requirement: A card is keyed by its studied language
A card SHALL belong to exactly one studied language, and a lemma SHALL be able to have one card per studied language: the deck, its counts and its synchronisation SHALL treat a card of one language and a card of another as distinct even when their lemmas are spelled the same.

#### Scenario: One spelling, two cards
- **WHEN** a reader studying English and Spanish holds a card for `pie` in each language
- **THEN** the two cards are scheduled, reviewed and synchronised independently, each with its own gloss and sentence
