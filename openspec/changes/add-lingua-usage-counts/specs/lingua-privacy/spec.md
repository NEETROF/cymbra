## MODIFIED Requirements

### Requirement: Lingua's privacy disclosures match what it collects
Cymbra's published privacy policy (French, English and Spanish) SHALL describe Lingua in its own annex:
- **Stays on the device:** page text and its analysis, reading exposures, and a card's page address — except the sentence a reader sends with « Traduction en ligne », and the translation of a word the reader chose to keep as its card's gloss.
- **Sent only on request:** with « Traduction en ligne » ticked, the sentence holding a selection, or a review card's sentence, is sent, when the reader presses the button, to Cymbra and from Cymbra to Google, or to Anthropic when Google is unavailable, to be translated; neither Cymbra nor the providers keep it, and the providers do not train on it. The setting is off by default and chosen per device. In the Cymbra Lingua app this needs the reader to sign in inside the app.
- **Downloaded on request:** the extended translation's models, by the extension and, once the reader accepts it there, by the Cymbra Lingua app, from Cymbra's model host, carrying no page text, no Lingua data and no account token.
- **Counted anonymously, as each browser allows:** the six steps of `lingua-usage-counts` (installed, set up, first page analysed, back on day 1, in days 7–13, in days 30–59), each sent once with the day of installation, the browser family and the extension's version; no identifier of any kind, even for a signed-in reader, and nothing about what is read; no network address kept by Cymbra, in its database, its logs or its reverse proxy's logs; each count sent as it happens, and at least once; counted by day of installation, an installation alone on its day being recognisable in the counters, its day of installation inferable; the day of installation kept on the device for at most 60 days; the counters deleted after 25 months; installations from before the first counting release never counted; the answer a reader chooses on the leaving page, counted the same way, opening that page being an ordinary visit to the site's host; on Firefox, governed by the browser's own "technical and interaction data" consent; in Safari, on by default; in Chrome and the other Chromium browsers, only after the reader agrees; turned off at any time in Settings → Data; legitimate interest (audience measurement) as the legal basis where no consent is asked, the switch as the way to object.
- **Synced for a signed-in reader:** statuses, level, deck without addresses — each card's gloss being the dictionary's or a machine translation of the word that the reader chose to keep, never a translated sentence — daily stats, and a random installation identifier.
- **Removal:** the Lingua-only erasure and account deletion.

The policy SHALL carry the Chrome Web Store Limited Use statement, naming the anonymous counts among the information it covers. The site's account deletion page SHALL state that the account serves every Cymbra app and SHALL mention the Lingua-only erasure. The App Store privacy answers for Cymbra Lingua SHALL be recorded with the Apple app and SHALL declare no browsing history and no tracking. Where a store listing states that page text never leaves the device, it SHALL name « Traduction en ligne » as the one exception and SHALL say that it is off by default. No store listing, reviewer note or App Review note SHALL say that Lingua makes no network request without an account, or that it has no analytics; each SHALL instead say that Lingua may send anonymous counts, with no identifier and nothing of what is read, and how to turn them off. The interface SHALL NOT name the providers; the annex SHALL.

#### Scenario: Reading the policy
- **WHEN** a reader opens `https://cymbra.app/confidentialite/`, `https://cymbra.app/en/privacy/` or `https://cymbra.app/es/privacidad/`
- **THEN** a Lingua annex lists what is synced, what stays on the device, what is counted anonymously, and how to erase it or turn it off

#### Scenario: Reading what online translation sends
- **WHEN** a reader follows the link in the setting's note to the annex
- **THEN** it states that the selected sentence is sent only when the button is pressed, names Google and Anthropic as the providers, says nobody keeps it, and says the setting is off by default

#### Scenario: Reading what a kept translation is
- **WHEN** a reader reads the annex's deck row
- **THEN** it states that a card's gloss may be a machine translation of the word the reader chose to keep, that it is synced with the card, and that a translated sentence is never kept

#### Scenario: Reading what the counts send
- **WHEN** a reader follows the link in the counts' note to the annex
- **THEN** it lists the six steps and the three things sent with them, says that no identifier and no address is kept, that each count leaves the device as it happens, that the counts are kept by day of installation so that an installation alone on its day can be recognised in them, that the day of installation stays on the device for at most 60 days, what opening the leaving page sends and to whom, how each browser asks, and where to turn the counts off

#### Scenario: A store description
- **WHEN** the Chrome Web Store, AMO or App Store description of Cymbra Lingua is read in French, English or Spanish
- **THEN** it says that what the reader reads never leaves the device, that Lingua may send anonymous counts with no identifier, and that a switch in Settings turns them off, and it no longer says that Lingua makes no network request without an account

#### Scenario: The App Review notes
- **WHEN** the App Review notes recorded with the Apple app are read
- **THEN** their "External services" paragraph names the anonymous counts and the switch, no longer says "no analytics", and the whole block measures at most 4,000 characters

#### Scenario: Filling the App Store privacy form
- **WHEN** the App Store Connect privacy form for Cymbra Lingua is filled from the recorded answers
- **THEN** it declares e-mail address, user ID, device ID, other user content and product interaction, all linked to the user, with no browsing history and no tracking
