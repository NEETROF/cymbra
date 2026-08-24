## Why

A closed beta campaign cannot be reopened. Not by the console, not by the API,
not by anything: `PlanService` offers `CloseEnrollment` and `CloseCampaign` with
no counterpart, and `admin-plan-console` lists create / close enrolment / close /
mint / revoke / list / export without ever naming the inverse. The spec does not
declare closing irreversible either — the operation was simply forgotten.

The gap was found the way gaps are found: by following the project's own
instructions. `add-drums-access` task 9.8 prescribes "close the campaign and
check its members lose access … **then re-open / re-enrol** for the rest of the
beta". Half of that task cannot be performed. Closing the `midi-drums` campaign
during the beta therefore ends it, and the only way back is an `UPDATE` on
`plans.beta_campaigns` — which is what we had to do.

Nothing is lost when a campaign closes, which is what makes the omission cheap
to fix and expensive to leave: `close_campaign` writes `closed_at` and touches no
membership. The rows stay; `membership_is_active` merely stops counting them
because it tests `campaign.closed_at`. Clearing that column restores every member
exactly as they were.

## What Changes

**The operation**

- `ReopenCampaign(key)` and `ReopenEnrollment(key)` on `PlanService`,
  admin-guarded and audited like every other campaign mutation. **Two** inverses,
  because `close()` writes both dates: closing a campaign closes its enrolment as
  a side effect, so a single reopen would restore a beta nobody new can join.
  They stay separate acts — a campaign may legitimately be live for its members
  and closed to newcomers — and reopening a campaign says when enrolment remains
  closed instead of letting the admin discover it.
- The console gets the action on a closed campaign's row, where "Fermer la
  campagne" sits on an open one.

**The semantics, stated at last**

Closing is a **pause**, not an end: reopening restores every member who was not
individually revoked. That is already how the data behaves, and it is the right
rule — `RevokeMembership` is the per-person tool and a revoked membership never
comes back, so the global gesture stays global. The alternative (deleting or
stamping memberships at close time) would destroy the record of who took part,
which is exactly what the member list and the CSV export exist to keep.

**The signal that was missing**

The defect is not only the absent button; it is that nothing said what closing
and reopening do to people:

- Reopening SHALL state how many memberships it will reactivate, before it acts.
- A closed campaign's member list SHALL show its members as inactive rather than
  listing them as if nothing had happened.

**One asymmetry to write down**

Closing a `premium_trial` campaign does not revoke the entitlements it already
granted: they live until their own `ends_at`, and `close_campaign` never touches
them. So closing ends a *feature* beta immediately but leaves a *trial*'s premium
running, and reopening a trial campaign gives nothing back. This is defensible —
a trial granted is a right acquired for its term — but it is currently written
nowhere, so an operator can only discover it by surprise.

## Impact

- `admin-plan-console` — the requirement gains the reopen operation, the
  reactivation notice, the inactive-member display, and the lifecycle statement.
- `music-access-codes` — unchanged: reopening does not resurrect spent codes.
- Backend: one RPC, one service method, one repo method (`closed_at = NULL`), one
  audit entry. No migration: the column is already nullable.
- Back office: one action, one confirmation dialog, one member-list state.
