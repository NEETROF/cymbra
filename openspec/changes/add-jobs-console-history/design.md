## Context

The Jobs console (archived change `add-admin-jobs-console`) is live. Its data comes
from four `SECURITY DEFINER` functions in `jobs`:

- `admin_list_jobs`: a page of the live queue;
- `admin_queue_counts`: counts per state;
- `admin_period_stats`: totals over a window;
- `admin_cancel`: cancels a queued job.

They are called by `cymbra-server` as `jobs_admin_svc`, a role with `USAGE` on the
schema, `EXECUTE` on those four functions, and no table privilege.

The data this change needs is already stored:

- **`jobs.job_attempts`** has one row per attempt: `job_id`, `job_name`,
  `channel_name`, `attempt`, `started_at`, `finished_at` and `outcome`
  (`running | succeeded | failed | abandoned`). The worker's `tracked` wrapper writes
  it around every handler, and the dead-letter sweep prunes it after 90 days. For an
  `abandoned` attempt, `finished_at` is when the crash was noticed, not when the
  handler stopped. The existing indexes are on `job_id`, on `started_at`, and a
  partial index on `finished_at`.
- **`jobs.dead_letter`** (`name`, `dead_lettered_at`; it also holds `payload_json`
  and `last_error`) and **`jobs.cancellations`** (`job_name`, `cancelled_at`,
  `cancelled_by`) feed the dead-lettered and cancelled totals.
- **`jobs.schedules`** maps a schedule `name` to a job `kind` with a `cron_expr`, a
  `timezone` and an `enabled` flag. It also holds `payload_json`. Operators may
  `UPDATE` the cadence without a redeploy. The seeds are:
  - hourly: `orphan_reap`, `session_reap`, `streak_reminder` at :05 and
    `consensus_honesty_settlement` at :15;
  - daily, UTC: `global_season_snapshot` 00:20, `usage_rollup` 03:10,
    `play_detail_prune` 03:30, `usage_purge` 03:40, `plans_reconcile` 04:10,
    `plans_withdraw` 04:40.

  The scheduler enqueues through `registry::spec(kind)`. No producer in the codebase
  enqueues any of these kinds, and no on-demand kind has a schedule. `push_dispatch`
  is documented as a kind that a feature may either enqueue or schedule, but it has
  no schedule row today.

What the console lacks is a way to read these rows. The owner's question, "which
kinds ran, when, and on their own or not", is answered by reading these rows, not by
recording anything new.

## Goals / Non-Goals

**Goals:**

- A global admin can list the attempts that finished over the selected period,
  newest first, and filter them by kind and outcome.
- A global admin sees the period figures per kind in one table, so no filtering is
  needed to learn which kinds ran.
- A global admin can tell a scheduled kind (and its cadence) from an on-demand kind
  everywhere a kind is shown.
- Least privilege and the contract's compatibility are preserved: the new access goes
  through new functions only, and the `.proto` changes are additive.

**Non-Goals:**

- New storage, and any change to the worker, the scheduler, retention or the
  producers.
- Payloads, schedule payloads or error text. Errors stay in logs and Tempo, keyed by
  the job id the history row shows (kept from the original design).
- Live push. The page keeps the refresh button and the opt-in 15 s auto-refresh.
- Listing cancellations or dead-letter moves as history events. They stay counted
  per kind (see Open Questions).
- Per-run provenance ("this run was started by the schedule"). The mark is per kind
  (D4).
- Editing schedules, running a scheduled job now, replaying dead letters, or a
  per-module view.

## Decisions

### D1 — The history lists finished attempts, not jobs

Each history row is one row of `jobs.job_attempts` whose outcome is `succeeded`,
`failed` or `abandoned`. `running` attempts belong to the queue table and are never
listed. The row shows:

- the job id, kind and channel;
- the outcome and the attempt number;
- the start and finish times;
- the run time: `finished_at − started_at` for `succeeded` and `failed`, and **unset**
  for `abandoned`, whose finish time is only when the crash was noticed.

A job that failed once and then succeeded shows as two rows, attempt 1 `failed` and
attempt 2 `succeeded`. That is the information an operator looks for after a
"failed attempts: 1" card. The final attempt of a dead-lettered job is its last
`failed` or `abandoned` row.

*Alternatives considered:*
- **One row per job**, with the last attempt's outcome. This needs a `GROUP BY job_id`
  over the window, and a job whose attempts straddle the window edge has no
  well-defined outcome. It also hides the retry the operator is asking about.
