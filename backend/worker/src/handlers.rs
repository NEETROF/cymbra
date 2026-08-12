//! Job handlers + registry (tasks 3.1, 3.2; the worker side of the `JobHandler`
//! seam, task 2.4). Coverage-excluded engine glue: each `#[job]` handler is a
//! thin adapter that deserializes the payload and calls into the relevant module
//! (email sender, user module). Job *names* match `cymbra_jobs::registry` so a
//! producer's `jobs.enqueue(name => ...)` dispatches here. Handlers are
//! at-least-once and MUST be idempotent (design D9).

use std::error::Error;
use std::sync::Arc;

use cymbra_platform::email::EmailSender;
use cymbra_storage::ObjectStorage;
use cymbra_user::{PgUserRepo, UserModule};
use serde::Deserialize;
use sqlx::PgPool;
use sqlxmq::{CurrentJob, JobRegistry};
use tracing::Instrument;

/// Error type sqlxmq handlers return; any module error converts into it.
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// Shared context injected into every handler by type (sqlxmq registry context).
/// `Clone + Send + Sync + 'static` as the macro requires.
#[derive(Clone)]
pub struct WorkerCtx {
    pub email: Arc<dyn EmailSender>,
    pub user: Arc<UserModule<PgUserRepo>>,
    /// `auth_svc` pool for the session-reaper job (auth owns `auth.sessions`).
    pub auth_pool: PgPool,
    /// `admin_svc` pool for the `purge_user` job — the only actor that may write
    /// both schemas, so the account erasure is atomic. Worker-only (design D0).
    pub admin_pool: PgPool,
    /// Object store for the `purge_score_object` job. `None` when the score-upload
    /// feature is unconfigured (then that job is a no-op).
    pub storage: Option<Arc<dyn ObjectStorage>>,
    pub reap_grace_secs: i64,
    /// Retention window (days) for the `play_detail_prune` job (change: add-play-
    /// activity-profile). Prunes `music.play_sessions` detail older than this.
    pub play_detail_retention_days: i64,
    /// Shared feature-flag service (change: add-feature-usage-analytics). The
    /// `usage_purge` job reads the raw-event retention window from it so it is
    /// BO-retunable without a redeploy (design D4).
    pub flags: Arc<cymbra_feature_flags::FlagService>,
    /// Push dispatcher for the `push_dispatch` job (change: add-push-notifications).
    /// `None` when Firebase credentials are unconfigured — the job then completes
    /// as a no-op rather than failing, so a deployment without push is inert.
    pub push: Option<Arc<cymbra_notifications::Dispatcher>>,
}

/// Payload for the `verification_email` job (and any transactional email). The
/// producer renders the branded multipart body (design D2); the worker only
/// transports `{to, subject, html, text}`.
#[derive(Deserialize)]
struct EmailJob {
    to: String,
    subject: String,
    html: String,
    text: String,
}

