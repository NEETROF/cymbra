## 1. Queue history, admin functions and grants (`backend/jobs/migrations`)

- [x] 1.1 Migration `0018_job_attempts_and_admin.sql`: `jobs.job_attempts` (outcome CHECK, index on `(job_id)` and `(finished_at)`) and `jobs.cancellations` (index on `cancelled_at`)
- [x] 1.2 `jobs.admin_queue` view with the D2 state rules and the single `RUNNING_GRACE` constant (a one-row SQL function the sweep also reads)
- [x] 1.3 `SECURITY DEFINER` functions `admin_list_jobs`, `admin_queue_counts`, `admin_period_stats` and `admin_cancel` (lock, protected kinds, running check, record, relink the ordered chain, `mq_delete`), all with `SET search_path = jobs` and `REVOKE EXECUTE … FROM PUBLIC`; also revoke `PUBLIC` on `jobs.enqueue`
- [x] 1.4 Conditional grants to `jobs_admin_svc` (USAGE on schema, EXECUTE on the four functions) when the role exists
- [x] 1.5 `backend/db/init/roles.sql.tpl` + `00-roles.sh`: create `jobs_admin_svc` (`CYMBRA_JOBS_ADMIN_DB_ROLE`/`_PASSWORD`, dev default), keep it out of `public`
- [x] 1.6 `backend/deploy/provision-jobs-admin-role.sql` (idempotent, password via psql variable, grants on the functions when they exist) + a DEPLOY.md section (`provision-optional-modules.sh` models one role + one schema, which this role is not)
- [x] 1.7 Integration tests (`#[ignore]`, `backend/jobs/tests/admin_queue_test.rs`): one fixture per state, cancel outcomes (`cancelled`/`gone`/`running`/`protected`), ordered-chain relink (A running, B cancelled, C still blocked on A), period figures, `jobs_admin_svc` refused on `mq_payloads`/`mq_msgs`/`enqueue`

## 2. Attempt tracking in the worker

- [x] 2.1 `cymbra-jobs`: `attempt.rs` (pure outcome mapping + `AttemptOutcome`) and engine glue `begin_attempt` (lock, skip when gone, abandon stale, insert running) / `finish_attempt` / `tracked`
- [x] 2.2 `backend/worker/src/handlers.rs`: every `#[sqlxmq::job]` body wrapped in `tracked`; a job gone at begin returns `Ok` without running
- [x] 2.3 Unit test pinning the worker registry's job names to `cymbra_jobs::registry::builtin()`
- [x] 2.4 `dead_letter_sweep`: skip exhausted messages with a `running` attempt younger than `RUNNING_GRACE`; close attempts past grace and attempts whose message vanished as `abandoned`; `prune_history` deletes `job_attempts`/`cancellations` older than 90 days in bounded batches on the sweep's cadence
- [x] 2.5 Integration tests (`backend/jobs/tests/attempt_tracking_test.rs`): success / failure / cancelled-before-start through the wrapper; reclaim closes the crashed attempt; sweep leaves a running final attempt alone and dead-letters it past grace; prune keeps the retention window

## 3. Registry

- [x] 3.1 `JobSpec::cancellable` (default `true`; `false` for `purge_user`, `purge_score_object`, `purge_soundfont_object`) + `protected_kinds()` + unit tests

## 4. Admin module and contract (new crate `backend/jobs-admin`)

- [x] 4.1 `proto/jobs_admin.proto` (`cymbra.jobs.v1.JobsAdminService`: `AdminListJobs`, `AdminGetJobStats`, `AdminListJobKinds`, `AdminCancelJob` with the `CancelOutcome` enum; no payload/error fields) + `build.rs` (`build_client(false)`) + tonic/prost deps
- [x] 4.2 `admin_core.rs`: state/kind filter parsing, page clamp (`limit ∈ [1,100]`), window validation (`from < to ≤ now` with a clock-skew clamp, `from ≥ now − 90d`), cancellability per row; unit tests
- [x] 4.3 `admin.rs`: consumer-declared `JobsAdminRepo` port (`#[cfg_attr(test, mockall::automock)]`) + `JobsAdminModule`; unit tests with the generated mock (the protected kinds travel with every cancel, outcomes mapped, invalid input never reaches the repo)
- [x] 4.4 `pg_admin.rs`: Postgres adapter calling the `jobs.admin_*` functions
- [x] 4.5 `admin_grpc.rs`: global-admin gate, actor from the interceptor identity, proto ↔ domain mapping; unit tests for the gate and argument errors

## 5. Server wiring and CI plumbing

- [x] 5.1 `backend/platform/src/config.rs`: optional `jobs_admin_database_url` (`CYMBRA_JOBS_ADMIN_DATABASE_URL`) + config test
- [x] 5.2 `backend/server/src/main.rs`: mount `JobsAdminService` behind the strict interceptor when the URL is set; log when disabled
- [x] 5.3 `buf.yaml` module `backend/jobs-admin/proto`; `.github/coverage-ignore-regex.txt` covers the jobs generated package; `back-office-check.yml` watches the new proto; `backend/.env.example`, `backend/deploy/.env.prod.example`, `backend-it.yml` env

## 6. Back office (`apps/back-office`)

- [x] 6.1 `tool/gen_proto.sh` compiles `jobs_admin.proto`; `lib/transport.ts` adds the `jobs` client
- [x] 6.2 `stores/jobs.ts`: `page`/`stats`/`kinds` as `Async<T>`, params (state, kind, offset) and period (preset or custom), `cancel` handling with localized toasts per outcome, reload after cancel; unit tests (vitest) over a fake client
- [x] 6.3 `views/JobsView.vue`: queue `StatCards`, period selector + period `StatCards`, filters, table, `TablePager`, Cancel via `ConfirmDialog` (only on cancellable rows), Refresh + opt-in 15 s auto-refresh paused while the dialog is open or the tab hidden
- [x] 6.4 Router `/jobs` (`meta: { admin: true, adminScope: "global" }`), nav entry + icon for global admins
- [x] 6.5 `i18n/locales/en.json` + `fr.json` (`nav.jobs`, `jobs.*`: states, outcomes, period presets, validation, empty state, history-start note), aligned
- [x] 6.6 `lib/e2e-seam.ts` jobs fake (cancel mutates fixtures) + `e2e/jobs.spec.ts`: global admin sees cards/rows/pagination, cancel flow updates row + cancelled figure, running/protected rows without Cancel, music admin has no entry and is redirected

## 7. Verification

- [x] 7.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov` with the shared ignore regex, jobs/auth/user integration tests against local Postgres
- [x] 7.2 Back office: `yarn gen`, `yarn typecheck`, `yarn lint`, `rtk proxy yarn format:check`, `yarn test`, `yarn e2e`
- [ ] 7.3 Dogfood on the local stack: enqueue jobs of several kinds, watch states move, cancel a ready and a blocked job, check period figures, confirm a `music/admin` is refused
- [x] 7.4 `openspec validate add-admin-jobs-console --strict`
