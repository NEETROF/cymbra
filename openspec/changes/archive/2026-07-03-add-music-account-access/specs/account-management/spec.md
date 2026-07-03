## ADDED Requirements

### Requirement: Email sign-up

The app SHALL let a new user create an account with an email and a password,
calling Cymbra ID's `SignUpLocal`. The app SHALL validate the password against
the backend policy (minimum length) before submitting and SHALL surface
`ALREADY_EXISTS` as a clear "email already in use" message.

#### Scenario: Successful sign-up
- **WHEN** the user submits a valid, unused email and a policy-compliant password
- **THEN** the app calls `SignUpLocal` and advances the user to the email-verification step

#### Scenario: Email already registered
- **WHEN** `SignUpLocal` returns `ALREADY_EXISTS`
- **THEN** the app shows an "email already in use" message and offers to sign in or reset the password instead

#### Scenario: Weak password
- **WHEN** the entered password is shorter than the backend minimum length
- **THEN** the app blocks submission and explains the password requirement

### Requirement: Email verification by code

The app SHALL verify a newly registered email by having the user enter the code
sent to that email, submitting it via Cymbra ID's `VerifyEmail`. The app SHALL
offer a resend action wired to `ResendVerification`, and SHALL surface rate-limit
(`RESOURCE_EXHAUSTED`) and expired/invalid-code errors clearly. Local sign-in
SHALL be blocked until the email is verified.

#### Scenario: Successful verification
- **WHEN** the user enters the correct, unexpired code
- **THEN** the app calls `VerifyEmail`, marks the email verified, and proceeds to sign-in (and handle onboarding)

#### Scenario: Resend verification code
- **WHEN** the user requests a new code
- **THEN** the app calls `ResendVerification` and informs the user a new code was sent

#### Scenario: Sign-in blocked before verification
- **WHEN** the user attempts `SignInLocal` and the backend reports the email is unverified (`FAILED_PRECONDITION`)
- **THEN** the app routes the user to the verification step instead of signing in

#### Scenario: Too many resends
- **WHEN** `ResendVerification` returns `RESOURCE_EXHAUSTED`
- **THEN** the app tells the user to wait before requesting another code

### Requirement: Email sign-in

The app SHALL let a verified user sign in with email and password via
`SignInLocal(audience="music")`, surfacing wrong-credential and lockout
(`RESOURCE_EXHAUSTED`) states distinctly.

#### Scenario: Successful email sign-in
- **WHEN** a verified user submits the correct email and password
- **THEN** the app stores the returned session and continues into the app (running handle onboarding if needed)

#### Scenario: Wrong password
- **WHEN** `SignInLocal` returns `UNAUTHENTICATED` for a wrong password
- **THEN** the app shows an invalid-credentials message without revealing whether the email exists

#### Scenario: Account locked out
- **WHEN** `SignInLocal` returns `RESOURCE_EXHAUSTED` due to too many failed attempts
- **THEN** the app shows a lockout message and asks the user to retry later

### Requirement: Forgot password reset by code

The app SHALL offer a "forgot password" flow that calls
`RequestPasswordReset(email)` and then lets the user enter the emailed code and a
new password via `ResetPassword`. The request step SHALL behave identically
whether or not the email exists (no account enumeration), and the app SHALL inform
the user that all sessions are signed out after a successful reset.

#### Scenario: Request a reset
- **WHEN** the user submits an email in the forgot-password flow
- **THEN** the app calls `RequestPasswordReset` and shows the same "check your email" confirmation regardless of whether the email is registered

#### Scenario: Complete a reset
- **WHEN** the user enters a valid reset code and a policy-compliant new password
- **THEN** the app calls `ResetPassword`, informs the user existing sessions were signed out, and returns them to email sign-in

#### Scenario: Invalid or expired reset code
- **WHEN** `ResetPassword` reports the code is invalid or expired
- **THEN** the app explains the code is no longer valid and offers to request a new one

