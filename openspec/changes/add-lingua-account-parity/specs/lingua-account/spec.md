# lingua-account — Cymbra account lifecycle in the Lingua browser extension

## ADDED Requirements

### Requirement: Account creation by email from the extension
The extension SHALL let a reader create a Cymbra account with an email and a password through `SignUpLocal`, sending the browser's UI language as `locale`, and SHALL then lead to the verification-code step for that email. A duplicate email SHALL be reported as an existing account with links to sign in and to reset the password, and a password rejected by the policy SHALL be reported as too weak, without leaving the form.

#### Scenario: New reader signs up
- **WHEN** a reader enters an unused email and a policy-compliant password on the sign-up view and submits
- **THEN** `SignUpLocal` is called with that email, password and the browser locale, and the page shows the code step for that email

#### Scenario: Email already has an account
- **WHEN** `SignUpLocal` fails with `ALREADY_EXISTS`
- **THEN** the page says an account already uses this email and offers "sign in" and "forgot password", and the form keeps the email

#### Scenario: Weak password
- **WHEN** `SignUpLocal` fails with `INVALID_ARGUMENT`
- **THEN** the page says the password is too weak and stays on the sign-up view

### Requirement: Email verification by code
The extension SHALL let the reader enter the emailed verification code (`VerifyEmail`) and request a new one (`ResendVerification` with the browser locale). After a successful verification it SHALL sign the reader in with `SignInLocal` when the password from the same sign-up is still held in the page's memory, and otherwise SHALL return to the sign-in view with the email prefilled. An invalid or expired code SHALL be reported as such with the resend action available.

#### Scenario: Verification signs the reader in
- **WHEN** the reader who just signed up enters a valid code
- **THEN** `VerifyEmail` succeeds, `SignInLocal(audience="lingua")` is called with the held credentials, the session is stored, and a sync is scheduled

#### Scenario: Verification after the page was reloaded
- **WHEN** the reader reloads the account page on the code step and enters a valid code
- **THEN** the email is still shown, `VerifyEmail` succeeds, and the page returns to sign-in with the email prefilled because no password is held

#### Scenario: Expired code
- **WHEN** `VerifyEmail` fails with `INVALID_ARGUMENT`
- **THEN** the page says the code is invalid or expired and offers to resend it

### Requirement: Unverified sign-in leads to the code step
The extension SHALL treat a local sign-in rejected with `FAILED_PRECONDITION` as an unverified email, not as a credential error: it SHALL open the verification-code step for that email instead of showing a wrong-password message.

#### Scenario: Unverified account signs in from the account page
- **WHEN** the reader signs in on the account page with correct credentials for an unverified email
- **THEN** no error is shown and the page moves to the code step for that email

#### Scenario: Unverified account signs in from the popup
- **WHEN** the reader signs in from the popup with correct credentials for an unverified email
- **THEN** the account page opens on the code step with the email prefilled, and the password is not passed to it

### Requirement: Password reset by code
The extension SHALL let the reader request a password reset (`RequestPasswordReset` with the browser locale) and complete it with the emailed code and a new password (`ResetPassword`). The request SHALL show the same confirmation whether or not an account exists for the email, and a completed reset SHALL return to the sign-in view with the email prefilled.

#### Scenario: Reset completes
- **WHEN** the reader enters a valid reset code and a policy-compliant new password
- **THEN** `ResetPassword` succeeds and the page shows the sign-in view with the email prefilled

#### Scenario: Request does not reveal the account
- **WHEN** the reader requests a reset for an email with no account
- **THEN** the page shows the same "if an account exists, a code was sent" confirmation as for an existing account

### Requirement: Sign in with Apple in the extension
The extension SHALL offer "Continue with Apple" by running `identity.launchWebAuthFlow` against Apple's authorize endpoint with the Services ID as client id, no `scope`, `response_type=code id_token` and `response_mode=fragment` (or, if Apple refuses the extension redirect URLs, the allow-listed backend relay), SHALL check the returned `state`, SHALL exchange the fragment's `id_token` through `SignInOidc(audience="lingua")`, and SHALL never exchange the authorization code. A user-cancelled flow SHALL send no RPC and show no error, and a failed exchange SHALL be worded as an Apple sign-in failure, never as a password error.

