//! Postgres adapters over the `plans` schema (role `plans_svc`) — thin I/O glue
//! behind the ports; coverage-excluded like the other `pg*.rs` adapters and
//! exercised by the `#[ignore]` integration tests against a real database.

use crate::model::{
    AccessCode, Campaign, CampaignKind, EntitlementRow, EntitlementStatus, EventProvider,
    Membership, MembershipRow, MembershipSource, Redemption, Source,
};
use crate::ports::{
    AccessCodeRepo, AuditEntry, AuditRepo, BillingEventRepo, CampaignRepo, Enrolment,
    EntitlementRepo, EntitlementWrite, MembershipRepo, NewCampaign,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cymbra_platform::{AppError, Result};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

fn internal(ctx: &str, e: impl std::fmt::Display) -> AppError {
    AppError::Internal(anyhow::anyhow!("{ctx}: {e}"))
}

fn parse_uuid(s: &str) -> Result<Uuid> {
    Uuid::parse_str(s).map_err(|_| AppError::InvalidArgument(format!("invalid uuid: {s}")))
}

// ----------------------------------------------------------------- entitlements

fn row_from_pg(r: &PgRow) -> Result<EntitlementRow> {
    let source: String = r.try_get("source").map_err(|e| internal("source", e))?;
    let status: String = r.try_get("status").map_err(|e| internal("status", e))?;
    Ok(EntitlementRow {
        id: r.try_get("id").map_err(|e| internal("id", e))?,
        user_id: r
            .try_get::<Uuid, _>("user_id")
            .map_err(|e| internal("user_id", e))?
            .to_string(),
        source: Source::parse(&source)
            .ok_or_else(|| internal("source", format!("unknown value {source}")))?,
        provider_ref: r
            .try_get("provider_ref")
            .map_err(|e| internal("provider_ref", e))?,
        campaign_id: r
            .try_get("campaign_id")
            .map_err(|e| internal("campaign_id", e))?,
        starts_at: r
            .try_get("starts_at")
            .map_err(|e| internal("starts_at", e))?,
        ends_at: r.try_get("ends_at").map_err(|e| internal("ends_at", e))?,
        status: EntitlementStatus::parse(&status)
            .ok_or_else(|| internal("status", format!("unknown value {status}")))?,
        revoked_at: r
            .try_get("revoked_at")
            .map_err(|e| internal("revoked_at", e))?,
        withdrawn_at: r
            .try_get("withdrawn_at")
            .map_err(|e| internal("withdrawn_at", e))?,
    })
}

const ENTITLEMENT_COLS: &str = "id, user_id, source, provider_ref, campaign_id, starts_at, ends_at, \
                                status, revoked_at, withdrawn_at";

/// Active-row predicate in SQL, mirroring [`crate::core::row_is_active`]:
/// not terminal, not revoked, started, and before `ends_at` (+ grace when in
/// billing retry). `$1` = now, `$2` = grace days.
const ACTIVE_SQL: &str = "status NOT IN ('refunded', 'revoked') AND revoked_at IS NULL \
     AND starts_at <= $1 \
     AND (ends_at IS NULL OR ends_at > $1 \
          OR (status = 'billing_retry' AND ends_at + make_interval(days => $2) > $1))";

#[derive(Clone)]
pub struct PgEntitlementRepo {
    pool: PgPool,
}

impl PgEntitlementRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EntitlementRepo for PgEntitlementRepo {
    async fn list_for_user(&self, user_id: &str) -> Result<Vec<EntitlementRow>> {
        let uid = parse_uuid(user_id)?;
        let rows = sqlx::query(&format!(
            "SELECT {ENTITLEMENT_COLS} FROM plan_entitlements WHERE user_id = $1 ORDER BY starts_at"
        ))
        .bind(uid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| internal("list entitlements", e))?;
        rows.iter().map(row_from_pg).collect()
    }

    async fn get(&self, id: Uuid) -> Result<Option<EntitlementRow>> {
        let row = sqlx::query(&format!(
            "SELECT {ENTITLEMENT_COLS} FROM plan_entitlements WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| internal("get entitlement", e))?;
        row.as_ref().map(row_from_pg).transpose()
    }

    async fn find_by_provider_ref(
        &self,
        source: Source,
        provider_ref: &str,
    ) -> Result<Option<EntitlementRow>> {
        let row = sqlx::query(&format!(
            "SELECT {ENTITLEMENT_COLS} FROM plan_entitlements WHERE source = $1 AND provider_ref = $2"
        ))
        .bind(source.as_str())
        .bind(provider_ref)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| internal("find entitlement by ref", e))?;
        row.as_ref().map(row_from_pg).transpose()
    }

    async fn upsert(&self, w: EntitlementWrite) -> Result<EntitlementRow> {
        let uid = parse_uuid(&w.user_id)?;
        let terminal = w.status.is_terminal();
        // Forward-only `ends_at` unless the new status is terminal (refund /
        // revocation ends the row now). A NULL on either side means open-ended.
        // `withdrawn_at` means "this lapse has already been withdrawn", and one
        // row is reused across subscribe → lapse → resubscribe (one row per
        // `(source, provider_ref)`, a key that is stable per user+product). A
        // write that leaves the row LIVE again therefore clears the stamp —
        // keeping it would make `withdrawal_pending` skip the row on the next
        // lapse, so the offline cache secret would never rotate again. Live =
        // non-terminal ($9) and open-ended or ending in the future, on either
        // the incoming ($10) or the stored side; a repeated `ended` write on an
        // already-lapsed row keeps the stamp, so the sweep does not rotate twice.
        let row = sqlx::query(&format!(
            "INSERT INTO plan_entitlements \
               (id, user_id, source, provider_ref, campaign_id, starts_at, ends_at, status) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (source, provider_ref) DO UPDATE SET \
               status = EXCLUDED.status, \
               ends_at = CASE \
                 WHEN $9 THEN EXCLUDED.ends_at \
                 WHEN plan_entitlements.ends_at IS NULL OR EXCLUDED.ends_at IS NULL THEN NULL \
                 ELSE GREATEST(plan_entitlements.ends_at, EXCLUDED.ends_at) END, \
               campaign_id = COALESCE(EXCLUDED.campaign_id, plan_entitlements.campaign_id), \
               withdrawn_at = CASE \
                 WHEN $9 THEN plan_entitlements.withdrawn_at \
                 WHEN $10 OR plan_entitlements.ends_at IS NULL \
                      OR plan_entitlements.ends_at > now() THEN NULL \
                 ELSE plan_entitlements.withdrawn_at END, \
               updated_at = now() \
             RETURNING {ENTITLEMENT_COLS}"
        ))
        .bind(Uuid::now_v7())
        .bind(uid)
        .bind(w.source.as_str())
        .bind(&w.provider_ref)
        .bind(w.campaign_id)
        .bind(w.starts_at)
        .bind(w.ends_at)
        .bind(w.status.as_str())
        .bind(terminal)
        .bind(w.ends_at.is_none_or(|e| e > Utc::now()))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| internal("upsert entitlement", e))?;
        row_from_pg(&row)
    }

    async fn revoke(&self, id: Uuid, at: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            "UPDATE plan_entitlements SET status = 'revoked', revoked_at = $2, updated_at = now() \
             WHERE id = $1 AND revoked_at IS NULL",
        )
        .bind(id)
        .bind(at)
        .execute(&self.pool)
        .await
        .map_err(|e| internal("revoke entitlement", e))?;
        Ok(())
    }

    async fn mark_withdrawn(&self, ids: &[Uuid], at: DateTime<Utc>) -> Result<u64> {
        let res = sqlx::query(
            "UPDATE plan_entitlements SET withdrawn_at = $2, updated_at = now() \
             WHERE id = ANY($1) AND withdrawn_at IS NULL",
        )
        .bind(ids)
        .bind(at)
        .execute(&self.pool)
        .await
        .map_err(|e| internal("mark withdrawn", e))?;
        Ok(res.rows_affected())
    }

    async fn users_with_unwithdrawn_ended_rows(
        &self,
        before: DateTime<Utc>,
    ) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT DISTINCT user_id FROM plan_entitlements \
             WHERE withdrawn_at IS NULL \
               AND (revoked_at IS NOT NULL OR status IN ('refunded', 'revoked') \
                    OR (ends_at IS NOT NULL AND ends_at <= $1))",
        )
        .bind(before)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| internal("sweep candidates", e))?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<Uuid, _>("user_id").ok())
            .map(|u| u.to_string())
            .collect())
    }

    async fn list_ending_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        sources: &[Source],
    ) -> Result<Vec<EntitlementRow>> {
        let sources: Vec<&str> = sources.iter().map(|s| s.as_str()).collect();
        let rows = sqlx::query(&format!(
            "SELECT {ENTITLEMENT_COLS} FROM plan_entitlements \
             WHERE source = ANY($1) AND ends_at >= $2 AND ends_at < $3 \
               AND status NOT IN ('refunded', 'revoked') AND revoked_at IS NULL"
        ))
        .bind(&sources)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| internal("list ending", e))?;
        rows.iter().map(row_from_pg).collect()
    }

    async fn active_user_ids(
        &self,
        now: DateTime<Utc>,
        grace_days: u32,
        trial_only: bool,
    ) -> Result<Vec<String>> {
        let rows = sqlx::query(&format!(
            "SELECT DISTINCT user_id FROM plan_entitlements \
             WHERE {ACTIVE_SQL} AND ($3 = FALSE OR campaign_id IS NOT NULL)"
        ))
        .bind(now)
        .bind(i32::try_from(grace_days).unwrap_or(i32::MAX))
        .bind(trial_only)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| internal("active user ids", e))?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<Uuid, _>("user_id").ok())
            .map(|u| u.to_string())
            .collect())
    }

    async fn purge_user(&self, user_id: &str) -> Result<()> {
        let uid = parse_uuid(user_id)?;
        sqlx::query("DELETE FROM plan_entitlements WHERE user_id = $1")
            .bind(uid)
            .execute(&self.pool)
            .await
            .map_err(|e| internal("purge entitlements", e))?;
        Ok(())
    }
}

