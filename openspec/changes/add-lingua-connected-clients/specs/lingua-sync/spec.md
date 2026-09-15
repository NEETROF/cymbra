# lingua-sync — Cymbra ID account and synchronisation: clients

## ADDED Requirements

### Requirement: Optional account, local-first preserved
The extension and the container app SHALL remain fully usable without an account: highlighting, statuses, decks, review and local stats all work with no connection and no sign-in, and synchronisation SHALL start only after an explicit sign-in (opt-in). Signing out SHALL stop synchronisation without altering local state.

#### Scenario: Account-free use unchanged
- **WHEN** a user without an account reads a page with the extension installed
- **THEN** highlighting, the word popup and review work in full and no request is sent to the Cymbra backend

#### Scenario: Signing out loses nothing
- **WHEN** a signed-in user signs out of the extension
- **THEN** their local statuses, cards and stats stay intact and usable offline, and no further sync request is sent

### Requirement: Extension sign-in over gRPC-web bearer
The extension SHALL sign in over gRPC-web with a bearer `TokenPair` (never the `/web/auth` cookie surface): auth/refresh interceptors modelled on the back office, single-flight refresh with exactly one retry, the access token kept in `storage.session` and the refresh token in `storage.local`. Two methods SHALL be offered: Google OIDC through `chrome.identity.launchWebAuthFlow` (client id added to the `CYMBRA_GOOGLE_AUDIENCE` CSV) and email/password (`SignInLocal`).

#### Scenario: Google sign-in from the extension
- **WHEN** the user picks "Continue with Google" in the extension
- **THEN** the `launchWebAuthFlow` flow produces an id_token, `SignInOidc` returns a `TokenPair` for the `lingua` audience, and subsequent calls carry `Authorization: Bearer`

#### Scenario: Expired access token refreshed silently
- **WHEN** a sync call fails with UNAUTHENTICATED while a valid refresh token is present
- **THEN** the client refreshes exactly once (single-flight), replays the call with the new token, and the user sees no interruption

#### Scenario: Browser restart
- **WHEN** the user restarts their browser
- **THEN** the access token from `storage.session` is gone, the refresh token from `storage.local` obtains a new `TokenPair`, and the session continues without re-entering credentials

### Requirement: Native sign-in in the Apple app with Sign in with Apple
The container app SHALL sign in natively over gRPC-web against the `lingua` audience, offering Sign in with Apple, Continue with Google and email/password at the same level (Sign in with Apple first on iOS), plus native email sign-up with code verification and password reset. Its tokens SHALL live in a Keychain access group shared only with its Safari extension, and signing out SHALL revoke the refresh token (`Logout`) and clear that Keychain. The app SHALL hold no learning state: the session is the only account data it owns.

#### Scenario: Sign in with Apple present
- **WHEN** the iOS sign-in screen shows "Continue with Google"
- **THEN** "Continue with Apple" is offered at the same level and yields a `TokenPair` for the `lingua` audience through the existing Apple OIDC

#### Scenario: Creating an email account in the app
- **WHEN** a user without an account signs up with an email and a password in the app
- **THEN** a verification code is sent, entering it signs them in, and no web page is needed

#### Scenario: Forgotten password
- **WHEN** a user with an email account asks to reset their password in the app
- **THEN** they receive a reset code and set a new password without leaving the app

### Requirement: The Safari extension uses the app's session
On Safari the extension SHALL hold no sign-in form and no refresh token: it SHALL obtain short-lived access tokens from the container app's native extension handler over native messaging, and only the native side SHALL refresh, serialised across the app and the extension, because the server revokes the whole token family when a refresh token is replayed. Without a session in the app, the extension SHALL show the signed-out state with a way to open the app and SHALL send no sync request.

#### Scenario: The Safari extension inherits the session
- **WHEN** the user is signed in to the container app and reads a page in Safari
- **THEN** the extension syncs under the app's account, with no sign-in screen in Safari

#### Scenario: Signing out in the app
- **WHEN** the user signs out in the container app
- **THEN** the Safari extension shows the signed-out state and sends no further sync request, its local state intact

#### Scenario: One refresh for two callers
- **WHEN** the app and the Safari extension both need a new access token at the same moment
- **THEN** a single refresh request is sent, both receive the new token, and the session is not revoked

### Requirement: Local store merged at first sign-in
At the first sign-in of a device holding pre-account local state, the client SHALL push that state in full as operations preserving their original timestamps, then pull the merged state; the merge SHALL be the protocol's ordinary last-write-wins (no special case) and SHALL lose nothing: every local status, card or aggregate missing from the server is created.

#### Scenario: First sign-in after months of local use
- **WHEN** a user with 2,000 local statuses and 150 local cards signs in for the first time
- **THEN** all of it goes up with its original timestamps, and the merged state pulled back contains the union with any pre-existing server state

#### Scenario: A second device already used locally
- **WHEN** a second device with its own local state signs in to the same account
- **THEN** the two states are merged by last-write-wins and no card from either device is lost

### Requirement: The Claude Code plugin stays out of synchronisation
The Claude Code plugin's local store (`~/.lingua/`) SHALL NOT acquire any network path in this change: the ingestion's "no network connection" invariant SHALL remain tested and true, plugin data synchronisation being a later change.

#### Scenario: Ingestion still offline
- **WHEN** the `Stop` hook ingests a transcript while the user has a Lingua account signed in inside their extension
- **THEN** the ingestion opens no network connection and the plugin's store stays purely local
