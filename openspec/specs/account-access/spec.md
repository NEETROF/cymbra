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

### Requirement: An unresolved account is re-resolved without user action

The app SHALL keep re-attempting account resolution for as long as a session is
authenticated but its account is unresolved. Keeping the user signed in after a
transient `GetAccount` failure is correct, but that degraded state MUST NOT be
terminal: it hides the user's handle, leaves the app with no user identity, and
silently disables everything keyed on it (play-session capture, feature-flag
targeting, the favorites index, the offline cache, plan resolution). It SHALL NOT
require the user to sign out, restart, or take any action to recover.

Re-attempts SHALL use a bounded backoff with a maximum interval, so an
unreachable backend is never hot-looped. Resolution SHALL be single-flight: at
most one account resolution is in flight at a time, and a concurrent trigger
joins the pending attempt rather than issuing a second call. Re-attempts SHALL
stop as soon as the account resolves or the session ends. A resolution still in
flight when its session ends SHALL be discarded: its outcome MUST NOT restore the
ended session, and a session that replaces it MUST issue its own resolution
rather than join or inherit the stale one.

A **terminal** failure encountered during a re-attempt (the session is revoked,
or the account no longer exists) SHALL clear the stored session and route the
user to the entry screen, exactly as it does on the first attempt.

#### Scenario: A transient failure at sign-in recovers on its own
- **WHEN** the account cannot be fetched after a successful sign-in because of a transient failure, and the backend becomes reachable again
- **THEN** the app re-resolves the account with no user action, and the handle, user identity and every identity-keyed feature become available

#### Scenario: A transient failure at launch recovers on its own
- **WHEN** the app launches with a stored session, the account cannot be fetched because of a transient failure, and the backend becomes reachable again
- **THEN** the app re-resolves the account with no user action

#### Scenario: An unreachable backend is not hot-looped
- **WHEN** account resolution keeps failing transiently
- **THEN** successive re-attempts are spaced by a growing delay up to a maximum interval, rather than repeating immediately

#### Scenario: Concurrent resolutions are coordinated
- **WHEN** a re-attempt is triggered while an account resolution is already in flight
- **THEN** exactly one resolution call is made and the trigger reuses its outcome

#### Scenario: Re-attempts stop once the account resolves
- **WHEN** an account resolution succeeds
- **THEN** no further re-attempt is scheduled

#### Scenario: Re-attempts stop when the session ends
- **WHEN** the user signs out, deletes the account, or the session is otherwise torn down while re-attempts are scheduled
- **THEN** the pending re-attempt is cancelled and no further account resolution is issued

#### Scenario: A resolution in flight when the session ends is discarded
- **WHEN** the user signs out while an account resolution is in flight, and that resolution later completes
- **THEN** the user stays signed out whatever its outcome, and no further account resolution is issued

#### Scenario: A new session never inherits a stale resolution
- **WHEN** the user signs out while an account resolution is in flight and signs in again before it completes
- **THEN** the new session issues its own account resolution, and the stale one's outcome is neither applied to it nor able to sign it out

#### Scenario: A terminal failure during a re-attempt signs the user out
- **WHEN** a re-attempt fails because the session is revoked or the account no longer exists
- **THEN** the stored session is cleared and the entry screen is shown

### Requirement: Account re-resolution runs only in the foreground

The app SHALL suspend account re-resolution whenever it is not in the foreground
and SHALL resume it on returning to the foreground. A pending re-attempt SHALL be
cancelled when the app is paused, hidden, or detached, so neither a timer nor a
network call outlives the foreground session. This is not implied by the host
platform: a backgrounded desktop app keeps executing (unlike a mobile process,
which is frozen), so an ungated backoff loop would keep spending battery and
issuing calls against a backend the user is not looking at.

On returning to the foreground with a still-unresolved account, the app SHALL
attempt resolution immediately and reset the backoff, rather than waiting out the
delay accumulated before it was suspended.

#### Scenario: Leaving the foreground cancels the pending re-attempt
- **WHEN** the app is paused, hidden, or detached while a re-attempt is scheduled
- **THEN** the scheduled re-attempt is cancelled and no account resolution call is issued while the app is out of the foreground

#### Scenario: Returning to the foreground retries immediately
- **WHEN** the app returns to the foreground and the session is still authenticated with an unresolved account
- **THEN** account resolution is attempted immediately and the backoff is reset

#### Scenario: Returning to the foreground with a resolved account does nothing
- **WHEN** the app returns to the foreground and the account is already resolved
- **THEN** no account resolution is issued

#### Scenario: A resume during an in-flight attempt does not double-fire
- **WHEN** the app returns to the foreground while an account resolution is already in flight
- **THEN** no second resolution is issued and the in-flight one is awaited

