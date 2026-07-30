# account-access Specification

## Purpose
TBD - created by archiving change add-music-account-access. Update Purpose after archive.
## Requirements
### Requirement: Account entry is the launch experience

The app SHALL present an account entry screen as the first screen whenever there
is no resolvable session (no stored guest choice and no valid Cymbra ID session).
The entry screen SHALL offer exactly four mutually exclusive choices: continue
with Google, continue with Apple, continue with email, and continue without an
account (guest). The screen SHALL follow the Cymbra theme (`CymbraColors`,
Material 3 dark).

#### Scenario: First launch with no prior choice
- **WHEN** the app starts and no session and no guest choice is stored
- **THEN** the account entry screen is shown as the home screen with the four entry options

#### Scenario: Returning user with a valid session
- **WHEN** the app starts and a valid (or silently refreshable) Cymbra ID session is stored
- **THEN** the entry screen is skipped and the app opens directly on the library

#### Scenario: Returning guest
- **WHEN** the app starts and the persisted choice is guest
- **THEN** the entry screen is skipped and the app opens directly on the library in guest mode

### Requirement: Sign in scoped to the music audience

All sign-in calls to Cymbra ID SHALL pass the audience `music`. The app SHALL
store the returned access and refresh tokens and treat the user as signed in for
the `music` audience only.

#### Scenario: Audience attached on sign-in
- **WHEN** the app calls any Cymbra ID sign-in RPC (`SignInLocal` or `SignInOidc`)
- **THEN** the request carries audience `music`

### Requirement: Google sign-in

The app SHALL let the user authenticate with Google by obtaining a Google
`id_token` via the native Google sign-in SDK and exchanging it through Cymbra
ID's `SignInOidc`. The app SHALL NOT perform the OAuth token exchange itself. A
`SignInOidc` failure SHALL surface a provider-appropriate message and SHALL NOT
reuse the email-credential copy "Incorrect email or password."

#### Scenario: Successful Google sign-in
- **WHEN** the user picks "continue with Google" and completes the Google consent flow
- **THEN** the app sends the returned `id_token` to `SignInOidc(audience="music")` and, on success, stores the session and continues into the app

#### Scenario: Google flow cancelled
- **WHEN** the user dismisses the native Google sheet without completing it
- **THEN** no RPC is sent and the user returns to the entry screen with no error surfaced

#### Scenario: Google sign-in failure is not shown as a password error
- **WHEN** `SignInOidc` fails with `UNAUTHENTICATED` after a Google sign-in
- **THEN** the app shows a Google/sign-in-context message, not "Incorrect email or password."

### Requirement: Apple sign-in

The app SHALL let the user authenticate with Apple by obtaining an Apple
`id_token` via the native Sign in with Apple SDK and exchanging it through Cymbra
ID's `SignInOidc`. Sign in with Apple SHALL be offered on Apple platforms wherever
Google sign-in is offered (App Store requirement). A `SignInOidc` failure SHALL
surface a provider-appropriate message and SHALL NOT reuse the email-credential
copy "Incorrect email or password."

#### Scenario: Successful Apple sign-in
- **WHEN** the user picks "continue with Apple" and completes the Apple flow
- **THEN** the app sends the returned `id_token` to `SignInOidc(audience="music")` and, on success, stores the session and continues into the app

#### Scenario: Apple flow cancelled
- **WHEN** the user cancels the Apple sheet
- **THEN** no RPC is sent and the user returns to the entry screen with no error surfaced

#### Scenario: Apple sign-in failure is not shown as a password error
- **WHEN** `SignInOidc` fails with `UNAUTHENTICATED` after an Apple sign-in
- **THEN** the app shows an Apple/sign-in-context message, not "Incorrect email or password."

### Requirement: Guest mode is fully offline

The app SHALL provide a guest mode that performs no calls to Cymbra ID and grants
no access to any online (backend-bound) service. Choosing guest SHALL persist the
choice so the entry screen is not shown again on subsequent launches. The app
SHALL expose an explicit way to leave guest mode and reach the entry screen to
create or sign in to an account.

