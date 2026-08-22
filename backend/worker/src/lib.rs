//! Library surface of `cymbra-worker` — the testable job logic the binary
//! (`main.rs`) wires into the sqlxmq runner. Thin DB I/O over live Postgres, so
//! it is exercised by the `#[ignore]` integration tests in `tests/` (and
//! coverage-excluded like the other I/O glue).

use sqlx::PgPool;

/// Completely erase a deleted user's data across the `user_account` and `auth`
/// schemas in ONE `admin_svc` transaction (change: complete-account-deletion).
///
/// `admin_svc` is the only actor that may write both schemas, which is what
/// makes the erasure **atomic** (all-or-nothing). The steps:
/// 1. Resolve the user's email from the `local` identity (the `local_credentials`
///    primary key). An OIDC-only account has no local identity → `None` → the
///    credentials delete is skipped (OIDC-only-safe).
/// 2. `DELETE` the `auth.local_credentials` row by that email (if any).
/// 3. `DELETE` every `auth.sessions` row for the user (refresh-token erasure).
/// 4. `DELETE` the `user_account.users` row (cascades identities + roles).
/// 5. `DELETE` the user's `music.user_scores` rows and, in the same transaction,
///    enqueue a `purge_score_object` job per stored object so the bytes are
///    erased too (no cross-schema FK, so no DB cascade — the enqueue is what
///    reaches the object store).
/// 6. `DELETE` the user's `music.user_library` saves (saved catalog scores).
///    These reference PUBLIC catalog rows, so no object cleanup is needed.
///
/// Every delete is a no-op when its rows are already gone, so the whole function
/// is **idempotent**: re-running it for an already-purged user commits nothing to
/// delete (and enqueues nothing) and succeeds. Callers (the `purge_user` job
/// handler) get at-least-once delivery, so this idempotency is required.
pub async fn purge_user(admin_pool: &PgPool, user_id: &str) -> anyhow::Result<()> {
    purge_user_with(admin_pool, user_id, None, None).await
}

