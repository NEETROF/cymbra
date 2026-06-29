## Why

Cymbra ID is growing background work that today has no proper home:

- The **orphan reaper** (`fix-handle-onboarding-escape`) shipped as a naive
  per-pod `tokio::time::interval` loop in `cymbra-id`. With horizontal scaling
  (multiple pods) every replica runs it — the canonical singleton-job problem.
- **Verification email** is sent **inline in the sign-up request path**
  (`AuthModule::sign_up_local`): SMTP latency is on the user's request, and an
  SMTP failure returns an error to a user **whose account was already created**.
- More async work is anticipated: request-triggered tasks (email, webhooks,
  uploads), periodic maintenance (purges, aggregates), and later event fan-out.

We need a durable, cluster-safe async-job substrate so this work runs off the
request path, exactly once across pods, with retries and visibility — instead of
ad-hoc timers and inline side effects.

## What Changes

- **Introduce a durable, Postgres-backed job queue** using **`sqlxmq`** (depends
  on `sqlx 0.8`, matching the repo). Chosen for two decisive properties: jobs are
  **enqueued inside the producing module's business transaction** (no dual-write,
  no outbox), and **ordered channels** give per-type ordering. *(apalis rejected:
  its `push()` uses its own pool, breaking transactional enqueue; underway was the
  runner-up.)*
- **New separate worker binary `cymbra-worker`** that executes jobs and owns
  recurring scheduling. The user-facing `cymbra-id` only **enqueues** jobs; it
  never executes them.
- **Shared `jobs` schema owned by a least-privilege `worker_svc` role.** Each
  module role (`auth_svc`, `user_svc`) gets a narrow **`GRANT INSERT`** into the
  queue — a documented, deliberate, write-only exception to the per-module
  isolation invariant (design D0). Module/type separation is a **channel column**,
  not separate tables.
- **Ordering policy via channels**: ordered channels for job types that must run
  sequentially; unordered for parallel types.
- **Recurring tasks**: a `jobs.schedules` table (cron per task, timezone, enabled
  — runtime-tunable, dashboard-visible) driven by a small scheduler in
  `cymbra-worker`. Since `sqlxmq` has no built-in cron, the scheduler enqueues due
  jobs via an **idempotent time-bucket dedup** (`ON CONFLICT DO NOTHING`) so
  exactly one runs across pods. `pg_try_advisory_xact_lock` is the noted lock-based
  fallback; session-level advisory locks are avoided (pool-leak/crash-failover).
- **Reliability**: bounded retries with exponential backoff, **configurable per
  `(module, type)`** via a `jobs.retry_policy` table; on exhaustion a job moves to
  a **dead-letter** table with an alert. **No infinite retry.**
- **Observability**: **Grafana** over a Postgres datasource — `pending` /
  `inflight` / `failed` views, per-channel depth and age, DLQ alerting.
- **First slice**: move verification email out of the `sign_up_local` request path
  (enqueued in its transaction; idempotent, retried), and **relocate the orphan
  reaper** from the in-process loop to a scheduled job.
- **BREAKING (ops/deployment)**: a new `cymbra-worker` deployment is required for
  background work to run; the reaper no longer runs inside `cymbra-id`.

## Capabilities

### New Capabilities
- `job-infrastructure`: The durable async-job substrate — transactional enqueue,
  the separate worker, ordered/parallel channels, per-`(module,type)` bounded
  retries + dead-letter, recurring scheduling with cluster-wide singleton
  semantics, and queue observability.

### Modified Capabilities
<!-- The reaper (fix-handle-onboarding-escape) and verification-email flow
     (add-music-account-access) are not yet archived into openspec/specs/, so there
     is no delta target. Their migration onto this substrate is captured as the
     first-slice work in tasks/design, not as a spec delta here. -->

## Impact

- **New backend crate(s)**: a thin `cymbra-jobs` engine wrapper around `sqlxmq`
  (enqueue seam, channel/retry policy, DLQ) + a `cymbra-worker` binary. New
  dependency: `sqlxmq` (sqlx 0.8).
- **Database**: new `jobs` schema + migrations (`mq_*`, `jobs.schedules`,
  `jobs.retry_policy`, dead-letter), a new `worker_svc` login role, and
  `GRANT INSERT` to each module role (the documented D0 exception).
- **Producers**: `AuthModule::sign_up_local` (and similar side effects) stop doing
  inline I/O and enqueue a job in their transaction instead.
- **Reaper**: `UserModule::reap_orphans` is invoked by a scheduled job, not the
  in-process loop; the loop in `backend/server/src/main.rs` is removed.
- **Ops**: a new worker deployment, Grafana dashboard + datasource, and a DLQ
  alert. Worker needs its own DB credentials (delivered like existing secrets).
- **Tests/coverage**: ≥80% (Rust) maintained — keep enqueue/retry/schedule policy
  in pure, host-testable modules; the `sqlxmq`/DB glue sits behind a seam and is
  coverage-excluded like the other I/O adapters.
- **Key spikes (resolve in design)**: validate `sqlxmq` `spawn` **inside a module
  role's transaction** against the shared `jobs` schema (the riskiest assumption),
  and the `jobs` schema placement / `search_path` for the `mq_*` tables.
- **Out of scope**: Redis Streams / NATS / Kafka and the event-fan-out layer
  (adopted only on a measured signal — throughput ceiling, multi-consumer fan-out,
  or cross-service eventing); click-to-act dashboard UI (Grafana is read-only for
  now); the external-secrets/Vault decision (separate ops topic).
