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
cargo run -p cymbra-server --bin cymbra-id
```

## Find a request's telemetry in Grafana

1. **Traces** — *Explore → Tempo → Search* (service `cymbra-id`). Open a
   `grpc.request` span; expand to see DB/auth child spans.
2. **Logs for that trace** — from the span, click **Logs for this span**
   (Tempo → Loki, filtered by `trace_id`). Or in *Explore → Loki*, click a log's
   **TraceID** derived field to jump back to the trace.
3. **Metrics** — *Explore → Prometheus*:
   - rate: `rate(rpc_server_requests_total[1m])`
   - latency: `histogram_quantile(0.95, rate(rpc_server_duration_seconds_bucket[5m]))`
   - resources: `process_memory_bytes`, `process_cpu_percent`, plus host
     `system_*` series from the collector.

## Notes

- No bearer tokens, passwords, secrets, or raw PII are placed in spans, metric
  labels, or logs (enforced by review + `logging::redact_bearer`; see
  `platform/src/logging.rs`).
- Production retention/sampling (Tempo/Loki retention, trace sampling ratio) is
  still to tune — this stack is sized for local dev.
