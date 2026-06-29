## Context

Background work in Cymbra ID has no proper substrate (see proposal): the orphan
reaper ships as a per-pod `tokio::time::interval` loop (not cluster-safe under
horizontal scaling), and verification email is sent inline in `sign_up_local`
(SMTP latency on the request; error returned after the account already exists).
More async work is anticipated (request-triggered tasks, periodic maintenance,
later fan-out). A decisive constraint shapes everything: Cymbra ID is **one
Postgres database** (`cymbra_id`) with **schema-isolated modules** and a confined
login role each (`auth_svc`, `user_svc`) — a module physically cannot read another
module's schema (design D0). Redis/Valkey exists but only as a narrow cache/
coordination layer, not a system of record.

## Goals / Non-Goals

**Goals:**
- A durable, cluster-safe job substrate where work runs off the request path,
  exactly once across pods, with bounded retries, dead-lettering, and visibility.
- **Transactional enqueue**: a job is created atomically with the producing
  module's business write (no dual-write, no outbox).
- Per-type ordering: some job types strictly sequential, others parallel.
- Reuse an existing crate; reuse existing infra (Postgres) — no new system now.

**Non-Goals:**
- Redis Streams / NATS / Kafka and the event **fan-out** layer (adopted only on a
  measured signal — throughput ceiling, multi-consumer fan-out, cross-service).
- Exactly-once execution across external side effects (we target at-least-once +
  idempotent handlers).
- A click-to-act admin UI (Grafana read-only for now); the Vault/external-secrets
  decision (separate ops topic).

## Decisions