// -------------------------------------------------------------------- campaigns

fn campaign_from_pg(r: &PgRow) -> Result<Campaign> {
    let kind: String = r.try_get("kind").map_err(|e| internal("kind", e))?;
    let duration: Option<i32> = r
        .try_get("duration_days")
        .map_err(|e| internal("duration_days", e))?;
    let kind = match kind.as_str() {
        "premium_trial" => CampaignKind::PremiumTrial {
            duration_days: duration
                .and_then(|d| u32::try_from(d).ok())
                .unwrap_or(CampaignKind::DEFAULT_TRIAL_DAYS),
        },
        "feature" => CampaignKind::Feature,
        other => return Err(internal("kind", format!("unknown value {other}"))),
    };
    Ok(Campaign {
        id: r.try_get("id").map_err(|e| internal("id", e))?,
        key: r.try_get("key").map_err(|e| internal("key", e))?,
        name: r.try_get("name").map_err(|e| internal("name", e))?,
        kind,
        enrollment_closes_at: r
            .try_get("enrollment_closes_at")
            .map_err(|e| internal("enrollment_closes_at", e))?,
        closed_at: r
            .try_get("closed_at")
            .map_err(|e| internal("closed_at", e))?,
        created_by: r
            .try_get::<Uuid, _>("created_by")
            .map_err(|e| internal("created_by", e))?
            .to_string(),
        created_at: r
            .try_get("created_at")
            .map_err(|e| internal("created_at", e))?,
    })
}

