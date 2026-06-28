## Why

The post-auth handle screen is a **hard dead-end**: it has no back, sign-out, or
cancel affordance, and its "Continue" button stays disabled until a *valid,
available* handle is entered. A user whose desired handle is already taken (often by
their own other account) is trapped — quitting the app is the only escape. Worse,
the account is provisioned at sign-in (before any handle), so abandoning leaves a
**handle-less orphan account** in the database keyed by `(provider, subject)`. This
is an urgent usability + data-hygiene defect, independent of account linking.

## What Changes

- **Escape hatch on the handle screen.** Add a clear "Use a different account / Sign
  out" action so the user can always leave: it signs out (best-effort `Logout`),
  clears the local session, and returns to the entry screen.
- **Delete-on-abandon.** When the user explicitly abandons onboarding for a *just-
  created* account (no handle yet), the app deletes that account via `DeleteAccount`
  so no orphan is left behind. Sign-out without delete remains possible if the
  account already has a handle (existing user).
- **Backend orphan reaper (safety net).** A backend job purges handle-less accounts
  older than a grace period, covering hard app kills where delete-on-abandon can't
  run. **BREAKING**: none — additive maintenance job.

## Capabilities

### New Capabilities
- `handle-onboarding`: The behavior and guarantees of the post-authentication
  handle-selection gate — that it is always escapable, that abandoning a brand-new
  account cleans it up, and that handle-less accounts do not accumulate.

### Modified Capabilities
<!-- account-access / account-management (add-music-account-access) are not yet
     archived into openspec/specs/, so there is no delta target. The handle gate's
     "always escapable" guarantee is captured as a new handle-onboarding requirement. -->

## Impact

- **App** (`apps/music/`): add an escape action to `handle_onboarding_screen.dart`
  (or its `AuthScaffold`), wired to `SessionNotifier.signOut` plus a new
  abandon-and-delete path for handle-less accounts; reuse existing `DeleteAccount`
  for OIDC accounts (delete re-auth uses the fresh OIDC token already in hand).
- **Backend**: a scheduled/maintenance task in the user module that deletes accounts
  with a null handle and `created_at` older than a configurable grace period.
- **No new app or proto dependencies.** Uses existing `Logout`/`DeleteAccount`.
- **Tests/coverage**: ≥80% (Flutter + Rust) maintained; escape + abandon-delete
  covered with fakes; reaper covered by a host-testable unit (pure selection logic).
- **Out of scope**: the "sign in to link an existing account" flow (handled in
  `add-account-identity-linking`); revealing the existing account's sign-in method.
