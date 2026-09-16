## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Global-admin-only access to the jobs console

The jobs console operations SHALL be available only to callers holding `admin` in the `global` scope. These operations are: listing queued jobs, listing finished job history, reading job statistics and their per-kind breakdown, listing job kinds with their schedules, and cancelling a job. A caller without that role MUST receive `PERMISSION_DENIED`, or `UNAUTHENTICATED` without a session, and no data. The back office SHALL show the Jobs navigation entry, and route to the console, only for such admins; the server-side check remains authoritative.

#### Scenario: Global admin reaches the console

- **WHEN** a `global/admin` opens `/jobs`
- **THEN** the Jobs page loads with the queue table and statistics

#### Scenario: Module admin is refused

- **WHEN** a `music/admin` without `global/admin` calls any jobs console operation
- **THEN** the service returns `PERMISSION_DENIED` and no job data

#### Scenario: Module admin cannot read the history

- **WHEN** a `music/admin` without `global/admin` requests the finished job history
- **THEN** the service returns `PERMISSION_DENIED` and no attempt rows

#### Scenario: Module admin sees no entry point

- **WHEN** a `music/admin` without `global/admin` is signed in to the back office
- **THEN** no Jobs navigation entry is shown and navigating to `/jobs` redirects away

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
