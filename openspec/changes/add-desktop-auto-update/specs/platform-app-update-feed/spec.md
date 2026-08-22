## ADDED Requirements

### Requirement: Anonymous update feed per product and channel

The backend SHALL expose an unauthenticated HTTP endpoint that returns the
current release envelope for a given product and channel. The response SHALL NOT
depend on the caller — no account, no install identifier, no client version
parameter — so that it is cacheable and cannot be used to count or track
installs.

#### Scenario: A servable release exists

- **WHEN** a client requests the feed for a product and channel that has a
  release ingested, not paused, and with a rollout percentage above zero
- **THEN** the endpoint SHALL return `200` with the stored envelope — the
  base64 manifest bytes, the base64 signature, the key id and the rollout
  percentage — and a `Cache-Control` header allowing shared caching

#### Scenario: No servable release

- **WHEN** the requested product/channel has no ingested release, or every
  candidate release is paused or at rollout zero
- **THEN** the endpoint SHALL return `204 No Content` with no body

#### Scenario: Identical response for every caller

- **WHEN** two different clients request the same product and channel at the
  same time
- **THEN** they SHALL receive byte-identical responses

#### Scenario: Unknown product or channel

- **WHEN** the request names a product or channel the feed does not serve
- **THEN** the endpoint SHALL return `204 No Content` rather than an error
  disclosing which products exist

### Requirement: The feed serves signed payloads it cannot produce

The backend SHALL treat the manifest as an opaque signed byte string. It SHALL
NOT sign, re-serialize, or otherwise alter the manifest bytes between ingest and
delivery, so that a compromise of the backend cannot forge a release.

#### Scenario: Bytes survive the round trip unchanged

- **WHEN** an envelope is ingested and later served
- **THEN** the manifest bytes and signature returned SHALL be byte-identical to
  those ingested

#### Scenario: The backend holds no signing key

- **WHEN** the backend configuration is inspected
- **THEN** it SHALL contain the public verification key only, and no private
  signing material

### Requirement: Ingest verifies the signature before storing

The backend SHALL expose a credential-gated ingest endpoint for publishing a
release envelope, and SHALL verify the Ed25519 signature against a configured
trusted public key — selected by the envelope's key id — before storing
anything.

#### Scenario: Valid envelope is stored

- **WHEN** an authorized caller submits an envelope whose signature verifies
  against the trusted key named by its key id
- **THEN** the backend SHALL store it with its rollout percentage and return
  success

#### Scenario: Invalid signature is rejected

- **WHEN** an authorized caller submits an envelope whose signature does not
  verify, or whose key id is not trusted
- **THEN** the backend SHALL reject the request without storing anything

#### Scenario: Missing or wrong credential

- **WHEN** a caller submits an envelope without a valid ingest credential
- **THEN** the backend SHALL reject the request with an authentication failure
  and store nothing

#### Scenario: Re-ingesting the same version

- **WHEN** an envelope is ingested for a product, channel and version that
  already exists
- **THEN** the backend SHALL replace the stored record rather than creating a
  duplicate, so that a corrected manifest can be republished

### Requirement: Staged rollout and pause

Each stored release SHALL carry a rollout percentage between 0 and 100 and a
paused flag. The feed SHALL expose the rollout percentage to the client for
client-side bucketing, and SHALL withhold a release entirely when it is paused
or at rollout zero.

#### Scenario: Kill-switch stops distribution immediately

- **WHEN** the rollout percentage of the current release is set to zero, or the
  release is paused
- **THEN** subsequent feed requests SHALL stop returning that release, within
  the advertised cache lifetime

#### Scenario: Rollout percentage is delivered, not enforced server-side

- **WHEN** a release is served at a partial rollout percentage
- **THEN** the response SHALL carry that percentage and the backend SHALL NOT
  vary the response per caller to implement it

#### Scenario: Only the newest servable release is offered

- **WHEN** several releases exist for a product and channel
- **THEN** the feed SHALL return the highest version that is neither paused nor
  at rollout zero

### Requirement: Minimum supported version is carried by the signed manifest

The minimum client version a release declares as still supported SHALL live
inside the signed manifest, not in the mutable policy fields, so that a client
cannot be pushed into a forced-update state by anything other than a signed
release.

#### Scenario: Forced-update floor is signed

- **WHEN** a client verifies a manifest
- **THEN** the minimum supported version it acts on SHALL be one covered by the
  verified signature

### Requirement: The feed path is reachable through the production edge

The feed and ingest paths SHALL be part of the production reverse-proxy HTTP
allow-list, because the edge routes by path and a path outside the allow-list is
forwarded to the gRPC handler instead, answering with an empty successful
response rather than a transport error.

#### Scenario: Production edge routes the feed to the HTTP surface

- **WHEN** the feed path is requested against the production endpoint
- **THEN** it SHALL be served by the HTTP surface, and SHALL NOT return a
  gRPC-style response
