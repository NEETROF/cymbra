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
The extension SHALL sign in over gRPC-web with a bearer `TokenPair` (never the `/web/auth` cookie surface): auth/refresh interceptors modelled on the back office, single-flight refresh with exactly one retry, the access token kept in `storage.session` and the refresh token in `storage.local`. The sign-in methods SHALL be those specified by `lingua-account` (at least Google OIDC through `chrome.identity.launchWebAuthFlow`, client id added to the `CYMBRA_GOOGLE_AUDIENCE` CSV, and email/password through `SignInLocal`).

#### Scenario: Google sign-in from the extension
- **WHEN** the user picks "Continue with Google" in the extension
- **THEN** the `launchWebAuthFlow` flow produces an id_token, `SignInOidc` returns a `TokenPair` for the `lingua` audience, and subsequent calls carry `Authorization: Bearer`

#### Scenario: Expired access token refreshed silently
- **WHEN** a sync call fails with UNAUTHENTICATED while a valid refresh token is present
- **THEN** the client refreshes exactly once (single-flight), replays the call with the new token, and the user sees no interruption

#### Scenario: Browser restart
- **WHEN** the user restarts their browser
- **THEN** the access token from `storage.session` is gone, the refresh token from `storage.local` obtains a new `TokenPair`, and the session continues without re-entering credentials

### Requirement: Apple and Google sign-in on Safari through the host app
On Safari, which has no `identity.launchWebAuthFlow`, the container app SHALL provide Sign in with Apple and Continue with Google natively (Sign in with Apple first) and SHALL hand the resulting id_token to its Safari extension through the App Group, readable once and for at most five minutes. The Safari extension SHALL exchange that id_token through `SignInOidc` against the `lingua` audience and SHALL own the resulting session like every other variant. The container app SHALL hold neither a session nor learning state, and the email flows SHALL stay in the extension.

#### Scenario: Apple sign-in from Safari
- **WHEN** the reader picks "Continue with Apple" in the Safari extension, completes the native Apple sheet in the container app and goes back to Safari
- **THEN** the extension exchanges the handed id_token through `SignInOidc`, is signed in to the same Cymbra account as with Apple on any other client, and schedules a sync

#### Scenario: A handed id_token is used once
- **WHEN** the extension has collected a handed id_token, or the token is older than five minutes
- **THEN** the App Group no longer holds it, and an expired token is discarded without any RPC

#### Scenario: Email flows stay in the extension
- **WHEN** a Safari reader signs up, verifies an email or resets a password
- **THEN** it happens in the extension's account page, with no native screen involved

### Requirement: Synchronisation keeps up with the reader
A signed-in device SHALL exchange with the server without the reader having to restart their browser: after a local mutation, when a Lingua surface opens (popup, in-page drawer, side panel), when a page loads or the reader returns to a tab, and when the extension's background wakes. Those triggers SHALL be throttled — at most one exchange every 10 s for a surface the reader opened, every 60 s for a page load — counted from the last successful one, which SHALL survive a background restart. A trigger SHALL be acknowledged only once its exchange is over, so the requesting surface keeps the background alive until then. Réglages SHALL show when this device last synced and SHALL offer a manual exchange, reporting a failure by category, never as a raw message. No exchange SHALL happen while signed out.

#### Scenario: A card captured on another device
- **WHEN** a reader adds a card on their phone, then opens the extension's panel on their Mac
- **THEN** the Mac exchanges with the server without being restarted, and the panel shows the new card

#### Scenario: Repeated openings
- **WHEN** the reader opens the popup three times within a few seconds
- **THEN** a single exchange runs

#### Scenario: A background the browser may suspend
- **WHEN** a surface triggers an exchange
- **THEN** it is told the exchange is done only when it is, and the exchange completes even though the browser suspends an idle background

#### Scenario: Synchronising on demand
- **WHEN** the reader chooses « Synchroniser maintenant » in Réglages while the server is unreachable
- **THEN** they see a categorized message and the displayed last-sync time is unchanged

#### Scenario: Signed out
- **WHEN** any of those moments happens on a signed-out device
- **THEN** no request is sent

### Requirement: A word reclassified on another device converges here
Every decision about a word SHALL carry the time it was made, **including putting the word in the deck**, so two devices deciding differently converge on the later decision rather than on an undated one. When a pulled `known` or `ignored` wins for a word this device has carded, the card SHALL stop coming due here — the retirement the gesture performs on the device that decided, which that device could not perform for a card it did not hold. Retiring SHALL NOT date the card before its own last edit.

#### Scenario: Deck on one device, known on the other
- **WHEN** a reader puts a word in the deck on their Mac, then marks the same word known on their phone, and both devices sync
- **THEN** both show it as known and its card no longer comes up for review, on either device

#### Scenario: The deck wins when it comes last
- **WHEN** the word was marked known first and put in the deck afterwards
- **THEN** both devices show it as being learned, with its card

#### Scenario: A card edited after the status it retires
- **WHEN** a known dated before this device's last review of that card is pulled and wins the status
- **THEN** the card is retired and keeps its own, later date, so the retirement reaches the other devices

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
