## MODIFIED Requirements

### Requirement: No network requests
The extension SHALL issue no network request unless the reader has signed in or turned on extended translation, and SHALL then issue only the requests those choices need.
- **Signed in:** the requests of signing in, and those to the Cymbra backend for the account and
  the synchronisation of what `lingua-privacy` lists as synced.
- **Extended translation on:** the download of the model from Cymbra's static host, once per model
  version, and nothing else.

Page text SHALL NOT be part of any request. The pack, the glosses and the engine are part of the
extension, so reading, highlighting, the word popup and — once the model is on the device —
translation SHALL work with no network connection.

#### Scenario: Working offline
- **WHEN** the user reads an already-loaded page with no network connection
- **THEN** highlighting and the word popup (gloss included) work in full

#### Scenario: No account and no extended translation
- **WHEN** a reader who is not signed in and has not turned on extended translation reads pages for a day
- **THEN** the extension has made no network request

#### Scenario: Translating offline
- **WHEN** a reader whose model is on the device selects a phrase with no network connection
- **THEN** the sentence is translated