const CAMPAIGN_COLS: &str =
    "id, key, name, kind, duration_days, enrollment_closes_at, closed_at, created_by, created_at";

#[derive(Clone)]
pub struct PgCampaignRepo {
    pool: PgPool,
}

impl PgCampaignRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CampaignRepo for PgCampaignRepo {
    async fn create(&self, new: NewCampaign) -> Result<Campaign> {
        let (kind, duration) = match new.kind {
            CampaignKind::PremiumTrial { duration_days } => (
                "premium_trial",
                Some(i32::try_from(duration_days).unwrap_or(i32::MAX)),
            ),
            CampaignKind::Feature => ("feature", None),
        };
        let created_by = parse_uuid(&new.created_by)?;
        let row = sqlx::query(&format!(
            "INSERT INTO beta_campaigns (id, key, name, kind, duration_days, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING {CAMPAIGN_COLS}"
        ))
        .bind(Uuid::now_v7())
        .bind(&new.key)
        .bind(&new.name)
        .bind(kind)
        .bind(duration)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(d) if d.is_unique_violation() => {
                AppError::AlreadyExists(format!("campaign key {} exists", new.key))
            }
            _ => internal("create campaign", e),
        })?;
        campaign_from_pg(&row)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Campaign>> {
        let row = sqlx::query(&format!(
            "SELECT {CAMPAIGN_COLS} FROM beta_campaigns WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| internal("get campaign", e))?;
        row.as_ref().map(campaign_from_pg).transpose()
    }

    async fn get_by_key(&self, key: &str) -> Result<Option<Campaign>> {
        let row = sqlx::query(&format!(
            "SELECT {CAMPAIGN_COLS} FROM beta_campaigns WHERE key = $1"
        ))
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| internal("get campaign by key", e))?;
        row.as_ref().map(campaign_from_pg).transpose()
    }

    async fn list(&self, include_closed: bool) -> Result<Vec<Campaign>> {
        let rows = sqlx::query(&format!(
            "SELECT {CAMPAIGN_COLS} FROM beta_campaigns \
             WHERE ($1 OR closed_at IS NULL) ORDER BY created_at DESC"
        ))
        .bind(include_closed)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| internal("list campaigns", e))?;
        rows.iter().map(campaign_from_pg).collect()
    }

    async fn close_enrollment(&self, id: Uuid, at: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            "UPDATE beta_campaigns SET enrollment_closes_at = $2 \
             WHERE id = $1 AND enrollment_closes_at IS NULL",
        )
        .bind(id)
        .bind(at)
        .execute(&self.pool)
        .await
        .map_err(|e| internal("close enrollment", e))?;
        Ok(())
    }

    async fn reopen(&self, id: Uuid) -> Result<()> {
        // Only `closed_at`: the enrolment deadline is its own decision, and its
        // own inverse below.
        sqlx::query("UPDATE beta_campaigns SET closed_at = NULL WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| internal("reopen campaign", e))?;
        Ok(())
    }

    async fn reopen_enrollment(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE beta_campaigns SET enrollment_closes_at = NULL WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| internal("reopen enrollment", e))?;
        Ok(())
    }

    async fn close(&self, id: Uuid, at: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            "UPDATE beta_campaigns SET closed_at = $2, \
               enrollment_closes_at = COALESCE(enrollment_closes_at, $2) \
             WHERE id = $1 AND closed_at IS NULL",
        )
        .bind(id)
        .bind(at)
        .execute(&self.pool)
        .await
        .map_err(|e| internal("close campaign", e))?;
        Ok(())
    }
}

