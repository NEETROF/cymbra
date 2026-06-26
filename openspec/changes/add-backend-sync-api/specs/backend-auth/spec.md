## ADDED Requirements

### Requirement: Local email/password sign-up

The auth module SHALL let a user create an account with an email and password. The
password MUST be stored only as an **argon2id** hash (never in clear text). On
sign-up the module MUST create the account with the email marked **unverified**
and send a verification message to the email. A sign-up for an email that already
has a local credential MUST be rejected.

#### Scenario: New email signs up

- **WHEN** a user signs up with an email that has no existing local credential and
  an acceptable password
- **THEN** an account and a `local` identity for that email are created
- **AND** only an argon2id hash of the password is stored, with the email marked
  unverified
- **AND** a verification message is sent to the email

#### Scenario: Duplicate local email is rejected

- **WHEN** a user signs up with an email that already has a local credential
- **THEN** the module rejects it with gRPC status `ALREADY_EXISTS`

### Requirement: Email verification required before local sign-in

The auth module SHALL require a local account's email to be verified before it can
sign in. Verification MUST be performed with a single-use, expiring token/code. A
local sign-in attempt for an unverified email MUST be rejected.

#### Scenario: Valid verification confirms the email

- **WHEN** a user submits a valid, unexpired verification token for their email
- **THEN** the email is marked verified

#### Scenario: Sign-in blocked until verified

- **WHEN** a user attempts local sign-in with correct credentials but an unverified
  email
- **THEN** the module rejects it with gRPC status `FAILED_PRECONDITION`

#### Scenario: Invalid or expired verification token

- **WHEN** a verification token is unknown, already used, or expired
- **THEN** the module rejects it and the email remains unverified

### Requirement: OIDC sign-in via Google and Apple

The auth module SHALL authenticate a user from an external OIDC provider ID token
through a pluggable `IdentityVerifier`, supporting **Google** and **Apple**. It
MUST validate the token against the matching issuer's JWKS and verify `iss`,
`aud`, and `exp`, then resolve or provision the account by `(provider, subject)`.
Adding another provider MUST be possible by adding a verifier without changing
callers.

#### Scenario: Valid provider token signs in

- **WHEN** a user presents a Google or Apple ID token with a valid signature,
  issuer, audience, and unexpired lifetime
- **THEN** the account is resolved (or provisioned on first sign-in) by
  `(provider, subject)` and a session is issued

#### Scenario: Token from an untrusted issuer is rejected

- **WHEN** a presented ID token fails signature/issuer/audience/expiry validation,
  or comes from an issuer the backend does not trust
- **THEN** the module rejects it with gRPC status `UNAUTHENTICATED`

### Requirement: Internal session tokens

Sign-in SHALL target an **app audience** (e.g. `music` or `live`), validated
against a configured allow-list; an unknown audience MUST be rejected. On success
(local or OIDC) the auth module SHALL issue the backend's **own** session tokens —
a short-lived access token and a refresh token — signed by the backend. The access
token SHALL set `aud` to the target app and carry the account's `user_id` and the
**effective role set for that audience** (roles whose scope is `global` or that
app's scope, read from the user module — never from the provider token). Protected
gRPC methods MUST be authorized by validating the internal **access** token (not
the provider token), and an interceptor MUST reject requests whose access token is
missing, invalid, or expired. The refresh token MUST be
exchangeable for a new access token, and the refresh token MUST be rotated on use;
a revoked or expired refresh token MUST be rejected.

#### Scenario: Successful sign-in issues an audience-scoped token

- **WHEN** a sign-in succeeds for an allowed audience (valid local credentials with
  a verified email, or a valid OIDC token)
- **THEN** the module returns a signed access token whose `aud` is that app and
  whose roles are the effective set for that audience, plus a refresh token

#### Scenario: Unknown audience is rejected

- **WHEN** a sign-in targets an audience that is not in the configured allow-list
- **THEN** the module rejects it with gRPC status `INVALID_ARGUMENT` and issues no
  tokens

#### Scenario: Wrong local password is rejected

- **WHEN** a local sign-in supplies an incorrect password
- **THEN** the module rejects it with gRPC status `UNAUTHENTICATED` and issues no
  tokens

#### Scenario: Protected call requires a valid internal token

- **WHEN** a request to a protected method has a missing, invalid, or expired
  internal access token
- **THEN** the interceptor rejects it with gRPC status `UNAUTHENTICATED`

#### Scenario: Refresh rotates the token

- **WHEN** a valid, unexpired refresh token is presented
- **THEN** a new access token is issued and the refresh token is rotated

#### Scenario: Revoked or expired refresh is rejected

- **WHEN** a refresh token that is expired, already rotated, or revoked is presented
- **THEN** the module rejects it with gRPC status `UNAUTHENTICATED`

### Requirement: Explicit provider linking

The auth module SHALL let an authenticated user link an additional identity — an
OIDC provider or a local email/password credential — to their **current** account.
The module MUST NOT auto-merge accounts by email. Linking an identity that is
already bound to a different account MUST be rejected.

#### Scenario: Authenticated user links another provider

- **WHEN** a signed-in user links a new identity (e.g. adds Google, or adds a local
  email/password) and that identity is not bound elsewhere
- **THEN** the identity is attached to the user's current account

#### Scenario: Linking an already-bound identity is rejected

- **WHEN** a signed-in user tries to link an identity already bound to a different
  account
- **THEN** the module rejects it with gRPC status `ALREADY_EXISTS` and links nothing

### Requirement: Role-based authorization guard

The backend SHALL provide a role-based authorization guard (`require_role(r)`, with
`is_admin` = `require_role("admin")`) that reads the **role set** from the validated
internal access token. The guard provides the enforcement mechanism only; concrete
admin endpoints are out of scope.

#### Scenario: Guard blocks users without the required role

- **WHEN** a user whose role set lacks the required role invokes a guarded method
- **THEN** the backend rejects it with gRPC status `PERMISSION_DENIED`

#### Scenario: Guard allows users holding the required role

- **WHEN** a user whose role set contains the required role invokes the guarded
  method
- **THEN** the request is permitted to proceed
