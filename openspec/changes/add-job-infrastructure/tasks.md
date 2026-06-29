## 1. Spike: transactional enqueue (do first — riskiest assumption)

- [x] 1.1 POC: from a `auth_svc`-role transaction, `sqlxmq` `spawn` a job into the shared `jobs` schema and commit/rollback atomically with an `auth` write; confirm the row appears iff committed — asserted by `jobs/tests/spike_transactional_enqueue.rs` (`#[ignore]`, backend-it); enqueue uses the `jobs.enqueue` `SECURITY DEFINER` wrapper, not raw `spawn` (see design spike finding #2)
- [x] 1.2 Resolve `search_path` / schema qualification for the `mq_*` tables under each module role; confirm a module role with only `GRANT INSERT` can enqueue but cannot read another schema — resolved to `EXECUTE` on `jobs.enqueue` only (write-only, no `mq_*` read); asserted by the same spike test
- [x] 1.3 Decision checkpoint: if the spike fails, record the fallback (per-module queue tables or outbox) in design.md before proceeding — spike held (sqlxmq↔sqlx 0.8 compat confirmed); D3 refined to a SECURITY DEFINER enqueue wrapper; findings recorded in design.md

## 2. Foundation: schema, role, engine seam

- [x] 2.1 Migrations for a `jobs` schema: `sqlxmq` `mq_*` objects, `jobs.schedules`, `jobs.retry_policy`, and a dead-letter table — `backend/jobs/migrations/0001..0006`; vendored sqlxmq migrations + `jobs.*` control tables, `jobs.schedule_occurrences` dedup ledger, and observability views
- [x] 2.2 `worker_svc` login role owning `jobs`; write-only access to each module role (`auth_svc`, `user_svc`) — documented D0 exception — role in `db/init/01-schemas-roles.sql`; grant is `EXECUTE` on `jobs.enqueue` only (refined from bare `GRANT INSERT`, spike finding #2)
- [x] 2.3 `cymbra-jobs` crate: an enqueue seam + channel/retry/DLQ policy types, engine kept swappable; pure policy logic host-testable, the DB glue behind a seam (coverage-excluded) — crate engine-agnostic (sqlxmq lives in the worker); `channel`/`retry`/`schedule`/`dlq`/`registry` pure + tested, `engine`/`scheduler` glue excluded
- [x] 2.4 `JobHandler` seam + job-type registry shared by producers and worker — `registry::{JobSpec, spec, builtin}` (pure, shared) + the worker's `#[job]` handlers/`JobRegistry`

## 3. Worker binary

- [x] 3.1 `cymbra-worker` binary: build the `JobRunner`, register handlers, `set_concurrency(min, max)` — `backend/worker/src/main.rs` + `handlers.rs`
- [x] 3.2 Channel policy: ordered channels for sequential `(module, kind)`, unordered for parallel — `channel::{Channel, Ordering}` + per-type `JobSpec`; `jobs.enqueue` sets the sqlxmq `ordered` flag
- [x] 3.3 Graceful shutdown + readiness/health for the worker — Ctrl-C/SIGTERM graceful shutdown; `/healthz` + `/readyz` (`worker/src/health.rs`)

## 4. Reliability: retries + dead-letter

- [x] 4.1 Read `jobs.retry_policy` per `(module, kind)` at enqueue → set retries/backoff on the job — `engine::load_retry_policy` + `EnqueueRequest::for_job(spec, payload, override)` maps it onto the job (`RetryPolicy::sqlxmq_retries`/`base_backoff`)
- [x] 4.2 On retry exhaustion, move the job to the dead-letter table and emit an alert signal/metric — `engine::dead_letter_sweep` (worker loop) moves exhausted `mq_msgs` → `jobs.dead_letter` and logs an error-level alert signal (Grafana alert on the table)
- [x] 4.3 Tests: bounded retry then success; exhaustion → dead-letter; ordered channel advances after a poison job is dead-lettered — `jobs/tests/reliability_test.rs` (`#[ignore]`, drives the real runner)

## 5. Recurring scheduler

- [x] 5.1 Scheduler in `cymbra-worker`: evaluate `jobs.schedules` (cron + timezone, enabled) and compute due occurrences — `schedule::Schedule` (pure) + `scheduler::run_scheduler_tick` (loop in the worker)
- [x] 5.2 Idempotent enqueue per occurrence via `dedup_key = '<name>:<bucket>'` + UNIQUE / `ON CONFLICT DO NOTHING` → exactly one across replicas — `schedule::{bucket,dedup_key}` + `jobs.schedule_occurrences` PK + `ON CONFLICT DO NOTHING` in `run_scheduler_tick`
- [x] 5.3 Per-schedule missed-run policy (skip vs catch-up) honored — `schedule::MissedRun` + `Schedule::occurrences_to_enqueue`
- [x] 5.4 Tests (host-testable): bucket/dedup is singleton across N simulated replicas; disabled schedule enqueues nothing; cron/timezone due-time computation — `schedule.rs` unit tests

## 6. First slice (migrate existing work) — DEFERRED to a follow-up step

> Substrate + worker are in place; the worker already has the `verification_email`
> and `orphan_reap` handlers and the enqueue seam. What remains is the invasive
> request-path change (and is intentionally left as the final, separately-reviewed
> step — it touches the live sign-up path and is runtime-verified via integration
> tests).

- [ ] 6.1 Move verification email out of `AuthModule::sign_up_local` request path → enqueue an `email` job in the same transaction; handler idempotent (dedupe by token), retried
- [ ] 6.2 Relocate the orphan reaper to a scheduled job (reuses `UserModule::reap_orphans`); remove the in-process `tokio::interval` loop in `backend/server/src/main.rs`
- [ ] 6.3 Tests: sign-up succeeds without SMTP and enqueues the email; reaper runs via a scheduled job exactly once

## 7. Observability

- [x] 7.1 SQL views: `jobs.pending`, `jobs.inflight`, `jobs.failed` (+ per-channel depth/age) — migration `0006_jobs_tables.sql` (`pending`/`inflight`/`failed`/`channel_depth`)
- [x] 7.2 Grafana dashboard (Postgres datasource) + a dead-letter alert; document setup — `observability/grafana/{datasources,dashboards,alerting}` + README section
- [x] 7.3 `.env.example` + config entries for the worker (DB creds, concurrency, schedule defaults) — `backend/.env.example` worker section + `worker/src/config.rs`

## 8. Validation

- [x] 8.1 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] 8.2 `cargo llvm-cov` green, coverage ≥ 80% (pure policy/scheduler logic covered; engine/DB glue excluded) — 95.67% lines workspace-wide with `engine.rs`/`scheduler.rs`/`handlers.rs` added to the exclusion regex (rust.yml + sonar.yml)
- [x] 8.3 Deployment note: `cymbra-worker` must be deployed for background work; the reaper no longer runs in `cymbra-id` — backend `README.md` (the reaper-loop removal lands with the deferred first slice, 6.2)
- [x] 8.4 `openspec validate add-job-infrastructure --strict` passes