// ------------------------------------------------------------------ memberships

fn membership_from_pg(r: &PgRow) -> Result<Membership> {
    let source: String = r.try_get("m_source").map_err(|e| internal("m_source", e))?;
    Ok(Membership {
        row: MembershipRow {
            campaign_id: r.try_get("id").map_err(|e| internal("id", e))?,
            user_id: r
                .try_get::<Uuid, _>("m_user_id")
                .map_err(|e| internal("m_user_id", e))?
                .to_string(),
            enrolled_at: r
                .try_get("m_enrolled_at")
                .map_err(|e| internal("m_enrolled_at", e))?,
            ends_at: r
                .try_get("m_ends_at")
                .map_err(|e| internal("m_ends_at", e))?,
            revoked_at: r
                .try_get("m_revoked_at")
                .map_err(|e| internal("m_revoked_at", e))?,
            source: MembershipSource::parse(&source)
                .ok_or_else(|| internal("m_source", format!("unknown value {source}")))?,
        },
        campaign: campaign_from_pg(r)?,
    })
}

const MEMBERSHIP_JOIN_SQL: &str = "SELECT c.id, c.key, c.name, c.kind, c.duration_days, \
       c.enrollment_closes_at, c.closed_at, c.created_by, c.created_at, \
       m.user_id AS m_user_id, m.enrolled_at AS m_enrolled_at, m.ends_at AS m_ends_at, \
       m.revoked_at AS m_revoked_at, m.source AS m_source \
     FROM beta_memberships m JOIN beta_campaigns c ON c.id = m.campaign_id";

