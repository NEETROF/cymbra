## 1. Queue history, admin functions and grants (`backend/jobs/migrations`)

- [ ] 1.1 Migration `0018_job_attempts_and_admin.sql`: `jobs.job_attempts` (outcome CHECK, index on `(job_id)` and `(finished_at)`) and `jobs.cancellations` (index on `cancelled_at`)
- [ ] 1.2 `jobs.admin_queue` view with the D2 state rules and the single `RUNNING_GRACE` constant (a one-row SQL function the sweep also reads)
- [ ] 1.3 `SECURITY DEFINER` functions `admin_list_jobs`, `admin_queue_counts`, `admin_period_stats` and `admin_cancel` (lock, running check, record, relink the ordered chain, `mq_delete`), all with `SET search_path = jobs` and `REVOKE EXECUTE … FROM PUBLIC`
- [ ] 1.4 Conditional grants to `jobs_admin_svc` (USAGE on schema, EXECUTE on the four functions) when the role exists
- [ ] 1.5 `backend/db/init/roles.sql.tpl` + `00-roles.sh`: create `jobs_admin_svc` (`CYMBRA_JOBS_ADMIN_DB_ROLE`/`_PASSWORD`, dev default), keep it out of `public`
- [ ] 1.6 `backend/deploy/provision-jobs-admin-role.sql` (idempotent, password via psql variable, grants on the functions when they exist) + wire into `provision-optional-modules.sh` if its one-role-one-schema model allows, otherwise a DEPLOY.md section
- [ ] 1.7 Integration tests (`#[ignore]`, `backend/jobs/tests/admin_queue_test.rs`): one fixture per state, cancel outcomes (`cancelled`/`gone`/`running`), ordered-chain relink (A running, B cancelled, C still blocked on A), `jobs_admin_svc` refused on `mq_payloads`/`mq_msgs`, grants present when provisioned after migrating

## 2. Attempt tracking in the worker

- [ ] 2.1 `cymbra-jobs`: `attempt.rs` (pure outcome mapping + `AttemptOutcome`) and engine glue `begin_attempt` (lock, skip when gone, abandon stale, insert running) / `finish_attempt`
- [ ] 2.2 `backend/worker/src/handlers.rs`: one `tracked` helper wrapping every `#[sqlxmq::job]` body; a job gone at begin returns `Ok` without running
- [ ] 2.3 Unit test pinning the worker registry's job names to `cymbra_jobs::registry::builtin()`
- [ ] 2.4 `dead_letter_sweep`: skip exhausted messages with a `running` attempt younger than `RUNNING_GRACE`; close attempts past grace and attempts whose message vanished as `abandoned`; prune `job_attempts`/`cancellations` older than 90 days in bounded batches
- [ ] 2.5 Integration tests: success / failure / cancelled-before-start through the wrapper; sweep leaves a running final attempt alone and dead-letters it past grace

## 3. Registry

- [ ] 3.1 `JobSpec::cancellable` (default `true`; `false` for `purge_user`, `purge_score_object`, `purge_soundfont_object`) + unit tests

## 4. Admin module and contract (`backend/jobs`)

- [ ] 4.1 `proto/jobs_admin.proto` (`cymbra.jobs.v1.JobsAdminService`: `AdminListJobs`, `AdminGetJobStats`, `AdminListJobKinds`, `AdminCancelJob` with the `CancelOutcome` enum; no payload/error fields) + `build.rs` (`build_client(false)`) + tonic/prost deps
- [ ] 4.2 `admin_core.rs`: state/kind filter parsing, page clamp (`limit ∈ [1,100]`), window validation (`from < to ≤ now`, `from ≥ now − 90d`), cancellation decision for protected kinds; unit tests
- [ ] 4.3 `admin.rs`: consumer-declared `JobsAdminRepo` port (`#[cfg_attr(test, mockall::automock)]`) + `JobsAdminModule`; unit tests with the generated mock (protected kind never reaches `cancel`, outcomes mapped)
- [ ] 4.4 `pg_admin.rs`: Postgres adapter calling the `jobs.admin_*` functions
- [ ] 4.5 `admin_grpc.rs`: global-admin gate, actor from the interceptor identity, proto ↔ domain mapping; unit tests for the gate and argument errors

## 5. Server wiring and CI plumbing

- [ ] 5.1 `backend/platform/src/config.rs`: optional `jobs_admin_database_url` (`CYMBRA_JOBS_ADMIN_DATABASE_URL`) + config test
- [ ] 5.2 `backend/server/src/main.rs`: mount `JobsAdminService` behind the strict interceptor when the URL is set; log when disabled
- [ ] 5.3 `buf.yaml` module `backend/jobs/proto`; `.github/coverage-ignore-regex.txt` covers the jobs generated package; `backend/.env.example`, `backend/deploy/.env.prod.example`, `backend-it.yml` env

## 6. Back office (`apps/back-office`)

- [ ] 6.1 `tool/gen_proto.sh` compiles `jobs_admin.proto`; `lib/transport.ts` adds the `jobs` client
- [ ] 6.2 `stores/jobs.ts`: `page`/`stats`/`kinds` as `Async<T>`, params (state, kind, offset) and period (preset or custom), `cancel` handling with localized toasts per outcome, reload after cancel; unit tests (vitest) over a fake client
- [ ] 6.3 `views/JobsView.vue`: queue `StatCards`, period selector + period `StatCards`, filters, table, `TablePager`, Cancel via `ConfirmDialog` (only on cancellable rows), Refresh + opt-in 15 s auto-refresh paused while the dialog is open or the tab hidden
- [ ] 6.4 Router `/jobs` (`meta: { admin: true, adminScope: "global" }`), nav entry + icon for global admins
- [ ] 6.5 `i18n/locales/en.json` + `fr.json` (`nav.jobs`, `jobs.*`: states, outcomes, period presets, validation, empty state, history-start note), aligned
- [ ] 6.6 `lib/e2e-seam.ts` jobs fake (cancel mutates fixtures) + `e2e/jobs.spec.ts`: global admin sees cards/rows/pagination, cancel flow updates row + cancelled figure, running/protected rows without Cancel, music admin has no entry and is redirected

## 7. Verification

- [ ] 7.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov --workspace --fail-under-lines 80` with the shared ignore regex, jobs integration tests against local Postgres
- [ ] 7.2 Back office: `yarn gen`, `yarn typecheck`, `yarn lint`, `rtk proxy yarn format:check`, `yarn test`, `yarn e2e`
- [ ] 7.3 Dogfood on the local stack: enqueue jobs of several kinds, watch states move, cancel a ready and a blocked job, check period figures, confirm a `music/admin` is refused
- [ ] 7.4 `openspec validate add-admin-jobs-console --strict`
