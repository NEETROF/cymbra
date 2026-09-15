## MODIFIED Requirements

### Requirement: Local credential hardening

The auth module SHALL enforce a configurable **password policy** at sign-up and
password reset, **rate-limit** local sign-in attempts with a temporary lockout
after repeated failures, and **throttle** verification/reset email sends. Limits
MUST be tracked centrally (shared cache) so they hold across instances. Every limit
MUST key the email in a normalized form (trimmed, lower-cased), and the sign-in
lockout and the per-email send throttles MUST be keyed on the pair (email, client
address), so that failures or requests from one address cannot lock out or exhaust
the budget of the same email at another address. The module MUST additionally limit
failed sign-ins and email sends per client address across emails, and MUST cap failed
sign-ins and email sends per email across addresses with a separate, higher ceiling.
The client address MUST be resolved from the first `X-Forwarded-For` hop set by the
reverse proxy, else `X-Real-IP`, else the peer address.

#### Scenario: Weak password rejected

- **WHEN** a sign-up or reset supplies a password failing the policy
- **THEN** the module rejects it with `INVALID_ARGUMENT` and stores nothing

#### Scenario: Repeated failures trigger lockout

- **WHEN** local sign-in for an email fails more than the configured threshold from one
  client address
- **THEN** further attempts for that email from that address are temporarily refused
  with `RESOURCE_EXHAUSTED` until the window elapses

#### Scenario: Another address is not locked out

- **WHEN** an email is locked out for one client address and its owner signs in with the
  correct password from a different address
- **THEN** the sign-in succeeds

#### Scenario: Credential stuffing from one address

- **WHEN** one client address accumulates more failed sign-ins across emails than the
  per-address limit
- **THEN** further sign-in attempts from that address are refused with
  `RESOURCE_EXHAUSTED` until the window elapses

#### Scenario: Distributed attack on one account

- **WHEN** failed sign-ins for one email across all addresses exceed the per-email ceiling
  within its window
- **THEN** further password sign-ins for that email are refused with `RESOURCE_EXHAUSTED`
  until the window elapses

#### Scenario: Letter case does not reset the counters

- **WHEN** the same email is submitted with different letter case or surrounding spaces
- **THEN** the attempts count against the same counters

#### Scenario: Email sends are throttled

- **WHEN** verification or reset emails for one email are requested from one address faster
  than the configured rate
- **THEN** the excess requests are throttled and no additional email is sent

#### Scenario: One address cannot exhaust another's email budget

- **WHEN** one client address has exhausted its verification or reset budget for an email
- **THEN** a request for the same email from another address, within the per-email ceiling,
  still sends the email
