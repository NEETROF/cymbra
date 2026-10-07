## ADDED Requirements

### Requirement: The account and the onboarding speak the interface language
The account surface — its flows, errors, connected accounts and page — and the onboarding — its page, its languages and level steps and its account offer — SHALL read their copy from the catalogue's modules for the interface language, read before they render their copy and without an engine; their pages SHALL carry `lang`; a date they show SHALL be formatted for the interface language, the French as before; the French SHALL be byte for byte what it was. After this change no file outside the catalogue SHALL hold a French literal, and the lint's baseline SHALL be empty.

#### Scenario: Every reader today
- **WHEN** a reader of French opens the account page and the onboarding
- **THEN** every text is byte for byte what it was

#### Scenario: An English-native reader signs in
- **WHEN** the interface language is English and the reader opens the account page
- **THEN** the flows, the buttons and the errors in plain words are the English catalogue's, and the page carries `lang="en"`

#### Scenario: The baseline is empty
- **WHEN** the lint runs after this change
- **THEN** its baseline names no file, and no file outside the catalogue holds a French literal

### Requirement: The account's e-mails follow the interface language
The locale the extension sends to Cymbra ID on sign-up, and the one it chooses the deletion link by, SHALL be the interface language when it is `fr`, `en` or `es`, and SHALL be the browser's language instead when the browser is in a language Cymbra speaks and the extension does not, so that the shared account locale another Cymbra app adopts is not moved from that language.

#### Scenario: A French reader with a French browser
- **WHEN** a reader whose interface language is French signs up from a French browser
- **THEN** the locale sent is `fr`, and the deletion link opens the French page

#### Scenario: A Spanish-native reader
- **WHEN** a reader whose interface language is Spanish signs up
- **THEN** the locale sent is `es`, the account's e-mails come in Spanish, and the deletion link opens the English page until the site has a Spanish one

#### Scenario: An Italian browser
- **WHEN** a reader whose interface language is French signs up from an Italian browser
- **THEN** the locale sent is `it`, and Music's Italian e-mails stay Italian

#### Scenario: A browser in a language Cymbra does not speak
- **WHEN** a reader whose interface language is English signs up from a German browser
- **THEN** the locale sent is `en`
