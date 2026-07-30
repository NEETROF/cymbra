## MODIFIED Requirements

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
