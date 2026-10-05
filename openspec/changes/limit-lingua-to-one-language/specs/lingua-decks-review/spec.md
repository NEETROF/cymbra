## ADDED Requirements

### Requirement: The cards of a language no longer studied wait
A review session and its count of due cards SHALL cover only the languages the reader studies. The cards of a language the reader no longer studies SHALL stay in the deck, unchanged and outside the review, until the reader studies that language again.

#### Scenario: After a change of language
- **WHEN** a reader with English cards due changes to Spanish and opens the review
- **THEN** the review offers their Spanish cards only, and their English cards keep their schedule

#### Scenario: Back to English
- **WHEN** the same reader studies English again
- **THEN** their English cards come up in the review, due as they were

#### Scenario: Every reader today
- **WHEN** a reader who studies English alone, with English cards only, opens the review
- **THEN** the review offers the same cards, in the same order, as before
