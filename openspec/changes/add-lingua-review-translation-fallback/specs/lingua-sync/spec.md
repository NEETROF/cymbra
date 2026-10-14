## MODIFIED Requirements

### Requirement: Complete card synchronisation, media excluded
`DeckService` SHALL synchronise complete cards — lemma, surface form, source sentence, gloss, whether the reader chose that gloss, FSRS state — identified by a stable client id, last-write-wins per card (deletions included). The page a card was captured from stays on the device (`add-lingua-privacy-controls`): the card's `source` field is deprecated, sent empty and never stored. Whether the reader chose the gloss — by keeping a machine translation or by writing it — SHALL be an optional field of the card operation, absent meaning not, stored and returned as received, so that clients built before it keep working. A client applying an incoming card that does not carry the mark but carries the same gloss text as its local card SHALL keep the local mark. The contents of the `media` field SHALL NOT be synchronised in this change (the schema slot stays local).

#### Scenario: Card created on mobile, reviewed on desktop
- **WHEN** a card created while reading in Safari on iOS is synced and the user then opens the side panel on their Mac
- **THEN** the card appears there with its source sentence and FSRS state, and reviewing it on the Mac propagates back

#### Scenario: Card with an image
- **WHEN** a local card carries media and is synchronised
- **THEN** all of its fields go up except the media content, which stays on the originating device

#### Scenario: A kept gloss reaches another device
- **WHEN** the reader keeps « se relâcher » for « slacken » in the app on their iPhone, then opens the side panel on their Mac after Safari has synchronised
- **THEN** the Mac's card shows « se relâcher », marked as the reader's, and does not ask for a translation

#### Scenario: A client that predates the mark
- **WHEN** a client built before the mark existed pushes a card
- **THEN** the server stores the card as not chosen by the reader, and accepts it as before

#### Scenario: An echo without the mark
- **WHEN** a device holding « se relâcher » marked as the reader's applies a newer pulled card with the same gloss text and no mark
- **THEN** it takes the newer card and keeps the mark

### Requirement: Apple and Google sign-in on Safari through the host app
On Safari, which has no `identity.launchWebAuthFlow`, the container app SHALL provide Sign in with Apple and Continue with Google natively (Sign in with Apple first) and SHALL hand the resulting id_token to its Safari extension through the App Group, readable once and for at most five minutes. The Safari extension SHALL exchange that id_token through `SignInOidc` against the `lingua` audience and SHALL own the resulting session like every other variant. The container app SHALL hold no learning state of its own, and SHALL hold a session only of its own, for online translation in its review, never the extension's; the email flows SHALL stay in the extension.
The container app's session SHALL be opened by the reader inside the app with the same Apple or
Google sheet, exchanged through `SignInOidc` against the `lingua` audience, its refresh token kept in
the app's Keychain, and SHALL be a session family of its own, which the reader can end from the app
and which ends with account deletion or sign-out everywhere. The extension's session, access token
or refresh token SHALL NOT be handed to the app by any channel.

#### Scenario: Apple sign-in from Safari
- **WHEN** the reader picks "Continue with Apple" in the Safari extension, completes the native Apple sheet in the container app and goes back to Safari
- **THEN** the extension exchanges the handed id_token through `SignInOidc`, is signed in to the same Cymbra account as with Apple on any other client, and schedules a sync

#### Scenario: A handed id_token is used once
- **WHEN** the extension has collected a handed id_token, or the token is older than five minutes
- **THEN** the App Group no longer holds it, and an expired token is discarded without any RPC

#### Scenario: Email flows stay in the extension
- **WHEN** a Safari reader signs up, verifies an email or resets a password
- **THEN** it happens in the extension's account page, with no native screen involved

#### Scenario: The app signs in for online translation
- **WHEN** a reader with access presses « Traduction en ligne » in the app for the first time and completes the Apple sheet
- **THEN** the app holds a session of its own for the same account, the translation is asked, and the Safari extension's session is untouched

#### Scenario: Signing out of the app
- **WHEN** the reader ends the app's session from the app
- **THEN** the app's session is revoked, the extension stays signed in, and the app's review offers no online translation until the reader signs in again
