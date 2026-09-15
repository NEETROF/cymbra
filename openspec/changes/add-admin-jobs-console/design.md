## Context

The job substrate (`add-job-infrastructure`) is sqlxmq 0.6 vendored into the `jobs`
schema, owned by `worker_svc`, and run by `cymbra-worker`. Four facts about it
shape this design:

1. **A completed job is deleted.** `CurrentJob::complete()` calls
   `mq_delete(ARRAY[id])`, which removes the `mq_msgs` and `mq_payloads` rows. Nothing
   stays behind to count "jobs done over a period". The only lasting trace is
   `jobs.dead_letter`, for exhausted jobs.
2. **A running job has no row of its own.** `mq_poll` claims a message by
   decrementing `attempts` and setting `attempt_at = NOW() + retry_backoff` (or `NULL`
   on the last attempt). The runner's keep-alive (on by default) keeps pushing
   `attempt_at` forward while the handler runs. When a handler returns an error,
   **nothing is written**: the message just waits until `attempt_at`. In the database,
   "running" and "failed, waiting for its retry" are identical.
3. **The error handler has no job id.** `JobRegistry::set_error_handler` receives
   `(name, error)`. `NamedJob`'s dispatch function is private, so the registry cannot
   be wrapped generically either.
4. **Ordered channels are a linked list.** Each ordered message points at its
   predecessor through `after_message_id`, with `ON DELETE SET DEFAULT` (nil) and a
   unique index on `(channel_name, channel_args, after_message_id)`. sqlxmq only
   ever deletes the **head** of a chain.

On the access side, `cymbra-server` holds no `admin_svc` pool, and the ops role must
never be given to a server module (`roles.sql.tpl`). Module roles get exactly one
thing in `jobs`: `EXECUTE` on the `SECURITY DEFINER` `jobs.enqueue`. Payloads carry
personal data: a `verification_email` payload holds the recipient address and the
rendered body.

The back office already has the pieces this page needs: `StatCards`, `TablePager`,
`ConfirmDialog`, the `Async<T>` union, the `api()` seam with its e2e fake, and a
router guard that checks admin scopes.

## Goals / Non-Goals

**Goals:**

- A `global` admin sees the jobs currently in the queue, with a truthful state for
  each: running, ready, scheduled, waiting for a retry, blocked, or exhausted.
- They see how many jobs are in the queue now, and how many completed, failed,
  were dead-lettered or cancelled over a selectable period, plus the average run
  time.
- They can cancel a queued job, and nothing unsafe can happen as a result: a
  handler is never cut mid-run, an erasure obligation is never dropped, and an
  ordered chain is never broken.
- Least privilege: the server can read queue *metadata* and cancel, and nothing
  more (no payloads, no raw table access).

**Non-Goals:**

- Cancelling a **running** job (no cooperative cancellation exists in handlers).
- Retry-now, requeue, dead-letter replay, or editing schedules / retry policies.
- Showing a job's payload or its error text. Errors stay in logs and traces
  (Tempo), which already carry them with the job id.
- Live push updates. The page refreshes on demand, with an opt-in periodic refresh.
- A per-module view for `music`/`live`/`lingua` admins. See Open Questions.

## Decisions

### D1 — Record attempts in the worker, around each handler

A new `jobs.job_attempts` table holds one row per attempt: `job_id`, `job_name`,
`channel_name`, `attempt`, `started_at`, `finished_at`, and `outcome`, one of
`running | succeeded | failed | abandoned`. A new seam in `cymbra-jobs`
(`attempt.rs` holds the pure outcome mapping, the engine glue holds the SQL) wraps
**every** handler body:

