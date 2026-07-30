## MODIFIED Requirements

### Requirement: Link a local (email + password) credential

A user whose account has no `local` identity SHALL be able to add an email +
password credential so they can also sign in with email. The app SHALL only offer
this action when no `local` identity is present.

The email/`local` identity SHALL be bound to the account only **after** the emailed
verification code is confirmed. Submitting the email + password SHALL validate the
input and send a verification code, but SHALL NOT create the `local` credential or
reserve the email until verification. An unconfirmed set-password SHALL leave no
trace: it SHALL expire on its own without reserving the email or requiring cleanup.
When the code is verified, the account SHALL gain a usable `local` credential (no
further verification step) linked to that email. If the email became bound to
another account before the code is verified, verification SHALL fail cleanly and
SHALL still leave nothing reserved.

#### Scenario: Submitting sends a code without binding yet

- **WHEN** a Google-only user chooses "Set a password" and submits a valid email and password
- **THEN** a verification email is sent and the user is taken to the code-entry screen to verify in place (staying signed in)
- **AND** the account does not yet gain a `local` identity and the email is not reserved

#### Scenario: Verifying the code binds the credential

- **WHEN** the user confirms the emailed code for that pending set-password
- **THEN** the account gains a `local` identity for that email and can sign in with the email + password immediately (no further verification)

#### Scenario: An unconfirmed set-password reserves nothing and expires

- **WHEN** a user submits a set-password for an email but never confirms the code
- **THEN** the email remains free for anyone else to use
- **AND** the pending set-password expires on its own with nothing to clean up

#### Scenario: Contended email — first to verify wins

- **WHEN** two users each submit a set-password for the same currently-free email and both receive a code
- **THEN** the first to confirm their code binds the email to their account
- **AND** the second user's verification fails with an already-exists error, having reserved nothing

#### Scenario: Weak password or existing local identity is rejected on submit

- **WHEN** the submitted password is too weak, or the account already has a `local` identity
- **THEN** the submission is rejected immediately, before any code is sent

#### Scenario: Action hidden when already present

- **WHEN** the account already has a `local` identity
- **THEN** the "Set a password" action is not offered
