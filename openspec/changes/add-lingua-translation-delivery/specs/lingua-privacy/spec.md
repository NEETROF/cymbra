## ADDED Requirements

### Requirement: The model download carries nothing of the reader's
The model download SHALL request only the model's files at their fixed addresses, and SHALL carry no page text, no Lingua data, no account token and no installation identifier.
Cymbra SHALL NOT associate a download with an account or an installation. The Lingua annex of the
privacy policy (French and English) SHALL state that turning on extended translation downloads a
translation model from Cymbra once, and that translation then runs on the device. Where a store
listing says the extension makes no network request without an account, it SHALL say instead that
it makes none without an account and without extended translation; where it says the text of the
pages read never leaves the device, that SHALL stay unconditional.

#### Scenario: Inspecting the download
- **WHEN** the requests made while the model downloads are inspected
- **THEN** each asks for one model file at a fixed address, with no cookie, token, identifier or page text

#### Scenario: Reading the listing
- **WHEN** a reader reads the Chrome Web Store or AMO listing
- **THEN** it says no network request is made without an account and without extended translation, and that page text never leaves the device
