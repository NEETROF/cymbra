## ADDED Requirements

### Requirement: The served protobuf surface is checked for breaking changes

Continuous integration SHALL reject a change that breaks wire compatibility of any served
`.proto` against the surface already published to clients: a removed RPC, a removed or
renumbered field, a changed field type, or a renamed message or service.

The check SHALL run on every change that touches a `.proto`, and its failure SHALL block
the merge. An intentional break SHALL be possible only through an explicit, recorded
override in the change itself — never by silently editing the file.

#### Scenario: Removing an RPC is refused

- **WHEN** a change deletes an RPC that exists in the published surface
- **THEN** the compatibility job fails and the change cannot merge unmodified

#### Scenario: Renumbering a field is refused

- **WHEN** a change reuses or renumbers an existing field tag
- **THEN** the compatibility job fails

#### Scenario: Adding a field is allowed

- **WHEN** a change adds a new field with an unused tag, or a new RPC
- **THEN** the compatibility job passes

#### Scenario: An intentional break is explicit

- **WHEN** a change must break compatibility
- **THEN** it carries a recorded override naming the break, and the reason it is safe for
  the clients already in the field

### Requirement: A retired RPC stays reachable while shipped clients call it

An RPC that a released client binary calls SHALL NOT be removed from the served surface
while that release is still in use. It SHALL first be superseded — the client migrated,
the release shipped — and only then removed.

#### Scenario: A shipped client keeps working across a server deploy

- **WHEN** the server is deployed with a newer contract
- **THEN** a client from a previously released version still completes the calls it makes

#### Scenario: Retiring an RPC is sequenced

- **WHEN** an RPC is to be retired
- **THEN** the replacement is served first, the client release migrates to it, and the old
  RPC is removed in a later change
