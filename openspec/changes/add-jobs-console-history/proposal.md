## Why

The back-office Jobs console (`/jobs`) shows only the jobs **still in the queue**:
sqlxmq deletes a job when it completes, so finished work surfaces only as period
totals ("completed over the period"). An operator who sees jobs running "on their
own" cannot tell which kinds finished, when, or whether they were started by a
schedule or by a user action. The kind filter narrows the totals one kind at a time
and never gives a breakdown. The data to answer this already exists: every attempt
is recorded in `jobs.job_attempts` (kept 90 days), and every recurring task is a row
of `jobs.schedules`. The console does not read either of them yet.

## What Changes

- **History view** in the Jobs console: a server-paginated list of **finished
  attempts**, newest first, over the page's selected period. The period stays within
  the 90-day retention. The list can be filtered by kind and by outcome
  (`succeeded`, `failed`, `abandoned`). Each row shows the job id, kind, channel,
  outcome, attempt number, start and finish times, and the run time. Abandoned
  attempts have no run time. No payload and no error text: errors stay in logs and
  traces (Tempo), keyed by the job id the row shows.
- **Per-kind breakdown** of the period figures: completed, failed attempts,
  dead-lettered, cancelled, average run time and last finish time, per kind. It is
  returned by the existing `AdminGetJobStats` as a new additive field, from the same
  snapshot as the totals, so the rows always add up to the cards. The console lists
  every registered kind, so a scheduled kind that did **not** run in the period shows
  as a zero row.
- **Scheduled kinds are marked** with their cadence, read at request time from
  `jobs.schedules`. `AdminListJobKinds` gains an additive `schedules` list per kind.
  The console labels such kinds "Scheduled · hourly at :05" or "Scheduled · daily at
  03:30 (UTC)", and every other kind "On demand". The label appears in the kind
  filter, the breakdown, the history rows and the queue rows. A disabled schedule is
  shown as paused.
- **Contract (additive only)**: a new RPC `AdminListJobHistory` in
  `jobs_admin.proto`, a new field on `AdminGetJobStatsResponse` and a new field on
  `JobKind`. Nothing is removed or renumbered, so the `buf breaking` (FILE) gate
  passes without a breaking marker.
- **Database**: a new idempotent migration `0019` adds four `SECURITY DEFINER`
  functions (history page, history count, per-kind figures, schedule list) and one
  index on the history. It grants them to `jobs_admin_svc`. The role still has no
  table privilege. None of the functions returns a job payload, a schedule payload or
  error text. The provisioning script and the dev role bootstrap grant the same
  functions, so the grants converge whatever the order.
- The console stays **global-admin only**, with the same server-side gate on the new
  RPC.

## Capabilities

### New Capabilities

_None._ The history, the breakdown and the schedule marking extend the existing
Jobs console.

### Modified Capabilities

- `admin-jobs-console`:
  - adds the finished-attempt history, the per-kind breakdown and the
    scheduled-kind marking;
  - extends the global-admin gate to the new history operation;
  - extends the no-payload rule to history rows and schedule payloads.
- `job-infrastructure`: the narrow queue-administration role's closed list of
  functions grows to cover reading the attempt history, the per-kind figures and the
  schedule list, still without any table privilege.

## Impact

**Products**
- **Back office** (new behaviour):
  - a Queue / History switch and the history table;
  - the per-kind breakdown under the period cards;
  - schedule badges on kinds;
  - store resources, e2e seam fake and en/fr strings.

  It **consumes** the existing admin session, the global-scope route guard,
  `StatCards`, `TablePager`, the period selector and the `Async<T>` union.
- **Platform (jobs)** (new, read-only): the admin SQL functions, the migration and
  the grants. It **consumes** the existing `jobs.job_attempts`, `jobs.dead_letter`,
  `jobs.cancellations` and `jobs.schedules`. There is no new table and no change to
  the worker, the scheduler, retention or any job producer.
- **Cymbra ID / Music / Live / Lingua / site / Flutter app**: no change.

**Code**
- `backend/jobs/migrations/0019_admin_job_history.sql`: functions, index and
  conditional grants.
- `backend/jobs/tests/admin_queue_test.rs`: SQL integration tests (`#[ignore]`).
- `backend/jobs-admin`:
  - `proto/jobs_admin.proto`: the new RPC, messages and fields;
  - `admin_core.rs`: outcome vocabulary, history row shaping, kind/schedule join;
  - `admin.rs`: repo port methods and module operations;
  - `pg_admin.rs`: function calls, stats read in one snapshot;
  - `admin_grpc.rs`: the new RPC and fields.
- `backend/db/init/roles.sql.tpl` and `backend/deploy/provision-jobs-admin-role.sql`:
  grant the new functions.
- `apps/back-office`:
  - `stores/jobs.ts`;
  - `views/JobsView.vue` plus two presentational components;
  - `lib/cadence.ts` (cron → cadence label);
  - `lib/e2e-seam.ts`;
  - `i18n/locales/{en,fr}.json`;
  - `test/jobs.spec.ts` and `e2e/jobs.spec.ts`.

**Deploy**: no new role and no new environment variable. After the worker has applied
migration `0019`, re-run `provision-jobs-admin-role.sql`. This step is only needed if
the migration ran while the role was missing: the migration grants conditionally.
Rolling back means reverting the server. The functions are inert without a caller.