### D1 — Engine: `sqlxmq` (Postgres-backed) behind a `cymbra-jobs` seam
`sqlxmq` (depends on `sqlx 0.8`, matches the repo) is chosen for two decisive
properties: `spawn(&mut *tx)` enqueues **inside the caller's sqlx transaction**,
and **ordered channels** provide per-type ordering. We wrap it in a thin
`cymbra-jobs` crate (enqueue seam, channel/retry/DLQ policy) so the engine is
swappable. *Alternatives:* **apalis** — richest (cron, board UI) but `push()` uses
its own pool → no in-tx enqueue (breaks the #1 goal); rejected. **underway** —
transactional enqueue + built-in cron, but no first-class per-type ordering;
runner-up. **Hand-rolled SKIP LOCKED** — rejected, reinvents a solved problem.

### D2 — A separate `cymbra-worker` binary
Jobs are executed by a dedicated worker process, not inside `cymbra-id`. The
user-facing service only **enqueues**; it never executes. This isolates job
throughput from API latency (no noisy-neighbor) and lets the worker scale
independently. Handler code lives behind a `JobHandler` seam so producers and
workers share types without sharing process lifecycle.

### D3 — Shared `jobs` schema owned by `worker_svc`; modules `GRANT INSERT`
Because it is **one database**, a module's transaction (on its own role/connection)
can `INSERT` into a shared `jobs` schema atomically with its business write — *if*
granted. So: a `jobs` schema owned by a new least-privilege `worker_svc`; each
module role gets a **narrow, write-only `GRANT INSERT`** on the queue table. This
is a **documented, deliberate exception to D0** — a one-way write channel into the
queue, never cross-module reads. *Alternative:* per-module job tables — rejected;
the separate worker would then need cross-schema SELECT/claim into every module
schema, a worse and bidirectional D0 hole. Module/type separation becomes the
**channel column**, not separate tables.

### D4 — Ordering via channels
`sqlxmq` ordered channels run jobs in the same channel strictly sequentially while
other channels proceed in parallel. Sequential-by-type → one ordered channel per
such type; parallel types → unordered channels. Channel name encodes `(module,
kind)`.

### D5 — Recurring scheduling = singleton via idempotent time-bucket enqueue
`sqlxmq` has no built-in cron, so a small scheduler in `cymbra-worker` reads a
`jobs.schedules` table (`name`, `cron_expr`, `timezone`, `enabled` — runtime-
tunable, dashboard-visible) and, when a task is due, **enqueues via an idempotent
time-bucket key** (`dedup_key = '<name>:<bucket>'`, `ON CONFLICT DO NOTHING` on a
UNIQUE constraint). Even with several worker replicas evaluating the same cron,
exactly one job is created. Execution then flows through the normal queue (retries/
DLQ/observability for free). *Alternatives:* `pg_try_advisory_xact_lock` per tick
— viable fallback (xact-scoped, pool-friendly, auto-release on commit/crash) but
runs the work inside the lock's transaction; **noted, not default**. Session-level
`pg_advisory_lock` — avoided (leaks on a connection pool, slow crash-failover); use
only for a future *continuous-leader* responsibility. External k8s CronJob — an
option that trades a moving part for platform coupling.

### D6 — Bounded retries per `(module, type)` + dead-letter
Retries are bounded with exponential backoff, **configurable per `(module, kind)`**
via a runtime-tunable `jobs.retry_policy` table (read at enqueue → set on the job).
On exhaustion the job moves to a **dead-letter** table and raises an alert. **No
infinite retry** — on an ordered channel a permanently-failing job is head-of-line
blocking (freezes the whole type), so bounded-retry → DLQ is what keeps ordered
channels live. Near-infinite retry is reserved for known-transient failures, on a
**non-ordered** channel, with a capped max-backoff.

### D7 — Pickup is event-driven, tuned by concurrency (not poll frequency)
`sqlxmq` is LISTEN/NOTIFY-driven: a fresh enqueue (incl. a scheduled job's INSERT)
NOTIFYs the worker → pickup in ~ms; future/retry jobs wake the worker at their due
time; idle is quiet with a ~minute keep-alive re-check as a backstop for missed
NOTIFYs. The operational tunable is **concurrency** (`set_concurrency(min, max)`),
not a poll interval.

### D8 — Observability: Grafana over Postgres
Everything is queryable SQL, so observability is a Grafana dashboard on a Postgres
datasource: `pending` / `inflight` / `failed` views, per-channel depth and age, and
a DLQ alert. Read-only for now. *Alternative:* `apalis-board` (turnkey click-to-act
UI) — only reconsidered if interactive retry/cancel from a UI becomes a hard
requirement, and only if it doesn't cost the transactional-enqueue property.

### D9 — At-least-once + idempotent handlers
Delivery is at-least-once (claim + lease; a dead worker's lease expires and another
reclaims). Handlers MUST be idempotent (idempotency key per job; e.g. verification
email dedupes by token). Exactly-once across side effects is a non-goal.

### D10 — First slice
(1) Move verification email out of `sign_up_local`'s request path — enqueue an
`email` job in its transaction; idempotent, retried. (2) Relocate the orphan reaper
to a scheduled job (D5) and remove the in-process loop in `main.rs`.

## Risks / Trade-offs

- **Riskiest assumption: `spawn` inside a module role's transaction against the
  shared `jobs` schema** → POC/spike before committing (grants + `search_path` for
  the `mq_*` tables). If it can't be made to work cleanly, fall back to per-module
  queue tables or an outbox.
- **D0 relaxation** → mitigated by keeping the grant **write-only** (INSERT) and
  documenting it; no module gains read access to another's data.
- **`sqlxmq` maintenance pace** (single maintainer) → contained behind the
  `cymbra-jobs` seam so the engine can be swapped (underway / hand-rolled) without
  touching producers/handlers.
- **Ordered-channel head-of-line blocking** → bounded retries + DLQ; never infinite
  retry on an ordered channel.
- **Scheduler clock skew across pods** → the time-bucket granularity must exceed
  expected skew; the UNIQUE dedup makes double-enqueue impossible regardless.
- **Long maintenance jobs** → the scheduler only enqueues; heavy work runs in the
  worker, never held inside a tick/lock.

## Migration Plan

Additive: new `cymbra-jobs` crate + `cymbra-worker` binary + `jobs` schema
migrations + `worker_svc` role and grants. Phase 1 = the first slice (D10) behind
no user-visible change. Rollback = revert producers to inline I/O and keep the
in-process reaper loop until the worker path is proven. **Deployment note**: the
`cymbra-worker` must be deployed for background work to run (the reaper no longer
runs inside `cymbra-id`) — call this out in the release.

## Open Questions

- Role naming: `worker_svc` vs `jobs_svc` (role follows the schema it owns).
- Per-task **missed-run policy** (skip vs catch-up) when the worker was down at a
  scheduled time.
- Default retry/backoff values per `(module, kind)`.
- Whether a minimal read-only admin endpoint is wanted alongside Grafana before any
  click-to-act UI is considered.
- Exact escalation thresholds (measured) that would justify a Redis Streams / broker
  fan-out layer.
