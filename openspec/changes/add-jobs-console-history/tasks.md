## 1. Admin SQL functions and grants (`backend/jobs/migrations`)

- [ ] 1.1 Migration `0019_admin_job_history.sql`, re-runnable (`CREATE OR REPLACE FUNCTION`, `CREATE INDEX IF NOT EXISTS`). Add the index `job_attempts_name_finished_idx ON job_attempts (job_name, finished_at) WHERE finished_at IS NOT NULL`
- [ ] 1.2 `SECURITY DEFINER` functions, all `STABLE` with `SET search_path = jobs` and `REVOKE EXECUTE … FROM PUBLIC`:
  - `admin_list_attempts(p_from, p_to, p_name, p_outcome, p_limit, p_offset)`: finished attempts only, ordered `finished_at DESC, id DESC`;
  - `admin_count_attempts(p_from, p_to, p_name, p_outcome)`;
  - `admin_period_stats_by_kind(p_from, p_to, p_name)`: one grouped aggregate per source (attempts, `dead_letter`, `cancellations`), combined with `UNION ALL`, only kinds with activity, ordered by kind;
  - `admin_list_schedules()`: `name, kind, cron_expr, timezone, enabled`, with no `payload_json`.
- [ ] 1.3 Grant the four functions to `jobs_admin_svc` in a conditional `DO` block, when the role exists
- [ ] 1.4 `backend/db/init/roles.sql.tpl` and `backend/deploy/provision-jobs-admin-role.sql`: grant the new functions in their own block, guarded by `to_regprocedure('jobs.admin_list_attempts(timestamptz, timestamptz, text, text, integer, integer)')`. Update the comments that say "four functions" and the jobs-console section of `DEPLOY.md`
- [ ] 1.5 Integration tests (`#[ignore]`, `backend/jobs/tests/admin_queue_test.rs`, unique job names):
  - history order and paging;
  - kind and outcome filters;
  - `running` excluded;
  - the per-kind rows add up to `admin_period_stats` (dead-lettered and cancelled rows included);
  - the schedule list returns no payload;
  - `jobs_admin_svc` is refused on `job_attempts`, `schedules` and `dead_letter`, and can execute the four new functions.

## 2. Contract (`backend/jobs-admin/proto/jobs_admin.proto`, additive only)

- [ ] 2.1 Add `rpc AdminListJobHistory`, the `AttemptOutcome` enum (`ATTEMPT_OUTCOME_UNSPECIFIED`, `_SUCCEEDED`, `_FAILED`, `_ABANDONED`), and the messages `AdminListJobHistoryRequest` (`window`, `kind`, `outcome`, `limit`, `offset`), `FinishedAttempt` (`job_id`, `kind`, `channel`, `outcome`, `attempt`, `started_at_ms`, `finished_at_ms`, `optional duration_ms`) and `AdminListJobHistoryResponse` (`attempts`, `total`). Nothing in them carries a payload or error text
- [ ] 2.2 Add `KindPeriodStats` (`kind`, `PeriodStats period`, `optional last_finished_at_ms`) and `AdminGetJobStatsResponse.by_kind = 4`. Add `JobSchedule` (`name`, `cron`, `timezone`, `enabled`) and `JobKind.schedules = 4`
- [ ] 2.3 `buf breaking` against `main` passes with no breaking marker

## 3. Admin module (`backend/jobs-admin`)

- [ ] 3.1 `admin_core.rs`:
  - `HistoryOutcome` with `as_db`/`from_db` and the proto-filter mapping;
  - `AttemptRow` → `FinishedAttempt` shaping, with no run time for `abandoned`;
  - the `KindPeriodStats` row;
  - `job_kinds(schedules)`, which joins the registry with the schedules and drops unknown kinds.

  Unit tests for each
- [ ] 3.2 `admin.rs`:
  - `JobsAdminRepo` gains `history`, `history_count` and `schedules`;
  - `period_stats` returns the totals and the per-kind rows together;
  - `JobsAdminModule::history` validates the window and page before touching the repo;
  - `kinds()` becomes `async` and fallible.

  Unit tests use the generated `MockJobsAdminRepo`: invalid window or page never reaches the repo, the kind filter is trimmed, a total past the last page is kept, and the schedules are joined