#[derive(Clone)]
pub struct PgMembershipRepo {
    pool: PgPool,
}

impl PgMembershipRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MembershipRepo for PgMembershipRepo {
    async fn list_for_user(&self, user_id: &str) -> Result<Vec<Membership>> {
        let uid = parse_uuid(user_id)?;
        let rows = sqlx::query(&format!(
            "{MEMBERSHIP_JOIN_SQL} WHERE m.user_id = $1 ORDER BY m.enrolled_at"
        ))
        .bind(uid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| internal("list memberships", e))?;
        rows.iter().map(membership_from_pg).collect()
    }

    async fn list_members(&self, campaign_id: Uuid) -> Result<Vec<MembershipRow>> {
        let rows = sqlx::query(&format!(
            "{MEMBERSHIP_JOIN_SQL} WHERE m.campaign_id = $1 ORDER BY m.enrolled_at"
        ))
        .bind(campaign_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| internal("list members", e))?;
        rows.iter()
            .map(|r| membership_from_pg(r).map(|m| m.row))
            .collect()
    }

    async fn active_member_ids(
        &self,
        campaign_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT m.user_id FROM beta_memberships m JOIN beta_campaigns c ON c.id = m.campaign_id \
             WHERE m.campaign_id = $1 AND m.revoked_at IS NULL \
               AND (m.ends_at IS NULL OR m.ends_at > $2) \
               AND (c.closed_at IS NULL OR c.closed_at > $2)",
        )
        .bind(campaign_id)
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| internal("active member ids", e))?;
        Ok(rows
            .iter()
            .filter_map(|r| r.try_get::<Uuid, _>("user_id").ok())
            .map(|u| u.to_string())
            .collect())
    }

    async fn enrol(&self, e: Enrolment) -> Result<()> {
        let uid = parse_uuid(&e.user_id)?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| internal("begin", err))?;

        if let Some(code_id) = e.code_id {
            // Consume one use; zero rows ⇒ spent or revoked meanwhile.
            let res = sqlx::query(
                "UPDATE access_codes SET uses = uses + 1 \
                 WHERE id = $1 AND revoked_at IS NULL AND uses < max_uses",
            )
            .bind(code_id)
            .execute(&mut *tx)
            .await
            .map_err(|err| internal("consume code", err))?;
            if res.rows_affected() == 0 {
                return Err(AppError::NotFound("code invalid or already used".into()));
            }
            sqlx::query(
                "INSERT INTO access_code_redemptions (code_id, user_id, redeemed_at) VALUES ($1, $2, $3)",
            )
            .bind(code_id)
            .bind(uid)
            .bind(e.enrolled_at)
            .execute(&mut *tx)
            .await
            .map_err(|err| internal("record redemption", err))?;
        }

        sqlx::query(
            "INSERT INTO beta_memberships (campaign_id, user_id, enrolled_at, ends_at, source) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(e.campaign_id)
        .bind(uid)
        .bind(e.enrolled_at)
        .bind(e.ends_at)
        .bind(e.source.as_str())
        .execute(&mut *tx)
        .await
        .map_err(|err| match &err {
            sqlx::Error::Database(d) if d.is_unique_violation() => {
                AppError::AlreadyExists("already_member".into())
            }
            _ => internal("insert membership", err),
        })?;

        if let Some(w) = e.trial_row {
            sqlx::query(
                "INSERT INTO plan_entitlements \
                   (id, user_id, source, provider_ref, campaign_id, starts_at, ends_at, status) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
                 ON CONFLICT (source, provider_ref) DO UPDATE SET \
                   ends_at = EXCLUDED.ends_at, status = EXCLUDED.status, \
                   withdrawn_at = NULL, updated_at = now()",
            )
            .bind(Uuid::now_v7())
            .bind(uid)
            .bind(w.source.as_str())
            .bind(&w.provider_ref)
            .bind(w.campaign_id)
            .bind(w.starts_at)
            .bind(w.ends_at)
            .bind(w.status.as_str())
            .execute(&mut *tx)
            .await
            .map_err(|err| internal("insert trial row", err))?;
        }

        tx.commit().await.map_err(|err| internal("commit", err))
    }

    async fn revoke(&self, campaign_id: Uuid, user_id: &str, at: DateTime<Utc>) -> Result<()> {
        let uid = parse_uuid(user_id)?;
        sqlx::query(
            "UPDATE beta_memberships SET revoked_at = $3 \
             WHERE campaign_id = $1 AND user_id = $2 AND revoked_at IS NULL",
        )
        .bind(campaign_id)
        .bind(uid)
        .bind(at)
        .execute(&self.pool)
        .await
        .map_err(|e| internal("revoke membership", e))?;
        Ok(())
    }

    async fn purge_user(&self, user_id: &str) -> Result<()> {
        let uid = parse_uuid(user_id)?;
        sqlx::query("DELETE FROM beta_memberships WHERE user_id = $1")
            .bind(uid)
            .execute(&self.pool)
            .await
            .map_err(|e| internal("purge memberships", e))?;
        Ok(())
    }
}

