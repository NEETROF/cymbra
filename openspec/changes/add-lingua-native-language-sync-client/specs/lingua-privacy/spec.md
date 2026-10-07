## ADDED Requirements

### Requirement: The disclosures name the languages a card and a day carry
Before the release that sends them, the Lingua annex of the published privacy policy, in French and English, SHALL say that a synced card carries its translation and the language of that translation, and that a day's statistics carry the reader's native language; the App Store privacy answers recorded with the Apple app SHALL name both, under the categories already declared, with no new category and no new linkage.

#### Scenario: Reading the policy
- **WHEN** a reader opens the Lingua annex of `https://cymbra.app/confidentialite/` or `https://cymbra.app/en/privacy/`
- **THEN** the deck row names the translation's language, and the statistics row names the native language

#### Scenario: The App Store answers
- **WHEN** the App Store Connect privacy form for Cymbra Lingua is filled from the recorded answers
- **THEN** it declares the same categories and linkage as before, and the recorded answers' « What it is in Lingua » column names the gloss language and the native language
