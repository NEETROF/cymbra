## MODIFIED Requirements

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
- **THEN** the locale sent is `es`, the account's e-mails come in Spanish, and the deletion link opens the Spanish page

#### Scenario: An Italian browser
- **WHEN** a reader who chose French as their language signs up from an Italian browser
- **THEN** the locale sent is `it`, Music's Italian e-mails stay Italian, and the deletion link opens the French page

#### Scenario: A browser in a language Cymbra does not speak
- **WHEN** a reader who chose English as their language signs up from a German browser
- **THEN** the locale sent is `en`