1. **Begin**, in one transaction on the job's own pool (`CurrentJob::pool()`):
   `SELECT … FROM jobs.mq_msgs WHERE id = $1 FOR UPDATE`.
   - If the row is gone, the job was cancelled between claim and start: return `Ok`
     without running the handler.
   - Otherwise, close any still-`running` attempt of this job as `abandoned` (a
     crashed worker's lease expired and the job was reclaimed), then insert the new
     `running` attempt.
2. **Run** the handler.
3. **Finish**: `Ok` → `succeeded`, `Err` → `failed`, with `finished_at = now()`.

*Alternatives considered:*
- **Triggers on `mq_msgs`.** A trigger sees the claim (the UPDATE) and the completion
  (the DELETE) but never a handler failure, so fact 2 would remain. It would also hide
  queue logic in triggers on a vendored table.
- **Hooking the sqlxmq dispatch.** The dispatch function is private and the error
  handler has no id (fact 3).
- **Postgres statistics / Grafana only.** They cannot answer "which job" or "did it
  succeed".

**Enforcement.** Every `#[sqlxmq::job]` in `handlers.rs` goes through one helper.
A unit test pins the registered handler names to `registry::builtin()`, so a new
handler is noticed. An integration test drives a succeeding, a failing and a
cancelled-before-start job through the wrapper.

### D2 — The state is derived once, in SQL

A `jobs.admin_queue` view projects each live message (`id != uuid_nil()`) with a
single `state`. The first rule that matches wins:

| state        | rule |
|--------------|------|
| `running`    | a `running` attempt exists AND (`attempt_at > now()` — lease held — OR (`attempt_at IS NULL` AND the attempt started less than `RUNNING_GRACE` ago)) |
| `exhausted`  | `attempt_at IS NULL` (no retries left, waiting for the dead-letter sweep) |
| `blocked`    | `mq_uuid_exists(after_message_id)` (ordered, predecessor still queued) |
| `ready`      | `attempt_at <= now()` |
| `retry_wait` | `attempt_at > now()` AND at least one attempt recorded |
| `scheduled`  | `attempt_at > now()` AND no attempt recorded (a delayed enqueue) |

`RUNNING_GRACE` is 1 hour and is defined once, in SQL, where both the view and the
sweep (D6) read it.

Filtering by state, paginating and counting per state all have to happen server-side,
so the rule cannot live only in Rust. It lives in the one view, and the `#[ignore]`
integration tests cover one fixture per state. Rust (`admin_core.rs`) owns what is
host-testable: parsing the state and kind filters, clamping the page, validating the
window, and the cancellation decision.

### D3 — Cancellation is one `SECURITY DEFINER` function and one Rust rule

`jobs.admin_cancel(p_job_id uuid, p_actor text) RETURNS text` does the following,
in the caller's transaction:

1. Lock the message `FOR UPDATE`. If it is missing, return `gone`.
2. If a `running` attempt holds the lease (the D2 `running` rule), return `running`.
3. Record `jobs.cancellations(job_id, job_name, channel_name, cancelled_by, cancelled_at)`,
   and close any stale `running` attempt as `abandoned`.
4. **Relink the ordered chain**:
   - store `v_pred := after_message_id`;
   - set this message's `after_message_id = NULL`, which frees its slot in the unique
     index;
   - repoint the successor (`after_message_id = p_job_id`) at `v_pred`;
   - then `mq_delete(ARRAY[p_job_id])`.

   Deleting a non-head message directly would set the successor to nil. That either
   violates the unique index (the head already holds nil) or lets the successor run
   concurrently with the head. Both break ordering. Deleting the head keeps sqlxmq's
   own behaviour: the successor becomes the head.
5. Return `cancelled`.

The begin step of D1 locks the same row, so claim, start and cancel serialize. Either
the cancellation sees the new `running` attempt and refuses, or the handler finds the
message gone and skips.

**Protected kinds.** `JobSpec` gains `cancellable: bool`, `true` by default. It is
`false` for `purge_user`, `purge_score_object` and `purge_soundfont_object`. The Rust
module reads the job's name first and refuses a protected kind before calling
`admin_cancel`. The name of a job id never changes, so reading it first is not a
race. A kind missing from the registry is cancellable: no handler would run it anyway.

The RPC answers with an outcome, `CANCELLED | GONE | RUNNING | PROTECTED`, rather than
error codes. The console turns each one into a precise message. `GONE` is not an
error for the operator: the job finished, or someone else cancelled it.

*Alternative considered:* keeping the protected list in SQL. That would duplicate the
registry, and `cymbra-server` is the only caller.

### D4 — A dedicated narrow role, `jobs_admin_svc`

The new role gets `USAGE` on schema `jobs` and `EXECUTE` on `admin_list_jobs`,
`admin_queue_counts`, `admin_period_stats` and `admin_cancel`. All four are
`SECURITY DEFINER`, owned by `worker_svc`, with `SET search_path = jobs`, and have
`EXECUTE` revoked from `PUBLIC`. The role gets no table privileges, so it physically
cannot read `mq_payloads`, and no function returns a payload column.

`cymbra-server` reads the optional `CYMBRA_JOBS_ADMIN_DATABASE_URL`. When it is unset,
`JobsAdminService` is not mounted, like the Lingua and analytics consoles.

**Grant ordering converges.** The migration grants to `jobs_admin_svc` only when the
role exists (`DO $$ … IF EXISTS (pg_roles) …`), and the provisioning script, run as
superuser, grants on the functions when they exist. Whichever runs first, the second
completes the grants. On a fresh development database, `roles.sql.tpl` creates the
role before the worker migrates.

*Alternatives considered:*
- **`admin_svc`** reads and writes every schema. It is forbidden in a server module.
- **`worker_svc`** owns the queue, including payloads and `mq_*` DML. That is far
  more than listing and cancelling.
- **Granting the functions to `user_svc`** would make an identity module a queue
  administrator, against `job-infrastructure`'s module-isolation requirement.

### D5 — Contract, gate and pagination

- `backend/jobs/proto/jobs_admin.proto`, package `cymbra.jobs.v1`, service
  `JobsAdminService`:
  - `AdminListJobs(state?, kind?, limit, offset)` returns rows plus `total`;
  - `AdminGetJobStats(window, kind?)` returns counts per state now, plus the period
    figures;
  - `AdminListJobKinds` returns the registry names, which feed the filter;
  - `AdminCancelJob(job_id)` returns the outcome.
- **Gate.** Every RPC requires `admin` in the **`global`** scope
  (`has_role_in_scope("global", "admin")`), because the queue spans every product
  (identity emails, erasure, Music renders, plan reconciliation). The actor recorded
  on a cancellation is the interceptor identity's `user_id`, never a field in the body.
- **Pagination.** Offset/limit with `limit ∈ [1, 100]` (default 25) and a `total`,
  ordered by `created_at ASC, id ASC` (queue order, oldest first). Offsets are
  acceptable here: the live queue is small, and the page is a snapshot the operator
  refreshes, not an infinite feed. A job that leaves the queue between two pages only
  shifts the next page.
- **Window.** The window is a pair of epoch-millisecond bounds, `from < to`, with
  `to ≤ now` and `from ≥ now − 90 days` (the retention window). Anything else is
  `INVALID_ARGUMENT`. Presets (1 h, 24 h, 7 d, 30 d) are computed client-side.
- **Period figures.** Over `[from, to)`:
  - `completed`: distinct jobs with a `succeeded` attempt finished in the window;
  - `failed_attempts`: `failed` attempts finished in the window;
  - `dead_lettered`: from `jobs.dead_letter.dead_lettered_at`;
  - `cancelled`: from `jobs.cancellations`;
  - `avg_run_ms`: over `succeeded` attempts.

### D6 — Dead-letter sweep: running-attempt grace and retention

- The sweep skips an exhausted message (`attempt_at IS NULL AND attempts <= 0`) that
  still has a `running` attempt started less than `RUNNING_GRACE` ago. Today the last
  attempt's message looks exhausted the moment it is claimed, so a sweep tick during a
  long last attempt dead-letters a job that then succeeds. This is a pre-existing race
  that the history would turn into wrong figures. Once the grace has elapsed, the
  attempt is treated as crashed: the job is dead-lettered and the attempt closed as
  `abandoned`.
- The same tick deletes `job_attempts` and `cancellations` rows older than 90 days,
  in bounded batches. That avoids a scheduled job and a handler just for hygiene.

### D7 — Back office

- **Route and nav.** `/jobs` with `meta: { admin: true, adminScope: "global" }`. The
  nav item is shown only when `auth.adminScopes` includes `global`. The server gate
  stays authoritative.
- **Store.** `stores/jobs.ts` is the only place that calls `api().jobs`. It holds
  three `Async<T>` resources:
  - `page`: rows plus total;
  - `stats`: queue counts plus period figures;
  - `kinds`: the filter list.

  It also holds the cancellation state and the parameters: state, kind, offset, and
  the period (preset or custom). After a cancellation the store reloads the page and
  the stats, and toasts the localized outcome.
- **View.** `views/JobsView.vue` renders, top to bottom:
  - queue `StatCards` (in queue, running, ready, retry wait, blocked + scheduled,
    exhausted);
  - a period selector with period `StatCards` (completed, failed attempts,
    dead-lettered, cancelled, average run time);
  - filters (state, kind);
  - the table;
  - `TablePager`.

  A **Cancel** button appears only on rows the server marked `cancellable`, which is
  every state but `running`, on a non-protected kind. It goes through
  `ConfirmDialog`. A **Refresh** button and an opt-in "auto-refresh every 15 s"
  toggle sit beside the title. The toggle pauses while the dialog is open or the tab
  is hidden.
- **Tests.** The e2e seam gains a `jobs` fake whose cancellation mutates its
  fixture in place. `e2e/jobs.spec.ts` covers: a global admin sees cards, rows and
  pagination; a cancellation removes the row and bumps the cancelled figure; running
  and protected rows offer no Cancel; a `music` admin gets no nav entry and is
  redirected. Strings are added to en and fr together.

## Risks / Trade-offs

- **A job's last attempt runs longer than `RUNNING_GRACE`.** After an hour it shows as
  `exhausted` and the sweep dead-letters it while it runs. → No current job runs
  anywhere near an hour. The grace is one SQL constant, and a long job type would
  raise it.
- **A handler added without the tracker.** Its runs would be invisible, and it could
  run after a cancellation. → The wrapper is the only documented way to write a
  handler, and a test pins the registry names. The `run` checklist in the worker
  module doc names it.
- **Worker crash between `complete()` and the `succeeded` write.** The attempt stays
  `running` with its message gone. → The period figures would miss one completion.
  The next sweep closes attempts whose message has disappeared as `abandoned`, so
  none stays `running` forever.
- **Extra round-trips per job.** Each job gains one begin transaction and one finish
  update, a couple of milliseconds against jobs that send mail or render audio.
  Throughput is far below where this matters.
- **Offset pagination over a moving queue** can show a row twice or skip one across
  pages. → The page is a snapshot with an explicit refresh, and the total is shown.
- **A deploy that forgets the role.** The console is inert (service not mounted); the
  worker is not affected, because the migration grants conditionally.

## Migration Plan

1. **Merge.** The worker migration creates the tables, the view and the functions,
   and grants when the role exists. The worker starts recording attempts
   immediately. The server ignores the console until it has a URL.
2. **Provision in production.** Run `provision-jobs-admin-role.sql` as superuser
   (password over stdin): it creates the role and grants on the functions. Then set
   `CYMBRA_JOBS_ADMIN_DATABASE_URL` in `.env` and roll the server.
3. **Rollback.** Unset the URL: the console goes dark. The history tables are inert
   without the console. Reverting the worker image stops the recording; the tables
   can stay.

History starts at deploy time, so period figures before it read zero. The console
states the date the history starts from when the window reaches before it.

## Open Questions

- Should `music`/`live`/`lingua` admins get a read-only view **filtered to their
  module's channels** (`<module>.<kind>`)? This change keeps the page global-only; a
  later change can add a scope → channel-prefix filter on the server.
- Retention of 90 days: long enough for month-over-month comparisons, short enough to
  stay small. Should it become a runtime flag, like the usage-analytics retention?