### Requirement: Unique-handle onboarding

After any successful sign-in, the app SHALL fetch the account via `GetAccount`
and, if it has no handle, SHALL require the user to choose a **unique** handle
before reaching the rest of the app. The app SHALL check availability live and
SHALL reject a taken or invalid handle, persisting the chosen handle via
`UpdateAccount`. A handle SHALL be 1–15 characters of UTF-8 letters and numbers
only (no spaces, punctuation, or symbols); uniqueness SHALL be evaluated
case-insensitively.

#### Scenario: New user must choose a handle
- **WHEN** sign-in succeeds and `GetAccount` returns no handle
- **THEN** the app shows a blocking "choose your handle" screen before the library

#### Scenario: Handle rejected for invalid format
- **WHEN** the user enters a handle longer than 15 characters or containing a non-letter/non-number character
- **THEN** the app blocks submission and explains the handle rules

#### Scenario: Handle availability checked live
- **WHEN** the user types a valid candidate handle
- **THEN** the app checks availability against the backend and indicates whether the handle is free, taken, or invalid

#### Scenario: Case-insensitive collision
- **WHEN** the user submits a handle that differs from an existing handle only by letter case
- **THEN** the handle is treated as taken and the app asks for a different one

#### Scenario: Handle taken at submit time
- **WHEN** the user submits a handle that was claimed between the check and the submit
- **THEN** `UpdateAccount` is rejected for the uniqueness conflict and the app asks the user to choose another handle

#### Scenario: Existing user keeps their handle
- **WHEN** sign-in succeeds and `GetAccount` returns an existing handle
- **THEN** the handle onboarding screen is skipped and the user proceeds into the app

### Requirement: Backend enforces handle uniqueness

The system SHALL enforce handle uniqueness in the Cymbra ID backend: the account
model SHALL carry a handle, a **case-insensitive** uniqueness constraint SHALL
prevent two accounts from holding the same handle (ignoring letter case), and a
handle-availability check SHALL be exposed to the client. The backend SHALL
validate the handle policy (1–15 UTF-8 letters/numbers) and SHALL guarantee
uniqueness at write time, not only by the pre-check.

#### Scenario: Concurrent claims of the same handle
- **WHEN** two accounts attempt to claim the same handle (in any letter case)
- **THEN** at most one succeeds and the other receives a conflict error

#### Scenario: Availability check is advisory
- **WHEN** the client asks whether a handle is available
- **THEN** the backend answers based on current state, while the write path remains the authority that guarantees uniqueness

### Requirement: Account deletion

The app SHALL let a signed-in user permanently delete their account via
`DeleteAccount`, gated behind **fresh re-authentication** and an explicit
confirmation that states the action is irreversible. Re-authentication SHALL match
the user's method: an email user re-enters their password (verified via
`SignInLocal`), and a Google/Apple user re-runs the native sign-in to produce a
fresh `id_token`. `DeleteAccount` SHALL only be called after re-authentication
succeeds and the user confirms. On success the app SHALL clear the local session
and return to the entry screen. Account deletion SHALL NOT be offered in guest mode.

#### Scenario: Confirmed deletion after re-authentication
- **WHEN** a signed-in user re-authenticates successfully and then confirms account deletion
- **THEN** the app calls `DeleteAccount`, clears secure storage, and returns to the entry screen

#### Scenario: Re-authentication fails
- **WHEN** the user enters a wrong password (or the re-run OIDC sign-in fails) at the deletion gate
- **THEN** no `DeleteAccount` call is made, the account remains intact, and the app surfaces an authentication error

#### Scenario: Deletion requires explicit confirmation
- **WHEN** the user re-authenticates but does not confirm the irreversible step
- **THEN** no `DeleteAccount` call is made and the account remains intact

#### Scenario: Not available to guests
- **WHEN** the app is in guest mode
- **THEN** no account-deletion action is presented