- **A unified event feed** of attempts, cancellations and dead-letter moves. It reads
  three sources, one of which (`dead_letter`) keeps a payload and error text in the
  same row, and it adds an actor column. Deferred to an open question. The per-kind
  breakdown already counts these events.

### D2 — Window, filters and pagination

- **Window.** The history uses the page's period (the same selector as the cards) and
  filters on `finished_at ∈ [from, to)`. It is validated by the existing
  `admin_core::window`: `from < to`, end not in the future (up to 5 minutes of skew is
  clamped), start within the 90-day retention. One period drives the cards, the
  breakdown and the history, so their figures agree.
- **Filters.** The kind filter is the page's existing filter (blank means every kind).
  The outcome filter applies to the history only. `UNSPECIFIED` means every finished
  outcome, and there is no `RUNNING` value.
- **Order and pages.** Rows are ordered by `finished_at DESC, id DESC`. Pages use
  offset/limit with `limit ∈ [1, 100]` (default 25), validated by `admin_core::page`.
  The total comes from a separate count function, so an offset past the end still
  reports how many rows match. This is the same shape as the queue table, so
  `TablePager` and the store's step-back-on-empty-page logic apply unchanged.
- **Stable pages.** The store computes the window once, on load, refresh, or a period
  or filter change, and reuses it when paging. New attempts finish after `to`, so they
  cannot shift the pages being browsed. A refresh recomputes a preset window, and the
  newest rows appear on page 1.

*Alternative considered:* keyset pagination on `(finished_at, id)`. It stays stable
without freezing the window, but it gives no page count and does not fit
`TablePager`. With a frozen window the list only shrinks at its old end, when rows
pass the 90-day prune, which is negligible.

### D3 — The per-kind breakdown is part of `AdminGetJobStats`

`AdminGetJobStatsResponse` gains `repeated KindPeriodStats by_kind = 4`, where
`KindPeriodStats { string kind = 1; PeriodStats period = 2; optional int64
last_finished_at_ms = 3; }` reuses the existing `PeriodStats` message. For each kind
over the window it carries:

- completed jobs;
- failed attempts;
- dead-lettered jobs;
- cancelled jobs;
- the average run time of succeeded attempts;
- the latest finish time of any finished attempt.

- **Why not a separate RPC.** The breakdown has the same window, validation, kind
  filter and refresh cycle as the totals. As a separate call it would double the
  requests per refresh, add a second store resource, and let the rows drift from the
  cards between the two calls. The current workaround, one stats call per kind, is
  what the owner complained about.
- **Rows add up to the cards.** `pg_admin` reads `admin_period_stats` and the new
  `admin_period_stats_by_kind` in one `REPEATABLE READ READ ONLY` transaction. The
  counts are additive across kinds: `completed` is `COUNT(DISTINCT job_id)`, and a
  job has exactly one kind. The per-kind average is computed per kind, never derived
  from the totals.
- **Only active kinds are returned.** The server returns only the kinds with at least
  one non-zero figure, ordered by name. The kind filter applies, so a filtered request
  returns at most one row. The view merges these rows with the kind list and shows
  **every registered kind**, with zeros where nothing happened, so a scheduled kind
  that did not run stands out. A kind that appears in the history but is no longer
  registered still gets its row.
- **SQL.** The new function takes one grouped aggregate per source (attempts grouped
  by `job_name`, `dead_letter` by `name`, `cancellations` by `job_name`), combines
  them with `UNION ALL`, and groups the result by kind. It reads no payload or error
  column.
- **Old server, new back office.** In proto3, a missing repeated field reads as empty.
  If the period totals are non-zero but `by_kind` is empty, the view shows "breakdown
  unavailable" rather than a table of zeros.

### D4 — Scheduled kinds are marked per kind, from `jobs.schedules`, at request time

`JobKind` gains `repeated JobSchedule schedules = 4`, with
`JobSchedule { string name = 1; string cron = 2; string timezone = 3; bool enabled = 4; }`.

- **Read at request time.** `AdminListJobKinds` joins the registry kinds with the rows
  of the new `jobs.admin_list_schedules()`. The table can be tuned at runtime, so a
  list compiled into the server would drift from what actually runs. The join is a
  pure function in `admin_core`. It drops schedules whose kind the registry does not
  know, because the scheduler skips those too. The RPC used to be a pure registry
  read; it now reads the database and can fail with `INTERNAL`. The store already
  holds kinds in an `Async` union.
- **A list, not a flag.** A kind can have zero, one or several schedules.
  `push_dispatch` is designed to be scheduled once per notification category.
