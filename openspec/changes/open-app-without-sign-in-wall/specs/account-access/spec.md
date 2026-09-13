## RENAMED Requirements

- FROM: `### Requirement: Account entry is the launch experience`
- TO: `### Requirement: Account entry is shown when an account session ends`

## MODIFIED Requirements

### Requirement: Account entry is shown when an account session ends

The app SHALL present the account entry screen when a user who had an account session no
longer has one — after signing out, after deleting their account, or when the session can no
longer be refreshed — and no guest choice is stored. The account entry screen SHALL NOT be the
first screen of a first run: a first-time user reaches the app through the welcome (see
`welcome-onboarding`). The entry screen SHALL offer exactly four mutually exclusive choices:
continue with Google, continue with Apple, continue with email, and continue without an account
(guest). The screen SHALL follow the Cymbra theme (`CymbraColors`, Material 3 dark).

#### Scenario: First launch does not open on the entry screen
- **WHEN** the app starts for the first time with no session and no guest choice stored
- **THEN** the first-run language and welcome steps are shown, and the account entry screen is not

#### Scenario: Entry screen after an account session ends
- **WHEN** a signed-in user signs out, deletes their account, or holds a session that can no longer be refreshed, and no guest choice is stored
- **THEN** the account entry screen is shown with the four entry options

#### Scenario: Returning user with a valid session
- **WHEN** the app starts and a valid (or silently refreshable) Cymbra ID session is stored
- **THEN** the entry screen is skipped and the app opens directly on the library

#### Scenario: Returning guest
- **WHEN** the app starts and the persisted choice is guest
- **THEN** the entry screen is skipped and the app opens directly on the library in guest mode

### Requirement: Guest mode is fully offline

The app SHALL provide a guest mode that performs no calls to Cymbra ID and grants no access to
any online (backend-bound) service. Choosing guest — from the welcome or from the entry screen —
SHALL persist the choice so that neither the welcome nor the entry screen is shown again on
subsequent launches. The app SHALL expose an explicit way for a guest to sign in or create an
account from within the app, through the contextual sign-in surface, and SHALL keep the user in
guest mode until authentication succeeds. Next to a direct sign-in button, a guest SHALL have a
menu reaching the subscription paywall, language and the legal pages (terms, privacy, licenses);
it SHALL NOT offer the profile, connected accounts, sign-out or account deletion.

#### Scenario: Entering guest mode
- **WHEN** the user picks "continue without an account" on the welcome or on the entry screen
- **THEN** the choice is persisted, no Cymbra ID RPC is made, and the app opens the library with full local functionality

#### Scenario: Online services blocked for guests
- **WHEN** a guest attempts to use a feature that depends on a Cymbra ID session
- **THEN** the feature is unavailable (hidden or disabled) and the app offers to sign in or create an account instead of calling the backend

#### Scenario: Guest account menu
- **WHEN** a guest looks at the account controls
- **THEN** the sign-in button opens sign-in in one tap, and the menu beside it opens the subscription paywall, language and the legal pages, with no profile, connected accounts, sign-out or deletion entry

#### Scenario: Guest upgrades to an account
- **WHEN** a guest chooses to sign in or create an account from within the app and authentication succeeds
- **THEN** the new session replaces the guest choice and the user returns to the screen they were on, without passing through the entry screen

#### Scenario: Guest leaves the sign-in surface without authenticating
- **WHEN** a guest opens the contextual sign-in surface and leaves it without authenticating
- **THEN** they remain a guest, on the screen they came from

### Requirement: Google sign-in

The app SHALL let the user authenticate with Google by obtaining a Google `id_token` via the
native Google sign-in SDK and exchanging it through Cymbra ID's `SignInOidc`. The app SHALL NOT
perform the OAuth token exchange itself. A `SignInOidc` failure SHALL surface a
provider-appropriate message and SHALL NOT reuse the email-credential copy "Incorrect email or
password."

#### Scenario: Successful Google sign-in
- **WHEN** the user picks "continue with Google" and completes the Google consent flow
- **THEN** the app sends the returned `id_token` to `SignInOidc(audience="music")` and, on success, stores the session and continues into the app

#### Scenario: Google flow cancelled
- **WHEN** the user dismisses the native Google sheet without completing it
- **THEN** no RPC is sent and the user returns to the sign-in surface the flow was started from, with no error surfaced

#### Scenario: Google sign-in failure is not shown as a password error
- **WHEN** `SignInOidc` fails with `UNAUTHENTICATED` after a Google sign-in
- **THEN** the app shows a Google/sign-in-context message, not "Incorrect email or password."

### Requirement: Apple sign-in

The app SHALL let the user authenticate with Apple by obtaining an Apple `id_token` via the
native Sign in with Apple SDK and exchanging it through Cymbra ID's `SignInOidc`. Sign in with
Apple SHALL be offered on Apple platforms wherever Google sign-in is offered (App Store
requirement). A `SignInOidc` failure SHALL surface a provider-appropriate message and SHALL NOT
reuse the email-credential copy "Incorrect email or password."

#### Scenario: Successful Apple sign-in
- **WHEN** the user picks "continue with Apple" and completes the Apple flow
- **THEN** the app sends the returned `id_token` to `SignInOidc(audience="music")` and, on success, stores the session and continues into the app

#### Scenario: Apple flow cancelled
- **WHEN** the user cancels the Apple sheet
- **THEN** no RPC is sent and the user returns to the sign-in surface the flow was started from, with no error surfaced

#### Scenario: Apple sign-in failure is not shown as a password error
- **WHEN** `SignInOidc` fails with `UNAUTHENTICATED` after an Apple sign-in
- **THEN** the app shows an Apple/sign-in-context message, not "Incorrect email or password."
