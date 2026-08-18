//! `cymbra-worker` — the background-job worker (design D2).
//!
//! Connects as `worker_svc`, runs the `jobs` migrations, starts the sqlxmq runner
//! (executes queued jobs), and spawns the recurring scheduler and the dead-letter
//! sweep. Serves a small health surface. The user-facing `cymbra-server` only
//! enqueues; this binary is what actually runs background work.

mod config;
mod flags;
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
    // metrics/logs over OTLP when enabled. Same pipeline as cymbra-server.
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
    // The push registry lives in `user_account` (device tokens, category prefs,
    // timezone), so the dispatcher reads it through the same user_svc pool.
    let push_pool = user_pool.clone();
    let user = Arc::new(UserModule::new(PgUserRepo::new(user_pool)));
    // auth_svc pool for the session-reaper job (auth owns `auth.sessions`).
    let auth_pool = db::connect(&cfg.auth_database_url, 5).await?;
    // admin_svc pool for the purge_user job — worker-only cross-schema erasure.
    let admin_pool = db::connect(&cfg.admin_database_url, 5).await?;
    let email: Arc<dyn EmailSender> = Arc::new(SmtpSender::new(&cfg.smtp_url, &cfg.smtp_from)?);

    // Object store for the purge_score_object job (only when configured).
    let storage: Option<Arc<dyn cymbra_storage::ObjectStorage>> = match cfg.score_storage.as_ref() {
        Some(s3) => Some(Arc::new(cymbra_storage::LocalFirstStore::from_config(
            &cfg.score_local_root,
            &cymbra_storage::S3Params {
                bucket: s3.bucket.clone(),
                endpoint: s3.endpoint.clone(),
                region: s3.region.clone(),
                access_key: s3.access_key.clone(),
                secret_key: s3.secret_key.clone(),
                allow_http: s3.allow_http,
            },
        )?)),
        None => None,
    };

    // Shared feature-flag service (read-only) — the usage-purge job reads the
    // raw-event retention window from it (change: add-feature-usage-analytics, D4).
    let flag_service = flags::build_flag_service(cfg.flags_database_url.as_deref()).await?;

    // Push dispatcher for the `push_dispatch` job (change: add-push-notifications).
    // Wired only when Firebase credentials are configured; otherwise the job stays
    // inert so a deployment without push starts normally.
    let push: Option<Arc<cymbra_notifications::Dispatcher>> = match cfg
        .fcm_service_account_json
        .as_deref()
    {
        Some(raw) => {
            let sender: Arc<dyn cymbra_notifications::PushSender> = Arc::new(
                cymbra_notifications::FcmSender::from_service_account_json(raw)?,
            );
            let registry: Arc<dyn cymbra_notifications::PushRegistry> =
                Arc::new(cymbra_notifications::PgPushRegistry::new(push_pool));
            Some(Arc::new(cymbra_notifications::Dispatcher::new(
                registry, sender,
            )))
        }
        None => {
            tracing::info!("push notifications disabled (CYMBRA_FCM_SERVICE_ACCOUNT_JSON unset)");
            None
        }
    };

    // Score audio-teaser renderer for the `score_preview_render` job (change:
    // add-score-daily-access-rewards). Needs both the score store (MusicXML in,
    // WAV out) and the SoundFont store (the configured preview font); either
    // unconfigured leaves the job dormant. The catalog + font rows are read on the
    // admin pool (the worker's cross-schema actor).
    let soundfont_store: Option<Arc<dyn cymbra_storage::ObjectStorage>> =
        match cfg.soundfont_storage.as_ref() {
            Some(sf) => Some(Arc::new(cymbra_storage::LocalFirstStore::from_config(
                &sf.local_root,
                &cymbra_storage::S3Params {
                    bucket: sf.bucket.clone(),
                    endpoint: sf.endpoint.clone(),
                    region: sf.region.clone(),
                    access_key: sf.access_key.clone(),
                    secret_key: sf.secret_key.clone(),
                    allow_http: sf.allow_http,
                },
            )?)),
            None => None,
        };
    let score_preview = match (storage.clone(), soundfont_store) {
        (Some(score_store), Some(font_store)) => {
            Some(Arc::new(cymbra_music::ScorePreviewRenderer::new(
                score_store,
                font_store,
                Arc::new(cymbra_music::PgCatalogSearchRepo::new(admin_pool.clone())),
                Arc::new(cymbra_music::PgSoundFontRepo::new(admin_pool.clone())),
                Arc::new(flags::WorkerScorePreviewConfig::new(flag_service.clone())),
            )))
        }
        _ => {
            tracing::info!("score preview render dormant (score or soundfont store unset)");
            None
        }
    };

    // Plans (change: add-premium-subscription): the reconciliation + withdrawal
    // sweeps and the erasure's web-subscription cancellation. Wired only when the
    // plans DB is configured; the provider clients come from the same environment
    // block the server reads (channels missing their variables are skipped).
    let (plans, plan_reconciler, web_cancel, rc_erase) = match cfg.plans_database_url.as_deref() {
        Some(url) => {
            let plans_pool = cymbra_plans::pg::connect(url, 2).await?;
            let rotator: Arc<dyn cymbra_plans::CacheSecretRotator> =
                Arc::new(flags::OfflineSecretRotator::new(Arc::new(
                    cymbra_music::PgOfflineSecretRepo::new(admin_pool.clone()),
                )));
            let svc = Arc::new(cymbra_plans::PlanService::new(cymbra_plans::PlanDeps {
                entitlements: Arc::new(cymbra_plans::pg::PgEntitlementRepo::new(
                    plans_pool.clone(),
                )),
                campaigns: Arc::new(cymbra_plans::pg::PgCampaignRepo::new(plans_pool.clone())),
                memberships: Arc::new(cymbra_plans::pg::PgMembershipRepo::new(plans_pool.clone())),
                codes: Arc::new(cymbra_plans::pg::PgAccessCodeRepo::new(plans_pool.clone())),
                billing_events: Arc::new(cymbra_plans::pg::PgBillingEventRepo::new(
                    plans_pool.clone(),
                )),
                audit: Arc::new(cymbra_plans::pg::PgAuditRepo::new(plans_pool)),
                config: Arc::new(flags::WorkerPlanConfig::new(flag_service.clone())),
                clock: Arc::new(cymbra_plans::SystemClock),
                rotator: Some(rotator),
            }));
            let channels = cymbra_plans::billing::env::BillingChannels::build(
                &cymbra_plans::billing::env::BillingEnv::from_env(),
            );
            let reconciler = Arc::new(cymbra_plans::billing::reconcile::Reconciler {
                customers: channels.rc_customers.clone(),
                allow_sandbox: channels
                    .revenuecat
                    .as_ref()
                    .is_some_and(|rc| rc.allow_sandbox),
            });
            let web_cancel: Option<Arc<dyn cymbra_plans::WebSubscriptionCanceller>> = channels
                .web
                .clone()
                .map(|w| w as Arc<dyn cymbra_plans::WebSubscriptionCanceller>);
            (
                Some(svc),
                reconciler,
                web_cancel,
                channels.rc_eraser.clone(),
            )
        }
        None => {
            tracing::info!("plans jobs inert (CYMBRA_PLANS_DATABASE_URL unset)");
            (
                None,
                Arc::new(cymbra_plans::billing::reconcile::Reconciler::default()),
                None,
                None,
            )
        }
    };

    let ctx = WorkerCtx {
        email,
        user,
        auth_pool,
        admin_pool,
        storage,
        reap_grace_secs: cfg.orphan_reap_grace.as_secs() as i64,
        play_detail_retention_days: cfg.play_detail_retention_days as i64,
        flags: flag_service.clone(),
        push,
        score_preview,
        plans,
        plan_reconciler,
        plan_paywall: Arc::new(flags::WorkerPaywallConfig::new(flag_service.clone())),
        web_cancel,
        rc_erase,
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