- **Not returned.** The function returns no `payload_json` (a scheduled
  `push_dispatch` carries its message there), no `last_evaluated_at` and no
  `missed_run_policy`. The page needs none of them.
- **Cadence label, client-side.** A pure helper, `lib/cadence.ts`, parses the cron into
  a discriminated union:
  - `{ hourly, minute }` for `M * * * *`;
  - `{ daily, hour, minute, timezone }` for `M H * * *`;
  - `{ custom, cron, timezone }` for anything else.

  The view renders it with an exhaustive `ts-pattern` match and en/fr strings:
  "Scheduled · hourly at :05", "Scheduled · daily at 03:30 (UTC)", or the raw
  expression. All ten seeded schedules match the first two shapes. A kind whose
  schedules are all disabled reads "Schedule paused". A kind with no schedule reads
  "On demand".
- **Where the mark appears.** The mark is shown in the kind filter, the breakdown, the
  history rows and the **queue rows**; the queue is where the owner saw jobs "running
  on their own". The mark is derived in the view from the kinds resource, with no
  per-row server field: each list query stays a single-table read, and there is one
  source for the mark.

*Alternatives considered:*
- **A structured cadence enum in the contract.** It puts presentation vocabulary into
  the `.proto`, and every new cron shape would need a contract change. The raw cron is
  the source of truth and always renders, at worst verbatim.
- **A cron-to-text library** such as `cronstrue`. It adds a dependency for two shapes.
- **Per-run provenance.** `schedule_occurrences` does not record the job id, so it
  would need new storage at enqueue time, which is out of scope. The per-kind mark is
  exact today, since no kind is both scheduled and enqueued on demand. The risk is
  noted below.

### D5 — Least privilege: four new functions, one idempotent migration

`backend/jobs/migrations/0019_admin_job_history.sql` runs as `worker_svc` with
`search_path = jobs`. It is written to be re-runnable. It adds:

- `admin_list_attempts(p_from timestamptz, p_to timestamptz, p_name text, p_outcome text, p_limit int, p_offset int)`
  → `(job_id, job_name, channel_name, attempt, outcome, started_at, finished_at)`;
- `admin_count_attempts(p_from timestamptz, p_to timestamptz, p_name text, p_outcome text)`
  → `bigint`;
- `admin_period_stats_by_kind(p_from timestamptz, p_to timestamptz, p_name text)`
  → `(job_name, completed, failed_attempts, dead_lettered, cancelled, avg_run_ms, last_finished_at)`;
- `admin_list_schedules()` → `(name, kind, cron_expr, timezone, enabled)`.

Every function is created with `CREATE OR REPLACE FUNCTION … LANGUAGE sql STABLE
SECURITY DEFINER SET search_path = jobs`, and its `EXECUTE` is revoked from `PUBLIC`.
The two history functions always exclude `running`.

The migration also:
- adds `CREATE INDEX IF NOT EXISTS job_attempts_name_finished_idx ON job_attempts
  (job_name, finished_at) WHERE finished_at IS NOT NULL` for the kind-filtered
  history;
- grants the four functions to `jobs_admin_svc` in the same conditional `DO` block as
  0018.

The role still holds no table privilege.

**Grant convergence.** `roles.sql.tpl` and `provision-jobs-admin-role.sql` grant the
new functions in their own block, guarded by
`to_regprocedure('jobs.admin_list_attempts(timestamptz, timestamptz, text, text, integer, integer)')`.
They do not reuse the existing `admin_cancel` guard, because a database can hold the
0018 functions without the 0019 ones. The `job-infrastructure` requirement that lists
the role's functions is updated to match.

*Alternative considered:* granting `SELECT` on `job_attempts` and `schedules`. It
would be simpler, but `schedules.payload_json` would become readable, and the rule
"the console role holds no table privilege" would gain its first exception.

### D6 — Rust layering

- **`admin_core`** gains:
  - `HistoryOutcome` (`Succeeded | Failed | Abandoned`, with `as_db`/`from_db`). It is
    distinct from `cymbra_jobs::attempt::AttemptOutcome`, which has no `Abandoned`.
  - `AttemptRow` → `FinishedAttempt` shaping, including the unset run time for
    `abandoned`.
  - The `KindPeriodStats` row type.
  - `job_kinds(schedules)`, the registry × schedule join.
- **`JobsAdminRepo`** (mockall-doubled) gains `history`, `history_count` and
  `schedules`. `period_stats` returns the totals and the per-kind rows together, so
  the one-snapshot guarantee lives in the adapter.
