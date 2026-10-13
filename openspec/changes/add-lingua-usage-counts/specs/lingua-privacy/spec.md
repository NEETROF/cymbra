## MODIFIED Requirements

### Requirement: Lingua's privacy disclosures match what it collects
Cymbra's published privacy policy (French, English and Spanish) SHALL describe Lingua in its own annex:
- **Stays on the device:** page text and its analysis, reading exposures, and a card's page address — except the sentence a reader sends with « Traduction en ligne ».
- **Sent only on request:** with « Traduction en ligne » ticked, the sentence holding a selection is sent, when the reader presses the button, to Cymbra and from Cymbra to Google, or to Anthropic when Google is unavailable, to be translated; neither Cymbra nor the providers keep it, and the providers do not train on it. The setting is off by default and chosen per device.
- **Counted anonymously unless turned off:** the six steps of `lingua-usage-counts` (installed, set up, first page analysed, back on day 1, in days 7–13, in days 30–59), each sent once with the day of installation, the browser family and the extension's version; no identifier of any kind, even for a signed-in reader, and nothing about what is read; no network address kept by Cymbra; the day of installation kept on the device for at most 60 days; the answer a reader chooses on the leaving page, counted the same way; on by default, turned off in Settings → Data, and on Firefox through the browser's own data-collection consent; legitimate interest as the legal basis, the switch as the way to object.
- **Synced for a signed-in reader:** statuses, level, deck without addresses, daily stats, and a random installation identifier.
- **Removal:** the Lingua-only erasure and account deletion.

The policy SHALL carry the Chrome Web Store Limited Use statement. The site's account deletion page SHALL state that the account serves every Cymbra app and SHALL mention the Lingua-only erasure. The App Store privacy answers for Cymbra Lingua SHALL be recorded with the Apple app and SHALL declare no browsing history and no tracking. Where a store listing states that page text never leaves the device, it SHALL name « Traduction en ligne » as the one exception and SHALL say that it is off by default. No store listing or reviewer note SHALL say that Lingua makes no network request without an account; each SHALL instead say that, by default, Lingua sends only the anonymous counts, and how to turn them off. The interface SHALL NOT name the providers; the annex SHALL.

#### Scenario: Reading the policy
- **WHEN** a reader opens `https://cymbra.app/confidentialite/`, `https://cymbra.app/en/privacy/` or `https://cymbra.app/es/privacidad/`
- **THEN** a Lingua annex lists what is synced, what stays on the device, what is counted anonymously and how to erase it or turn it off

#### Scenario: Reading what online translation sends
- **WHEN** a reader follows the link in the setting's note to the annex
- **THEN** it states that the selected sentence is sent only when the button is pressed, names Google and Anthropic as the providers, says nobody keeps it, and says the setting is off by default

#### Scenario: Reading what the counts send
- **WHEN** a reader follows the link in the counts' note to the annex
- **THEN** it lists the six steps and the two dimensions, says that no identifier and no address is kept, that the day of installation stays on the device for at most 60 days, and where to turn the counts off

#### Scenario: A store description
- **WHEN** the Chrome Web Store, AMO or App Store description of Cymbra Lingua is read in French, English or Spanish
- **THEN** it says that what the reader reads never leaves the device, that anonymous counts are sent by default, and that a switch in Settings turns them off

#### Scenario: Filling the App Store privacy form
- **WHEN** the App Store Connect privacy form for Cymbra Lingua is filled from the recorded answers
- **THEN** it declares e-mail address, user ID, device ID, other user content and product interaction, all linked to the user, with no browsing history and no tracking
