## Context

`RcConfig.allow_sandbox` is a process-wide boolean threaded into two **pure**
mappers, `map_event` and `map_customer`. Both drop a transaction when it is sandbox
and the flag is false. Neither has anything but the Cymbra account id
(`ev.app_user_id`, `user_id`) — no email, and for OIDC accounts the email claim is
read then discarded, so any rule phrased over addresses would silently never match.

Production runs with the flag false. That was verified end to end on 2026-09-06: a
TestFlight purchase reached RevenueCat and wrote nothing until the flag was set to
true on the box, at which point a `SyncStorePlan` triggered by "Restore" produced the
entitlement immediately.

The back office already solves the shape of problem this change needs. The account
directory lives in the **user** service; plan attributes live in **plans**. The
console composes them: `GetPlansForAccounts` decorates a page of accounts with plan
badges, and `ListAccountIdsByPlan` answers "which accounts match this plan filter"
so the console can intersect. The sandbox-account mark is another plans attribute and
follows the same path.

## Goals / Non-Goals

**Goals:**
- Sandbox transactions are honoured for designated accounts and nobody else.
- The designation is set from the back office, restricted to admins, audited, and
  visible in the directory.
- The pure mappers stay pure.
- Production stops carrying a switch that, left on, accepts sandbox transactions for
  every account.

**Non-Goals:**
- No client change. The app already sends the same purchase either way.
- No change to how production transactions are handled.
- Not a general "impersonate a plan" tool. The mark grants nothing on its own; it
  only decides whether an account's sandbox purchases are believed.
- No revisit of the stale pre-RevenueCat wording in `music-subscription-billing`.

## Decisions

### D1 — A plain mark, not an expiring grant

The mark is present or absent. No expiry.

An expiry is the obvious safety reflex, and it is wrong here: the accounts that carry
the mark are review accounts, used in bursts weeks apart. An expiry that lapsed
between two App Store submissions would reproduce **exactly** the failure this change
exists to fix — a reviewer purchases, nothing unlocks, and nothing says why. Silence
is the failure mode we are removing, so we do not reintroduce it on a timer.

The safety an expiry buys is bought instead by visibility: the directory filter
answers "who is marked?" at any moment, and the audit trail answers "who marked them,
and when?". Both are cheap to consult and neither can lapse.

Bounding the blast radius helps: the mark cannot grant premium. It only makes a
sandbox purchase count like a production one, so an account must still complete a
purchase to obtain anything.

*Alternative considered*: `expires_at` with a default of 90 days. Rejected for the
reason above; may be revisited if the mark is ever used for something wider than
review accounts.

### D2 — The mark is a plans-owned table, keyed by account

`plans.sandbox_accounts(user_id uuid primary key, created_at timestamptz, created_by text)`.
Presence is the mark; clearing deletes the row.

The port exposes a batch read as well as a single one. `GetPlansForAccounts`
decorates a whole page of the directory, so a per-row `is_sandbox_account` would be an N+1
against the directory's own pagination.

It is a **billing policy**, not an identity attribute, so it does not belong on a
users table owned by another domain — the same reasoning that keeps entitlements in
`plans`. Keying by account id and nothing else keeps the lookup a primary-key hit.

History is not kept in this table: clearing deletes the row, and the existing admin
audit trail already records both directions with the actor. Two records of the same
fact would drift.

### D3 — The mappers keep their signature; only the caller changes

`map_event` and `map_customer` continue to take a boolean. What changes is its
meaning and who computes it: instead of "this process accepts sandbox", it becomes
"sandbox is accepted **for this account**", resolved by the caller that already holds
the account id — the webhook handler from `ev.app_user_id`, `sync_customer` from its
`user_id` argument.

This is the smallest change that satisfies the constraint. The mappers stay pure and
keep receiving a decided answer rather than a policy to evaluate, so their existing
unit tests keep working by passing `true`/`false` as they do today. The parameter is
renamed to say what it now means.

*Alternative considered*: passing a `SandboxPolicy` trait object into the mappers.
Rejected — it would make the pure functions do I/O, which is the property that makes
them testable.

### D3b — Transfers are decided over every account they name

The sandbox check runs **before** the `TRANSFER` branch and before `app_user_id` is
parsed, so a single boolean resolved from `app_user_id` is not enough for a transfer:
the event names `transferred_from` and `transferred_to`, which may be several
accounts.

A sandbox transfer is honoured only when **every** account it names is marked.
Without that, a marked account could buy in the sandbox and transfer the entitlement onto an
ordinary account — granting premium to someone the mark never covered, which is the
hole this change exists to close.

