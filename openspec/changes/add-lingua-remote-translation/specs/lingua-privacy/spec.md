## MODIFIED Requirements

### Requirement: Lingua's privacy disclosures match what it collects
Cymbra's published privacy policy (French and English) SHALL describe Lingua in its own annex:
- **Stays on the device:** page text and its analysis, reading exposures, and a card's page address — page text SHALL be qualified as leaving the device only when the reader has chosen remote translation, in which case the annex SHALL state that the sentence is translated and not kept.
- **Synced for a signed-in reader:** statuses, level, deck without addresses, daily stats, and a random installation identifier.
- **Removal:** the Lingua-only erasure and account deletion.

The site's account deletion page SHALL state that the account serves every Cymbra app and SHALL mention the Lingua-only erasure. The App Store privacy answers for Cymbra Lingua SHALL be recorded with the Apple app and SHALL declare no browsing history and no tracking. Where a store listing or the annex states that page text never leaves the device, it SHALL be corrected to name remote translation as the one exception, and SHALL say that it is off by default and chosen per device.

#### Scenario: Reading the policy
- **WHEN** a reader opens `https://cymbra.app/confidentialite/` or `https://cymbra.app/en/privacy/`
- **THEN** a Lingua annex lists what is synced, what stays on the device and how to erase it

#### Scenario: Reading what remote translation sends
- **WHEN** a reader opens the annex while deciding whether to turn on remote translation
- **THEN** it states that the selected sentence is sent to Cymbra, translated, and kept nowhere, and that the setting is off by default

#### Scenario: Filling the App Store privacy form
- **WHEN** the App Store Connect privacy form for Cymbra Lingua is filled from the recorded answers
- **THEN** it declares e-mail address, user ID, device ID, other user content and product interaction, all linked to the user, with no browsing history and no tracking
