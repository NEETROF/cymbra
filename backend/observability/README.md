# Cymbra ID — observability

The backend emits all three OpenTelemetry signals over **OTLP/gRPC**:

- **Traces** — one span per gRPC request (`grpc.request`, with `rpc.method`) plus
  child spans for downstream work.
- **Metrics** — RED (`rpc.server.requests`, `rpc.server.duration.seconds`) and
  **resource** usage (`process.memory.bytes`, `process.cpu.percent`); host
  CPU/mem/disk/net come from the Collector's `hostmetrics` receiver.
- **Logs** — `tracing` events bridged to the OTel Logs signal, each in-span record
  carrying `trace_id` / `span_id` for correlation.

Export is **disabled by default** (`CYMBRA_OTLP_ENABLED=false`) so tests and
offline runs need no telemetry infrastructure.

## Start the stack

```bash
docker compose -f backend/observability/docker-compose.yml up -d
```

This brings up an **OpenTelemetry Collector** (OTLP in on `:4317`), **Tempo**
(traces), **Prometheus** (metrics), **Loki** (logs), and **Grafana**
(`http://localhost:3000`, anonymous admin) with datasources pre-provisioned.

## Point the backend at it

```bash
export CYMBRA_OTLP_ENABLED=true
export CYMBRA_OTLP_ENDPOINT=http://localhost:4317
cargo run -p cymbra-server --bin cymbra-server
```

## Find a request's telemetry in Grafana

1. **Traces** — *Explore → Tempo → Search* (service `cymbra-server`). Open a
   `grpc.request` span; expand to see DB/auth child spans.
2. **Logs for that trace** — from the span, click **Logs for this span**
   (Tempo → Loki, filtered by `trace_id`). Or in *Explore → Loki*, click a log's
   **TraceID** derived field to jump back to the trace.
3. **Metrics** — *Explore → Prometheus*:
   - rate: `rate(rpc_server_requests_total[1m])`
   - latency: `histogram_quantile(0.95, rate(rpc_server_duration_seconds_bucket[5m]))`
   - resources: `process_memory_bytes`, `process_cpu_percent`, plus host
     `system_*` series from the collector.

## Job queue (change: add-job-infrastructure)

The async-job substrate is observed with plain SQL over Grafana's Postgres
datasource (design D8) — no extra exporter. Provisioned automatically with the
stack:

- **Datasource** "Jobs (Postgres)" (`grafana/datasources.yaml`) — dev connects as
  `worker_svc` to the backend Postgres via `host.docker.internal`. Point this at a
  dedicated read-only grant in production.
- **Dashboard** "Cymbra — Job Queue" (`grafana/dashboards/jobs.json`, folder
  *Jobs*) — pending / in-flight / dead-lettered counts, per-channel depth and
  oldest age, and a recent-dead-letters table. Built on the `jobs.pending`,
  `jobs.inflight`, `jobs.failed`, and `jobs.channel_depth` views.
- **Alert** "Job dead-letter queue is non-empty" (`grafana/alerting.yaml`, folder
  *Jobs*) — fires whenever `jobs.dead_letter` is non-empty (a job exhausted its
  retries). Wire a contact point to your channel of choice.

The worker also serves `/healthz` and `/readyz` on `CYMBRA_WORKER_HTTP_ADDR`.
Queue pickup is event-driven (LISTEN/NOTIFY); the tuning knob is concurrency, not
a poll interval (design D7).

## Notes

- No bearer tokens, passwords, secrets, or raw PII are placed in spans, metric
  labels, or logs (enforced by review + `logging::redact_bearer`; see
  `platform/src/logging.rs`).
- Production retention/sampling (Tempo/Loki retention, trace sampling ratio) is
  still to tune — this stack is sized for local dev.
