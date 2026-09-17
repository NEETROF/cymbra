## MODIFIED Requirements

### Requirement: Token model and refresh

Cymbra ID SHALL issue an internal, asymmetrically-signed **access** token and a **refresh** token on successful sign-in (local or OIDC, after validating the provider token). Protected gRPC methods MUST be authorized by validating the internal **access** token (not the provider token), and an interceptor MUST reject requests whose access token is missing, invalid, or expired. Access-token validation SHALL be **offline** (signature + claims against the published JWKS), so protected calls MUST NOT depend on the session store. The access token SHALL be **short-lived** (target ~15 minutes) and the refresh token **long-lived and sliding** (target ~30 days) — the refresh token is the effective session length. The refresh token MUST be exchangeable for a new access token and MUST be **rotated on use** with **reuse detection**: presenting an expired, revoked, or already-rotated refresh token MUST be rejected, and replay of a rotated token SHALL revoke the whole session family.

Rotation SHALL tolerate a client that was interrupted before it could store the new token: the session SHALL remember the token it last replaced and when, and a presentation of that immediately-previous token **within a configured grace period** (default 60 seconds) SHALL NOT be treated as reuse. It SHALL rotate the family again and return a usable pair, leaving the session live. Beyond that grace, or for any token older than the immediately-previous one, reuse detection applies unchanged. Rotation and reuse detection MUST be **atomic** — the check-and-rotate SHALL be a single durable, conditional operation so that concurrent refreshes cannot both take the same branch.

An expired **access** token alone MUST NOT require re-authentication while the refresh token is still valid. Each session/refresh token is **bound to the audience chosen at sign-in**; `Refresh` preserves that audience (it takes no audience parameter), and tokens are never shared across apps — a user signs in to each app **independently (one login per app)**.

#### Scenario: Expired access token is refreshed without re-login

- **WHEN** a client's access token has expired but its refresh token is still valid
- **THEN** the client obtains a new access token via refresh, with no credential re-entry by the user

#### Scenario: Successful sign-in issues an audience-scoped token

- **WHEN** a sign-in succeeds for an allowed audience (valid local credentials with a verified email, or a valid OIDC token)
- **THEN** the module returns a signed access token whose `aud` is that app and whose roles are the effective set for that audience, plus a refresh token

#### Scenario: Unknown audience is rejected

- **WHEN** a sign-in targets an audience that is not in the configured allow-list
- **THEN** the module rejects it with gRPC status `INVALID_ARGUMENT` and issues no tokens

#### Scenario: Wrong local password is rejected

- **WHEN** a local sign-in supplies an incorrect password
- **THEN** the module rejects it with gRPC status `UNAUTHENTICATED` and issues no tokens

#### Scenario: Protected call requires a valid internal token

- **WHEN** a request to a protected method has a missing, invalid, or expired internal access token
- **THEN** the interceptor rejects it with gRPC status `UNAUTHENTICATED`

#### Scenario: Refresh rotates the token

- **WHEN** a valid, unexpired refresh token is presented
- **THEN** a new access token is issued and the refresh token is rotated

#### Scenario: Revoked or expired refresh is rejected

- **WHEN** a refresh token that is expired or revoked is presented
- **THEN** the module rejects it with gRPC status `UNAUTHENTICATED`

#### Scenario: A client killed before it stored the new token

- **WHEN** the token that was replaced less than the grace period ago is presented again
- **THEN** the module rotates the family again and returns a usable token pair, and the session stays live

#### Scenario: Reused refresh token revokes the session

- **WHEN** an already-rotated refresh token is replayed after the grace period, or a token older than the immediately-previous one is replayed
- **THEN** the module rejects it and revokes the whole session family so the stolen token chain is dead

#### Scenario: Concurrent refresh of the same token

- **WHEN** the same refresh token is presented by two concurrent requests
- **THEN** exactly one rotates it, and the other — being the immediately-previous token inside the grace — also receives a usable pair, with the family left live
