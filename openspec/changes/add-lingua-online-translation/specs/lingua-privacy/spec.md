## MODIFIED Requirements

### Requirement: Lingua's privacy disclosures match what it collects
Cymbra's published privacy policy (French and English) SHALL describe Lingua in its own annex:
- **Stays on the device:** page text and its analysis, reading exposures, and a card's page address — except the sentence a reader sends with « Traduction en ligne ».
- **Sent only on request:** with « Traduction en ligne » ticked, the sentence holding a selection is sent, when the reader presses the button, to Cymbra and from Cymbra to Anthropic, or to Google when Anthropic is unavailable, to be translated; neither Cymbra nor the providers keep it, and the providers do not train on it. The setting is off by default and chosen per device.
- **Synced for a signed-in reader:** statuses, level, deck without addresses, daily stats, and a random installation identifier.
- **Removal:** the Lingua-only erasure and account deletion.

The site's account deletion page SHALL state that the account serves every Cymbra app and SHALL mention the Lingua-only erasure. The App Store privacy answers for Cymbra Lingua SHALL be recorded with the Apple app and SHALL declare no browsing history and no tracking. Where a store listing states that page text never leaves the device, it SHALL name « Traduction en ligne » as the one exception and SHALL say that it is off by default. The interface SHALL NOT name the providers; the annex SHALL.

#### Scenario: Reading the policy
- **WHEN** a reader opens `https://cymbra.app/confidentialite/` or `https://cymbra.app/en/privacy/`
- **THEN** a Lingua annex lists what is synced, what stays on the device and how to erase it

#### Scenario: Reading what online translation sends
- **WHEN** a reader follows the link in the setting's note to the annex
- **THEN** it states that the selected sentence is sent only when the button is pressed, names Anthropic and Google as the providers, says nobody keeps it, and says the setting is off by default

#### Scenario: Filling the App Store privacy form
- **WHEN** the App Store Connect privacy form for Cymbra Lingua is filled from the recorded answers
- **THEN** it declares e-mail address, user ID, device ID, other user content and product interaction, all linked to the user, with no browsing history and no tracking
