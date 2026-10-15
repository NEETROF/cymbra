## MODIFIED Requirements

### Requirement: Apple and Google sign-in on Safari through the host app
On Safari, which has no `identity.launchWebAuthFlow`, the container app SHALL provide Sign in with Apple and Continue with Google natively (Sign in with Apple first) and SHALL hand the resulting id_token to its Safari extension through the App Group, readable once and for at most five minutes. The Safari extension SHALL exchange that id_token through `SignInOidc` against the `lingua` audience and SHALL own the resulting session like every other variant. The container app SHALL hold no deck of its own across openings, and SHALL hold a session only of its own, for online translation in its review and for synchronising the reviews made in it, never the extension's; the extension's email flows SHALL stay in the extension, and creating an account with an email SHALL happen only there.
The container app's session SHALL be opened by the reader inside the app, with the same Apple or
Google sheet exchanged through `SignInOidc`, or with the email and password of an existing account
through `SignInLocal`, both against the `lingua` audience, its refresh token kept in the app's
Keychain; it SHALL be a session family of its own, which the reader can end from the app and which
ends with account deletion or sign-out everywhere. The extension's session, access token or refresh
token SHALL NOT be handed to the app by any channel, nor the app's to the extension, and the reader's
password SHALL NOT be stored by the app.

#### Scenario: Apple sign-in from Safari
- **WHEN** the reader picks "Continue with Apple" in the Safari extension, completes the native Apple sheet in the container app and goes back to Safari
- **THEN** the extension exchanges the handed id_token through `SignInOidc`, is signed in to the same Cymbra account as with Apple on any other client, and schedules a sync

#### Scenario: A handed id_token is used once
- **WHEN** the extension has collected a handed id_token, or the token is older than five minutes
- **THEN** the App Group no longer holds it, and an expired token is discarded without any RPC

#### Scenario: Email flows stay in the extension
- **WHEN** a Safari reader signs up, verifies an email or resets a password for the extension's session
- **THEN** it happens in the extension's account page, with no native screen involved

#### Scenario: The app signs in for online translation
- **WHEN** a reader with access presses « Traduction en ligne » in the app for the first time and completes the Apple sheet
- **THEN** the app holds a session of its own for the same account, the translation is asked, and the Safari extension's session is untouched

#### Scenario: The app signs in with an email account
- **WHEN** a reader whose account was created with an email presses « Traduction en ligne » in the app, picks « Continuer avec un email » and signs in with their email and password
- **THEN** the app holds a session of its own for that account, the translation is asked, the Safari extension's session is untouched, and only the refresh token is kept, in the app's Keychain

#### Scenario: The app signs in to synchronise
- **WHEN** a reader signed in to Safari's extension signs in inside the app from its opening line, with the same account
- **THEN** the app holds a session of its own, synchronises the reviews made in it, and the extension's session is untouched

#### Scenario: Signing out of the app
- **WHEN** the reader ends the app's session from the app
- **THEN** the app's session is revoked, the extension stays signed in, the app stops synchronising, and the app's review offers no online translation until the reader signs in again

## ADDED Requirements

### Requirement: The container app synchronises as a device of its own
The container app SHALL synchronise as a device of its own: its own stable random device identifier, distinct from the extension's, and its own pull positions, so neither moves the other's. It SHALL use the same synchronisation protocol, rules and implementation as the extension — last-write-wins per card and per word, deletions included, the erasure mark read first — and SHALL need no change to the Lingua server: the account comes from the app's session, the device identifier serves only as a tie-break. Signing in to another account or signing out SHALL discard the previous account's pull positions.

#### Scenario: Two devices on one iPhone
- **WHEN** the app and the Safari extension of the same iPhone both synchronise the same account
- **THEN** each pulls from its own position, the server stores both devices' changes by last-write-wins, and the extension's next pull is not shortened by the app's

#### Scenario: An app device identifier lost
- **WHEN** the app's device identifier and pull positions are lost
- **THEN** the app generates a new identifier, pulls from the start, and the server's state is unchanged by the loss
