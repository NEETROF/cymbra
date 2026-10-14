## MODIFIED Requirements

### Requirement: Apple and Google sign-in on Safari through the host app
On Safari, which has no `identity.launchWebAuthFlow`, the container app SHALL provide Sign in with Apple and Continue with Google natively (Sign in with Apple first) and SHALL hand the resulting id_token to its Safari extension through the App Group, readable once and for at most five minutes. The Safari extension SHALL exchange that id_token through `SignInOidc` against the `lingua` audience and SHALL own the resulting session like every other variant. The container app SHALL hold no learning state of its own, and SHALL hold a session only of its own, for online translation in its review, never the extension's; the extension's email flows SHALL stay in the extension, and creating an account with an email SHALL happen only there.
The container app's session SHALL be opened by the reader inside the app, with the same Apple or
Google sheet exchanged through `SignInOidc`, or with the email and password of an existing account
through `SignInLocal`, both against the `lingua` audience, its refresh token kept in the app's
Keychain; it SHALL be a session family of its own, which the reader can end from the app and which
ends with account deletion or sign-out everywhere. The extension's session, access token or refresh
token SHALL NOT be handed to the app by any channel, and the reader's password SHALL NOT be stored by
the app.

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

#### Scenario: Signing out of the app
- **WHEN** the reader ends the app's session from the app
- **THEN** the app's session is revoked, the extension stays signed in, and the app's review offers no online translation until the reader signs in again