- [ ] 3.3 `pg_admin.rs`:
  - call the new functions;
  - read `admin_period_stats` and `admin_period_stats_by_kind` in one `REPEATABLE READ READ ONLY` transaction.
- [ ] 3.4 `admin_grpc.rs`:
  - `AdminListJobHistory` behind the same `global_admin` gate;
  - outcome proto ↔ domain mapping, with an unknown value rejected as `INVALID_ARGUMENT`;
  - `by_kind` and `schedules` filled.

  Unit tests for the gate on the new RPC and for the argument errors

## 4. Back office (`apps/back-office`)

- [ ] 4.1 `yarn gen` regenerates `jobs_admin_pb`
- [ ] 4.2 `lib/cadence.ts`: pure cron parser that returns a discriminated union (`hourly` / `daily` / `custom`, plus the paused state when every schedule is disabled). Vitest covers all ten seeded crons, a non-UTC timezone and the raw fallback
- [ ] 4.3 `stores/jobs.ts`:
  - new state: `history: Async<HistoryPage>`, `historyParams` (outcome, offset), a history window frozen between reloads, and `tab`;
  - new actions: `setTab` (loads the history lazily), `setHistoryFilters` and `goToHistoryPage`;
  - `JobStats.byKind` and `JobKindInfo.schedules`;
  - `setFilters`, `setPeriod`, `refresh` and `cancel` re-read the active list and the stats;
  - the step-back-on-empty-page logic is reused for the history.

  Vitest over the fake client
- [ ] 4.4 `components/JobKindBreakdown.vue` (props and events only):
  - every registered kind, merged with `byKind`, zero rows included;
  - a cadence badge on each kind;
  - "breakdown unavailable" when the totals are non-zero and `byKind` is empty;
  - selecting a row emits the kind.
- [ ] 4.5 `components/JobHistoryTable.vue` (props and events only):
  - columns: job id, kind with its cadence badge, channel, outcome tag, attempt, finish time (relative, with the absolute time in the title), run time;
  - outcome filter;
  - `TablePager`;
  - empty state.
- [ ] 4.6 `views/JobsView.vue`:
  - a Queue / History `tablist`;
  - the breakdown under the period cards, where selecting a kind sets the kind filter and opens History;
  - cadence badges in the kind filter options and on queue rows;
  - the history-start note on both tabs;
  - auto-refresh re-reads the active tab.
- [ ] 4.7 `i18n/locales/en.json` and `fr.json`, aligned: tab labels, history columns, outcomes, cadence templates (`hourly at :{mm}`, `daily at {hh}:{mm} ({tz})`, raw, paused, on demand), breakdown headings, "breakdown unavailable"
- [ ] 4.8 `lib/e2e-seam.ts` jobs fake: a history fixture with mixed outcomes over more than one page, schedules on some kinds, `by_kind` derived from the fixture. `e2e/jobs.spec.ts`:
  - history paging and the outcome filter;
  - a zero row for a scheduled kind that did not run;
  - hourly and daily badges, and "On demand";
  - selecting a breakdown row opens the filtered history;
  - a `music/admin` still has no entry and is redirected.

## 5. Verification

- [ ] 5.1 Rust:
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo llvm-cov --workspace --fail-under-lines 80` with the shared ignore regex;
  - the `admin_queue_test` integration tests against local Postgres.
- [ ] 5.2 Back office: `yarn gen`, `yarn typecheck`, `yarn lint`, `rtk proxy yarn format:check`, `yarn test`, `yarn e2e`
- [ ] 5.3 Dogfood on the local stack, signed in as a `global/admin`:
  - the History tab lists the real hourly runs (`session_reap`, `orphan_reap`, …) with their run time;
  - the breakdown matches the cards;
  - the nightly kinds show "daily at …" and a zero row when the period excludes their run;
  - changing a schedule's cron in `jobs.schedules` shows the new cadence after a reload.
- [ ] 5.4 `openspec validate add-jobs-console-history --strict`
