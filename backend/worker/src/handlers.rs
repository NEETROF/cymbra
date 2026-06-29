//! Job handlers + registry (tasks 3.1, 3.2; the worker side of the `JobHandler`
//! seam, task 2.4). Coverage-excluded engine glue: each `#[job]` handler is a
//! thin adapter that deserializes the payload and calls into the relevant module
//! (email sender, user module). Job *names* match `cymbra_jobs::registry` so a
//! producer's `jobs.enqueue(name => ...)` dispatches here. Handlers are
//! at-least-once and MUST be idempotent (design D9).

use std::error::Error;
use std::sync::Arc;

use cymbra_platform::email::EmailSender;
use cymbra_user::{PgUserRepo, UserModule};
use serde::Deserialize;
use sqlxmq::{CurrentJob, JobRegistry};

/// Error type sqlxmq handlers return; any module error converts into it.
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// Shared context injected into every handler by type (sqlxmq registry context).
/// `Clone + Send + Sync + 'static` as the macro requires.
#[derive(Clone)]
pub struct WorkerCtx {
    pub email: Arc<dyn EmailSender>,
    pub user: Arc<UserModule<PgUserRepo>>,
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
    let p: EmailJob = job
        .json()?
        .ok_or("verification_email: missing JSON payload")?;
    ctx.email.send(&p.to, &p.subject, &p.body).await?;
    job.complete().await?;
    Ok(())
}

/// Purge handle-less accounts abandoned during onboarding. Reuses the existing
/// `UserModule::reap_orphans` policy (design D10); naturally idempotent (a
/// second run finds nothing left to purge).
#[sqlxmq::job("orphan_reap")]
pub async fn orphan_reap(mut job: CurrentJob, ctx: WorkerCtx) -> Result<(), BoxError> {
    let now = chrono::Utc::now().timestamp();
    let purged = ctx.user.reap_orphans(now, ctx.reap_grace_secs).await?;
    if purged > 0 {
        tracing::info!(reaped = purged, "orphan accounts purged (scheduled job)");
    }
    job.complete().await?;
    Ok(())
}

/// Build the job registry with all handlers registered and the shared context set.
pub fn registry(ctx: WorkerCtx) -> JobRegistry {
    let mut registry = JobRegistry::new(&[verification_email, orphan_reap]);
    registry.set_context(ctx);
    registry
}