- **`JobsAdminModule`** gains `history(...)`. `kinds()` becomes `async` and fallible.
- **`admin_grpc`** adds `AdminListJobHistory`, gated by the same `global_admin` helper.

Host tests use the generated `MockJobsAdminRepo`, following the `rust-testing` skill.
The SQL is covered by `#[ignore]` cases in `backend/jobs/tests/admin_queue_test.rs`,
which use unique job names as the existing cases do.

### D7 — Back office

- **Store (`stores/jobs.ts`).** It stays the only caller of `api().jobs`.
  - New state:
    - `history: Async<HistoryPage>`;
    - `historyParams { outcome, offset }`;
    - the frozen history window;
    - `tab: "queue" | "history"`.
  - New actions:
    - `setTab`, which loads the history lazily the first time;
    - `setHistoryFilters`, which resets the offset;
    - `goToHistoryPage`.
  - Existing actions: `setFilters` (kind), `setPeriod`, `refresh` and `cancel` re-read
    the **active** list together with the stats.
  - `JobStats` gains `byKind`, and `JobKindInfo` gains `schedules`.
- **View (`views/JobsView.vue`).**
  - A Queue / History `tablist`, using the `viewtoggle` pattern of `UsageView`.
  - Two presentational components that receive props and emit events, and never touch
    the store or the API:
    - `JobKindBreakdown.vue`, under the period cards. Selecting a kind sets the page
      kind filter and opens History.
    - `JobHistoryTable.vue`, with outcome tags (succeeded: accepted, failed: pending,
      abandoned: rejected) and `TablePager`.
  - The existing "history starts on …" note is shown on both tabs.
- **Tests.**
  - Vitest:
    - store: the frozen window while paging, the lazy tab load, the reload of the
      active list only;
    - `lib/cadence.ts`: every shape, disabled schedules, the raw fallback.
  - e2e seam: the `jobs` fake gains a history fixture with mixed outcomes over more
    than one page, schedules on some kinds, and a `by_kind` computed from its fixture.
  - `e2e/jobs.spec.ts` covers:
    - history paging and the outcome filter;
    - the zero row of a scheduled kind that did not run;
    - the cadence badges;
    - that module admins are still refused.
  - en/fr strings are added together (`test/i18n.spec.ts`).

## Risks / Trade-offs

- **A kind becomes both scheduled and on demand** (for example, a `push_dispatch`
  schedule is added). Every run of that kind would then read "Scheduled".
  → The label says the kind has a schedule, not that the run came from it. Per-run
  provenance is an open question, and would need recording the origin at enqueue
  time.
- **Auto-refresh over a 30-day window** now also runs the grouped per-kind query.
  → It is a bounded scan of an indexed table holding a few thousand rows per month,
  and runs only while an operator has opted in.
- **Back office deployed before the backend.**
  → The History tab shows a localized error (`UNIMPLEMENTED`). The breakdown reads
  "breakdown unavailable" (D3). Kinds temporarily read "On demand". The migration plan
  deploys the backend first.
- **Server up before the worker has applied 0019** (they start together).
  → The stats, kinds and history RPCs return `INTERNAL` for the few seconds of the
  worker's boot. The console is operator-only, and a refresh fixes it. No degraded
  code path is kept for this.
- **A non-UTC schedule** is labelled in its own timezone, not in the browser's.
  → The timezone is always printed next to the time.
- **Offset paging** can still shift when the 90-day prune removes the oldest rows.
  → This only affects the last page, and the store's step-back logic covers an
  emptied page.

## Migration Plan

1. Merge. The worker applies `0019` on boot and grants the functions, because
   `jobs_admin_svc` already exists in production.
2. Roll the backend (`deploy.sh <version>`: worker and server). If the role was
   provisioned after the migration, re-running `provision-jobs-admin-role.sql` grants
   the new functions. The script is idempotent.
3. Deploy the back office after the backend.
4. **Rollback.** Revert the back office and the server. The functions and the index
   are inert without a caller and can stay. No data is written by this change.

## Open Questions

- Should History also list **cancellations** (with the cancelling admin) and
  **dead-letter moves** as events, rather than only counting them per kind?
- Is **per-run provenance** wanted ("started by schedule X at its 03:10 occurrence"
  versus "enqueued on demand")? It needs a new column written at enqueue time, which
  is deliberately not part of this change.
- The earlier open questions still stand: a read-only per-module view, and a
  retention runtime flag.
