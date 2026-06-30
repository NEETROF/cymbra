//! OpenTelemetry init (group 6): traces + metrics + logs over OTLP, all
//! **disablable** for tests/offline runs. `tracing` stays the single logging API;
//! when OTLP is enabled, an appender bridges its events to the OTel Logs signal
//! (each in-span record carrying `trace_id`/`span_id`). Completes task 5.4.

use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

/// Owns the OTel providers; [`Telemetry::shutdown`] flushes them on exit.
pub struct Telemetry {
    tracer_provider: Option<opentelemetry_sdk::trace::TracerProvider>,
    meter_provider: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
    logger_provider: Option<opentelemetry_sdk::logs::LoggerProvider>,
}

impl Telemetry {
    /// Flush + shut down the exporters (call before process exit).
    pub fn shutdown(self) {
        if let Some(t) = self.tracer_provider {
            let _ = t.shutdown();
        }
        if let Some(m) = self.meter_provider {
            let _ = m.shutdown();
        }
        if let Some(l) = self.logger_provider {
            let _ = l.shutdown();
        }
    }

    fn disabled() -> Self {
        Self {
            tracer_provider: None,
            meter_provider: None,
            logger_provider: None,
        }
    }
}

fn resource(service_name: &str) -> Resource {
    Resource::new(vec![KeyValue::new(
        "service.name",
        service_name.to_string(),
    )])
}

/// Initialize logging + (optionally) the three OTel signals for `service_name`
/// (e.g. `cymbra-id`, `cymbra-worker`). Decoupled from any one binary's config so
/// every backend process can share it. Idempotent-safe.
pub fn init(
    service_name: &str,
    otlp_enabled: bool,
    otlp_endpoint: Option<&str>,
) -> anyhow::Result<Telemetry> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,cymbra=debug"));
    let console = fmt::layer();

    if !otlp_enabled {
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(console)
            .try_init();
        return Ok(Telemetry::disabled());
    }

    let endpoint = otlp_endpoint.unwrap_or("http://localhost:4317").to_string();

    // --- traces ---
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()?;
    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(span_exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(resource(service_name))
        .build();
    let tracer = tracer_provider.tracer(service_name.to_string());
    let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // --- metrics ---
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()?;
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(
        metric_exporter,
        opentelemetry_sdk::runtime::Tokio,
    )
    .build();
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource(service_name))
        .build();
    opentelemetry::global::set_meter_provider(meter_provider.clone());

    // --- logs (bridge tracing -> OTel Logs) ---
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()?;
    let logger_provider = opentelemetry_sdk::logs::LoggerProvider::builder()
        .with_batch_exporter(log_exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(resource(service_name))
        .build();
    let log_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(console)
        .with(trace_layer)
        .with(log_layer)
        .try_init();

    Ok(Telemetry {
        tracer_provider: Some(tracer_provider),
        meter_provider: Some(meter_provider),
        logger_provider: Some(logger_provider),
    })
}
