## ADDED Requirements

### Requirement: Review and statistics speak the interface language
The review — its surfaces, grades, counts and messages — and the statistics — their page, counters, ladder and notes — SHALL read their copy from the catalogue's modules for the interface language, handed by their hosts; a count SHALL take plural forms chosen for the interface language, the French keeping its strings and raw counts; the statistics' page SHALL carry `lang`; the French SHALL be byte for byte what it was, its no-break spaces included.

#### Scenario: Every reader today
- **WHEN** a reader of French opens the review and the statistics
- **THEN** every text is byte for byte what it was, the ladder's no-break spaces included

#### Scenario: An English-native reader's review
- **WHEN** the interface language is English and three cards remain
- **THEN** the review reads "3 cards to review", and one card "1 card to review"

#### Scenario: A Spanish-native reader's statistics
- **WHEN** the interface language is Spanish and the reader opens the statistics
- **THEN** the counters and the ladder's notes are the Spanish catalogue's, with Spanish number formatting

#### Scenario: Off the baseline
- **WHEN** the lint runs after this change
- **THEN** none of review's and statistics' files is on the baseline, and none holds a French literal
