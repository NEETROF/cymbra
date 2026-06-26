## ADDED Requirements

### Requirement: OpenTelemetry tracing

The backend SHALL instrument request handling with OpenTelemetry traces. Each
inbound gRPC request MUST produce a span with the method name, status, and a
trace/correlation id, and spans for downstream work (database, auth validation)
MUST be children of the request span. Trace context MUST propagate across async
boundaries within a request.

#### Scenario: Request produces a trace

- **WHEN** a gRPC request is handled
- **THEN** a span is recorded for the request with method name and resulting status
- **AND** downstream operations within the request appear as child spans of it

#### Scenario: Trace carries a correlation id

- **WHEN** a request is traced
- **THEN** the span carries a trace/correlation id that also appears in the
  structured logs for that request

### Requirement: OTLP export

The backend SHALL export telemetry (traces, and metrics where emitted) over OTLP
to a configurable endpoint. The endpoint and sampling MUST be configurable, and
export MUST be disablable for tests/local runs without telemetry infrastructure.

#### Scenario: Telemetry exported to the configured endpoint

- **WHEN** an OTLP endpoint is configured and the backend handles requests
- **THEN** traces (and metrics, if enabled) are exported to that endpoint

#### Scenario: Telemetry can be disabled

- **WHEN** telemetry export is disabled by configuration
- **THEN** the backend runs normally and emits no OTLP export

### Requirement: No secrets or PII in telemetry

Telemetry (spans, attributes, metrics, logs) MUST NOT contain bearer tokens,
secrets, or raw personal data.

#### Scenario: Sensitive values are absent from telemetry

- **WHEN** a request carrying a bearer token is traced
- **THEN** no bearer token or secret appears in any span attribute, metric label,
  or log field

### Requirement: Grafana exploration stack

The project SHALL provide a local observability stack — an OpenTelemetry Collector,
a trace backend (e.g. Tempo), a metrics backend (e.g. Prometheus), and Grafana —
wired to receive the backend's OTLP output, plus technical documentation for using
these tools.

#### Scenario: Stack started and receiving data

- **WHEN** a developer starts the observability stack and runs the backend against it
- **THEN** the backend's traces and metrics are visible/queryable in Grafana

#### Scenario: Documentation enables usage

- **WHEN** a developer follows the observability documentation
- **THEN** they can start the stack, open Grafana, and find the request traces and
  metrics for a sample request without further guidance