The consequence for the caller: for `TRANSFER` it resolves the mark for the union of
`transferred_from`, `transferred_to` and `app_user_id`, and passes `true` only if all
are marked. For every other type, one lookup on `app_user_id`.

### D3c — An unparseable app_user_id is simply not marked

The sandbox check precedes the `Uuid::parse_str` guard, so a sandbox event carrying
something that is not a Cymbra account id — RevenueCat keeps an `$RCAnonymousID`
alias next to the account id, because the SDK configures before sign-in and then
logs in — resolves to "not marked" and is skipped as `Sandbox` rather than
`MalformedUser`.

That is the right outcome and needs no special case: an id we cannot resolve is an
account we cannot vouch for. It is named here only because the skip *reason* shifts,
and the ingest counters are read when diagnosing a purchase that did not land.

### D4 — No cache on the lookup

One indexed primary-key read per ingested event. Webhook volume is a handful of
events per purchase, and the reconciliation sweep runs nightly.

A cache would buy nothing measurable and would delay the effect of **clearing** the
mark, which is the safety-relevant direction. A stale "still marked" entry is
exactly the kind of quiet wrongness this change is meant to remove.

### D5 — Admin RPC alongside the existing plan admin surface

`SetSandboxAccount(user_id, enabled)`, gated and audited like `GrantPremium` and
`RevokeEntitlement`. The current state rides on `LookupAccountPlanResponse`, which the
account page already fetches, so the console needs no extra round trip to render the
checkbox.

### D6 — The directory filter extends the existing cross-domain path

`ListAccountIdsByPlanRequest` gains `sandbox_accounts_only`, and `AccountPlanBadge` gains
`sandbox_account` so a filtered list can show *why* a row matched. This reuses the route
the console already takes for the plan and beta filters rather than teaching the user
service about a plans concept.

### D7 — The environment flag is removed, not kept as a fallback

`CYMBRA_REVENUECAT_ALLOW_SANDBOX` disappears from `RcConfig`, `RevenueCatEnv`,
`backend/.env.example`, `backend/plans/README.md` and the production `.env`.

Keeping it "for staging" was the original intent, but no staging environment exists —
the repo deploys a single box from `docker-compose.prod.yml`, and nothing else
references the variable. Its only real use has been the manual production flip, which
this change replaces. A global switch retained solely to reopen the hole this change
closes is a liability, not a fallback.

A developer running the backend locally marks their own account instead, through
their own back office or a one-line insert.

## Risks / Trade-offs

- **A marked account is a standing hole for that account** → The mark grants nothing
  by itself, the accounts are ours, and the directory filter makes the list of marked
  accounts a one-click question rather than tribal knowledge.
- **Forgetting to mark the review account before submitting reproduces the rejection**
  → The console shows the mark on the account page next to the plan, where whoever
  prepares a submission is already looking; and the submission checklist in
  `apps/music/store/` is the place to name it.
- **Removing the flag is a breaking config change** → It is commented out in
  `.env.example` and false in production, so removing it changes no behaviour. The
  deploy note carries the one-line removal from the box's `.env`.
- **A sandbox transfer onto an unmarked account would leak premium** → Decided in
  D3b: every account a transfer names must be marked, or the event is dropped.
- **Deploy order** → Backend and back office ship together. If the backend lands
  first, the mark simply cannot be set yet; sandbox stays refused, which is today's
  production behaviour. No window is worse than the status quo.

## Migration Plan

1. Migration `0003_sandbox_accounts.sql` creates the table. Nothing to backfill: no
   account is marked until an admin says so.
2. Deploy the backend, then the back office.
3. Mark the two review accounts, and confirm the directory filter returns exactly
   them.
4. Delete `CYMBRA_REVENUECAT_ALLOW_SANDBOX` from the box's `.env` and roll
   `server` + `worker`. Confirm with `docker inspect` — `printenv` and `echo $VAR`
   misreport on this stack.
5. Verify end to end before resubmitting: a sandbox purchase on a marked account
   writes an entitlement row; the same purchase on an unmarked account writes none.

Rollback is the previous image plus restoring the `.env` line; the table can stay,
unread.

## Open Questions

Resolved during implementation:

- **Should `apps/music/store/README.md` name the mark as a pre-submission step?**
  Yes, done. It also had to say why the review accounts need *both* the mark and
  their Premium grant — the grant lets a reviewer see the paid features, the mark
  lets the purchase work if they try it — because holding only one of the two is
  exactly how this fails quietly.
