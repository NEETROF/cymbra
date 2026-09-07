## Why

Production drops every `SANDBOX` store transaction: `CYMBRA_REVENUECAT_ALLOW_SANDBOX`
is false and the mappers skip those events with `SkipReason::Sandbox`. That was
measured, not assumed — a TestFlight purchase on 2026-09-06 succeeded at Apple,
was recorded by RevenueCat, and produced **no entitlement row** until the flag was
flipped by hand on the box.

App Review buys in the sandbox. A reviewer completing the purchase we are being
asked to document would therefore pay and unlock nothing, which reads as a broken
app and costs a rejection under guideline 2.1 or 3.1.1. The App Store already
refused 1.30.0 once and asked us to describe the purchase flow, so a reviewer
trying it is likely, not hypothetical.

The current workaround — flip the global flag, test, flip it back — opens the door
for **every** account for as long as it is open, and depends on someone remembering
to close it. It is not something to repeat on a published app.

## What Changes

- Accounts can be marked **store testers**. A tester's sandbox transactions are
  applied; everyone else's are still dropped.
- The back office gains a checkbox on the account page to set and clear that mark,
  restricted to admins and written to the existing audit trail.
- The account directory gains a filter for store testers, so the mark is
  discoverable and reviewable rather than something only its author remembers.
- **BREAKING** `CYMBRA_REVENUECAT_ALLOW_SANDBOX` is removed. It is documented
  "staging only" and there is no staging environment: the repo deploys one box from
  `docker-compose.prod.yml`, and nothing else references the variable. Its only real
  use has been the manual production flip described above, which this change
  replaces with something narrower and auditable. Keeping both would leave a global
  switch whose sole remaining purpose is to open the hole this change closes.
- The sandbox decision moves from a process-wide boolean to a per-account lookup
  resolved **before** the pure mappers are called, so `map_event` and `map_customer`
  stay pure and keep taking a decided answer rather than a policy.

## Capabilities

### New Capabilities

None. This narrows an existing rule and extends two admin surfaces.

### Modified Capabilities

- `music-subscription-billing`: sandbox acceptance becomes a property of the
  purchasing account. The rule was never specified — it lived only in code — so the
  delta **adds** it rather than modifying anything, and adds the transfer rule that
  keeps a sandbox entitlement from reaching an unmarked account.
- `admin-plan-console`: the account page gains an audited, admin-only control for
  marking an account a store tester.
- `admin-account-directory`: the directory gains a store-tester filter.

## Impact

- `backend/plans`: a store-tester store and its port; `RcConfig` loses
  `allow_sandbox`; the webhook handler and `SyncStorePlan` resolve the flag per
  account before mapping; a new admin RPC to set it, audited like the existing
  admin grants.
- Database: a `plans`-owned table. The mark is a billing policy, not an identity
  attribute, so it does not belong on a users table owned by another domain.
- `apps/back-office`: checkbox on the account page, filter on the directory.
- Configuration: `CYMBRA_REVENUECAT_ALLOW_SANDBOX` disappears from
  `backend/.env.example`, `backend/plans/README.md` and the production `.env`.
- Not affected: the app. Nothing changes client-side — the same purchase flow is
  either honoured or ignored server-side.

## Open question for design

Whether the mark is a plain boolean or a grant that expires. A boolean is simpler
and matches how review accounts are actually used (long-lived, few, deliberate); an
expiry closes the hole on its own if someone forgets. Design decides.
