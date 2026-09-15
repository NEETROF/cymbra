## Why

Background work (verification emails, account erasure, score renders, plan
reconciliation, …) runs on `cymbra-worker`, but operators cannot see the queue from
the back office. The only views are SQL views read through Grafana, which show depth
and age but not *which* jobs are waiting, which one is running, or how much work was
done over a period. When emails arrived ~45 minutes late (a long job holding the only
polling slot), nothing in the console showed the stuck queue, and there is no safe
way to remove a job that should not run: today that means `psql` against `jobs.mq_*`.

## What Changes

- New back-office **Jobs** page (`/jobs`), for `global` admins only:
  - summary cards: jobs in the queue now, split by state (running, ready, scheduled,
    waiting for a retry, blocked behind an ordered predecessor, exhausted), plus
    figures for a **selectable period** (last hour / 24 h / 7 days / 30 days, or a
    custom range): completed jobs, failed attempts, dead-lettered jobs, cancelled
    jobs, and the average run time of completed jobs;
  - a **paginated table** of the jobs currently in the queue (kind, channel, state,
    attempts made / left, enqueued at, next attempt at, started at), filterable by
    state and job kind;
  - a **Cancel** action on a queued job, behind a confirmation.
- The worker records **every attempt** of every job (start, end, outcome) in a new
  `jobs.job_attempts` history, and every operator cancellation (who, when) in
  `jobs.cancellations`. Completed jobs are deleted from the queue by sqlxmq, so
  without this history "jobs completed over a period" cannot be answered, and a
  running job cannot be told apart from one waiting for a retry.
- **Cancellation rules**, enforced server-side: a running job cannot be cancelled
  (a handler cannot be interrupted safely); account and object **erasure jobs
  cannot be cancelled** (`purge_user`, `purge_score_object`,
  `purge_soundfont_object`: a legal obligation, not a queue-hygiene choice);
  cancelling a job on an ordered channel lets the next one proceed; cancelling one
  occurrence of a recurring job leaves the schedule enabled.
- A new `JobsAdminService` gRPC contract (`backend/jobs/proto/jobs_admin.proto`) with
  list, stats, kinds and cancel RPCs. It never returns a job's payload: payloads carry
  personal data (a verification email's recipient and body).
- A new narrow database role, `jobs_admin_svc`, used by `cymbra-server` for the
  console. It holds `EXECUTE` on a handful of `SECURITY DEFINER` functions in `jobs` and
  no table privileges, mirroring how `jobs.enqueue` is granted to module roles.
- The dead-letter sweep no longer dead-letters a job whose **last attempt is still
  running**. This race already exists, but the new history would make it show up
  as a wrong "dead-lettered" count. The sweep also prunes history older than the
  retention window (90 days).

## Capabilities

### New Capabilities

- `admin-jobs-console`: the back-office Jobs page and the admin-only operations
  behind it (queue listing with pagination and filters, current and per-period
  statistics, cancellation of a queued job, access control, payload privacy).

### Modified Capabilities

- `job-infrastructure`: adds the attempt-history requirement (every attempt recorded
  with its outcome, bounded retention), operator cancellation semantics at the queue
  level (never runs afterwards, running jobs and erasure jobs refused, ordered
  successors proceed), the narrow queue-administration access role, and the
  dead-letter sweep leaving a still-running last attempt alone.

## Impact

**Products**
- **Back office**: new (the Jobs page, a store, the e2e seam fake, en/fr strings). It
  consumes the existing admin session and scope claims, `StatCards`, `TablePager`
  and `ConfirmDialog`.
- **Cymbra ID / Music / Live / site**: no user-facing change. Their jobs become
  visible and cancellable from the console; nothing about how they enqueue changes.
- **Platform (jobs + worker)**: new attempt history, cancellation, admin SQL
  functions, and the sweep fix. The job producers are unchanged.

**Code**
- `backend/jobs`: migration (history tables, admin functions, grants),
  `JobSpec::cancellable`, the attempt-tracking seam, the admin module
  (`admin_core.rs`, `admin.rs`, `pg_admin.rs`, `admin_grpc.rs`), the proto and a
  `build.rs`.
- `backend/worker`: every handler wrapped by the attempt tracker; the sweep
  gains a grace rule for running attempts, plus pruning.
- `backend/server` + `backend/platform/src/config.rs`: optional
  `CYMBRA_JOBS_ADMIN_DATABASE_URL`. When it is unset, the service is not mounted
  and the console is inert.
- `backend/db/init` (`roles.sql.tpl`, `00-roles.sh`), `backend/deploy`
  (`provision-jobs-admin-role.sql`, `.env.prod.example`, `DEPLOY.md`), `backend/.env.example`,
  `backend-it.yml` env, `buf.yaml`, `.github/coverage-ignore-regex.txt`.
- `apps/back-office`: `tool/gen_proto.sh`, `lib/transport.ts`, `stores/jobs.ts`,
  `views/JobsView.vue`, router, nav, `i18n/locales/{en,fr}.json`, `lib/e2e-seam.ts`,
  `e2e/jobs.spec.ts`.

**Deploy**: one new role in production, provisioned before the server is given its URL.
The migration and the provisioning script both grant, so either order converges.