#### Scenario: Entering guest mode
- **WHEN** the user picks "continue without an account"
- **THEN** the choice is persisted, no Cymbra ID RPC is made, and the app opens the library with full local functionality

#### Scenario: Online services blocked for guests
- **WHEN** a guest attempts to use a feature that depends on a Cymbra ID session
- **THEN** the feature is unavailable (hidden or disabled) and the app offers to sign in or create an account instead of calling the backend

#### Scenario: Guest upgrades to an account
- **WHEN** a guest chooses to sign in or create an account from within the app
- **THEN** the app returns to the entry screen and, on successful authentication, replaces the guest choice with the new session

### Requirement: Secure session storage

The app SHALL store access and refresh tokens in platform secure storage
(`flutter_secure_storage`, backed by Keychain/Keystore) and SHALL NOT persist
tokens in plain preferences or log them.

#### Scenario: Tokens persisted securely
- **WHEN** a sign-in succeeds
- **THEN** the access and refresh tokens are written to secure storage and are available on the next launch

### Requirement: Silent token refresh

The app SHALL refresh the session without user interaction when the access token
is expired or a protected RPC returns `UNAUTHENTICATED`, by calling Cymbra ID's
`Refresh` with the stored refresh token and replacing the stored token pair.

Refresh SHALL be **coordinated**: at most one `Refresh` call for the stored
session is in flight at any time. When several protected RPCs need a refresh
concurrently, they SHALL all await the outcome of that single `Refresh` and reuse
its result, and MUST NOT each replay the stored refresh token (which the backend
rotates and treats as reuse, revoking the whole session family).

The app SHALL classify a refresh outcome before touching stored state:
- A **terminal rejection** — the refresh token is expired or revoked
  (`UNAUTHENTICATED` / `INVALID_ARGUMENT`) — SHALL clear the stored session and
  route the user back to the entry screen.
- A **transient failure** — the backend is unreachable or the call times out
  (`UNAVAILABLE`, deadline exceeded, offline) — SHALL NOT clear the stored
  session. The stored token pair SHALL be left intact so a later retry (this
  launch when back online, or the next launch) can recover, and the failure
  SHALL NOT be surfaced to the session bootstrap as `UNAUTHENTICATED`.

#### Scenario: Access token expired
- **WHEN** a protected RPC fails with `UNAUTHENTICATED` and a refresh token is stored
- **THEN** the app calls `Refresh`, stores the new token pair, and retries the original RPC once

#### Scenario: Concurrent refreshes are coordinated
- **WHEN** several protected RPCs fail with `UNAUTHENTICATED` at the same time with one stored session
- **THEN** exactly one `Refresh` is sent, every waiting call reuses its rotated token pair, and the stored refresh token is never replayed by a second concurrent call

#### Scenario: Refresh token no longer valid
- **WHEN** `Refresh` fails with `UNAUTHENTICATED` or `INVALID_ARGUMENT` because the refresh token is expired or revoked
- **THEN** the app clears the stored session and shows the entry screen

#### Scenario: Refresh fails transiently (offline / weak network)
- **WHEN** `Refresh` fails with a transient error (`UNAVAILABLE`, deadline exceeded, or no connectivity)
- **THEN** the stored session is left intact, the user is not signed out, and a later online retry can recover the session

#### Scenario: Expired access token at launch while offline
- **WHEN** the app launches with a stored session whose access token is expired and the network is unavailable so the refresh cannot complete
- **THEN** the app keeps the user signed in (resolving the account when connectivity returns) instead of clearing the session and routing to the entry screen

### Requirement: Sign out

The app SHALL let a signed-in user sign out, calling Cymbra ID's `Logout` to
revoke the refresh token and clearing the locally stored session.

#### Scenario: User signs out
- **WHEN** the user chooses sign out
- **THEN** the app calls `Logout` with the stored refresh token, clears secure storage, and returns to the entry screen

#### Scenario: Sign out while offline
- **WHEN** the user signs out and the `Logout` RPC cannot reach the backend
- **THEN** the app still clears the local session and returns to the entry screen

