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
}

/// Payload for the `verification_email` job (and any transactional email).
#[derive(Deserialize)]
struct EmailJob {
    to: String,
    subject: String,
    body: String,
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
        ctx.email.send(&p.to, &p.subject, &p.body).await?;
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

/// Build the job registry with all handlers registered and the shared context set.
pub fn registry(ctx: WorkerCtx) -> JobRegistry {
    let mut registry = JobRegistry::new(&[
        verification_email,
        orphan_reap,
        session_reap,
        purge_user,
        purge_score_object,
    ]);
    registry.set_context(ctx);
    registry
}
