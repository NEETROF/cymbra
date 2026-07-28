# mobile-session-signout Specification

## Purpose
TBD - created by archiving change add-mobile-sign-out-everywhere. Update Purpose after archive.
## Requirements
### Requirement: A user can sign out of all devices from the app

The mobile app SHALL provide an account action that revokes **all** of the signed-in
user's sessions by calling `AuthService.RevokeAllSessions`. On success it SHALL tear down
the local session (clear the stored tokens, sign out of any OIDC provider, and return to
the entry screen), because the current device's refresh token is revoked too.

#### Scenario: Sign out from all devices succeeds

- **WHEN** a signed-in user confirms "sign out from all devices"
- **THEN** the app revokes all of the account's sessions server-side, clears the local tokens, and returns to the entry screen

### Requirement: A failed revoke keeps the user signed in

If the `RevokeAllSessions` call fails, the app MUST NOT clear the local session; it SHALL
keep the user signed in and surface the failure, so the user is not left locally signed
out while other devices remain active.

#### Scenario: The revoke call fails

- **WHEN** the "sign out from all devices" action's `RevokeAllSessions` call returns an error
- **THEN** the local session is preserved (still signed in) and the error is surfaced, rather than tearing down locally

### Requirement: The destructive action is confirmed

The app SHALL require an explicit confirmation before signing out of all devices, since
it ends sessions across every device.

#### Scenario: Confirmation is required

- **WHEN** the user taps "sign out from all devices"
- **THEN** a confirmation is shown, and the revoke happens only if the user confirms

