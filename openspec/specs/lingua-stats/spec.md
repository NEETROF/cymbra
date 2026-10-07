# lingua-stats Specification

## Purpose
TBD - created by archiving change add-lingua-connected-clients. Update Purpose after archive.
## Requirements
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

### Requirement: Aggregates per day, per language and per device
`StatsService` SHALL store learning aggregates keyed by (UTC day, studied language, device) — exposures, words learned, reviews done — pushed by idempotent upsert from each device; no fine-grained timestamped event SHALL be stored server-side, day × language being the finest grain allowed.

#### Scenario: Two devices active on the same day
- **WHEN** the user does 20 reviews on their Mac and 10 on their iPhone on the same day
- **THEN** each device upserts its own aggregate row and the consolidated read for that day reports 30 reviews

#### Scenario: Replayed upsert without double counting
- **WHEN** a device re-pushes the aggregate row for a day it has already sent
- **THEN** the row is replaced (upsert by key), never added to itself

### Requirement: Consolidated multi-device read
`StatsService` SHALL expose a read of consolidated aggregates — summed across devices, as series per day and per language, over a requested date range — for the authenticated user only (never another account's stats).

#### Scenario: A thirty-day series
- **WHEN** a client requests the last 30 days of stats for English
- **THEN** the response contains at most one value per day and per measure, summed across every device on the account

### Requirement: The language of a daily stat is normalised on receipt
The language of a daily aggregate SHALL be normalised on receipt with the same rule as statuses and cards (trimmed, lowercased, primary subtag, empty read as `en`), so that one language never splits into several series.

#### Scenario: A regional code on a stat
- **WHEN** a device upserts its daily row with language `es-MX`
- **THEN** the row is stored and consolidated under `es`

### Requirement: A device's daily statistics carry their language
A device SHALL send one daily statistic per day and studied language, each with its language, instead of sending every count as English.

#### Scenario: A day of English and Spanish
- **WHEN** a device that read English and Spanish today syncs
- **THEN** it sends an English statistic and a Spanish statistic for today, each with its own counts

