## Context

The gap was found by executing `add-drums-access` task 9.8 during the drums beta:
"close the campaign and check its members lose access … then re-open / re-enrol".
The first half worked; the second had no implementation. Recovering meant
`UPDATE plans.beta_campaigns SET closed_at = NULL WHERE key = 'midi-drums'`.

What the code already does, and what this change only has to expose:

- `close_campaign(key, actor)` writes `closed_at` and audits. It touches no
  membership and no entitlement (`backend/plans/src/service.rs`).
- `membership_is_active` requires three independent conditions: not revoked, the
  campaign not closed, the membership not expired
  (`backend/plans/src/core.rs`). Clearing `closed_at` restores exactly the
  members whose other two conditions still hold.
- `enrollment_closes_at` is a **different** column, tested on the redemption path
  only — but `close()` writes it too
  (`enrollment_closes_at = COALESCE(enrollment_closes_at, $2)`), so closing a
  campaign closes its enrolment as a side effect. That is why this change needs
  **two** inverses, not one: reopening only the campaign would restore a beta
  that nobody new can join, and the operator would be stuck at exactly the same
  point as before.

## Goals / Non-Goals

- **Goal**: make the inverse of `CloseCampaign` reachable, and write down what
  closing and reopening mean for people.
- **Non-goal**: changing what closing does. The behaviour is right; it is
  undocumented and one-way.
- **Non-goal**: reopening enrolment as part of reopening a campaign. Two
  decisions, two controls.

## Decisions

### Closing is a pause; reopening restores

The alternative — deleting memberships, or stamping each one with an end date at
close time — was rejected on two grounds. It destroys the record of who took part
(the member list and the CSV export exist to keep it, and a beta cohort is
something you thank or convert later). And it duplicates a gesture that already
exists and is better targeted: `RevokeMembership` removes one person, for good.
A global control that quietly performs N individual revocations would make the
two indistinguishable afterwards.

The cost of this choice is surprise: an admin who closed a campaign to *end* a
wave and reopens it for a *new* one gets the old cohort back. That is why the
reactivation count is part of this change rather than a nicety — the behaviour is
correct, so the fix is to announce it.

### The trial asymmetry is documented, not changed

Closing a `premium_trial` campaign leaves already-granted entitlements running to
their own `ends_at`. Revoking them at close time would take back a right the
product granted, on a date the recipient never agreed to. Keeping it silent,
though, means an operator closing a trial campaign believes they have stopped
something they have not. The spec now says both halves.

## Risks

- **An operator reopens a campaign expecting an empty one.** Mitigated by the
  reactivation count in the confirmation; the alternative design would have
  mitigated it by losing data.
- **Reopen becomes a way to resurrect a spent code.** It is not: codes carry
  their own redemption state, and `redeem` re-checks `AccessCode::is_redeemable`
  independently of the campaign's closed flag.
