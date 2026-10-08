## ADDED Requirements

### Requirement: Review and statistics speak the interface language
The review — its surfaces, grades, counts and messages — and the statistics — their page, counters, ladder and notes — SHALL read their copy from the catalogue's modules for the interface language, handed by their hosts; a count SHALL take plural forms chosen for the interface language, and a number the interface language's format, the French keeping its strings and the numbers it writes today; the statistics' page SHALL carry `lang`; the French SHALL be byte for byte what it was, its no-break spaces included.

#### Scenario: Every reader today
- **WHEN** a reader of French opens the review and the statistics
- **THEN** every text is byte for byte what it was, the ladder's no-break spaces included

#### Scenario: An English-native reader's review
- **WHEN** the interface language is English and three cards remain
- **THEN** the review reads "3 cards to review", and one card "1 card to review"

#### Scenario: A Spanish-native reader's statistics
- **WHEN** the interface language is Spanish and the reader opens the statistics
- **THEN** the counters and the ladder's notes are the Spanish catalogue's, the ladder's numbers grouped as the RAE writes them; the ladder's title, the language names and the clause naming borrowed figures stay the labels module's French, which change 19 (`add-lingua-native-language-labels`) localises

#### Scenario: Off the baseline
- **WHEN** the lint runs after this change
- **THEN** none of review's and statistics' files is on the baseline, and none holds a French literal
