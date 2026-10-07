## ADDED Requirements

### Requirement: Statistics per studied language
The extension SHALL keep the daily statistics (words read, new words met, words learned, reviews) per day and per studied language, reading statistics kept before per day only as English. When the reader studies several languages, the statistics page SHALL offer a choice of language, the reader's first language by default, and SHALL show the selected language's level ladder, estimate, marked words and daily figures.

#### Scenario: Two languages, two sets of figures
- **WHEN** a reader read Spanish and English today and opens the statistics in Spanish
- **THEN** the day's figures and the ladder are the Spanish ones

#### Scenario: Statistics kept before
- **WHEN** the extension updates on a device with daily statistics kept per day only
- **THEN** they are shown, and synced, as English

#### Scenario: Every reader today
- **WHEN** a reader studies English alone
- **THEN** no choice is shown, and the statistics are the English ones, as before
