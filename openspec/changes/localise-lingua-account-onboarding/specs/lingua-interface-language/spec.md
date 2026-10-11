## ADDED Requirements

### Requirement: The account and the onboarding speak the interface language
The account surface — its flows, errors, connected accounts and page — and the onboarding — its page, its languages and level steps and its account offer — SHALL read their copy from the catalogue's modules for the interface language, read before they render their copy and without an engine; their pages SHALL carry `lang`; a date they show SHALL be formatted for the interface language, the French as before; an error in plain words SHALL be worded in the interface language; the French SHALL be byte for byte what it was. After this change none of the account's and the onboarding's files SHALL hold a French literal or be on the lint's baseline.

#### Scenario: Every reader today
- **WHEN** a reader of French opens the account page and the onboarding
- **THEN** every text is byte for byte what it was

#### Scenario: An English-native reader signs in
- **WHEN** the interface language is English and the reader opens the account page
- **THEN** the flows, the buttons and the errors in plain words are the English catalogue's, and the page carries `lang="en"`

#### Scenario: Off the baseline
- **WHEN** the lint runs after this change
- **THEN** its baseline names none of the account's and the onboarding's files, and none of them holds a French literal

### Requirement: The account's e-mails follow the interface language
The locale the extension sends to Cymbra ID — on sign-up, on resending the verification code, on requesting a password reset and on setting a password — SHALL follow the interface language only once the reader has chosen their native language on this device (as add-lingua-native-language-choice records an answer; an update's mark or a new install's preset alone is no choice), and until then SHALL write over no account's language: the browser's language as the browser gives it — its whole tag, `fr` when it gives none — at sign-up and when setting a password, no locale on resending the code and on requesting a reset, so that Cymbra ID keeps the account's own, and the deletion link chosen by the browser's tag, as before. Once the reader has chosen, the locale SHALL be a bare primary subtag on all four requests: the interface language when it is `fr`, `en` or `es`, and the browser's language instead when the browser is in a language Cymbra speaks and the extension does not, so that the shared account locale another Cymbra app adopts is not moved from that language; and the deletion link SHALL be chosen by the interface language alone.

#### Scenario: A reader who has not chosen
- **WHEN** a reader who has not chosen their native language on this device — every reader while one native language ships, an installed extension marked as chosen by its update included — signs up from an English browser (`en-GB`), asks for a new code and for a password reset, then sets a password
- **THEN** the sign-up and the password carry `en-GB` as before, the new code and the reset carry no locale, the account's e-mails come in English as before, Cymbra Music's language is not moved, and the deletion link opens the English page

#### Scenario: A choice made on another device
- **WHEN** a reader who chose English on one device asks for a password reset on another, a French browser where nothing is chosen
- **THEN** the request carries no locale, the account keeps `en`, the e-mail comes in English, and Cymbra Music keeps English

#### Scenario: A French reader with a French browser
- **WHEN** a reader who chose French as their language signs up from a French browser
- **THEN** the locale sent is `fr` (not `fr-FR`), on each of the four requests, and the deletion link opens the French page

#### Scenario: A Spanish-native reader
- **WHEN** a reader who chose Spanish as their language signs up
- **THEN** the locale sent is `es`, the account's e-mails come in Spanish, and the deletion link opens the English page until the site has a Spanish one

#### Scenario: An Italian browser
- **WHEN** a reader who chose French as their language signs up from an Italian browser
- **THEN** the locale sent is `it`, Music's Italian e-mails stay Italian, and the deletion link opens the French page

#### Scenario: A browser in a language Cymbra does not speak
- **WHEN** a reader who chose English as their language signs up from a German browser
- **THEN** the locale sent is `en`
