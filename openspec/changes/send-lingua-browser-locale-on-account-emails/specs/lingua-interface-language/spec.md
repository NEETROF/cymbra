## MODIFIED Requirements

### Requirement: The account's e-mails follow the interface language
The locale the extension sends to Cymbra ID — on sign-up, on resending the verification code, on requesting a password reset and on setting a password — SHALL be one locale on all four requests, following the interface language only once the reader has chosen their native language on this device (as add-lingua-native-language-choice records an answer; an update's mark or a new install's preset alone is no choice): until then the browser's language as the browser gives it — its whole tag, `fr` when it gives none — and the deletion link chosen by the browser's tag, as before. Once the reader has chosen, the locale SHALL be a bare primary subtag on all four requests: the interface language when it is `fr`, `en` or `es`, and the browser's language instead when the browser is in a language Cymbra speaks and the extension does not, so that the shared account locale another Cymbra app adopts is not moved from that language; and the deletion link SHALL be chosen by the interface language alone. Cymbra ID writes each of these e-mails in the account's stored language and records a request's locale only on an account that has none (`user-locale-preference`), so no request the extension sends, from any device, whether the reader chose there or not, SHALL replace a language the account has: a request's locale decides the e-mail, and is recorded, only for an account that has no language.

#### Scenario: A reader who has not chosen
- **WHEN** a reader who has not chosen their native language on this device — every reader while one native language ships, an installed extension marked as chosen by its update included — signs up from an English browser (`en-GB`), asks for a new code and for a password reset, then sets a password
- **THEN** each of the four requests carries `en-GB`, the account's e-mails come in English as before, Cymbra Music's language is not moved, and the deletion link opens the English page

#### Scenario: A choice made on another device
- **WHEN** a reader who chose English on one device asks for a password reset on another, a French browser where nothing is chosen
- **THEN** the request carries `fr-FR`, the account keeps `en`, the e-mail comes in English, and Cymbra Music keeps English

#### Scenario: A French reader with a French browser
- **WHEN** a reader who chose French as their language signs up from a French browser
- **THEN** the locale sent is `fr` (not `fr-FR`), on each of the four requests, and the deletion link opens the French page

#### Scenario: A Spanish-native reader
- **WHEN** a reader who chose Spanish as their language signs up
- **THEN** the locale sent is `es`, the account's e-mails come in Spanish, and the deletion link opens the page `lingua-privacy` gives a Spanish interface

#### Scenario: An Italian browser
- **WHEN** a reader who chose French as their language signs up from an Italian browser
- **THEN** the locale sent is `it`, Music's Italian e-mails stay Italian, and the deletion link opens the French page

#### Scenario: A browser in a language Cymbra does not speak
- **WHEN** a reader who chose English as their language signs up from a German browser
- **THEN** the locale sent is `en`

#### Scenario: An account with no language
- **WHEN** a reader whose account has no stored language — created with Google, its password set before Cymbra ID recorded a language for it, never opened in Cymbra Music — asks for a password reset from a French browser (`fr-FR`) where nothing is chosen
- **THEN** the request carries `fr-FR`, the e-mail comes in French where it came in English, the account keeps `fr-FR` from then on, and Cymbra Music's interface is not moved

#### Scenario: A choice does not move an account's language
- **WHEN** a reader whose account's language is English chooses French on this device and asks for a password reset there
- **THEN** the request carries `fr`, the account keeps `en`, and the e-mail comes in English
