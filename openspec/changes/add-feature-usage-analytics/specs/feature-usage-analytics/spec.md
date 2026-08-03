## ADDED Requirements

### Requirement: Curated action taxonomy

The system SHALL maintain the set of trackable actions as a single client-owned
registry (one shared constant, not scattered string literals). The `action` field
SHALL travel as a **shape-validated string** (bounded length, restricted charset,
e.g. lower `snake_case`), NOT a frozen wire enum, so new actions can be added by a
normal client release without a coordinated backend deploy. The backend SHALL
accept any well-formed action string and MUST reject only malformed ones; it MUST
NOT reject an action merely for being previously unseen.

#### Scenario: Well-formed action is accepted

- **WHEN** the client reports an event whose action is a well-formed string,
  including one the backend has never seen before
- **THEN** the backend accepts and stores it

#### Scenario: Malformed action is rejected

- **WHEN** the client reports an event whose action violates the shape rules (too
  long, or containing disallowed characters)
- **THEN** the backend rejects that event without persisting it and the rest of the
  batch is processed normally

### Requirement: Batched event ingestion

The system SHALL expose a batched `ReportEvents` RPC that accepts one or more
usage events in a single call. Each event MUST carry `action`, `platform`,
`device_class`, `app_version`, `locale`, and a client-supplied `occurred_at`
timestamp. Each event MAY additionally carry an optional `subject_id` (a
high-cardinality reference such as the score UUID) and an optional `variant` (a
low-cardinality, shape-validated qualifier such as the play mode or the settings
category — never a setting value). Ingestion MUST be authenticated and MUST be
idempotent-safe for client retries (a retried batch MUST NOT corrupt aggregates).

#### Scenario: Batch is ingested

- **WHEN** an authenticated client sends a batch of valid events to `ReportEvents`
- **THEN** each valid event is persisted as one row in `usage_events` with its
  dimensions preserved

#### Scenario: Unauthenticated call is refused

- **WHEN** an unauthenticated caller invokes `ReportEvents`
- **THEN** the call is rejected and nothing is persisted

#### Scenario: Malformed event does not fail the batch

- **WHEN** a batch contains a mix of valid events and one event missing a required
  dimension
- **THEN** the valid events are persisted and the malformed event is skipped

#### Scenario: Optional context fields are preserved

- **WHEN** an event carries a `subject_id` (e.g. a score UUID) and/or a `variant`
  (e.g. `fall_note`)
- **THEN** both are persisted on the raw event; `variant` also participates in the
  action aggregate grain while `subject_id` remains only in raw

#### Scenario: Variant carries category not value

- **WHEN** a `settings_change` event is reported after the user changes a setting
- **THEN** the event's `variant` records the setting category (e.g. `piano_type`)
  and no setting value is recorded anywhere

### Requirement: Platform and device-class dimensions

The system SHALL record, for every event, the originating `platform` (one of
`ios`, `android`, `macos`, `windows`, `linux`, `web`) and a `device_class`
(one of `phone`, `tablet`, `desktop`). The client SHALL derive both values;
the backend SHALL reject events whose `platform` or `device_class` is outside the
allowed sets.

#### Scenario: Device class is derived on the client

- **WHEN** the app runs on a tablet-sized device
- **THEN** events it reports carry `device_class = tablet`

#### Scenario: Out-of-range dimension is rejected

- **WHEN** an event arrives with a `platform` value outside the allowed set
- **THEN** the backend rejects that event

### Requirement: Period-salted pseudonymous identity

The system SHALL identify the reporting user by a `user_bucket` computed as a
salted hash of the account `user_id`, where the salt rotates each period (monthly).
The raw `user_id` MUST NOT be stored on any usage row. The scheme MUST allow
counting distinct users within a period but MUST NOT allow linking the same user
across two different periods.

#### Scenario: Same user within a period maps to one bucket

- **WHEN** a user reports events on two different days of the same month
- **THEN** both events resolve to the same `user_bucket`

#### Scenario: Same user across periods is unlinkable

- **WHEN** the same user reports events in two different months
- **THEN** the two `user_bucket` values differ and cannot be correlated to the same
  identity

