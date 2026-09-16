# admin-jobs-console Specification

## Purpose
TBD - created by archiving change add-admin-jobs-console. Update Purpose after archive.
## Requirements
### Requirement: Global-admin-only access to the jobs console

The jobs console operations SHALL be available only to callers holding `admin` in the `global` scope. These operations are: listing queued jobs, listing finished job history, reading job statistics and their per-kind breakdown, listing job kinds with their schedules, and cancelling a job. A caller without that role MUST receive `PERMISSION_DENIED`, or `UNAUTHENTICATED` without a session, and no data. The back office SHALL show the Jobs navigation entry (Administration section) and serve its route, `/admin/jobs`, only to such admins; the former `/jobs` path SHALL redirect to it. The server-side check remains authoritative.

#### Scenario: Global admin reaches the console

- **WHEN** a `global/admin` opens `/admin/jobs`, or the former `/jobs`
- **THEN** the Jobs page loads with the queue table and statistics

#### Scenario: Module admin is refused

- **WHEN** a `music/admin` without `global/admin` calls any jobs console operation
- **THEN** the service returns `PERMISSION_DENIED` and no job data

#### Scenario: Module admin cannot read the history

- **WHEN** a `music/admin` without `global/admin` requests the finished job history
- **THEN** the service returns `PERMISSION_DENIED` and no attempt rows

#### Scenario: Module admin sees no entry point

- **WHEN** a `music/admin` without `global/admin` is signed in to the back office
- **THEN** no Jobs navigation entry is shown and navigating to `/admin/jobs` redirects away

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

The jobs console operations SHALL return queue metadata only and MUST NOT return any job payload or payload-derived field (for example an email recipient or body). They also MUST NOT return a recurring schedule's payload, nor attempt error text. The rule applies equally to queue rows, history rows, statistics and the kind list. The database access used by the console MUST NOT be able to read job payloads, schedule payloads or dead-letter records directly.

#### Scenario: Listing a verification email job

- **WHEN** a `verification_email` job is listed
- **THEN** the row carries its id, kind, channel, state, attempt counts and timestamps, and no recipient address or message content

#### Scenario: History row of a failed verification email

- **WHEN** a failed `verification_email` attempt is listed in the history
- **THEN** the row carries its job id, kind, channel, outcome, attempt number, timestamps and run time, and no recipient address, message content or error text

#### Scenario: Kind list omits schedule payloads

- **WHEN** the kind list returns a kind with a schedule
- **THEN** the schedule carries its name, cron expression, timezone and enabled flag, and no payload

#### Scenario: Console database role cannot read payloads

- **WHEN** the console's database role attempts to select from the job payload table, the schedule table or the dead-letter table
- **THEN** the database refuses with a permission error

### Requirement: History of finished job attempts

The console SHALL list the job attempts that finished within the selected period as a server-paginated history, newest first by finish time, with the total number of matching attempts. The history MUST contain only attempts whose outcome is `succeeded`, `failed` or `abandoned`; running attempts belong to the queue table and MUST NOT be listed. Each row SHALL show the job id, kind, channel, outcome, attempt number, start time and finish time. Each row SHALL also show the run time: the finish time minus the start time for `succeeded` and `failed` attempts, and no run time for `abandoned` attempts. The history SHALL accept an optional kind filter, an optional outcome filter, a `limit` between 1 and 100 and an `offset`. Its period SHALL obey the same rules as the period statistics: within the 90-day retention window, not ending in the future, and starting before it ends. Any other period, and an out-of-range page size, MUST be rejected with `INVALID_ARGUMENT`. While the admin moves between pages, the console SHALL keep the period it loaded, so that attempts finishing in the meantime do not shift the pages. A refresh, a period change or a filter change SHALL load the period again.

#### Scenario: Newest finished attempts first, with total

- **WHEN** 40 attempts finished in the last 24 hours and a global admin opens the History view with "last 24 hours" and the default page size of 25
- **THEN** the history shows the 25 most recently finished attempts, newest first, with a total of 40 and a control to reach the next page

#### Scenario: A retried job shows each attempt

- **WHEN** a `verification_email` job failed on its first attempt and succeeded on its second, both within the period
- **THEN** the history shows two rows for that job id: attempt 1 `failed` and attempt 2 `succeeded`, each with its run time

#### Scenario: Filter by kind and outcome

- **WHEN** the admin filters the history on kind `plans_reconcile` and outcome `failed`
- **THEN** only failed `plans_reconcile` attempts from the period are listed, and the total reflects both filters

#### Scenario: Abandoned attempt has no run time

- **WHEN** an attempt was closed as `abandoned` after its worker crashed
- **THEN** its history row shows the outcome `abandoned` and its finish time, with no run time

#### Scenario: Running attempts are not history

- **WHEN** a job is currently running
- **THEN** its running attempt appears in the queue table and not in the history

