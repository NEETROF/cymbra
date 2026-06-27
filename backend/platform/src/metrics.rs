//! RED + resource metrics over the OTel meter (tasks 6.3, 6.4), plus a tower
//! layer that opens a span and times each gRPC request.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Instant;

use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Histogram};
use tower::{Layer, Service};
use tracing::Instrument;

/// RED instruments (request rate / errors / duration), recorded per request.
pub struct RedMetrics {
    requests: Counter<u64>,
    duration: Histogram<f64>,
}

impl RedMetrics {
    pub fn new() -> Self {
        let meter = opentelemetry::global::meter("cymbra-id");
        Self {
            requests: meter.u64_counter("rpc.server.requests").build(),
            duration: meter.f64_histogram("rpc.server.duration.seconds").build(),
        }
    }

    fn record(&self, method: &str, status: &str, secs: f64) {
        let attrs = [
            KeyValue::new("rpc.method", method.to_string()),
            KeyValue::new("status", status.to_string()),
        ];
        self.requests.add(1, &attrs);
        self.duration.record(secs, &attrs);
    }
}

impl Default for RedMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Register process CPU + resident-memory observable gauges (task 6.4). Host
/// CPU/mem/disk/net come from the Collector's hostmetrics receiver.
pub fn install_resource_metrics() {
    let pid = match sysinfo::get_current_pid() {
        Ok(p) => p,
        Err(_) => return,
    };
    let meter = opentelemetry::global::meter("cymbra-id");
    let sys = Arc::new(Mutex::new(sysinfo::System::new()));

    let sys_mem = sys.clone();
    let _mem = meter
        .u64_observable_gauge("process.memory.bytes")
        .with_callback(move |obs| {
            let mut s = sys_mem.lock().unwrap();
            s.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
            if let Some(p) = s.process(pid) {
                obs.observe(p.memory(), &[]);
            }
        })
        .build();

    let sys_cpu = sys.clone();
    let _cpu = meter
        .f64_observable_gauge("process.cpu.percent")
        .with_callback(move |obs| {
            let mut s = sys_cpu.lock().unwrap();
            s.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
            if let Some(p) = s.process(pid) {
                obs.observe(p.cpu_usage() as f64, &[]);
            }
        })
        .build();
}

/// tower layer: opens a `grpc.request` span and records RED metrics per request.
#[derive(Clone)]
pub struct ObserveLayer {
    red: Arc<RedMetrics>,
}

impl ObserveLayer {
    pub fn new(red: Arc<RedMetrics>) -> Self {
        Self { red }
    }
}

impl<S> Layer<S> for ObserveLayer {
    type Service = Observe<S>;
    fn layer(&self, inner: S) -> Self::Service {
        Observe {
            inner,
            red: self.red.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Observe<S> {
    inner: S,
    red: Arc<RedMetrics>,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for Observe<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let method = req.uri().path().to_string();
        let red = self.red.clone();
        // Per the tower contract, call the *ready* clone, not `&mut self.inner`.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let span = tracing::info_span!("grpc.request", rpc.method = %method);
        Box::pin(async move {
            let start = Instant::now();
            let res = inner.call(req).instrument(span).await;
            let status = if res.is_ok() { "ok" } else { "error" };
            red.record(&method, status, start.elapsed().as_secs_f64());
            res
        })
    }
}
