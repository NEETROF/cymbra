## 1. Spike: transactional enqueue (do first — riskiest assumption)

- [ ] 1.1 POC: from a `auth_svc`-role transaction, `sqlxmq` `spawn` a job into the shared `jobs` schema and commit/rollback atomically with an `auth` write; confirm the row appears iff committed
- [ ] 1.2 Resolve `search_path` / schema qualification for the `mq_*` tables under each module role; confirm a module role with only `GRANT INSERT` can enqueue but cannot read another schema
- [ ] 1.3 Decision checkpoint: if the spike fails, record the fallback (per-module queue tables or outbox) in design.md before proceeding

## 2. Foundation: schema, role, engine seam

- [ ] 2.1 Migrations for a `jobs` schema: `sqlxmq` `mq_*` objects, `jobs.schedules`, `jobs.retry_policy`, and a dead-letter table
- [ ] 2.2 `worker_svc` login role owning `jobs`; `GRANT INSERT` (write-only) to each module role (`auth_svc`, `user_svc`) — documented D0 exception
- [ ] 2.3 `cymbra-jobs` crate: an enqueue seam + channel/retry/DLQ policy types over `sqlxmq`, engine kept swappable; pure policy logic host-testable, the `sqlxmq`/DB glue behind a seam (coverage-excluded)
- [ ] 2.4 `JobHandler` seam + job-type registry shared by producers and worker

## 3. Worker binary

- [ ] 3.1 `cymbra-worker` binary: build the `JobRunner`, register handlers, `set_concurrency(min, max)`
- [ ] 3.2 Channel policy: ordered channels for sequential `(module, kind)`, unordered for parallel
- [ ] 3.3 Graceful shutdown + readiness/health for the worker

## 4. Reliability: retries + dead-letter

- [ ] 4.1 Read `jobs.retry_policy` per `(module, kind)` at enqueue → set retries/backoff on the job
- [ ] 4.2 On retry exhaustion, move the job to the dead-letter table and emit an alert signal/metric
- [ ] 4.3 Tests: bounded retry then success; exhaustion → dead-letter; ordered channel advances after a poison job is dead-lettered

## 5. Recurring scheduler

- [ ] 5.1 Scheduler in `cymbra-worker`: evaluate `jobs.schedules` (cron + timezone, enabled) and compute due occurrences
- [ ] 5.2 Idempotent enqueue per occurrence via `dedup_key = '<name>:<bucket>'` + UNIQUE / `ON CONFLICT DO NOTHING` → exactly one across replicas
- [ ] 5.3 Per-schedule missed-run policy (skip vs catch-up) honored
- [ ] 5.4 Tests (host-testable): bucket/dedup is singleton across N simulated replicas; disabled schedule enqueues nothing; cron/timezone due-time computation

## 6. First slice (migrate existing work)

- [ ] 6.1 Move verification email out of `AuthModule::sign_up_local` request path → enqueue an `email` job in the same transaction; handler idempotent (dedupe by token), retried
- [ ] 6.2 Relocate the orphan reaper to a scheduled job (reuses `UserModule::reap_orphans`); remove the in-process `tokio::interval` loop in `backend/server/src/main.rs`
- [ ] 6.3 Tests: sign-up succeeds without SMTP and enqueues the email; reaper runs via a scheduled job exactly once

## 7. Observability

- [ ] 7.1 SQL views: `jobs.pending`, `jobs.inflight`, `jobs.failed` (+ per-channel depth/age)
- [ ] 7.2 Grafana dashboard (Postgres datasource) + a dead-letter alert; document setup
- [ ] 7.3 `.env.example` + config entries for the worker (DB creds, concurrency, schedule defaults)

## 8. Validation

- [ ] 8.1 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] 8.2 `cargo llvm-cov` green, coverage ≥ 80% (pure policy/scheduler logic covered; engine/DB glue excluded)
- [ ] 8.3 Deployment note: `cymbra-worker` must be deployed for background work; the reaper no longer runs in `cymbra-id`
- [ ] 8.4 `openspec validate add-job-infrastructure --strict` passes
