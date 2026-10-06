## ADDED Requirements

### Requirement: A Spanish voice from Spain by default
For Spanish, the automatic voice choice SHALL prefer a voice of Spain to the voices of other regions within the same quality, on every browser, Firefox for Android's three-letter regions included. The extension SHALL offer no accent setting: the reader SHALL choose another voice in Réglages.

#### Scenario: A voice of Spain listed after a Mexican one
- **WHEN** the browser lists a Mexican Spanish voice before a voice of Spain, both of the same quality
- **THEN** the automatic choice for Spanish is the voice of Spain

#### Scenario: Firefox for Android
- **WHEN** Firefox for Android lists `spa-MEX` before `spa-ESP`
- **THEN** the automatic choice for Spanish is the `spa-ESP` voice

#### Scenario: A voice the reader downloaded
- **WHEN** the only enhanced Spanish voice is Mexican, and the voice of Spain is an ordinary one
- **THEN** the automatic choice is the enhanced Mexican voice