#### Scenario: Raw identity is never persisted

- **WHEN** any usage row is written
- **THEN** it contains a `user_bucket` and no plaintext `user_id`

### Requirement: Tiered event storage

The system SHALL persist raw events in `analytics.usage_events` (one row per event,
all dimensions) and MUST maintain two permanent aggregate tables:
`analytics.usage_action_daily` keyed by `(day, action, variant, platform,
device_class, app_version, locale)` holding an event count, and
`analytics.usage_user_daily` keyed by `(day, user_bucket, platform, device_class)`
holding per-user daily presence.
The aggregate tables MUST enable exact distinct-user counts over an arbitrary
window without reading raw events.

#### Scenario: Raw event is queryable within retention

- **WHEN** an event was ingested within the retention window
- **THEN** it can be retrieved from `analytics.usage_events` with all its dimensions

#### Scenario: Distinct users over a multi-day window are exact

- **WHEN** a user is present on several days of a period
- **THEN** counting that user once over the whole period is possible from
  `analytics.usage_user_daily` (a `COUNT(DISTINCT user_bucket)`), not by summing daily
  distinct counts

### Requirement: Daily aggregation

The system SHALL run a recurring job that folds each closed day of raw events into
`analytics.usage_action_daily` and `analytics.usage_user_daily`. Re-running the job for an
already-aggregated day MUST be idempotent (it MUST NOT double-count).

#### Scenario: A closed day is aggregated

- **WHEN** the rollup job runs after a day has ended
- **THEN** that day's action counts and per-user presence appear in the aggregate
  tables

#### Scenario: Rollup is idempotent

- **WHEN** the rollup job runs twice for the same closed day
- **THEN** the aggregate rows for that day are identical to a single run

### Requirement: Retention and purge

The system SHALL treat the raw-event retention window as a configuration value
held in `cymbra-feature-flags` (default six months), editable from the back-office
without a redeploy. A recurring purge job SHALL delete raw events older than the
configured window. The purge job MUST run only after the day it would remove has
been aggregated, so no data is lost from the permanent aggregates.

#### Scenario: Old raw events are purged

- **WHEN** the purge job runs and raw events exist older than the configured
  retention window
- **THEN** those raw rows are deleted from `analytics.usage_events`

#### Scenario: Aggregates survive purge

- **WHEN** raw events for a day are purged
- **THEN** that day's rows in `analytics.usage_action_daily` and
  `analytics.usage_user_daily` remain intact

#### Scenario: Retention is reconfigurable at runtime

- **WHEN** an administrator changes the retention config value in the back-office
- **THEN** subsequent purge runs honour the new window without a client or server
  redeploy

### Requirement: Remote collection kill-switch

The system SHALL provide a kill-switch flag in `cymbra-feature-flags` that, when
disabled, stops usage collection without requiring a client release. When
collection is disabled the client MUST NOT emit events.

#### Scenario: Collection disabled remotely

- **WHEN** the collection kill-switch flag is turned off
- **THEN** clients stop emitting usage events on their next flag refresh

### Requirement: Offline-tolerant client buffering

The Flutter client SHALL buffer usage events locally and flush them to
`ReportEvents` periodically on a best-effort basis. Events produced while offline
MUST be retained and sent on a later successful flush; a failed flush MUST NOT
lose buffered events and MUST NOT surface an error to the user.

#### Scenario: Events survive an offline period

- **WHEN** the device is offline when an event is produced and later regains
  connectivity
- **THEN** the buffered event is delivered on the next successful flush

#### Scenario: Flush failure is silent

- **WHEN** a flush attempt fails
- **THEN** the buffered events are kept for retry and no error is shown to the user

### Requirement: User consent control

The system SHALL let a user turn usage collection off from the app. Collection
defaults to **opt-out** (enabled by default) under a first-party
audience-measurement posture. When a user disables collection, the client MUST
stop emitting events for that user.

#### Scenario: User opts out

- **WHEN** a user disables usage collection in settings
- **THEN** the client stops emitting usage events for that user

#### Scenario: Default posture

- **WHEN** a user has never changed the setting
- **THEN** usage collection is enabled
