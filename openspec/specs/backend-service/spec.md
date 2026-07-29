# backend-service Specification

## Purpose
TBD - created by archiving change add-cymbra-id. Update Purpose after archive.
## Requirements
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

#### Scenario: Server starts and serves gRPC

- **WHEN** the backend process starts with valid configuration
- **THEN** it binds the configured address and serves the registered gRPC
  services over HTTP/2

#### Scenario: gRPC reflection available in non-production

- **WHEN** the server runs with reflection enabled
- **THEN** a gRPC client can list the available services and methods without a
  local copy of the `.proto` files

#### Scenario: Back office reaches the API over gRPC-web

- **WHEN** the Vue back office at an allowed origin calls a gRPC method over gRPC-web
- **THEN** the request is served with the same auth/role enforcement as native gRPC

#### Scenario: Disallowed origin is blocked by CORS

- **WHEN** a browser request originates from an origin not in the configured allow-list
- **THEN** the CORS policy blocks it

#### Scenario: No REST API is introduced

- **WHEN** the browser transport is added for the back office
- **THEN** it uses gRPC-web framing only and the backend still exposes no REST API

### Requirement: Configuration

The backend SHALL load configuration from environment variables (and optional
config file) at startup, covering the listen address, database URL, object-store
settings, and OIDC issuer/audience. It MUST fail fast with a clear error when a
required value is missing or invalid.

#### Scenario: Missing required configuration

- **WHEN** a required configuration value (e.g. database URL) is absent at startup
- **THEN** the process exits non-zero with an error message naming the missing key
- **AND** does not begin serving requests

### Requirement: Database connectivity and migrations

The backend SHALL connect to Postgres through a pooled connection and SHALL apply
schema migrations. Migrations MUST be versioned and idempotent across restarts.

#### Scenario: Migrations applied on startup

- **WHEN** the server starts against a database missing the latest schema
- **THEN** pending migrations are applied before the server accepts traffic

#### Scenario: Database unavailable

- **WHEN** the database cannot be reached at startup
- **THEN** startup fails with a clear error and the server does not report ready

### Requirement: Health and readiness

The backend SHALL expose liveness and readiness checks. Readiness MUST reflect
the availability of critical dependencies (the database and Redis).

#### Scenario: Ready when dependencies healthy

- **WHEN** the database and Redis are reachable
- **THEN** the readiness check reports serving/healthy

#### Scenario: Not ready when a dependency is down

- **WHEN** the database or Redis is unreachable
- **THEN** the readiness check reports not-serving

### Requirement: Structured logging

The backend SHALL emit structured logs for each request, including a
request/correlation identifier (the trace id when the request is traced) and the
resolved user identity when authenticated. Logs MUST NOT contain secrets or bearer
tokens. (Export of logs as an OpenTelemetry signal is covered by the
`observability` capability.)

#### Scenario: Request is logged

- **WHEN** a gRPC request is handled
- **THEN** a structured log entry is emitted with method name, status, and
  correlation id
- **AND** no bearer token or secret appears in the output

