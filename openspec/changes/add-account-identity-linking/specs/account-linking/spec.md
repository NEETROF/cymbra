## ADDED Requirements

### Requirement: View connected identities

A signed-in user SHALL be able to view the sign-in identities currently linked to
their account. The app SHALL obtain the list via `UserService.ListIdentities` and
display, per identity, the provider (Google, Apple, or email/password) and when it
was linked. Guests (no account) SHALL NOT see this screen.

#### Scenario: List shows all linked identities

- **WHEN** a signed-in user opens the Connected accounts screen
- **THEN** the app calls `ListIdentities` and renders one row per identity with its provider and linked-at date

#### Scenario: Providers not yet linked are offered as actions

- **WHEN** the account has a `local` identity but no `google` identity
- **THEN** the screen shows the linked email/password identity and a "Link Google" action

#### Scenario: Guest has no access

- **WHEN** the app is in guest mode (no account)
- **THEN** the Connected accounts screen is not reachable

### Requirement: Link a social identity

A signed-in user SHALL be able to link a Google or Apple identity to their current
account. The app SHALL mint a fresh `id_token` via the injectable OIDC token source
and call `AuthService.LinkIdentity(id_token)` with the current session's bearer
token. On success the identity list SHALL refresh to include the new provider.

#### Scenario: Successful Google link

- **WHEN** a signed-in user taps "Link Google" and completes the Google consent
- **THEN** the app calls `LinkIdentity` with the returned `id_token` and, on success, the list refreshes to include the Google identity

#### Scenario: User cancels the provider sheet

- **WHEN** the user dismisses the Google/Apple sheet without authorizing
- **THEN** no `LinkIdentity` call is made and the screen is unchanged (no error)

### Requirement: Link a local (email + password) credential

A user whose account has no `local` identity SHALL be able to add an email +
password credential so they can also sign in with email. The app SHALL only offer
this action when no `local` identity is present.

#### Scenario: Set a password when none exists

- **WHEN** a Google-only user chooses "Set a password" and submits a valid email and password
- **THEN** the account gains a `local` identity and the list refreshes to include it

#### Scenario: Action hidden when already present

- **WHEN** the account already has a `local` identity
- **THEN** the "Set a password" action is not offered

### Requirement: Identity already linked to another account

The app SHALL handle the backend's `ALREADY_EXISTS` response — returned when the
chosen identity already belongs to a *different* account — by surfacing a clear,
dedicated message and making no further change. The app SHALL NOT attempt to merge
the two accounts.

#### Scenario: Collision surfaces a dedicated error

- **WHEN** a user tries to link a Google identity that already owns another Cymbra account
- **THEN** the app shows a message such as "This Google account is already linked to another Cymbra account." and the identity list is unchanged

### Requirement: Unlink an identity with last-identity guard

A signed-in user SHALL be able to unlink an identity via
`AuthService.UnlinkIdentity(provider, subject)`, except the **last remaining**
identity, which cannot be removed (anti-lockout). The app SHALL prevent the action
for the last identity and SHALL surface the backend's refusal if it still occurs.

#### Scenario: Unlink a non-last identity

- **WHEN** an account has both `local` and `google` identities and the user unlinks Google
- **THEN** the app calls `UnlinkIdentity` and the list refreshes to show only the `local` identity

#### Scenario: Last identity cannot be unlinked

- **WHEN** only one identity remains
- **THEN** its unlink action is disabled with an explanation, and any server `FAILED_PRECONDITION` is shown as "You can't remove your only sign-in method."

### Requirement: Provider-appropriate error messaging

Authentication and linking failures SHALL surface messages appropriate to the flow.
A gRPC `UNAUTHENTICATED` outside the email-credential flow SHALL NOT be shown as
"Incorrect email or password." Linking flows SHALL distinguish link failure,
already-linked-elsewhere, and last-identity refusal.

#### Scenario: OIDC failure is not shown as a password error

- **WHEN** a Google sign-in or link fails with `UNAUTHENTICATED`
- **THEN** the message reflects the provider/link context, not "Incorrect email or password."

### Requirement: Link an existing account from the sign-in collision point

The app SHALL offer, after a social sign-in that lands on handle onboarding, a
user-driven "Already have an account? Sign in to link." option. The app SHALL NOT
reveal whether an account already exists for the user, nor which sign-in method it
uses — the user supplies the method by choosing it. On confirmation the app SHALL
re-authenticate the user into their chosen existing account, delete the just-created
orphan social account so its `(provider, subject)` is freed, and call `LinkIdentity`
with the social `id_token` to attach the identity to the existing account. The orphan
SHALL be deleted before `LinkIdentity` to avoid an `ALREADY_EXISTS` self-collision.

#### Scenario: Link the new social identity onto an existing email account

- **WHEN** a user who already has an email account signs in with Google, reaches handle onboarding, chooses "Sign in to link", and authenticates with their email/password
- **THEN** the app deletes the orphan Google account, signs in to the email account, links the Google identity to it, and lands the user in the app on their existing account (no second account, no new handle)

#### Scenario: No account information is disclosed

- **WHEN** the collision option is shown
- **THEN** it does not state that an account exists or which method it uses; the user chooses a method to attempt sign-in

#### Scenario: Expired social token is re-minted before linking

- **WHEN** the social `id_token` is no longer valid by the time `LinkIdentity` is called
- **THEN** the app re-mints a fresh `id_token` via the OIDC source before linking

### Requirement: Linking requires an authenticated session

Link and unlink actions SHALL require a valid authenticated session; they are never
available to guests. Whether a recent-auth (re-auth) gate is additionally required
is determined in design; if required, the app SHALL prompt for re-authentication
before performing the action.

#### Scenario: Unauthenticated caller is rejected

- **WHEN** a link or unlink is attempted without a valid session
- **THEN** the action is not performed and the user is routed to sign in