/// Send a transactional email. Idempotency is the producer's responsibility
/// (the verification flow enqueues exactly once per token), so a re-delivery
/// simply re-sends — acceptable for verification mail.
#[sqlxmq::job("verification_email")]
pub async fn verification_email(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    // One span per execution → a trace per job in Tempo (service `cymbra-worker`).
    let span = tracing::info_span!("job.verification_email", job_id = %job.id());
    async move {
        let p: EmailJob = job
            .json()?
            .ok_or("verification_email: missing JSON payload")?;
        let email = cymbra_platform::email_template::RenderedEmail {
            subject: p.subject,
            html: p.html,
            text: p.text,
        };
        ctx.email.send(&p.to, &email).await?;
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Purge handle-less accounts abandoned during onboarding. Reuses the existing
/// `UserModule::reap_orphans` policy (design D10); naturally idempotent (a
/// second run finds nothing left to purge).
#[sqlxmq::job("orphan_reap")]
pub async fn orphan_reap(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.orphan_reap", job_id = %job.id());
    async move {
        let now = chrono::Utc::now().timestamp();
        let purged = ctx.user.reap_orphans(now, ctx.reap_grace_secs).await?;
        if purged > 0 {
            tracing::info!(reaped = purged, "orphan accounts purged (scheduled job)");
        }
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Delete expired `auth.sessions` rows (change: durable-sessions-postgres). Lazy
/// expiry enforces correctness on read, so this is pure table hygiene; naturally
/// idempotent (a second run finds nothing to delete). Uses the auth-scoped pool.
#[sqlxmq::job("session_reap")]
pub async fn session_reap(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.session_reap", job_id = %job.id());
    async move {
        let deleted = cymbra_auth::reap_expired_sessions(&ctx.auth_pool).await?;
        if deleted > 0 {
            tracing::info!(reaped = deleted, "expired sessions purged (scheduled job)");
        }
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Payload for the `purge_user` job (change: complete-account-deletion).
#[derive(Deserialize)]
struct PurgeUserJob {
    user_id: String,
}

/// Completely erase a deleted user's data across `user_account` + `auth` in one
/// atomic `admin_svc` transaction (change: complete-account-deletion). A thin
/// adapter over [`cymbra_worker::purge_user`]; that function is idempotent, so
/// this at-least-once handler is safe to re-run (design D9).
#[sqlxmq::job("purge_user")]
pub async fn purge_user(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.purge_user", job_id = %job.id());
    async move {
        let p: PurgeUserJob = job.json()?.ok_or("purge_user: missing JSON payload")?;
        cymbra_worker::purge_user(&ctx.admin_pool, &p.user_id).await?;
        tracing::info!(user_id = %p.user_id, "account data purged");
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Payload for the `purge_score_object` job (change: add-user-score-upload).
#[derive(Deserialize)]
struct PurgeScoreObjectJob {
    object_key: String,
}

/// Delete one stored score object by key. Enqueued during account erasure (and,
/// later, on a failed single-score object delete). Idempotent: deleting a missing
/// key is a no-op, so at-least-once re-delivery is safe. A no-op with a warning
/// when the store is unconfigured (the feature is off).
#[sqlxmq::job("purge_score_object")]
pub async fn purge_score_object(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.purge_score_object", job_id = %job.id());
    async move {
        let p: PurgeScoreObjectJob = job
            .json()?
            .ok_or("purge_score_object: missing JSON payload")?;
        match &ctx.storage {
            Some(store) => {
                store.delete(&p.object_key).await?;
                tracing::info!(object_key = %p.object_key, "score object purged");
            }
            None => tracing::warn!(
                object_key = %p.object_key,
                "purge_score_object skipped: object store not configured"
            ),
        }
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Prune the heavy per-session play detail past its retention window (change:
/// add-play-activity-profile, D7). Scheduled (`play_detail_prune_daily`); NULLs
/// the `session_result` JSONB on old `music.play_sessions` rows, keeping the
/// summary. Idempotent (already-pruned rows are skipped), so a redelivery is safe.
#[sqlxmq::job("play_detail_prune")]
pub async fn play_detail_prune(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.play_detail_prune", job_id = %job.id());
    async move {
        let pruned =
            cymbra_worker::prune_play_detail(&ctx.admin_pool, ctx.play_detail_retention_days)
                .await?;
        tracing::info!(pruned, "play-detail retention prune complete");
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Fold each closed day of `analytics.usage_events` into the permanent daily
/// aggregates (change: add-feature-usage-analytics, D3). Scheduled
/// (`usage_rollup_daily`) on the ordered `analytics.maintenance` channel BEFORE the
/// purge. Idempotent (upserts recompute from raw), so a redelivery is safe. Runs as
/// `admin_svc` (the only worker actor allowed to write across schemas).
#[sqlxmq::job("usage_rollup")]
pub async fn usage_rollup(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.usage_rollup", job_id = %job.id());
    async move {
        cymbra_analytics::rollup_closed_days(&ctx.admin_pool).await?;
        tracing::info!("usage analytics daily rollup complete");
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Delete raw `analytics.usage_events` older than the configured retention window
/// (change: add-feature-usage-analytics, D4). Scheduled (`usage_purge_daily`) on
/// the ordered `analytics.maintenance` channel AFTER the rollup, so it never
/// removes an unaggregated day. Reads the window from the feature-flag service
/// (refreshed on demand so a BO change applies without a redeploy); idempotent and
/// safe to retry. Runs as `admin_svc`.
#[sqlxmq::job("usage_purge")]
pub async fn usage_purge(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.usage_purge", job_id = %job.id());
    async move {
        // Pick up any BO retention change (best-effort; falls back to last-known /
        // code default on a store outage — fail-safe).
        if let Err(e) = ctx.flags.refresh().await {
            tracing::warn!(error = %e, "usage_purge flag refresh failed; using last-known/default");
        }
        let days = ctx.flags.int(
            cymbra_feature_flags::registry::DATA_RETENTION_USAGE_EVENTS_DAYS,
            180,
            &cymbra_feature_flags::EvalContext::anonymous(""),
        );
        let purged = cymbra_analytics::purge_expired(&ctx.admin_pool, days).await?;
        tracing::info!(
            purged,
            retention_days = days,
            "usage retention purge complete"
        );
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Settle honesty by community consensus (change: add-curation-rewards, task 3.2).
/// Scheduled sweep: freezes each consensus-ready score's truth and settles its
/// unsettled ratings. Idempotent (settlement state), so at-least-once is safe.
#[sqlxmq::job("consensus_honesty_settlement")]
pub async fn consensus_honesty_settlement(
    mut job: CurrentJob,
    ctx: WorkerCtx,
) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.consensus_honesty_settlement", job_id = %job.id());
    async move {
        let settled = cymbra_worker::settle_consensus_honesty(&ctx.admin_pool).await?;
        if settled > 0 {
            tracing::info!(settled, "consensus honesty settlement complete");
        }
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Roll the global leaderboard over to a new season (change: add-global-
/// leaderboard, task 2.2). Scheduled (`global_season_snapshot_daily`): freezes the
/// season that has just closed into the hall of fame. Idempotent (an
/// already-snapshotted season is skipped), so both a redelivery and a run on a day
/// with no rollover are no-ops.
#[sqlxmq::job("global_season_snapshot")]
pub async fn global_season_snapshot(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.global_season_snapshot", job_id = %job.id());
    async move {
        let frozen =
            cymbra_worker::snapshot_global_season(&ctx.admin_pool, ctx.user.as_ref()).await?;
        if frozen > 0 {
            tracing::info!(frozen, "global leaderboard season snapshot complete");
        }
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Payload for the generic `push_dispatch` job (change: add-push-notifications,
/// design D6). The platform ships no concrete notification type: the feature that
/// owns one supplies its `category`, its already-localized message, and — when it
/// has its own candidate query — the `user_ids` that query resolved. Omitting
/// `user_ids` targets every user holding a registered device.
#[derive(Deserialize)]
struct PushDispatchJob {
    category: String,
    title: String,
    body: String,
    /// Optional routing payload delivered with the notification.
    #[serde(default)]
    data: std::collections::BTreeMap<String, String>,
    /// The feature's candidate set; absent ⇒ every registered device.
    #[serde(default)]
    user_ids: Option<Vec<String>>,
    /// What an absent user preference means for this category (the owning
    /// feature's product default). Defaults to opt-in.
    #[serde(default = "default_true")]
    default_pref: bool,
}

fn default_true() -> bool {
    true
}

/// Run one category's push dispatch: resolve the flags, select recipients, send,
/// prune dead tokens (change: add-push-notifications, design D5/D6). Scheduled by
/// the feature that owns the category (a `jobs.schedules` row) or enqueued on an
/// event. Idempotent enough for at-least-once delivery: a redelivery re-runs
/// selection, so a closed kill-switch or a passed schedule hour sends nothing.
#[sqlxmq::job("push_dispatch")]
pub async fn push_dispatch(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let span = tracing::info_span!("job.push_dispatch", job_id = %job.id());
    async move {
        let p: PushDispatchJob = job.json()?.ok_or("push_dispatch: missing JSON payload")?;
        match &ctx.push {
            Some(dispatcher) => {
                // Pick up any BO flag change (best-effort; a store outage falls
                // back to last-known/defaults, i.e. sends nothing — fail-safe).
                if let Err(e) = ctx.flags.refresh().await {
                    tracing::warn!(error = %e, "push_dispatch flag refresh failed; using last-known/default");
                }
                let flags =
                    cymbra_notifications::resolve_flags(&ctx.flags, &p.category, p.default_pref);
                let audience = match p.user_ids {
                    Some(ids) => cymbra_notifications::Audience::Users(ids),
                    None => cymbra_notifications::Audience::All,
                };
                let mut msg = cymbra_notifications::PushMessage::new(p.title, p.body);
                for (k, v) in p.data {
                    msg = msg.with_data(k, v);
                }
                let report = dispatcher
                    .dispatch(&flags, &audience, &msg, chrono::Utc::now())
                    .await?;
                tracing::info!(
                    category = %p.category,
                    selected = report.selected,
                    delivered = report.delivered,
                    retryable = report.retryable,
                    pruned = report.pruned,
                    "push dispatch complete"
                );
            }
            None => tracing::warn!(
                category = %p.category,
                "push_dispatch skipped: FCM credentials not configured"
            ),
        }
        job.complete().await?;
        Ok(())
    }
    .instrument(span)
    .await
}

/// Build the job registry with all handlers registered and the shared context set.
pub fn registry(ctx: WorkerCtx) -> JobRegistry {
    let mut registry = JobRegistry::new(&[
        verification_email,
        orphan_reap,
        session_reap,
        purge_user,
        purge_score_object,
        play_detail_prune,
        usage_rollup,
        usage_purge,
        consensus_honesty_settlement,
        global_season_snapshot,
        push_dispatch,
    ]);
    registry.set_context(ctx);
    registry
}
