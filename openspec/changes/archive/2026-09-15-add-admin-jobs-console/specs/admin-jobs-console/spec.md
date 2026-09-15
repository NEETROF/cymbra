## ADDED Requirements

### Requirement: Global-admin-only access to the jobs console

The jobs console operations (listing queued jobs, reading job statistics, listing job kinds, cancelling a job) SHALL be available only to callers holding `admin` in the `global` scope. A caller without it MUST receive `PERMISSION_DENIED` (or `UNAUTHENTICATED` without a session) and no data. The back office SHALL show the Jobs navigation entry and route only to such admins; the server-side check remains authoritative.

#### Scenario: Global admin reaches the console

- **WHEN** a `global/admin` opens `/jobs`
- **THEN** the Jobs page loads with the queue table and statistics

#### Scenario: Module admin is refused

- **WHEN** a `music/admin` without `global/admin` calls any jobs console operation
- **THEN** the service returns `PERMISSION_DENIED` and no job data

#### Scenario: Module admin sees no entry point

- **WHEN** a `music/admin` without `global/admin` is signed in to the back office
- **THEN** no Jobs navigation entry is shown and navigating to `/jobs` redirects away

### Requirement: Paginated table of queued jobs

The console SHALL list the jobs currently in the queue as a server-paginated table, ordered oldest first, with the total number of matching jobs. Each row MUST show the job id, kind, channel, state (`running`, `ready`, `scheduled`, `retry_wait`, `blocked`, `exhausted`), attempts made, attempts left, enqueue time, next attempt time, the start time of a running attempt, and whether it can be cancelled. The list SHALL accept an optional state filter, an optional kind filter, a `limit` between 1 and 100 and an `offset`.

#### Scenario: First page with total

- **WHEN** 60 jobs are queued and a global admin opens the console with the default page size of 25
- **THEN** the table shows the 25 oldest jobs and the total of 60, with a control to reach the next page

#### Scenario: Next page

- **WHEN** the admin moves to the next page
- **THEN** the table shows the following jobs in queue order and the same total

#### Scenario: Filter by state and kind

- **WHEN** the admin filters on state `retry_wait` and kind `verification_email`
- **THEN** only verification-email jobs waiting for a retry are listed, and the total reflects the filter

#### Scenario: Out-of-range page size is rejected

- **WHEN** the list operation is called with `limit` 0 or greater than 100
- **THEN** the service returns `INVALID_ARGUMENT`

#### Scenario: Running job is identified as running

- **WHEN** a worker is executing a job whose earlier attempt failed
- **THEN** the job is listed as `running`, not `retry_wait`, with the start time of the current attempt

### Requirement: Queue and period statistics

The console SHALL show summary figures for the queue as it is now: the total number of queued jobs and the count per state. It SHALL also show figures over a selectable period: jobs completed, failed attempts, jobs dead-lettered, jobs cancelled, and the average run time of completed jobs. The period SHALL be selectable as the last hour, 24 hours, 7 days or 30 days, or as a custom range within the 90-day retention window. An optional kind filter SHALL apply to all figures. An inverted range, a range ending in the future, or a range starting before the retention window MUST be rejected with `INVALID_ARGUMENT`.

#### Scenario: Queue figures match the queue

- **WHEN** 3 jobs are running and 12 are ready
- **THEN** the queue figures show 15 in the queue, 3 running and 12 ready

#### Scenario: Completed jobs over the last 24 hours

- **WHEN** 40 jobs completed in the last 24 hours and 5 completed two days ago, and the admin selects "last 24 hours"
- **THEN** the completed figure is 40

#### Scenario: Changing the period updates the figures

- **WHEN** the admin switches the period from "last 24 hours" to "last 7 days"
- **THEN** the period figures are recomputed for the 7-day window, and the queue figures are unchanged

#### Scenario: Invalid custom range

- **WHEN** a custom range whose start is after its end is requested
- **THEN** the service returns `INVALID_ARGUMENT` and the console shows a localized validation message

### Requirement: Cancel a queued job from the console

The console SHALL let a global admin cancel a queued job after an explicit confirmation. The operation SHALL report exactly one outcome: `CANCELLED`, `GONE` (the job is no longer queued), `RUNNING` (refused because an attempt is executing) or `PROTECTED` (refused because the job kind is an erasure obligation). A Cancel action MUST be offered only on rows the server marks cancellable. After a cancellation the console SHALL refresh the table and the statistics and show a localized message for the outcome.

#### Scenario: Cancel a ready job

- **WHEN** a global admin cancels a `ready` verification-email job and confirms
- **THEN** the outcome is `CANCELLED`, the row disappears from the table, and the cancelled figure for the current period increases by one

#### Scenario: Dismissed confirmation does nothing

- **WHEN** the admin opens the cancel confirmation and dismisses it
- **THEN** no cancellation is requested and the job stays queued

#### Scenario: Running job offers no Cancel

- **WHEN** a job is `running`
- **THEN** its row offers no Cancel action, and a direct cancel request returns `RUNNING` with the job still queued

#### Scenario: Erasure job offers no Cancel

- **WHEN** a `purge_user` job is queued
- **THEN** its row offers no Cancel action, and a direct cancel request returns `PROTECTED` with the job still queued

#### Scenario: Job finished before the confirmation

- **WHEN** the job completes between the table load and the confirmed cancellation
- **THEN** the outcome is `GONE`, the console says the job is no longer in the queue, and the table refreshes

### Requirement: Job payloads are never exposed by the console

The jobs console operations SHALL return queue metadata only and MUST NOT return any job payload or payload-derived field (for example an email recipient or body), nor attempt error text. The database access used by the console MUST NOT be able to read job payloads.

#### Scenario: Listing a verification email job

- **WHEN** a `verification_email` job is listed
- **THEN** the row carries its id, kind, channel, state, attempt counts and timestamps, and no recipient address or message content

#### Scenario: Console database role cannot read payloads

- **WHEN** the console's database role attempts to select from the job payload table
- **THEN** the database refuses with a permission error
