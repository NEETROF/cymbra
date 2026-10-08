## ADDED Requirements

### Requirement: The annex names the languages Lingua stores
Annex B of the privacy policy, in each language it is published in, SHALL say that statuses, levels, cards and days of statistics are each stored under the studied language they belong to, SHALL name the languages they carry — a gloss's language, the day's native language — and SHALL say that the native language stays on the device and that the interface language is sent to Cymbra ID as the account's e-mail language. It SHALL describe the translation models as those of the reader's pairs, with their sizes from the catalogue, and SHALL NOT present one studied language as the only one.

#### Scenario: A reader of Spanish reads the annex
- **WHEN** a reader studying Spanish reads Annex B in French, English or Spanish
- **THEN** it says their statuses, cards and days are stored per studied language, and names the model or models their pairs download

#### Scenario: The account's language
- **WHEN** a reader reads what their Cymbra account receives from Lingua
- **THEN** the annex names the interface language, sent for the language of the account's e-mails
