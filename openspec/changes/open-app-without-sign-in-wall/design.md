## Context

The first-run path today is `OnboardingGate` → language step → `WelcomeScreen` → `SessionGate`.
`SessionGate` routes `SessionUnauthenticated` to `EntryScreen`, a root home widget offering
Google, Apple, email and "continue without an account". All three exits of the welcome — Skip,
Sign in, Continue without an account — call the same `_finish()`, which only runs
`completeWelcome()`. None of them enters guest mode, so every one of them lands on the entry
screen. `onboarding_flow_test.dart` pins that outcome ("Skipping lands on the entry screen").

Contextual sign-in already exists and works: `inviteSignIn(context, ref, SignInBenefit)` shows a
dialog naming a benefit, pushes `SignInInvitationScreen` and resumes the caller once the session
resolves. Its benefits are `saveLibrary`, `earnPoints`, `leaderboards`, `goPublic` and
`keepProgress` — nothing about subscribing. It is called from the library and from the welcome's
"try now" only, and it shows no consent notice: the Terms / Privacy notice (`_LegalConsent`) is
private to `entry_screen.dart`.

On the paywall, `planProvider` returns `PlanSnapshotView.free` whenever
`canUseOnlineServicesProvider` is false, so a guest is rendered exactly like a signed-in free
user whose snapshot has `canPurchaseHere == false`: a "free" status and nothing else. The guest
account button calls `leaveGuest()`, which clears the guest flag and drops the user on the entry
screen.

Constraints that do not move: Premium is account-bound — the five server unlocks are granted per
account and `get_my_plan` requires an identity — and guest mode performs no call to Cymbra ID.

## Goals / Non-Goals

**Goals:**
- A first-time user reaches the app without meeting an account wall.
- "Continue without an account" does what it says.
- A guest can go from the paywall to a purchase through one sign-in, and land back on the paywall.
- Every sign-in or sign-up surface carries the consent notice.

**Non-Goals:**
- Anonymous purchase. Premium stays attached to an account.
- Changing where a user lands after signing out, deleting their account or losing their session.
- Any backend, proto, Cymbra ID or data change.

## Decisions

### D1. Guest mode is entered when the welcome is left, not at launch

Skip and Continue call `continueAsGuest()` **before** `completeWelcome()`. The order matters:
`OnboardingGate` hands over to `SessionGate` as soon as the welcome is complete, and if the
session were still unauthenticated at that frame, `EntryScreen` would flash before the guest
session arrived.

*Alternative — render the library for any unauthenticated session.* Rejected. It would also
change the destination after sign-out, account deletion and an expired session, which four more
specs pin (`account-management`, `handle-onboarding`, `mobile-session-signout`, the refresh rules
of `account-access`). Someone leaving an account is not in the situation of a first-time visitor,
and store review does not need that change.

*Alternative — enter guest mode at first launch, before the welcome.* Rejected: the welcome is
shown only while the session is unauthenticated, so it would stop appearing.

### D2. One resumable sign-in surface

The welcome's "Sign in" and the guest account button both go through `inviteSignIn` and the
pushed `SignInInvitationScreen`. `leaveGuest()` is no longer called from the account menu: the
guest choice is replaced only when authentication succeeds, so a user who backs out is still a
guest, on the screen they came from.

*Alternative — push `EntryScreen`.* Rejected: it is a root home widget, not a resumable route;
completing sign-in there restarts at the app root instead of returning the user to their action.

Task 1.3 verifies that authenticating from a guest session clears the persisted guest flag; if it
does not, the fix belongs in the session notifier, not in the screens.

### D3. `SignInBenefit.subscribe`

A new benefit whose copy carries the reason an account is needed, so the explanation is in the
product rather than only in a message to App Review: *"Sign in to subscribe. Premium is attached
to your Cymbra account, which is what makes it follow you on every device."* The key lands in all
four ARB files in the same change, with the drift check run before pushing.

### D4. The paywall's guest state is decided by the session, not the plan

A guest cannot be told apart from a signed-in free user by `PlanSnapshotView.free`, so the screen
reads `canUseOnlineServicesProvider`. When it is false the screen renders what Premium includes,
the account-bound explanation and a sign-in action — and it builds neither the purchase card nor
the restore action, so no plan-service or store call can start.

After sign-in no extra wiring is needed: `planProvider` is identity-scoped and rebuilds on the
auth change, so returning to the paywall re-reads the plan and the purchase card appears.

*Alternative — start the purchase automatically after sign-in.* Rejected. The guest never chose a
product: the store listing, which carries the prices, needs a store client bound to an account,
so no product button can exist before sign-in. Opening a purchase sheet the user did not ask for
would be worse than showing them the offer.

### D5. The consent notice becomes a shared widget

`_LegalConsent` moves out of `entry_screen.dart` into a shared widget used by `EntryScreen` and
`SignInInvitationScreen`, which covers the sign-up link reached from the latter. A guest does not
see it: guest mode uses no online service and creates no account, so acceptance belongs where an
account is created or used.

## Risks / Trade-offs

- **[Apple still objects under 5.1.1(v), since buying still needs an account]** → The change
  removes the reading that registration is needed to *use* the app, and puts the account-bound
  reason on screen at the moment of purchase. The reply to App Review argues the account basis.
  Anonymous purchase remains the fallback, as a separate change.
- **[A user who finished the welcome under an earlier build and never chose guest]** → They open on
  the entry screen, exactly as today. No regression, no migration.
- **[A guest never sees the Terms]** → Consistent with guest mode being offline; the notice is on
  every surface where an account is created or used.
- **[Tests pin the old behaviour]** → `onboarding_flow_test.dart` changes its assertion from the
  entry screen to the library in guest mode; the other entry-screen tests keep their meaning.

## Migration Plan

UI routing only: no data, no flag, no server change. It ships in the next music release and is
reverted by reverting the change.

## Open Questions

- Should signing out also land in guest mode rather than on the entry screen? Deferred. It moves
  four more specs and is not needed for store review.
