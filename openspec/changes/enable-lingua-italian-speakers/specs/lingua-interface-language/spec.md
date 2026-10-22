## MODIFIED Requirements

### Requirement: The interface language is the reader's native language, under its own key
The interface language SHALL be the reader's native language — `fr`, `en`, `es` or `it` — and SHALL be stored on the device under its own key, which the store's owner alone SHALL write — at its start when the key is absent, and after a write of the backup — from the native language as the extension serves it: the profile's, or French when no listed pair is glossed in it; a device that predates the key, whose key is absent or unknown, SHALL read `fr`. Every surface SHALL be able to read the interface language before it renders its copy, without an engine.

#### Scenario: Every installed reader
- **WHEN** a reader whose profile names no native language opens any surface
- **THEN** the surface reads `fr`, and the French interface is shown

#### Scenario: The key follows the profile
- **WHEN** the background reads or sets a profile whose native language is Spanish, with a pair glossed in Spanish listed
- **THEN** the key holds `es` once the owner has reacted to the write, and a surface opened afterwards reads `es`

#### Scenario: A surface before any engine
- **WHEN** the popup or the account page opens
- **THEN** it reads the interface language from the key before rendering its copy, with no engine loaded

#### Scenario: A restore from a file
- **WHEN** a backup whose profile names Spanish is restored from a file
- **THEN** the key holds `es` once the backup is written and the owner has reacted to it

#### Scenario: An Italian reader
- **WHEN** the background sets a profile whose native language is Italian, with en-it listed
- **THEN** the key holds `it` once the owner has reacted to the write, and a surface opened afterwards reads `it`

### Requirement: The account's e-mails follow the interface language
The locale the extension sends to Cymbra ID — on sign-up, on resending the verification code, on requesting a password reset and on setting a password — SHALL be one locale on all four requests, following the interface language only once the reader has chosen their native language on this device (as add-lingua-native-language-choice records an answer; an update's mark or a new install's preset alone is no choice): until then the browser's language as the browser gives it — its whole tag, `fr` when it gives none — and the deletion link chosen by the browser's tag, as before. Once the reader has chosen, the locale SHALL be a bare primary subtag on all four requests: the interface language when it is `fr`, `en`, `es` or `it`, and the browser's language instead when the browser is in a language Cymbra speaks and the extension does not, so that the shared account locale another Cymbra app adopts is not moved from that language; and the deletion link SHALL be chosen by the interface language alone. Cymbra ID writes each of these e-mails in the account's stored language and records a request's locale only on an account that has none (`user-locale-preference`), so no request the extension sends, from any device, whether the reader chose there or not, SHALL replace a language the account has: a request's locale decides the e-mail, and is recorded, only for an account that has no language.

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
- **THEN** the locale sent is `fr`, since the extension speaks Italian, and the deletion link opens the French page

#### Scenario: A browser in a language Cymbra does not speak
- **WHEN** a reader who chose English as their language signs up from a German browser
- **THEN** the locale sent is `en`

#### Scenario: An account with no language
- **WHEN** a reader whose account has no stored language — created with Google, its password set before Cymbra ID recorded a language for it, never opened in Cymbra Music — asks for a password reset from a French browser (`fr-FR`) where nothing is chosen
- **THEN** the request carries `fr-FR`, the e-mail comes in French where it came in English, the account keeps `fr-FR` from then on, and Cymbra Music's interface is not moved

#### Scenario: A choice does not move an account's language
- **WHEN** a reader whose account's language is English chooses French on this device and asks for a password reset there
- **THEN** the request carries `fr`, the account keeps `en`, and the e-mail comes in English

#### Scenario: An Italian-native reader
- **WHEN** a reader who chose Italian as their language signs up
- **THEN** the locale sent is `it`, the account's e-mails come in Italian, and the deletion link opens the page `lingua-privacy` gives an Italian interface