// ----------------------------------------------------------------- access codes

fn code_from_pg(r: &PgRow) -> Result<AccessCode> {
    Ok(AccessCode {
        id: r.try_get("id").map_err(|e| internal("id", e))?,
        campaign_id: r
            .try_get("campaign_id")
            .map_err(|e| internal("campaign_id", e))?,
        issued_by: r
            .try_get("issued_by")
            .map_err(|e| internal("issued_by", e))?,
        issued_to_hint: r
            .try_get("issued_to_hint")
            .map_err(|e| internal("issued_to_hint", e))?,
        max_uses: u32::try_from(
            r.try_get::<i32, _>("max_uses")
                .map_err(|e| internal("max_uses", e))?,
        )
        .unwrap_or(1),
        uses: u32::try_from(
            r.try_get::<i32, _>("uses")
                .map_err(|e| internal("uses", e))?,
        )
        .unwrap_or(0),
        revoked_at: r
            .try_get("revoked_at")
            .map_err(|e| internal("revoked_at", e))?,
        created_at: r
            .try_get("created_at")
            .map_err(|e| internal("created_at", e))?,
    })
}

const CODE_COLS: &str =
    "id, campaign_id, issued_by, issued_to_hint, max_uses, uses, revoked_at, created_at";

#[derive(Clone)]
pub struct PgAccessCodeRepo {
    pool: PgPool,
}

impl PgAccessCodeRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccessCodeRepo for PgAccessCodeRepo {
    async fn insert(
        &self,
        campaign_id: Uuid,
        code_hash: &str,
        issued_by: &str,
        issued_to_hint: Option<String>,
        max_uses: u32,
    ) -> Result<AccessCode> {
        let row = sqlx::query(&format!(
            "INSERT INTO access_codes (id, campaign_id, code_hash, issued_by, issued_to_hint, max_uses) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING {CODE_COLS}"
        ))
        .bind(Uuid::now_v7())
        .bind(campaign_id)
        .bind(code_hash)
        .bind(issued_by)
        .bind(issued_to_hint)
        .bind(i32::try_from(max_uses).unwrap_or(1))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| internal("insert code", e))?;
        code_from_pg(&row)
    }

    async fn find_by_hash(&self, code_hash: &str) -> Result<Option<AccessCode>> {
        let row = sqlx::query(&format!(
            "SELECT {CODE_COLS} FROM access_codes WHERE code_hash = $1"
        ))
        .bind(code_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| internal("find code", e))?;
        row.as_ref().map(code_from_pg).transpose()
    }

    async fn revoke(&self, id: Uuid, at: DateTime<Utc>) -> Result<()> {
        sqlx::query("UPDATE access_codes SET revoked_at = $2 WHERE id = $1 AND revoked_at IS NULL")
            .bind(id)
            .bind(at)
            .execute(&self.pool)
            .await
            .map_err(|e| internal("revoke code", e))?;
        Ok(())
    }

    async fn revoke_campaign(&self, campaign_id: Uuid, at: DateTime<Utc>) -> Result<u64> {
        // Only codes that could still be redeemed: a spent code is already inert
        // and stays a redemption record, so it is neither touched nor counted.
        let res = sqlx::query(
            "UPDATE access_codes SET revoked_at = $2 \
             WHERE campaign_id = $1 AND revoked_at IS NULL AND uses < max_uses",
        )
        .bind(campaign_id)
        .bind(at)
        .execute(&self.pool)
        .await
        .map_err(|e| internal("revoke campaign codes", e))?;
        Ok(res.rows_affected())
    }

    async fn redemptions(&self, campaign_id: Uuid) -> Result<Vec<Redemption>> {
        let rows = sqlx::query(
            "SELECT r.code_id, r.user_id, r.redeemed_at FROM access_code_redemptions r \
             JOIN access_codes c ON c.id = r.code_id WHERE c.campaign_id = $1 ORDER BY r.redeemed_at",
        )
        .bind(campaign_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| internal("list redemptions", e))?;
        rows.iter()
            .map(|r| {
                Ok(Redemption {
                    code_id: r.try_get("code_id").map_err(|e| internal("code_id", e))?,
                    user_id: r
                        .try_get::<Uuid, _>("user_id")
                        .map_err(|e| internal("user_id", e))?
                        .to_string(),
                    redeemed_at: r
                        .try_get("redeemed_at")
                        .map_err(|e| internal("redeemed_at", e))?,
                })
            })
            .collect()
    }

    async fn purge_user(&self, user_id: &str) -> Result<()> {
        let uid = parse_uuid(user_id)?;
        sqlx::query("DELETE FROM access_code_redemptions WHERE user_id = $1")
            .bind(uid)
            .execute(&self.pool)
            .await
            .map_err(|e| internal("purge redemptions", e))?;
        Ok(())
    }
}

