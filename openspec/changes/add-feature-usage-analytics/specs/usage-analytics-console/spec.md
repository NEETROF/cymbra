## ADDED Requirements

### Requirement: Usage reporting screen

The back-office SHALL provide a "Usage" screen, accessible to authorised admin
users, that reports feature-usage metrics from the analytics aggregate tables.
The screen MUST follow the back-office architecture rules: it MUST NOT call the
API directly (only a Pinia store / composable does, behind the injectable client
seam) and its async state MUST be a single `ts-pattern` discriminated union.

#### Scenario: Authorised admin opens the screen

- **WHEN** an authorised admin navigates to the Usage screen
- **THEN** the screen loads usage metrics for the default period

#### Scenario: Unauthorised user is denied

- **WHEN** a user without the required admin scope attempts to open the Usage screen
- **THEN** access is refused and no usage data is shown

### Requirement: Unique users by period and platform

The screen SHALL report the number of distinct users over a user-selected period,
broken down by `platform` and by `device_class`. The distinct-user count MUST be
exact within a period (derived from `analytics.usage_user_daily`, not by summing daily
distinct counts).

#### Scenario: Distinct users for a selected period

- **WHEN** the admin selects a date range
- **THEN** the screen shows the distinct-user count for that range, split by
  platform and device class

#### Scenario: Multi-day user counted once

- **WHEN** a user was active on several days of the selected range
- **THEN** that user contributes exactly one to the distinct-user total

### Requirement: Action breakdown with free-form filtering

The screen SHALL report which actions were performed over a selected period and
SHALL allow the admin to filter and combine freely across the available dimensions
(date range, platform, device class, action). Filters MUST be composable in any
combination.

#### Scenario: Filter by a single dimension

- **WHEN** the admin filters to a single platform
- **THEN** the action breakdown reflects only events from that platform

#### Scenario: Combine multiple filters

- **WHEN** the admin applies a date range, a device class, and an action together
- **THEN** the reported figures reflect the intersection of all three filters

### Requirement: Data-driven action filter list

The screen SHALL populate its action filter from the actions actually present in
the data (the distinct actions in the aggregates), NOT from a hard-coded list, so
newly introduced actions appear as filter options without a back-office redeploy.

#### Scenario: New action appears as a filter option

- **WHEN** the client starts emitting a new action and it lands in the aggregates
- **THEN** that action becomes selectable in the screen's action filter with no
  back-office code change

### Requirement: Aggregate-backed queries with raw fallback

The screen SHALL answer historical queries from the permanent aggregate tables and
MAY read `analytics.usage_events` directly only within the retention window for
maximum filtering flexibility. Queries outside the retention window MUST still be
answerable from the aggregates for the dimensions those aggregates retain.

#### Scenario: Query beyond retention window

- **WHEN** the admin selects a period older than the raw retention window
- **THEN** the screen still returns results computed from the aggregate tables

#### Scenario: Recent query uses raw detail

- **WHEN** the admin selects a period inside the retention window
- **THEN** the screen MAY use raw events to satisfy filter combinations the
  aggregates do not pre-compute