/// [`purge_user`] with the optional billing seams (changes: add-premium-subscription
/// and swap-store-billing-to-revenuecat D6): an active `web` plan row is cancelled
/// on the merchant-of-record and the account's store-aggregator customer is
/// deleted BEFORE the rows are erased, so a deleted account is never billed again
/// and leaves no trace at the aggregator. Store subscriptions themselves are the
/// user's to cancel on the store. Both calls happen outside the erasure
/// transaction: a provider failure retries the job, never half-erases. When the
/// `plans` schema is not deployed (no `CYMBRA_PLANS_DATABASE_URL`), the plans step
/// is skipped entirely.
pub async fn purge_user_with(
    admin_pool: &PgPool,
    user_id: &str,
    web_cancel: Option<&dyn cymbra_plans::WebSubscriptionCanceller>,
    rc_erase: Option<&dyn cymbra_plans::StoreCustomerEraser>,
) -> anyhow::Result<()> {
    let uid = uuid::Uuid::parse_str(user_id)
        .map_err(|_| anyhow::anyhow!("purge_user: invalid user_id {user_id:?}"))?;

    // The plans schema is optional infrastructure: probe it once.
    let plans_deployed: bool =
        sqlx::query_scalar("SELECT to_regclass('plans.plan_entitlements') IS NOT NULL")
            .fetch_one(admin_pool)
            .await
            .unwrap_or(false);

    if plans_deployed {
        // Cancel active web subscriptions on the provider first (outside the
        // erasure transaction: a provider failure must retry the job, not leave a
        // half-erased account).
        let web_refs: Vec<String> = sqlx::query_scalar(
            "SELECT provider_ref FROM plans.plan_entitlements \
             WHERE user_id = $1 AND source = 'web' AND revoked_at IS NULL \
               AND status NOT IN ('refunded', 'revoked', 'ended') \
               AND (ends_at IS NULL OR ends_at > now())",
        )
        .bind(uid)
        .fetch_all(admin_pool)
        .await?;
        if !web_refs.is_empty() {
            let canceller = web_cancel.ok_or_else(|| {
                anyhow::anyhow!(
                    "purge_user: active web subscription but no web billing provider configured"
                )
            })?;
            for r in &web_refs {
                canceller
                    .cancel(r)
                    .await
                    .map_err(|e| anyhow::anyhow!("cancel web subscription {r}: {e}"))?;
            }
        }
        // Forget the account at the store aggregator (idempotent: a missing
        // customer is a success). Unconfigured aggregator ⇒ nothing to forget.
        if let Some(eraser) = rc_erase {
            eraser
                .delete_customer(user_id)
                .await
                .map_err(|e| anyhow::anyhow!("delete aggregator customer: {e}"))?;
        }
    }

    let mut tx = admin_pool.begin().await?;

    // Resolve the local-identity email (the `local_credentials` PK). OIDC-only
    // accounts have no `local` identity → `None` → skip the credentials delete.
    let email: Option<String> = sqlx::query_scalar(
        "SELECT subject FROM user_account.user_identities \
         WHERE user_id = $1 AND provider = 'local'",
    )
    .bind(uid)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(email) = email {
        sqlx::query("DELETE FROM auth.local_credentials WHERE email = $1")
            .bind(&email)
            .execute(&mut *tx)
            .await?;
    }

    // Refresh-token sessions are keyed by the user_id string (auth.sessions.user_id
    // is TEXT), so bind the raw id, not the parsed UUID.
    sqlx::query("DELETE FROM auth.sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // The account row; the FKs cascade `user_identities` + `user_roles`.
    sqlx::query("DELETE FROM user_account.users WHERE id = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    // The user's contributed scores live in the `music` schema (no cross-schema
    // FK, so no cascade): delete the rows and, in the SAME transaction, enqueue a
    // `purge_score_object` job per stored object so its bytes are removed too. The
    // transactional enqueue makes the row deletes and the cleanup jobs atomic;
    // each object job is independently retryable if the store is transiently down.
    let object_keys: Vec<String> = sqlx::query_scalar(
        "DELETE FROM music.user_scores WHERE owner_id = $1 RETURNING object_key",
    )
    .bind(uid)
    .fetch_all(&mut *tx)
    .await?;
    if !object_keys.is_empty() {
        let spec = cymbra_jobs::registry::spec(cymbra_jobs::registry::PURGE_SCORE_OBJECT)
            .ok_or_else(|| anyhow::anyhow!("purge_score_object spec missing"))?;
        for key in &object_keys {
            let req = cymbra_jobs::EnqueueRequest::for_job(
                &spec,
                &serde_json::json!({ "object_key": key }),
                None,
            )?;
            sqlx::query(
                "SELECT jobs.enqueue($1, $2, $3, $4, $5, \
                 make_interval(secs => $6), make_interval(secs => $7), $8)",
            )
            .bind(&req.name)
            .bind(&req.channel_name)
            .bind(&req.channel_args)
            .bind(req.ordered)
            .bind(req.retries)
            .bind(req.retry_backoff.as_secs() as i32)
            .bind(req.delay.as_secs() as i32)
            .bind(&req.payload_json)
            .execute(&mut *tx)
            .await?;
        }
    }

    // The user's saved-catalog library (change: score-hub-search). These rows
    // reference PUBLIC catalog scores, so there is nothing to erase from the
    // object store — just drop the owner's saves in the same transaction.
    sqlx::query("DELETE FROM music.user_library WHERE owner_id = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    // The user's play sessions (change: add-play-activity-profile). Play stats are
    // profile data keyed by user_id (no cross-schema FK, so no cascade); the whole
    // record incl. the JSONB detail is inline, so no object cleanup is needed —
    // just drop the rows in the same transaction so no play data outlives the
    // account (RGPD erasure, design D7).
    sqlx::query("DELETE FROM music.play_sessions WHERE user_id = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    // The user's practice sessions (change: add-measure-range-practice). Scoreless
    // activity records, same shape of profile data as the play sessions above and
    // keyed the same way — erase them in the same transaction so no practice data
    // outlives the account.
    sqlx::query("DELETE FROM music.practice_sessions WHERE user_id = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    // The user's offline-cache secret (change: add-offline-score-cache). Removing it
    // is the account-deletion kill-switch: any residual encrypted cache files on the
    // user's old devices can no longer be decrypted, and a re-created account would
    // mint a brand-new secret. No object cleanup (the secret is inline); no
    // cross-schema FK, so drop the row explicitly in the same transaction.
    sqlx::query("DELETE FROM music.offline_cache_secrets WHERE user_id = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    // The user's leaderboard personal bests (change: add-play-leaderboards). These
    // are a durable per-(piece, mode) summary keyed by user_id (no cross-schema FK,
    // so no cascade from the account row); erase them in the same transaction so no
    // ranking data outlives the account (RGPD erasure, design D3).
    sqlx::query("DELETE FROM music.leaderboard_bests WHERE user_id = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    // The user's GLOBAL leaderboard data (change: add-global-leaderboard, task
    // 3.3): the per-season bests AND the frozen end-of-season standings. Both are
    // keyed by user_id with no cross-schema FK, so nothing cascades from the
    // account row — erase them here so no global ranking data (live or
    // hall-of-fame) outlives the account (RGPD erasure). The hall of fame is not
    // exempt: a deleted account leaves no trace on a past season's board either.
    for table in ["music.global_season_bests", "music.global_season_snapshots"] {
        sqlx::query(&format!("DELETE FROM {table} WHERE user_id = $1"))
            .bind(uid)
            .execute(&mut *tx)
            .await?;
    }

    // The user's curation-rewards data (change: add-curation-rewards): the append-only
    // points ledger, durable grants (redeemed rewards + earned badges), and the
    // engagement signal — all keyed by user_id (no cross-schema FK, so no cascade);
    // erase them in the same transaction so no rewards data outlives the account. The
    // per-rating settlement state lives on `score_ratings`, which the user does not own
    // (a rating references a public catalog score), so it is not erased here — it is
    // aggregate curation signal, not personal data.
    // The user's practice streak (change: add-practice-streak) goes with them:
    // it is derived profile data keyed by user_id with no cross-schema FK, so
    // nothing cascades from the account row.
    // The user's catalog day-access rows (change: add-score-daily-access-rewards)
    // go too: per-user per-day open/paid marks, keyed by user_id with no
    // cross-schema FK.
    for table in [
        "music.curation_points",
        "music.curation_grants",
        "music.score_engagements",
        "music.course_progress",
        "music.practice_streaks",
        "music.catalog_day_access",
    ] {
        sqlx::query(&format!("DELETE FROM {table} WHERE user_id = $1"))
            .bind(uid)
            .execute(&mut *tx)
            .await?;
    }

    // The user's plan data (change: add-premium-subscription): entitlement rows,
    // beta memberships, code redemptions and billing-event references — all keyed
    // by user_id in the `plans` schema (no cross-schema FK). Identifiers only, but
    // they still name the account: erase them in the same transaction. Campaigns,
    // codes and the admin audit are not the user's (they survive).
    if plans_deployed {
        for table in [
            "plans.access_code_redemptions",
            "plans.beta_memberships",
            "plans.billing_events",
            "plans.plan_entitlements",
        ] {
            sqlx::query(&format!("DELETE FROM {table} WHERE user_id = $1"))
                .bind(uid)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

/// Settle honesty by community consensus (change: add-curation-rewards, task 3.2) —
/// the worker side of the [`cymbra_music::CurationRewardsModule::run_consensus_sweep`]
/// sweep. For each score past the consensus minimum, freezes its aggregate as truth
/// and settles each still-unsettled rating's honesty bonus. Idempotent (guarded by
/// the per-rating + per-score settlement state), so at-least-once redelivery is safe.
/// Returns the number of ratings settled. Runs as `admin_svc` (the only worker actor
/// allowed to write `music`).
pub async fn settle_consensus_honesty(admin_pool: &PgPool) -> anyhow::Result<u64> {
    let repo = std::sync::Arc::new(cymbra_music::PgCurationRewardsRepo::new(admin_pool.clone()));
    let module = cymbra_music::CurationRewardsModule::new(repo);
    module
        .run_consensus_sweep()
        .await
        .map_err(|e| anyhow::anyhow!("consensus honesty settlement: {e}"))
}

/// Roll the global leaderboard over to a new season (change: add-global-
/// leaderboard, task 2.2) — the worker side of
/// [`cymbra_music::snapshot_closed_season`]. Freezes the season that has just
/// closed into the hall of fame (per mode) so later seasons never overwrite it,
/// while the new season accumulates from scratch. Per-piece all-time bests (#6)
/// are untouched. Idempotent (an already-snapshotted season is skipped), so
/// at-least-once redelivery — and the daily cadence on days with no rollover — are
/// both safe. Returns the number of standings newly frozen. Runs as `admin_svc`
/// (the only worker actor allowed to write `music`).
///
/// Needs the [`cymbra_user_port::UserPort`] because the snapshot FREEZES each
/// player's consent to be listed as it stands at the close of the season — the
/// archive's ranking then stays stable when someone later goes private (the age
/// safeguard is still re-checked live on every read).
pub async fn snapshot_global_season(
    admin_pool: &PgPool,
    user: &dyn cymbra_user_port::UserPort,
) -> anyhow::Result<u64> {
    let repo = cymbra_music::PgGlobalLeaderboardRepo::new(admin_pool.clone());
    let cfg = cymbra_music::GlobalConfig::default();
    cymbra_music::snapshot_closed_season(&repo, user, &cfg, chrono::Utc::now())
        .await
        .map_err(|e| anyhow::anyhow!("global season snapshot: {e}"))
}

/// Resolve the practice-streak reminder's batches (change: add-practice-streak,
/// tasks 3.2/3.3) — the worker side of
/// [`cymbra_music::StreakModule::reminder_groups`].
///
/// Returns the at-risk users (a live streak with **no play on their own local
/// day**) grouped by locale + streak length, one group per message the platform
/// will send. Users who already played today never appear, so no consent or
/// schedule gate has to compensate for them.
///
/// Runs as `admin_svc`: the sweep joins `music.practice_streaks` with
/// `user_account.users` for each account's timezone + locale, which the isolated
/// `music_svc` role cannot read. Purely a read — the send itself is the caller's.
pub async fn streak_reminder_groups(
    admin_pool: &PgPool,
    now: chrono::DateTime<chrono::Utc>,
    fallback_tz: &str,
) -> anyhow::Result<Vec<cymbra_music::ReminderGroup>> {
    let module = cymbra_music::StreakModule::new(std::sync::Arc::new(
        cymbra_music::PgStreakRepo::new(admin_pool.clone()),
    ));
    module
        .reminder_groups(now, fallback_tz)
        .await
        .map_err(|e| anyhow::anyhow!("streak reminder sweep: {e}"))
}

/// Prune the heavy per-session play detail past its retention window (change:
/// add-play-activity-profile, design D7). NULLs `session_result` (the full record
/// used for replay) on rows older than `retention_days`, **keeping** the
/// lightweight summary (`overall_sync_pct` + `played_at`) and the per-day
/// aggregate it drives. Idempotent (already-pruned rows are skipped by the
/// `session_result IS NOT NULL` guard) and safe to retry. Returns the number of
/// rows pruned. Runs as `admin_svc` (the only actor allowed to write `music`
/// from the worker).
/// Retention window of `music.catalog_day_access` rows (change:
/// add-score-daily-access-rewards, design D11): only today's rows are ever read.
pub const CATALOG_DAY_ACCESS_RETENTION_DAYS: u64 = 30;

pub async fn prune_play_detail(admin_pool: &PgPool, retention_days: i64) -> anyhow::Result<u64> {
    // The catalog day-access rows (change: add-score-daily-access-rewards) ride
    // the same daily prune: only the current server day is ever read, so anything
    // older than the fixed 30-day window is dead weight. Idempotent.
    let day_access = cymbra_music::PgCatalogDayAccessRepo::new(admin_pool.clone());
    let cutoff =
        chrono::Utc::now().date_naive() - chrono::Days::new(CATALOG_DAY_ACCESS_RETENTION_DAYS);
    match cymbra_music::CatalogDayAccessRepo::prune_before(&day_access, cutoff).await {
        Ok(n) => tracing::info!(pruned = n, "catalog day-access retention prune complete"),
        Err(e) => tracing::warn!(error = %e, "catalog day-access prune failed (next run retries)"),
    }
    let res = sqlx::query(
        "UPDATE music.play_sessions SET session_result = NULL \
         WHERE session_result IS NOT NULL \
         AND created_at < now() - make_interval(days => $1)",
    )
    .bind(retention_days as i32)
    .execute(admin_pool)
    .await?;
    Ok(res.rows_affected())
}
