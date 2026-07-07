## Context

`DeleteAccount` (user service) runs only `DELETE FROM users` in the `user_account`
schema (cascades to `user_identities` / `user_roles`). The **auth** module owns a
separate schema with `local_credentials` (email PK + password hash + verify/reset
tokens) and `sessions` (refresh tokens by `user_id`); nothing deletes them on
account removal. Constraints that shape the design:

- **Per-module schema isolation (D0):** `user_svc` / `auth_svc` can each only touch
  their own schema. No app service may cross schemas.
- **`admin_svc` is the sanctioned cross-schema writer**, but the ops-db-access design
  states it **must never be wired into an application service** — it belongs to the
  worker / ad-hoc ops.
- `local_credentials` has **no `user_id`** — it is keyed by email. The email is
  recoverable from `user_account.user_identities` where `provider = 'local'`
  (`subject` = email).
- The job substrate (`sqlxmq`, `cymbra-worker`) already exists, runs as `worker_svc`
  for the queue, and the worker process is the natural home for `admin_svc` work.

## Goals / Non-Goals

**Goals:**
- One `DeleteAccount` erases the user's data across `user_account` **and** `auth`.
- Erasure is atomic (single transaction) and idempotent (retry-safe).
- Freed email is re-registrable; the user's refresh tokens stop working.
- No `admin_svc` in any application service; no per-module role widening.

**Non-Goals:**
- No change to the `DeleteAccount` gRPC contract or the Flutter client.
- No "export my data" / full GDPR SAR flow (separate concern).
- No soft-delete / tombstones — this is a hard erasure.

## Decisions

### Decision: erase via a `purge_user` job run by the worker as `admin_svc`
`DeleteAccount` enqueues a `purge_user{user_id}` job (via the existing
`jobs.enqueue` seam) and returns success once enqueued. `cymbra-worker` runs the job
on an **`admin_svc` connection** in **one transaction**: resolve the email from
`user_identities` (provider `local`), `DELETE FROM auth.local_credentials` by that
email (skip if none), `DELETE FROM auth.sessions` by `user_id`, `DELETE FROM
user_account.users` by `user_id` (cascades identities/roles), commit. A single
`admin_svc` connection spanning both schemas is what makes the erasure atomic.

- **Why not orchestrate in the server composition root** (call `user.delete_account`
  + `auth.delete_credentials` + `auth.delete_sessions`)? Those are separate module
  ports on separate roles/connections → **not atomic** (partial-failure can strand
  data), and it would still need a cross-schema actor. Rejected.
- **Why not a synchronous `admin_svc` transaction inside the server?** That wires
  `admin_svc` into an application service, which the ops-db-access design explicitly
  forbids. Rejected.

### Decision: `DeleteAccount` returns after enqueue (async erasure)
The RPC stays thin; the client clears the local session immediately regardless. The
worker completes the erasure within its normal poll latency (seconds). sqlxmq's
retry + dead-letter give durability if a run fails.

### Decision: resolve email inside the job, not in the payload
The job carries only `user_id` (what the token gives). It reads the `local` identity
inside its own transaction to get the email, keeping the enqueue path trivial and
avoiding the email transiting the queue payload.

## Risks / Trade-offs

- **[Immediate re-signup race]** Between enqueue and job completion the old
  `local_credentials` row still exists, so a re-signup with the same email in that
  window would hit the PK. → Mitigation: worker poll latency is seconds and a human
  won't re-register that fast; documented. Future option: let sign-up reclaim a
  credential whose owning `user_account` no longer exists.
- **[Worker down]** Erasure is delayed until the worker runs. → Mitigation: the job
  persists in the queue and runs on recovery; DLQ + monitoring surface stuck jobs.
- **[admin_svc blast radius]** The purge job can write every schema. → Mitigation:
  it is a single, reviewed, narrowly-scoped handler; `admin_svc` stays worker-only.

## Migration Plan

- Additive: a new job kind + handler + an auth "delete credentials by email" query
  (the sessions-by-user delete already exists). No schema migration required
  (`user_identities` / cascades already exist).
- Deploy backend image; the worker registers the new job. Roll forward only; to
  disable, stop enqueuing (the handler is inert without jobs).

## Open Questions

- Should `DeleteAccount` optionally **block** until the purge completes (stronger
  read-your-deletion guarantee) at the cost of coupling the RPC to the worker?
  Default: no (async), revisit if the re-signup race proves real.
- Should sign-up proactively reclaim an orphaned `local_credentials` (belt-and-braces
  for the race)? Deferred unless needed.
