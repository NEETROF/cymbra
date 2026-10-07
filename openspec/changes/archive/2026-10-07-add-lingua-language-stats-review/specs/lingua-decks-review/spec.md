## ADDED Requirements

### Requirement: One review queue across languages, with a language filter
A review session SHALL take the due cards of every studied language in one queue; a queue that mixes languages SHALL be ordered by due date, and a single language SHALL keep its usual order. When the reader studies several languages, the review page SHALL offer to review all of them or a single one, and its count of due cards SHALL follow the choice.

#### Scenario: A mixed queue
- **WHEN** a reader of English and Spanish starts a review with cards due in both
- **THEN** the cards come in due-date order, whatever their language

#### Scenario: Spanish only
- **WHEN** the same reader chooses Spanish in the review page
- **THEN** the count and the session hold only the Spanish cards

#### Scenario: Every reader today
- **WHEN** a reader studies English alone
- **THEN** no filter is shown, and the review is as before

### Requirement: A review card says its language
The review card SHALL carry its studied language, and the review page SHALL show it when the reader studies several languages. A review or a word learned from a card SHALL be counted in the card's language.

#### Scenario: A Spanish card in a mixed queue
- **WHEN** a Spanish card comes up in a mixed review
- **THEN** it is marked as Spanish, and grading it counts a Spanish review
