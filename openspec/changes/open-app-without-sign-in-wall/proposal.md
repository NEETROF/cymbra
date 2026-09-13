## Why

A first-time user meets a sign-in wall before they can use the app: language → welcome →
the account entry screen. Worse, the welcome's "Continue without an account" only marks the
welcome as done, so it lands on that same wall. Two specs contradict each other —
`welcome-onboarding` forbids "a mandatory account wall", `account-access` makes "account entry
the launch experience" — and a guest who wants to subscribe finds no way forward on the
paywall. Apple refused macOS 1.32.0 under guideline 5.1.1(v), reading the app as requiring
registration, and the next build must not reinforce that reading.

## What Changes

- First run goes language → welcome → **the app, in guest mode**. "Skip" and "Continue without
  an account" enter guest mode and persist it; the welcome's "Sign in" opens the sign-in
  surface instead of the wall.
- The account entry screen **stops being the launch experience** for a first-time visitor. It
  stays where a user lands after signing out, deleting their account or losing their session.
- A guest upgrading to an account uses the **contextual, pushed sign-in surface** and comes back
  to where they were, instead of being sent to the entry screen.
- The paywall gains a **guest state**: what Premium includes, that it is attached to the Cymbra
  account — which is what makes it follow the user on every device — and a sign-in invitation
  with a new "subscribe" benefit. After sign-in the user is back on the paywall with the
  purchase actions available.
- The **consent notice** (Terms of Service and Privacy Policy) is shown on every sign-in and
  sign-up surface, including the pushed one, not only on the entry screen.
- Premium stays account-bound: no anonymous purchase, and guest mode stays fully offline.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `account-access`: account entry is no longer the first-run launch experience; a guest upgrades
  through the contextual sign-in surface; cancelling a Google or Apple sheet returns to the
  surface it was started from.
- `welcome-onboarding`: leaving the welcome without signing in enters guest mode and opens the
  app; the welcome's sign-in action opens the sign-in surface.
- `music-premium-paywall`: a guest state that invites sign-in to subscribe; restore purchases is
  offered to users with an account session.
- `legal-links`: the consent notice is required on every surface where a user signs in or creates
  an account.

## Impact

- **Products.** Cymbra Music only (`apps/music`). It **consumes** Cymbra ID sign-in unchanged
  (`SignInLocal`, `SignInOidc`, audience `music`). Nothing new in Cymbra ID, Live, the back
  office or the site; no backend, proto or data change.
- **Code.** `screens/onboarding/welcome_screen.dart`, `screens/onboarding/sign_in_invitation.dart`
  (new `SignInBenefit.subscribe`, consent notice), `screens/auth/entry_screen.dart` (consent
  notice extracted and shared), `screens/auth/account_menu.dart` (guest sign-in),
  `screens/plan_screen.dart` (guest state), the four ARB files.
- **Tests.** `onboarding_flow_test.dart` pins "skipping lands on the entry screen" and must change;
  `auth_entry_test.dart`, `legal_links_test.dart`, the plan screen tests and the integration
  `app_test.dart` are affected.
- **Release.** Must ship in the next music build, before iOS and macOS are resubmitted to App
  Review.
