# lingua-stats — learning aggregates: the clients' stats screen

## ADDED Requirements

### Requirement: Stats screen in the extension and the app
The extension and the container app SHALL offer a learning stats screen — words learned, reviews done, exposures, per day and per language — fed by the consolidated server read when the user is signed in and by the device's local aggregates otherwise; the screen SHALL state the scope shown (all devices or this device) and that agent sessions (the Claude Code plugin, local-only) are not counted in it.

#### Scenario: Consolidated stats once signed in
- **WHEN** a signed-in user opens the extension's stats screen after reviewing on two devices
- **THEN** the totals shown cover both devices and the screen states the "all devices" scope

#### Scenario: Local stats without an account
- **WHEN** a user without an account opens the stats screen
- **THEN** the device's local aggregates are shown with the "this device" scope, with no network request

### Requirement: Jargon-free stats vocabulary
Stats screens and responses SHALL NOT display the term "lemma": counts of unique lemmas SHALL be labelled "distinct words" and the canonical form "dictionary form", per the product's vocabulary rule.

#### Scenario: Label for words learned
- **WHEN** the stats screen shows the week's count of words learned
- **THEN** the label uses "words" or "distinct words", and the term "lemma" appears nowhere
