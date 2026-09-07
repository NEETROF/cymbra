## MODIFIED Requirements

### Requirement: Health and readiness

The backend SHALL expose liveness and readiness checks. Readiness MUST reflect
the availability of critical dependencies (the database and Redis).

When the backend is served by **more than one process**, each process SHALL expose its own
liveness and readiness checks, reflecting its own dependencies. A deployment SHALL be
reported healthy only when **every** component reports ready: a probe that covers one
process MUST NOT make a deployment green while another is failing or absent.

A component that depends on another component SHALL report ready on its own dependencies,
and SHALL NOT report not-ready merely because a peer is unavailable — a peer outage
degrades the affected calls, it does not take the component out of service.

#### Scenario: Ready when dependencies healthy

- **WHEN** the database and Redis are reachable
- **THEN** the readiness check reports serving/healthy

#### Scenario: Not ready when a dependency is down

- **WHEN** the database or Redis is unreachable
- **THEN** the readiness check reports not-serving

#### Scenario: A deployment is green only when every component is

- **WHEN** one component is ready and another is failing or was never started
- **THEN** the deployment is reported unhealthy

#### Scenario: A peer outage does not take a component out of service

- **WHEN** a component cannot reach a peer it calls
- **THEN** it still reports ready, and the calls that need the peer degrade individually

### Requirement: gRPC service foundation

The backend SHALL run as a gRPC server built on tonic, exposing its services
over HTTP/2. The server MUST NOT expose a REST API.

To serve the browser-based moderation back office, the backend SHALL additionally
expose the same gRPC services over a **browser-reachable gRPC-web** endpoint with a
**CORS policy restricted to the configured back-office origin(s)**. gRPC-web is a
framing of gRPC, not a REST API, so this preserves the no-REST constraint. The native
(HTTP/2 gRPC) surface used by the mobile/desktop app SHALL be unchanged, and every
method exposed over gRPC-web SHALL enforce the same authentication and role
authorization as over native gRPC.

The served surface MAY be distributed across several processes. When it is, the
**client-visible contract SHALL be unchanged**: a client SHALL reach every service it used
before at the same address, with the same authentication, whatever process now serves it.
Routing a service to a different process is an operational change, not a contract change.

#### Scenario: Server starts and serves gRPC

- **WHEN** the backend process starts with valid configuration
- **THEN** it binds the configured address and serves the registered gRPC
  services over HTTP/2

#### Scenario: A relocated service stays at the same address

- **WHEN** a service is moved to a separate process
- **THEN** an existing client reaches it unchanged, with no client release required

#### Scenario: Authorization is unchanged by relocation

- **WHEN** a relocated service receives a call
- **THEN** it enforces the same authentication and role authorization it enforced before
  the move
