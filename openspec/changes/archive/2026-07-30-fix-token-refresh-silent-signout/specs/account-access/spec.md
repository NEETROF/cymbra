## MODIFIED Requirements

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
