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

### Requirement: Resource-consumption metrics

The backend SHALL emit resource-consumption metrics as OpenTelemetry metrics over
OTLP: the **process** CPU usage and resident memory, **async-runtime saturation**
(task backlog / busy ratio), and **database connection-pool** usage (in-use /
idle). **Host-level** resource metrics (CPU, memory, disk, network) SHALL be
collectable via the OpenTelemetry Collector. All MUST be viewable in Grafana.

#### Scenario: Process resource metrics exported

- **WHEN** the backend runs with metrics enabled
- **THEN** the process CPU usage and resident memory are exported over OTLP and are
  queryable in Grafana

#### Scenario: Saturation is observable

- **WHEN** the async runtime or the DB connection pool is under load
- **THEN** the runtime task/queue metrics and pool in-use/idle metrics reflect it

#### Scenario: Host metrics via the collector

- **WHEN** the observability stack is running
- **THEN** host CPU, memory, disk, and network metrics are available in Grafana

### Requirement: Structured logs as an OpenTelemetry signal

The backend SHALL emit application logs through a single logging API (`tracing`)
and bridge them into the OpenTelemetry **Logs** signal, exported over OTLP. Each
emitted log record produced within a request span MUST carry the active
`trace_id` and `span_id` so logs are correlated with traces. A console log output
MUST remain available in parallel, and the OTLP log export MUST be disablable
independently for tests/offline runs.

#### Scenario: Logs exported with trace correlation

- **WHEN** code logs within a request span and log export is enabled
- **THEN** the log record is exported over OTLP carrying the request's `trace_id`
  and `span_id`

#### Scenario: Logs still produced when OTLP export is disabled

- **WHEN** OTLP log export is disabled
- **THEN** the backend still writes structured logs to the console and exports none

### Requirement: No secrets or PII in telemetry

Telemetry (spans, attributes, metrics, logs) MUST NOT contain bearer tokens,
secrets, or raw personal data.

#### Scenario: Sensitive values are absent from telemetry

- **WHEN** a request carrying a bearer token is traced
- **THEN** no bearer token or secret appears in any span attribute, metric label,
  or log field

### Requirement: Grafana exploration stack

The project SHALL provide a local observability stack — an OpenTelemetry Collector,
a trace backend (e.g. Tempo), a metrics backend (e.g. Prometheus), a logs backend
(e.g. Loki), and Grafana — wired to receive the backend's OTLP output, with
**trace↔logs correlation** configured, plus technical documentation for using these
tools.

#### Scenario: Stack started and receiving data

- **WHEN** a developer starts the observability stack and runs the backend against it
- **THEN** the backend's traces, metrics, and logs are visible/queryable in Grafana

#### Scenario: Jump between a trace and its logs

- **WHEN** a developer opens a request's trace in Grafana
- **THEN** they can navigate to that request's correlated logs (and back) via the
  shared `trace_id`

#### Scenario: Documentation enables usage

- **WHEN** a developer follows the observability documentation
- **THEN** they can start the stack, open Grafana, and find the request's traces,
  metrics, and correlated logs for a sample request without further guidance