#### Scenario: Apple account from Music signs in to Lingua
- **WHEN** a reader whose Cymbra account was created with Sign in with Apple in Cymbra Music picks "Continue with Apple" in the extension with the same Apple ID
- **THEN** `SignInOidc` returns a `TokenPair` for the `lingua` audience on that same Cymbra account

#### Scenario: Apple window closed
- **WHEN** the reader closes Apple's window before finishing
- **THEN** no RPC is sent, nothing is persisted as a sign-in error, and the surface shows no error

#### Scenario: State mismatch
- **WHEN** the redirect fragment carries a `state` different from the one sent
- **THEN** the id_token is discarded, no RPC is sent, and the sign-in fails as an Apple sign-in failure

### Requirement: Backend relay for Apple is allow-listed and inert by default
If the Apple relay is deployed, the backend SHALL accept only a form-encoded `POST` carrying `id_token` and `state`, SHALL redirect with `303` only to a target exactly matching an entry of its configured extension-redirect allow-list, SHALL answer `404` when the allow-list is empty, SHALL set `Cache-Control: no-store` and `Referrer-Policy: no-referrer`, and SHALL NOT log the request body or the redirect location.

#### Scenario: Allow-listed extension
- **WHEN** Apple posts an `id_token` whose `state` names an allow-listed extension redirect URL
- **THEN** the relay answers `303` to that URL with the `id_token` and `state` in the fragment, and no-store headers

#### Scenario: Unknown target
- **WHEN** the `state` names a URL absent from the allow-list
- **THEN** the relay answers `400` and redirects nowhere

#### Scenario: Relay not configured
- **WHEN** the allow-list is empty
- **THEN** the relay path answers `404`

### Requirement: Sign-in providers shown per browser
The extension SHALL show the Google and Apple buttons only when the provider's client id was configured at build time and the browser exposes `identity.launchWebAuthFlow`; otherwise it SHALL hide them (not disable them) and keep email sign-in, sign-up and reset available.

#### Scenario: Firefox for Android
- **WHEN** the extension runs on Firefox for Android, which has no `identity.launchWebAuthFlow`
- **THEN** neither Google nor Apple is shown and the email flows work

#### Scenario: Build without an Apple client id
- **WHEN** the extension was built with an empty Apple client id
- **THEN** the Apple button is absent from the popup and the account page, and Google and email are unaffected

### Requirement: Account flows hosted in a page that survives focus changes
The extension SHALL host sign-up, verification, and password reset in a full extension page opened in a tab, reachable from the popup ("create an account", "forgot password") and offered as a skippable step of onboarding. No account step SHALL block any reading, highlighting, deck or review feature.

#### Scenario: Reader fetches the code from their mailbox
- **WHEN** the reader on the code step switches to another application to read the email and comes back
- **THEN** the account page is still on the code step with the email shown

#### Scenario: Onboarding skipped
- **WHEN** the reader picks "later" on the onboarding account step
- **THEN** onboarding completes and the extension works exactly as without an account

### Requirement: Credentials never persisted
The extension SHALL NOT write a password to any storage area or log, and SHALL keep only the pending verification email in `storage.session` to resume the code step. The sign-up password SHALL live only in the account page's memory until used once for the automatic sign-in.

#### Scenario: Storage after sign-up
- **WHEN** a reader has signed up and is waiting on the code step
- **THEN** `storage.local` and `storage.session` contain the pending email and no password

### Requirement: Auth errors shown in plain words
The extension SHALL map every auth failure to a category (unauthenticated, already exists, rate limited, failed precondition, invalid argument, unavailable, unknown — a deadline overrun counting as unavailable) and SHALL show the reader a message chosen by that category and the flow's context; a raw gRPC or Connect error string SHALL never be displayed.

#### Scenario: Backend unreachable
- **WHEN** any account action fails with `UNAVAILABLE` or `DEADLINE_EXCEEDED`
- **THEN** the reader sees that Cymbra cannot be reached and can retry, with no technical text

#### Scenario: Too many attempts
- **WHEN** a sign-in, resend or reset fails with `RESOURCE_EXHAUSTED`
- **THEN** the reader sees that there were too many attempts and to try again later
