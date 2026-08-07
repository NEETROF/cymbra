## ADDED Requirements

### Requirement: Slash commands are served by a signature-verified interactions endpoint

The backend SHALL expose one HTTP endpoint that receives Discord interactions, and SHALL
verify every request's **Ed25519 signature** against the Discord application public key before
interpreting the payload. A request whose signature or timestamp header is missing or invalid
MUST be rejected with `401` and MUST NOT produce any side effect. The endpoint SHALL answer
Discord's `PING` verification with the `PONG` acknowledgement so the application can be
registered. No gateway (persistent WebSocket) connection SHALL be required.

#### Scenario: Valid signature is processed

- **WHEN** an interaction arrives with a signature that verifies against the application public key
- **THEN** the command is processed and a response is returned

#### Scenario: Invalid signature is rejected without side effect

- **WHEN** an interaction arrives with a missing, malformed, or non-verifying signature
- **THEN** the request is rejected with `401` and no command is executed

#### Scenario: Registration ping is acknowledged

- **WHEN** Discord sends the `PING` interaction to verify the endpoint
- **THEN** the endpoint replies with the `PONG` acknowledgement

#### Scenario: No gateway connection is needed

- **WHEN** the backend runs with no outbound WebSocket to Discord
- **THEN** slash commands still work

### Requirement: Slash commands answer inside Discord's acknowledgement window

A command handler SHALL acknowledge the interaction within Discord's timeout, deferring the
reply when the work cannot complete in time and following up once it does. A handler that
depends on slow work MUST NOT let the interaction expire.

#### Scenario: Fast command replies directly

- **WHEN** a command's answer is available immediately
- **THEN** the reply is returned in the initial response

#### Scenario: Slow command is deferred then followed up

- **WHEN** a command needs work that exceeds the acknowledgement window
- **THEN** the interaction is deferred immediately and the answer is sent as a follow-up

### Requirement: Slash command answers respect the public field set

A command's answer SHALL disclose only data the requester is already entitled to see on a
public surface. Player-specific answers SHALL apply the same fail-closed gate as public
listing (public profile, age-eligible) and MUST NOT disclose email addresses, raw account
identifiers, moderation state, or curator reliability figures. A command about a player who is
not publicly listable SHALL return a neutral "not available" answer that does not reveal
whether the account exists.

#### Scenario: Command about a listable player returns public fields

- **WHEN** a command asks about a player who is publicly listable
- **THEN** only public profile fields are returned

#### Scenario: Command about a non-listable player reveals nothing

- **WHEN** a command asks about a player who is private, not age-eligible, or unknown
- **THEN** the same neutral "not available" answer is returned in all three cases, so existence is not disclosed

### Requirement: Discord roles are granted from Cymbra account state

The system SHALL be able to grant and revoke Discord roles from the worker via Discord's REST
API, driven by Cymbra account state (for example a verified-account role). Role operations
SHALL be **idempotent**: granting a role the member already has, or revoking one they do not
have, MUST succeed without error. A role operation for an account that loses its qualifying
state SHALL revoke the role. The bot SHALL be configured with **least-privilege** permissions —
only what sending messages and managing the roles it owns requires — and MUST NOT be granted
administrator permission.

#### Scenario: Qualifying account receives the role

- **WHEN** an account reaches the state a Discord role represents and its Discord link is known
- **THEN** the role is granted to that member

#### Scenario: Repeated grant is harmless

- **WHEN** the same role grant is executed twice
- **THEN** the second execution succeeds and changes nothing

#### Scenario: Losing the state revokes the role

- **WHEN** an account no longer qualifies for a role it holds
- **THEN** the role is revoked

### Requirement: Bot credentials come from the environment and disable the feature when absent

The bot token, the application public key, and the channel webhook URLs SHALL be read from
configuration/environment and MUST NOT appear in source, in logs, or in error messages. When a
required credential is absent, the corresponding capability SHALL be **disabled rather than
degraded**: the interactions endpoint rejects requests it cannot verify, and role operations
are skipped as no-ops without failing their job.

#### Scenario: Missing public key disables interaction handling

- **WHEN** no application public key is configured
- **THEN** interactions cannot be verified and are rejected, and the rest of the backend is unaffected

#### Scenario: Missing bot token skips role operations

- **WHEN** no bot token is configured
- **THEN** a role operation completes as a no-op instead of failing the job

#### Scenario: Credentials never appear in output

- **WHEN** a Discord request fails and is logged
- **THEN** the log contains no token, no public key, and no webhook URL
