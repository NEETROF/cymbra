## ADDED Requirements

### Requirement: Practice when nothing is due
When no card is due in the language under review, the review SHALL offer a practice session (French UI copy: « S'entraîner quand même »), both where it says nothing is due and at the end of a session that leaves nothing due, provided the reader has at least one reviewed card that is not retired in that language. A practice session SHALL hold at most 10 reviewed, non-retired cards, the lowest predicted recall first, whether or not they are due, and SHALL include no never-reviewed card. Its answers SHALL leave every card's schedule unchanged — due date, memory and history — while a missed card SHALL still come back in the session as in a review; marking a word known or hiding it SHALL apply as in a review. Each card SHALL say it is practice in place of the count of cards remaining (French UI copy: « Entraînement · N carte(s) »), and each card's first answer SHALL count as a review in the day's statistics.

#### Scenario: Nothing due
- **WHEN** a reader with 10 reviewed English cards, none due, opens the review in English
- **THEN** the review says nothing is due and offers to practise anyway

#### Scenario: A practice session
- **WHEN** the reader chooses to practise
- **THEN** a session holds those cards, the one they are most likely to have forgotten first, each saying it is practice

#### Scenario: The schedule stays as it was
- **WHEN** the reader answers a practice card as recalled, or as missed
- **THEN** the card's next due date, memory and review history are what they were before the session

#### Scenario: Counted in the day
- **WHEN** the reader answers five practice cards
- **THEN** the day's count of reviews grows by five

#### Scenario: Nothing to practise
- **WHEN** the reader has no reviewed card in that language, only never-reviewed or retired ones
- **THEN** the review offers no practice

#### Scenario: Cards due
- **WHEN** cards are due in the language under review
- **THEN** the review offers the review of those cards, not practice
