# lingua-stats — learning aggregates: server

## ADDED Requirements

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
