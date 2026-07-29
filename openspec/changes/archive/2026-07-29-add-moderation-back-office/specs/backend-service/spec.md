## MODIFIED Requirements

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
