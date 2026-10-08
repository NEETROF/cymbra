## MODIFIED Requirements

### Requirement: The account's e-mails follow the interface language
The locale the extension sends to Cymbra ID — on sign-up, on resending the verification code, on requesting a password reset and on setting a password — SHALL be a bare primary subtag: the interface language when it is `fr`, `en` or `es`, and the browser's language instead when the browser is in a language Cymbra speaks and the extension does not, so that the shared account locale another Cymbra app adopts is not moved from that language. The deletion link SHALL be chosen by the interface language alone.

#### Scenario: A French reader with a French browser
- **WHEN** a reader whose interface language is French signs up from a French browser
- **THEN** the locale sent is `fr` (not `fr-FR`), on each of the four requests, and the deletion link opens the French page

#### Scenario: A Spanish-native reader
- **WHEN** a reader whose interface language is Spanish signs up
- **THEN** the locale sent is `es`, the account's e-mails come in Spanish, and the deletion link opens the Spanish page

#### Scenario: An Italian browser
- **WHEN** a reader whose interface language is French signs up from an Italian browser
- **THEN** the locale sent is `it`, Music's Italian e-mails stay Italian, and the deletion link opens the French page

#### Scenario: A browser in a language Cymbra does not speak
- **WHEN** a reader whose interface language is English signs up from a German browser
- **THEN** the locale sent is `en`
