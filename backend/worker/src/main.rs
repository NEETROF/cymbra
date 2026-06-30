//! `cymbra-worker` — the background-job worker (design D2).
//!
//! Connects as `worker_svc`, runs the `jobs` migrations, starts the sqlxmq runner
//! (executes queued jobs), and spawns the recurring scheduler and the dead-letter
//! sweep. Serves a small health surface. The user-facing `cymbra-id` only
//! enqueues; this binary is what actually runs background work.

mod config;
mod handlers;
mod health;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use cymbra_platform::db;
use cymbra_platform::email::{EmailSender, SmtpSender};
use cymbra_user::{PgUserRepo, UserModule};

use crate::config::WorkerConfig;
use crate::handlers::WorkerCtx;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::from_filename("backend/.env").or_else(|_| dotenvy::dotenv());
    let cfg = WorkerConfig::from_env().map_err(|e| anyhow::anyhow!("worker config: {e}"))?;
    // Shared OTel init (service `cymbra-worker`): stdout logs always; traces/
    // metrics/logs over OTLP when enabled. Same pipeline as cymbra-id.
    let telemetry = cymbra_platform::telemetry::init(
        "cymbra-worker",
        cfg.otlp_enabled,
        cfg.otlp_endpoint.as_deref(),
    )?;

    // --- queue connection (worker_svc) + migrations ---
    let queue_pool = db::connect(&cfg.worker_database_url, cfg.concurrency_max as u32 + 2).await?;
    cymbra_jobs::MIGRATOR.run(&queue_pool).await?;

    // --- per-module connections for the work the handlers perform ---
    let user_pool = db::connect(&cfg.user_database_url, 5).await?;
    let user = Arc::new(UserModule::new(PgUserRepo::new(user_pool)));
    let email: Arc<dyn EmailSender> = Arc::new(SmtpSender::new(&cfg.smtp_url, &cfg.smtp_from)?);

    let ctx = WorkerCtx {
        email,
        user,
        reap_grace_secs: cfg.orphan_reap_grace.as_secs() as i64,
    };

    // --- sqlxmq runner: executes queued jobs (event-driven; design D7) ---
    let registry = handlers::registry(ctx);
    let _runner = registry
        .runner(&queue_pool)
        .set_concurrency(cfg.concurrency_min, cfg.concurrency_max)
        .run()
        .await?;
    tracing::info!(
        min = cfg.concurrency_min,
        max = cfg.concurrency_max,
        "job runner started"
    );

    // --- recurring scheduler (design D5) ---
    spawn_scheduler(queue_pool.clone(), cfg.scheduler_interval);
    // --- dead-letter sweep (design D6) ---
    spawn_dlq_sweep(queue_pool.clone(), cfg.dlq_sweep_interval);

    // --- health surface ---
    let http_addr: SocketAddr = cfg.http_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    tracing::info!(%http_addr, "cymbra-worker serving health");
    let http = axum::serve(listener, health::router(queue_pool).into_make_service())
        .with_graceful_shutdown(shutdown_signal());

    http.await?;
    // Dropping `_runner` here stops polling; in-flight jobs finish or their lease
    // expires and another worker reclaims them (at-least-once; design D9).
    tracing::info!("cymbra-worker shutting down");
    telemetry.shutdown(); // flush OTLP exporters
    Ok(())
}

/// Evaluate `jobs.schedules` on an interval, enqueuing due occurrences.
fn spawn_scheduler(pool: sqlx::PgPool, interval: Duration) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(interval);
        loop {
            tick.tick().await;
            match cymbra_jobs::run_scheduler_tick(&pool, chrono::Utc::now()).await {
                Ok(n) if n > 0 => tracing::info!(enqueued = n, "scheduler enqueued occurrences"),
                Ok(_) => {}
                Err(e) => tracing::warn!(error = %e, "scheduler tick failed"),
            }
        }
    });
}

/// Move retry-exhausted jobs to the dead-letter store on an interval and alert.
fn spawn_dlq_sweep(pool: sqlx::PgPool, interval: Duration) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(interval);
        loop {
            tick.tick().await;
            match cymbra_jobs::dead_letter_sweep(&pool).await {
                // Non-zero is the alert signal (Grafana alerts on jobs.dead_letter
                // growth; this log line is the structured backstop).
                Ok(n) if n > 0 => tracing::error!(dead_lettered = n, "jobs moved to dead-letter"),
                Ok(_) => {}
                Err(e) => tracing::warn!(error = %e, "dead-letter sweep failed"),
            }
        }
    });
}

/// Resolve on Ctrl-C or SIGTERM for graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }
}
