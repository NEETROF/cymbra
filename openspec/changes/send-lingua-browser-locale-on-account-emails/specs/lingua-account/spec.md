## MODIFIED Requirements

### Requirement: Email verification by code
The extension SHALL let the reader enter the emailed verification code (`VerifyEmail`) and request a new one (`ResendVerification` with the locale the sign-up carries — the browser's language until the reader has chosen their native language on this device, the account locale once they have — which Cymbra ID records only on an account that has none). After a successful verification it SHALL sign the reader in with `SignInLocal` when the password from the same sign-up is still held in the page's memory, and otherwise SHALL return to the sign-in view with the email prefilled. An invalid or expired code SHALL be reported as such with the resend action available.

#### Scenario: Verification signs the reader in
- **WHEN** the reader who just signed up enters a valid code
- **THEN** `VerifyEmail` succeeds, `SignInLocal(audience="lingua")` is called with the held credentials, the session is stored, and a sync is scheduled

#### Scenario: Verification after the page was reloaded
- **WHEN** the reader reloads the account page on the code step and enters a valid code
- **THEN** the email is still shown, `VerifyEmail` succeeds, and the page returns to sign-in with the email prefilled because no password is held

#### Scenario: Expired code
- **WHEN** `VerifyEmail` fails with `INVALID_ARGUMENT`
- **THEN** the page says the code is invalid or expired and offers to resend it

### Requirement: Password reset by code
The extension SHALL let the reader request a password reset (`RequestPasswordReset` with the locale the sign-up carries — the browser's language until the reader has chosen their native language on this device, the account locale once they have — which Cymbra ID records only on an account that has none) and complete it with the emailed code and a new password (`ResetPassword`). The request SHALL show the same confirmation whether or not an account exists for the email, and a completed reset SHALL return to the sign-in view with the email prefilled.

#### Scenario: Reset completes
- **WHEN** the reader enters a valid reset code and a policy-compliant new password
- **THEN** `ResetPassword` succeeds and the page shows the sign-in view with the email prefilled

#### Scenario: Request does not reveal the account
- **WHEN** the reader requests a reset for an email with no account
- **THEN** the page shows the same "if an account exists, a code was sent" confirmation as for an existing account