#### Scenario: Period beyond retention is rejected

- **WHEN** the history is requested for a period starting 120 days ago
- **THEN** the service returns `INVALID_ARGUMENT`, and the console shows a localized validation message without requesting it

#### Scenario: Out-of-range page size is rejected

- **WHEN** the history operation is called with `limit` 0 or greater than 100
- **THEN** the service returns `INVALID_ARGUMENT`

#### Scenario: Paging keeps the loaded period

- **WHEN** new attempts finish while the admin moves from the first to the second history page
- **THEN** the second page continues the list the admin loaded, without repeating rows pushed down by the new attempts, and the new attempts appear on the first page after a refresh

### Requirement: Per-kind breakdown of the period figures

The console SHALL show the period figures broken down by job kind: for each kind, the jobs completed, failed attempts, jobs dead-lettered, jobs cancelled, the average run time of completed jobs, and the time of the latest finished attempt. The breakdown MUST cover the same period and the same kind filter as the period figures. For each figure except the average run time, the breakdown MUST add up to the period figure. The console SHALL list every registered job kind in the breakdown, showing zero for a kind with no activity in the period. It SHALL also list a kind that has activity in the period but is no longer registered. If the server returns no breakdown while the period figures are not zero, the console MUST show that the breakdown is unavailable rather than a table of zeros.

#### Scenario: Breakdown adds up to the period figures

- **WHEN** over the last 24 hours 30 `verification_email` jobs and 10 `session_reap` jobs completed, and the admin opens the console without a kind filter
- **THEN** the breakdown shows 30 completed for `verification_email` and 10 for `session_reap`, and the completed period figure is 40

#### Scenario: Scheduled kind that did not run is visible

- **WHEN** `plans_reconcile` has a daily schedule and no attempt of it finished in the selected period
- **THEN** the breakdown shows a `plans_reconcile` row with zero figures and no latest finish time, next to its schedule

#### Scenario: Kind filter narrows the breakdown

- **WHEN** the admin sets the kind filter to `orphan_reap`
- **THEN** the breakdown shows the `orphan_reap` row only, and its figures equal the period figures

#### Scenario: Selecting a kind opens its history

- **WHEN** the admin selects the `usage_rollup` row of the breakdown
- **THEN** the kind filter is set to `usage_rollup` and the History view opens for the same period

#### Scenario: Breakdown unavailable

- **WHEN** the statistics report 12 completed jobs in the period but no per-kind rows
- **THEN** the console states that the breakdown is unavailable instead of listing kinds with zero figures

### Requirement: Scheduled job kinds show their cadence

The console SHALL identify each job kind that has a recurring schedule, together with its cadence, and SHALL identify every other kind as on demand. The schedules MUST be read from the recurring-schedule store when the kind list is requested, so that a cadence changed by an operator is shown without a redeploy. The kind list SHALL return, for each registered kind, its schedules with their name, cron expression, timezone and enabled flag. A schedule whose kind is not registered MUST be omitted. The console SHALL render the cadence as follows:

- an hourly cron as "hourly at :MM";
- a daily cron as "daily at HH:MM" with its timezone;
- any other expression verbatim with its timezone;
- a kind whose schedules are all disabled as paused.

The mark SHALL appear in the kind filter, the per-kind breakdown, the history rows and the queue rows.

#### Scenario: Hourly kind is marked

- **WHEN** the kind list is loaded and `streak_reminder` is scheduled with `5 * * * *` in `UTC`
- **THEN** `streak_reminder` is shown as scheduled, hourly at :05, in the kind filter, the breakdown, and each of its history and queue rows

#### Scenario: Daily kind is marked with its timezone

- **WHEN** `play_detail_prune` is scheduled with `30 3 * * *` in `UTC`
- **THEN** it is shown as scheduled, daily at 03:30 (UTC)

#### Scenario: On-demand kind

- **WHEN** `verification_email` has no schedule
- **THEN** it is shown as on demand, and its history rows carry no schedule mark

#### Scenario: Cadence change is visible without a redeploy

- **WHEN** an operator changes the `orphan_reap_hourly` schedule to `30 * * * *` and the admin reloads the console
- **THEN** `orphan_reap` is shown as hourly at :30

#### Scenario: Disabled schedule is shown as paused

- **WHEN** the only schedule of `usage_purge` is disabled
- **THEN** `usage_purge` is shown with a paused schedule, not as running on its own

#### Scenario: Unusual cron is shown verbatim

- **WHEN** a kind is scheduled with `0 */6 * * 1-5` in `Europe/Paris`
- **THEN** its cadence is shown as that expression with the `Europe/Paris` timezone

#### Scenario: Schedule for an unknown kind is ignored

- **WHEN** the schedule store holds a row whose kind is not registered
- **THEN** the kind list does not include it

