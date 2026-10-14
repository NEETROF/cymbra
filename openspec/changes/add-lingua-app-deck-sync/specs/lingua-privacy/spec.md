## MODIFIED Requirements

### Requirement: Lingua's privacy disclosures match what it collects
Cymbra's published privacy policy (French, English and Spanish) SHALL describe Lingua in its own annex:
- **Stays on the device:** page text and its analysis, reading exposures, and a card's page address — except the sentence a reader sends with « Traduction en ligne », and the translation of a word the reader chose to keep as its card's gloss.
- **Sent only on request:** with « Traduction en ligne » ticked, the sentence holding a selection, or a review card's sentence, is sent, when the reader presses the button, to Cymbra and from Cymbra to Google, or to Anthropic when Google is unavailable, to be translated; neither Cymbra nor the providers keep it, and the providers do not train on it. The setting is off by default and chosen per device. In the Cymbra Lingua app this needs the reader to sign in inside the app.
- **Downloaded on request:** the extended translation's models, by the extension and, once the reader accepts it there, by the Cymbra Lingua app, from Cymbra's model host, carrying no page text, no Lingua data and no account token.
- **Synced for a signed-in reader:** statuses, level, deck without addresses — each card's gloss being the dictionary's or a machine translation of the word that the reader chose to keep, never a translated sentence — daily stats, and a random installation identifier. The extension synchronises them; once the reader signs in inside the Cymbra Lingua app with the account the extension uses, the app also synchronises the statuses and cards of the reviews made in it, under its own session and its own installation identifier, while daily stats still go through the extension.
- **Removal:** the Lingua-only erasure and account deletion.

The site's account deletion page SHALL state that the account serves every Cymbra app and SHALL mention the Lingua-only erasure. The App Store privacy answers for Cymbra Lingua SHALL be recorded with the Apple app and SHALL declare no browsing history and no tracking. Where a store listing states that page text never leaves the device, it SHALL name « Traduction en ligne » as the one exception and SHALL say that it is off by default. The interface SHALL NOT name the providers; the annex SHALL.

#### Scenario: Reading the policy
- **WHEN** a reader opens `https://cymbra.app/confidentialite/`, `https://cymbra.app/en/privacy/` or `https://cymbra.app/es/privacidad/`
- **THEN** a Lingua annex lists what is synced, what stays on the device and how to erase it

#### Scenario: Reading what online translation sends
- **WHEN** a reader follows the link in the setting's note to the annex
- **THEN** it states that the selected sentence is sent only when the button is pressed, names Google and Anthropic as the providers, says nobody keeps it, and says the setting is off by default

#### Scenario: Reading what a kept translation is
- **WHEN** a reader reads the annex's deck row
- **THEN** it states that a card's gloss may be a machine translation of the word the reader chose to keep, that it is synced with the card, and that a translated sentence is never kept

#### Scenario: Reading what the app synchronises
- **WHEN** a reader reads the annex's synced row
- **THEN** it states that the Cymbra Lingua app, once the reader signs in inside it, synchronises the statuses and cards of its reviews under its own session, with no page address, and that daily stats still go through the extension

#### Scenario: Filling the App Store privacy form
- **WHEN** the App Store Connect privacy form for Cymbra Lingua is filled from the recorded answers
- **THEN** it declares e-mail address, user ID, device ID, other user content and product interaction, all linked to the user, with no browsing history and no tracking
