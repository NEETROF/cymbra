## ADDED Requirements

### Requirement: gRPC service foundation

The backend SHALL run as a gRPC server built on tonic, exposing its services
over HTTP/2. The server MUST NOT expose a REST API in this change.

#### Scenario: Server starts and serves gRPC

- **WHEN** the backend process starts with valid configuration
- **THEN** it binds the configured address and serves the registered gRPC
  services over HTTP/2

#### Scenario: gRPC reflection available in non-production

- **WHEN** the server runs with reflection enabled
- **THEN** a gRPC client can list the available services and methods without a
  local copy of the `.proto` files

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