// --------------------------------------------------------------- billing events

#[derive(Clone)]
pub struct PgBillingEventRepo {
    pool: PgPool,
}

impl PgBillingEventRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BillingEventRepo for PgBillingEventRepo {
    async fn record_if_new(
        &self,
        provider: EventProvider,
        event_id: &str,
        user_id: Option<String>,
        payload_ref: &str,
    ) -> Result<bool> {
        let uid = user_id.as_deref().map(parse_uuid).transpose()?;
        let res = sqlx::query(
            "INSERT INTO billing_events (provider, event_id, user_id, payload_ref) \
             VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
        )
        .bind(provider.as_str())
        .bind(event_id)
        .bind(uid)
        .bind(payload_ref)
        .execute(&self.pool)
        .await
        .map_err(|e| internal("record billing event", e))?;
        Ok(res.rows_affected() == 1)
    }

    async fn mark_applied(&self, provider: EventProvider, event_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE billing_events SET applied_at = now() WHERE provider = $1 AND event_id = $2",
        )
        .bind(provider.as_str())
        .bind(event_id)
        .execute(&self.pool)
        .await
        .map_err(|e| internal("mark billing event applied", e))?;
        Ok(())
    }

    async fn purge_user(&self, user_id: &str) -> Result<()> {
        let uid = parse_uuid(user_id)?;
        sqlx::query("DELETE FROM billing_events WHERE user_id = $1")
            .bind(uid)
            .execute(&self.pool)
            .await
            .map_err(|e| internal("purge billing events", e))?;
        Ok(())
    }
}

// ------------------------------------------------------------------------ audit

#[derive(Clone)]
pub struct PgAuditRepo {
    pool: PgPool,
}

impl PgAuditRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepo for PgAuditRepo {
    async fn record(&self, e: AuditEntry) -> Result<()> {
        let target_user = e.target_user.as_deref().map(parse_uuid).transpose()?;
        sqlx::query(
            "INSERT INTO plan_admin_audit (actor, action, target_user, target_ref, reason) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&e.actor)
        .bind(&e.action)
        .bind(target_user)
        .bind(&e.target_ref)
        .bind(&e.reason)
        .execute(&self.pool)
        .await
        .map_err(|err| internal("audit", err))?;
        Ok(())
    }
}

/// Connect a `plans_svc` pool (search_path pinned by the role).
pub async fn connect(url: &str, max: u32) -> Result<PgPool> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max)
        .connect(url)
        .await
        .map_err(|e| internal("connect plans db", e))
}
