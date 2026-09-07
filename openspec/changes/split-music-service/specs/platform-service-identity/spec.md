## ADDED Requirements

### Requirement: A backend component can authenticate as itself

A backend component SHALL be able to prove its own identity when calling another backend
component, distinctly from any end-user identity. The credential SHALL name the calling
component, SHALL be verifiable by the callee without contacting a third party on the
request path, and SHALL be usable on calls that carry no end-user token at all — account
provisioning during sign-in happens before any user token exists.

A component credential SHALL NOT be accepted as an end-user identity, and an end-user token
SHALL NOT be accepted as a component credential.

#### Scenario: A component call carries its own identity

- **WHEN** one backend component calls another
- **THEN** the callee can determine which component is calling, independently of any
  end-user token on the request

#### Scenario: A pre-authentication call is possible

- **WHEN** a component performs an internal operation during sign-in, before any end-user
  token exists
- **THEN** the call is authenticated by the component credential alone

#### Scenario: The two identity kinds do not substitute for one another

- **WHEN** an end-user token is presented where a component credential is required, or the
  reverse
- **THEN** the call is refused

### Requirement: Internal operations are not reachable from the public surface

An internal component-to-component operation SHALL NOT be reachable by an end-user client,
on any served surface, whatever token it presents. Such operations include account
provisioning, effective role resolution, and visibility predicates.

Extracting a module SHALL NOT be an occasion to publish an internal operation on the public
contract: the internal surface is separate from the audience-facing one.

#### Scenario: An internal operation refuses an end-user token

- **WHEN** an end-user client calls an internal component operation with a valid end-user
  token
- **THEN** the call is refused

#### Scenario: Extraction does not widen the public contract

- **WHEN** a module is extracted and its callers move to a served transport
- **THEN** the operations they call are served on the internal surface, and the
  audience-facing contract gains no operation
