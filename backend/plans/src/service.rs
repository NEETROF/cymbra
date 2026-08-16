//! `PlanService` — the use cases over the ports: snapshot (with the on-demand
//! withdrawal check), enrolment by code or by admin, grants/revocations,
//! campaigns and codes, the sweep, and the provider-event application entry.
//!
//! Every decision is delegated to [`crate::core`]; this layer only sequences
//! reads and writes and maps refusals to `AppError`s.

use crate::codes::{generate_code, hash_code};
use crate::core;
use crate::model::{
    AccessCode, Campaign, CampaignKind, EntitlementRow, EntitlementStatus, Membership,
    MembershipRow, MembershipSource, PlanSnapshot, Redemption, Source, Unlock,
};
use crate::ports::{
    AccessCodeIssuer, AccessCodeRepo, AuditEntry, AuditRepo, BillingEventRepo, CacheSecretRotator,
    CampaignRepo, Clock, Enrolment, EntitlementRepo, EntitlementWrite, MembershipRepo, MintedCode,
    NewCampaign, PlanConfig, PlanConfigSource, PlanSource,
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use cymbra_platform::{AppError, Result};
use std::sync::Arc;
use uuid::Uuid;

/// Everything the service needs, injected (mocks in tests).
pub struct PlanDeps {
    pub entitlements: Arc<dyn EntitlementRepo>,
    pub campaigns: Arc<dyn CampaignRepo>,
    pub memberships: Arc<dyn MembershipRepo>,
    pub codes: Arc<dyn AccessCodeRepo>,
    pub billing_events: Arc<dyn BillingEventRepo>,
    pub audit: Arc<dyn AuditRepo>,
    pub config: Arc<dyn PlanConfigSource>,
    pub clock: Arc<dyn Clock>,
    /// `None` = no offline cache to withdraw (tests, or before the seam is wired).
    pub rotator: Option<Arc<dyn CacheSecretRotator>>,
}

/// Outcome of a successful redemption / enrolment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrolOutcome {
    pub campaign_key: String,
    pub campaign_name: String,
    pub kind: CampaignKind,
    /// End of the membership (trials) — `None` for feature betas.
    pub ends_at: Option<DateTime<Utc>>,
}

/// Everything the console shows for one account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountPlan {
    pub snapshot: PlanSnapshot,
    pub rows: Vec<EntitlementRow>,
    pub memberships: Vec<Membership>,
}

/// Directory filter (`ListAccountIdsByPlan`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanFilter {
    Any,
    Premium,
    Trial,
}

pub struct PlanService {
    d: PlanDeps,
}

/// Neutral refusal for unknown / revoked / spent codes (spec: never reveal
/// whether a code exists).
const CODE_REFUSED: &str = "code invalid or already used";

impl PlanService {
    pub fn new(deps: PlanDeps) -> Self {
        Self { d: deps }
    }

    fn cfg(&self) -> PlanConfig {
        self.d.config.plan_config()
    }

    fn grace(&self) -> Duration {
        Duration::days(i64::from(self.cfg().grace_days))
    }

    fn now(&self) -> DateTime<Utc> {
        self.d.clock.now()
    }

    // ------------------------------------------------------------------ reads

    /// The app-facing snapshot; runs the on-demand withdrawal check first
    /// (design D13). Kill-switch off ⇒ free, no memberships, no side effect.
    pub async fn snapshot(&self, user_id: &str) -> Result<PlanSnapshot> {
        if !self.cfg().enabled {
            return Ok(PlanSnapshot::free());
        }
        let now = self.now();
        let grace = self.grace();
        let rows = self.d.entitlements.list_for_user(user_id).await?;
        self.withdraw_rows_if_lapsed(user_id, &rows, now, grace)
            .await?;
        let memberships = self.d.memberships.list_for_user(user_id).await?;
        Ok(core::snapshot(&rows, &memberships, now, grace))
    }

    /// `true` iff the effective plan grants `unlock` (the seam music uses).
    pub async fn grants(&self, user_id: &str, unlock: Unlock) -> Result<bool> {
        Ok(self.snapshot(user_id).await?.grants(unlock))
    }

    /// Console lookup: raw rows + memberships + the computed snapshot.
    pub async fn account_plan(&self, user_id: &str) -> Result<AccountPlan> {
        let snapshot = self.snapshot(user_id).await?;
        let rows = self.d.entitlements.list_for_user(user_id).await?;
        let memberships = self.d.memberships.list_for_user(user_id).await?;
        Ok(AccountPlan {
            snapshot,
            rows,
            memberships,
        })
    }

    /// Directory filter: account ids matching `plan` and/or a beta campaign.
    pub async fn account_ids(
        &self,
        plan: PlanFilter,
        beta_campaign_key: Option<&str>,
    ) -> Result<Vec<String>> {
        let now = self.now();
        let cfg = self.cfg();
        let mut ids: Option<Vec<String>> = match plan {
            PlanFilter::Any => None,
            PlanFilter::Premium => Some(
                self.d
                    .entitlements
                    .active_user_ids(now, cfg.grace_days, false)
                    .await?,
            ),
            PlanFilter::Trial => Some(
                self.d
                    .entitlements
                    .active_user_ids(now, cfg.grace_days, true)
                    .await?,
            ),
        };
        if let Some(key) = beta_campaign_key {
            let campaign = self.campaign_by_key(key).await?;
            let members = self
                .d
                .memberships
                .active_member_ids(campaign.id, now)
                .await?;
            ids = Some(match ids {
                None => members,
                Some(prev) => prev.into_iter().filter(|u| members.contains(u)).collect(),
            });
        }
        Ok(ids.unwrap_or_default())
    }

    // ------------------------------------------------------------- enrolment

    /// Redeem `code_clear` for `user_id` = enrol in its campaign (spec
    /// `music-access-codes`). Refusals are neutral for unknown/revoked/spent.
    pub async fn redeem(&self, user_id: &str, code_clear: &str) -> Result<EnrolOutcome> {
        let hash = hash_code(code_clear);
        let code = self
            .d
            .codes
            .find_by_hash(&hash)
            .await?
            .filter(AccessCode::is_redeemable)
            .ok_or_else(|| AppError::NotFound(CODE_REFUSED.into()))?;
        let campaign = self
            .d
            .campaigns
            .get(code.campaign_id)
            .await?
            .ok_or_else(|| AppError::NotFound(CODE_REFUSED.into()))?;
        self.enrol(
            user_id,
            &campaign,
            MembershipSource::Code,
            Some(code.id),
            Some(code.id.to_string()),
        )
        .await
    }

    /// Nominative enrolment from the console (no code), audited.
    pub async fn enrol_by_admin(
        &self,
        user_id: &str,
        campaign_key: &str,
        actor: &str,
        reason: &str,
    ) -> Result<EnrolOutcome> {
        let campaign = self.campaign_by_key(campaign_key).await?;
        let out = self
            .enrol(user_id, &campaign, MembershipSource::Admin, None, None)
            .await?;
        self.audit(actor, "enrol", Some(user_id), Some(campaign_key), reason)
            .await?;
        Ok(out)
    }

    async fn enrol(
        &self,
        user_id: &str,
        campaign: &Campaign,
        source: MembershipSource,
        code_id: Option<Uuid>,
        provider_ref: Option<String>,
    ) -> Result<EnrolOutcome> {
        let now = self.now();
        let existing = self.d.memberships.list_for_user(user_id).await?;
        core::can_enrol(campaign, &existing, now).map_err(|r| match r {
            core::EnrolRefusal::CampaignClosed => {
                AppError::FailedPrecondition("campaign_closed".into())
            }
            core::EnrolRefusal::AlreadyMember => AppError::AlreadyExists("already_member".into()),
            core::EnrolRefusal::TrialActive => {
                AppError::FailedPrecondition("trial_already_active".into())
            }
        })?;
        let ends_at = core::membership_end(campaign.kind, now);
        let trial_row = campaign.kind.is_trial().then(|| EntitlementWrite {
            user_id: user_id.to_string(),
            source: match source {
                MembershipSource::Code => Source::Code,
                MembershipSource::Admin => Source::Admin,
            },
            provider_ref: provider_ref.unwrap_or_else(|| format!("{}:{user_id}", campaign.id)),
            campaign_id: Some(campaign.id),
            starts_at: now,
            ends_at,
            status: EntitlementStatus::Active,
        });
        self.d
            .memberships
            .enrol(Enrolment {
                campaign_id: campaign.id,
                user_id: user_id.to_string(),
                enrolled_at: now,
                ends_at,
                source,
                code_id,
                trial_row,
            })
            .await?;
        Ok(EnrolOutcome {
            campaign_key: campaign.key.clone(),
            campaign_name: campaign.name.clone(),
            kind: campaign.kind,
            ends_at,
        })
    }

    // ------------------------------------------------- grants / revocations

    /// Admin comp: a `premium` row from `now` to `ends_at`. An open-ended grant
    /// requires `confirm_open_ended`.
    pub async fn grant_premium(
        &self,
        user_id: &str,
        ends_at: Option<DateTime<Utc>>,
        confirm_open_ended: bool,
        actor: &str,
        reason: &str,
    ) -> Result<EntitlementRow> {
        if ends_at.is_none() && !confirm_open_ended {
            return Err(AppError::InvalidArgument(
                "open-ended grant requires explicit confirmation".into(),
            ));
        }
        let now = self.now();
        if ends_at.is_some_and(|e| e <= now) {
            return Err(AppError::InvalidArgument(
                "ends_at must be in the future".into(),
            ));
        }
        let row = self
            .d
            .entitlements
            .upsert(EntitlementWrite {
                user_id: user_id.to_string(),
                source: Source::Admin,
                provider_ref: Uuid::new_v4().to_string(),
                campaign_id: None,
                starts_at: now,
                ends_at,
                status: EntitlementStatus::Active,
            })
            .await?;
        self.audit(
            actor,
            "grant_premium",
            Some(user_id),
            Some(&row.id.to_string()),
            reason,
        )
        .await?;
        Ok(row)
    }

    /// Revoke a `code`/`admin` row; store/web rows are read-only here.
    pub async fn revoke_entitlement(&self, id: Uuid, actor: &str, reason: &str) -> Result<()> {
        let row = self
            .d
            .entitlements
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound("entitlement not found".into()))?;
        if row.source.is_paid_channel() {
            return Err(AppError::FailedPrecondition(
                "store and web rows end on the provider's side".into(),
            ));
        }
        let now = self.now();
        self.d.entitlements.revoke(id, now).await?;
        self.audit(
            actor,
            "revoke_entitlement",
            Some(&row.user_id),
            Some(&id.to_string()),
            reason,
        )
        .await?;
        self.withdraw_if_lapsed(&row.user_id).await?;
        Ok(())
    }

    /// Revoke a membership (and its trial row, if any).
    pub async fn revoke_membership(
        &self,
        campaign_key: &str,
        user_id: &str,
        actor: &str,
        reason: &str,
    ) -> Result<()> {
        let campaign = self.campaign_by_key(campaign_key).await?;
        let now = self.now();
        self.d.memberships.revoke(campaign.id, user_id, now).await?;
        let rows = self.d.entitlements.list_for_user(user_id).await?;
        for r in rows
            .iter()
            .filter(|r| r.campaign_id == Some(campaign.id) && r.revoked_at.is_none())
        {
            self.d.entitlements.revoke(r.id, now).await?;
        }
        self.audit(
            actor,
            "revoke_membership",
            Some(user_id),
            Some(campaign_key),
            reason,
        )
        .await?;
        self.withdraw_if_lapsed(user_id).await?;
        Ok(())
    }

    // ------------------------------------------------------------ campaigns

    pub async fn create_campaign(
        &self,
        key: &str,
        name: &str,
        kind: CampaignKind,
        actor: &str,
    ) -> Result<Campaign> {
        let key = key.trim().to_ascii_lowercase();
        if key.is_empty()
            || !key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(AppError::InvalidArgument(
                "campaign key must be [a-z0-9-]".into(),
            ));
        }
        if let CampaignKind::PremiumTrial { duration_days } = kind
            && duration_days == 0
        {
            return Err(AppError::InvalidArgument(
                "duration_days must be > 0".into(),
            ));
        }
        let c = self
            .d
            .campaigns
            .create(NewCampaign {
                key: key.clone(),
                name: name.trim().to_string(),
                kind,
                created_by: actor.to_string(),
            })
            .await?;
        self.audit(actor, "create_campaign", None, Some(&key), "")
            .await?;
        Ok(c)
    }

    pub async fn list_campaigns(&self, include_closed: bool) -> Result<Vec<Campaign>> {
        self.d.campaigns.list(include_closed).await
    }

    pub async fn campaign_by_key(&self, key: &str) -> Result<Campaign> {
        self.d
            .campaigns
            .get_by_key(key)
            .await?
            .ok_or_else(|| AppError::NotFound("campaign not found".into()))
    }

    /// Stop new enrolments; running trials keep their end.
    pub async fn close_enrollment(&self, key: &str, actor: &str) -> Result<()> {
        let c = self.campaign_by_key(key).await?;
        self.d.campaigns.close_enrollment(c.id, self.now()).await?;
        self.audit(actor, "close_enrollment", None, Some(key), "")
            .await
    }

    /// Close the campaign: every membership ends now (feature betas' lever).
    pub async fn close_campaign(&self, key: &str, actor: &str) -> Result<()> {
        let c = self.campaign_by_key(key).await?;
        self.d.campaigns.close(c.id, self.now()).await?;
        self.audit(actor, "close_campaign", None, Some(key), "")
            .await
    }

    pub async fn list_members(&self, key: &str) -> Result<Vec<MembershipRow>> {
        let c = self.campaign_by_key(key).await?;
        self.d.memberships.list_members(c.id).await
    }

    pub async fn list_redemptions(&self, key: &str) -> Result<Vec<Redemption>> {
        let c = self.campaign_by_key(key).await?;
        self.d.codes.redemptions(c.id).await
    }

    // ---------------------------------------------------------------- codes

    /// Mint `count` single-use codes; the clear texts are returned once.
    pub async fn mint_codes(
        &self,
        campaign_key: &str,
        count: u32,
        issued_by: &str,
        issued_to_hint: Option<String>,
    ) -> Result<Vec<String>> {
        if count == 0 || count > 500 {
            return Err(AppError::InvalidArgument("count must be 1..=500".into()));
        }
        let campaign = self.campaign_by_key(campaign_key).await?;
        if !campaign.accepts_enrolment(self.now()) {
            return Err(AppError::FailedPrecondition("campaign_closed".into()));
        }
        let mut out = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let clear = generate_code();
            self.d
                .codes
                .insert(
                    campaign.id,
                    &hash_code(&clear),
                    issued_by,
                    issued_to_hint.clone(),
                    1,
                )
                .await?;
            out.push(clear);
        }
        self.audit(
            issued_by,
            "mint_codes",
            None,
            Some(campaign_key),
            &format!("count={count}"),
        )
        .await?;
        Ok(out)
    }

    pub async fn revoke_code(&self, id: Uuid, actor: &str) -> Result<()> {
        self.d.codes.revoke(id, self.now()).await?;
        self.audit(actor, "revoke_code", None, Some(&id.to_string()), "")
            .await
    }

    pub async fn revoke_campaign_codes(&self, campaign_key: &str, actor: &str) -> Result<u64> {
        let c = self.campaign_by_key(campaign_key).await?;
        let n = self.d.codes.revoke_campaign(c.id, self.now()).await?;
        self.audit(
            actor,
            "revoke_campaign_codes",
            None,
            Some(campaign_key),
            &format!("count={n}"),
        )
        .await?;
        Ok(n)
    }

    // ------------------------------------------------- provider events (D3)

    /// Apply a provider-derived write (Apple/Google/web adapters call this
    /// after their own verification + idempotency check).
    pub async fn apply(&self, write: EntitlementWrite) -> Result<EntitlementRow> {
        let user_id = write.user_id.clone();
        let row = self.d.entitlements.upsert(write).await?;
        // A refund/revocation may have just lapsed the user.
        self.withdraw_if_lapsed(&user_id).await?;
        Ok(row)
    }

    /// Idempotency gate for provider notifications.
    pub async fn record_event(
        &self,
        provider: Source,
        event_id: &str,
        user_id: Option<&str>,
        payload_ref: &str,
    ) -> Result<bool> {
        self.d
            .billing_events
            .record_if_new(provider, event_id, user_id.map(str::to_string), payload_ref)
            .await
    }

    pub async fn mark_event_applied(&self, provider: Source, event_id: &str) -> Result<()> {
        self.d.billing_events.mark_applied(provider, event_id).await
    }

    // ------------------------------------------------------- withdrawal (D13)

    /// Rotate + stamp once if the user has lapsed past grace.
    pub async fn withdraw_if_lapsed(&self, user_id: &str) -> Result<bool> {
        if !self.cfg().enabled {
            return Ok(false);
        }
        let rows = self.d.entitlements.list_for_user(user_id).await?;
        self.withdraw_rows_if_lapsed(user_id, &rows, self.now(), self.grace())
            .await
    }

    async fn withdraw_rows_if_lapsed(
        &self,
        user_id: &str,
        rows: &[EntitlementRow],
        now: DateTime<Utc>,
        grace: Duration,
    ) -> Result<bool> {
        let pending = core::withdrawal_pending(rows, now, grace);
        if pending.is_empty() {
            return Ok(false);
        }
        let ids: Vec<Uuid> = pending.iter().map(|r| r.id).collect();
        // Claim first (WHERE withdrawn_at IS NULL) so concurrent observers of the
        // same lapse rotate exactly once.
        let stamped = self.d.entitlements.mark_withdrawn(&ids, now).await?;
        if stamped == 0 {
            return Ok(false);
        }
        if let Some(rotator) = &self.d.rotator {
            rotator.rotate(user_id).await?;
        }
        tracing::info!(user_id, rows = stamped, "plan lapsed: content withdrawn");
        Ok(true)
    }

    /// The daily sweep: every candidate re-checked through the core.
    pub async fn sweep_withdrawals(&self) -> Result<u64> {
        if !self.cfg().enabled {
            return Ok(0);
        }
        let now = self.now();
        let users = self
            .d
            .entitlements
            .users_with_unwithdrawn_ended_rows(now)
            .await?;
        let mut n = 0;
        for u in users {
            if self.withdraw_if_lapsed(&u).await? {
                n += 1;
            }
        }
        Ok(n)
    }

    // ---------------------------------------------------------------- erasure

    /// Account erasure: purge every plans row of the user.
    pub async fn purge_user(&self, user_id: &str) -> Result<()> {
        self.d.codes.purge_user(user_id).await?;
        self.d.memberships.purge_user(user_id).await?;
        self.d.billing_events.purge_user(user_id).await?;
        self.d.entitlements.purge_user(user_id).await
    }

    /// Active `web` rows of a user (erasure cancels them on the provider first).
    pub async fn active_web_rows(&self, user_id: &str) -> Result<Vec<EntitlementRow>> {
        let now = self.now();
        let grace = self.grace();
        Ok(self
            .d
            .entitlements
            .list_for_user(user_id)
            .await?
            .into_iter()
            .filter(|r| r.source == Source::Web && core::row_is_active(r, now, grace))
            .collect())
    }

    async fn audit(
        &self,
        actor: &str,
        action: &str,
        target_user: Option<&str>,
        target_ref: Option<&str>,
        reason: &str,
    ) -> Result<()> {
        self.d
            .audit
            .record(AuditEntry {
                actor: actor.to_string(),
                action: action.to_string(),
                target_user: target_user.map(str::to_string),
                target_ref: target_ref.map(str::to_string),
                reason: reason.to_string(),
            })
            .await
    }
}

#[async_trait]
impl PlanSource for PlanService {
    async fn snapshot(&self, user_id: &str) -> Result<PlanSnapshot> {
        PlanService::snapshot(self, user_id).await
    }
}

#[async_trait]
impl AccessCodeIssuer for PlanService {
    async fn mint(
        &self,
        campaign_key: &str,
        issued_by: &str,
        issued_to_hint: Option<String>,
    ) -> Result<MintedCode> {
        let campaign = self.campaign_by_key(campaign_key).await?;
        let mut codes = self
            .mint_codes(campaign_key, 1, issued_by, issued_to_hint)
            .await?;
        Ok(MintedCode {
            code: codes.remove(0),
            campaign_key: campaign.key,
            campaign_name: campaign.name,
            kind: campaign.kind,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{
        MockAccessCodeRepo, MockAuditRepo, MockBillingEventRepo, MockCacheSecretRotator,
        MockCampaignRepo, MockClock, MockEntitlementRepo, MockMembershipRepo, MockPlanConfigSource,
    };
    use chrono::TimeZone;
    use mockall::predicate::*;

    fn t(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, day, 12, 0, 0).unwrap()
    }

    struct Mocks {
        entitlements: MockEntitlementRepo,
        campaigns: MockCampaignRepo,
        memberships: MockMembershipRepo,
        codes: MockAccessCodeRepo,
        billing: MockBillingEventRepo,
        audit: MockAuditRepo,
        config: MockPlanConfigSource,
        clock: MockClock,
        rotator: MockCacheSecretRotator,
    }

    fn mocks(enabled: bool, now: DateTime<Utc>) -> Mocks {
        let mut config = MockPlanConfigSource::new();
        config.expect_plan_config().returning(move || PlanConfig {
            enabled,
            grace_days: 3,
        });
        let mut clock = MockClock::new();
        clock.expect_now().returning(move || now);
        let mut audit = MockAuditRepo::new();
        audit.expect_record().returning(|_| Ok(()));
        Mocks {
            entitlements: MockEntitlementRepo::new(),
            campaigns: MockCampaignRepo::new(),
            memberships: MockMembershipRepo::new(),
            codes: MockAccessCodeRepo::new(),
            billing: MockBillingEventRepo::new(),
            audit,
            config,
            clock,
            rotator: MockCacheSecretRotator::new(),
        }
    }

    fn service(m: Mocks) -> PlanService {
        PlanService::new(PlanDeps {
            entitlements: Arc::new(m.entitlements),
            campaigns: Arc::new(m.campaigns),
            memberships: Arc::new(m.memberships),
            codes: Arc::new(m.codes),
            billing_events: Arc::new(m.billing),
            audit: Arc::new(m.audit),
            config: Arc::new(m.config),
            clock: Arc::new(m.clock),
            rotator: Some(Arc::new(m.rotator)),
        })
    }

    fn row(source: Source, ends: Option<DateTime<Utc>>) -> EntitlementRow {
        EntitlementRow {
            id: Uuid::new_v4(),
            user_id: "u1".into(),
            source,
            provider_ref: "ref".into(),
            campaign_id: None,
            starts_at: t(1),
            ends_at: ends,
            status: EntitlementStatus::Active,
            revoked_at: None,
            withdrawn_at: None,
        }
    }

    fn trial_campaign() -> Campaign {
        Campaign {
            id: Uuid::new_v4(),
            key: "beta-premium".into(),
            name: "Beta premium".into(),
            kind: CampaignKind::PremiumTrial { duration_days: 90 },
            enrollment_closes_at: None,
            closed_at: None,
            created_by: "admin".into(),
            created_at: t(1),
        }
    }

    #[tokio::test]
    async fn kill_switch_off_is_free_without_reads() {
        let m = mocks(false, t(5));
        let s = service(m);
        assert_eq!(s.snapshot("u1").await.unwrap(), PlanSnapshot::free());
        assert!(!s.withdraw_if_lapsed("u1").await.unwrap());
        assert_eq!(s.sweep_withdrawals().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn snapshot_is_premium_with_an_active_row_and_no_withdrawal() {
        let mut m = mocks(true, t(5));
        m.entitlements
            .expect_list_for_user()
            .returning(|_| Ok(vec![row(Source::Apple, Some(t(20)))]));
        m.memberships
            .expect_list_for_user()
            .returning(|_| Ok(vec![]));
        m.entitlements.expect_mark_withdrawn().times(0);
        m.rotator.expect_rotate().times(0);
        let s = service(m);
        let snap = s.snapshot("u1").await.unwrap();
        assert!(snap.grants(Unlock::CatalogUnlimited));
        assert!(s.grants("u1", Unlock::OfflineCache).await.unwrap());
    }

    #[tokio::test]
    async fn lapsed_trial_withdraws_exactly_once_via_claim() {
        let mut m = mocks(true, t(15));
        let ended = row(Source::Code, Some(t(10)));
        let id = ended.id;
        m.entitlements
            .expect_list_for_user()
            .returning(move |_| Ok(vec![ended.clone()]));
        m.memberships
            .expect_list_for_user()
            .returning(|_| Ok(vec![]));
        // first observer stamps 1 row and rotates; second observer stamps 0 → no rotate
        let mut seq = mockall::Sequence::new();
        m.entitlements
            .expect_mark_withdrawn()
            .with(eq(vec![id]), always())
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Ok(1));
        m.rotator
            .expect_rotate()
            .with(eq("u1"))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));
        m.entitlements
            .expect_mark_withdrawn()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Ok(0));
        let s = service(m);
        let snap = s.snapshot("u1").await.unwrap();
        assert_eq!(snap.plan, crate::model::Plan::Free);
        assert!(!s.withdraw_if_lapsed("u1").await.unwrap());
    }

    #[tokio::test]
    async fn no_withdrawal_while_paid_row_active_after_trial_end() {
        let mut m = mocks(true, t(15));
        m.entitlements.expect_list_for_user().returning(|_| {
            Ok(vec![
                row(Source::Code, Some(t(10))),
                row(Source::Google, Some(t(30))),
            ])
        });
        m.memberships
            .expect_list_for_user()
            .returning(|_| Ok(vec![]));
        m.entitlements.expect_mark_withdrawn().times(0);
        m.rotator.expect_rotate().times(0);
        let s = service(m);
        assert_eq!(s.snapshot("u1").await.unwrap().source, Some(Source::Google));
    }

    #[tokio::test]
    async fn redeem_unknown_revoked_and_spent_codes_are_refused_neutrally() {
        let mut m = mocks(true, t(5));
        let mut spent = AccessCode {
            id: Uuid::new_v4(),
            campaign_id: Uuid::new_v4(),
            issued_by: "bo".into(),
            issued_to_hint: None,
            max_uses: 1,
            uses: 1,
            revoked_at: None,
            created_at: t(1),
        };
        let mut calls = 0;
        m.codes.expect_find_by_hash().returning(move |_| {
            calls += 1;
            match calls {
                1 => Ok(None),
                2 => {
                    spent.revoked_at = Some(t(2));
                    spent.uses = 0;
                    Ok(Some(spent.clone()))
                }
                _ => {
                    spent.revoked_at = None;
                    spent.uses = 1;
                    Ok(Some(spent.clone()))
                }
            }
        });
        let s = service(m);
        for _ in 0..3 {
            match s.redeem("u1", "ABC").await {
                Err(AppError::NotFound(msg)) => assert_eq!(msg, CODE_REFUSED),
                other => panic!("expected neutral refusal, got {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn redeem_trial_code_enrols_with_per_tester_end_and_consumes_code() {
        let mut m = mocks(true, t(5));
        let campaign = trial_campaign();
        let cid = campaign.id;
        let code = AccessCode {
            id: Uuid::new_v4(),
            campaign_id: cid,
            issued_by: "discord".into(),
            issued_to_hint: Some("member#1".into()),
            max_uses: 1,
            uses: 0,
            revoked_at: None,
            created_at: t(1),
        };
        let code_id = code.id;
        m.codes
            .expect_find_by_hash()
            .returning(move |_| Ok(Some(code.clone())));
        let c2 = campaign.clone();
        m.campaigns
            .expect_get()
            .with(eq(cid))
            .returning(move |_| Ok(Some(c2.clone())));
        m.memberships
            .expect_list_for_user()
            .returning(|_| Ok(vec![]));
        m.memberships
            .expect_enrol()
            .withf(move |e| {
                e.campaign_id == cid
                    && e.user_id == "u1"
                    && e.code_id == Some(code_id)
                    && e.source == MembershipSource::Code
                    && e.ends_at == Some(t(5) + Duration::days(90))
                    && e.trial_row.as_ref().is_some_and(|w| {
                        w.source == Source::Code
                            && w.campaign_id == Some(cid)
                            && w.ends_at == Some(t(5) + Duration::days(90))
                            && w.provider_ref == code_id.to_string()
                    })
            })
            .times(1)
            .returning(|_| Ok(()));
        let s = service(m);
        let out = s.redeem("u1", "whatever").await.unwrap();
        assert_eq!(out.campaign_key, "beta-premium");
        assert_eq!(out.ends_at, Some(t(5) + Duration::days(90)));
    }

    #[tokio::test]
    async fn redeem_refuses_second_trial_and_closed_campaign() {
        let mut m = mocks(true, t(5));
        let running = trial_campaign();
        let other = Campaign {
            id: Uuid::new_v4(),
            key: "trial-2".into(),
            ..trial_campaign()
        };
        let code = AccessCode {
            id: Uuid::new_v4(),
            campaign_id: other.id,
            issued_by: "bo".into(),
            issued_to_hint: None,
            max_uses: 1,
            uses: 0,
            revoked_at: None,
            created_at: t(1),
        };
        m.codes
            .expect_find_by_hash()
            .returning(move |_| Ok(Some(code.clone())));
        let o2 = other.clone();
        m.campaigns
            .expect_get()
            .returning(move |_| Ok(Some(o2.clone())));
        let membership = Membership {
            row: MembershipRow {
                campaign_id: running.id,
                user_id: "u1".into(),
                enrolled_at: t(2),
                ends_at: Some(t(2) + Duration::days(90)),
                revoked_at: None,
                source: MembershipSource::Code,
            },
            campaign: running.clone(),
        };
        m.memberships
            .expect_list_for_user()
            .returning(move |_| Ok(vec![membership.clone()]));
        m.memberships.expect_enrol().times(0);
        let s = service(m);
        assert!(matches!(
            s.redeem("u1", "x").await,
            Err(AppError::FailedPrecondition(msg)) if msg == "trial_already_active"
        ));
    }

    #[tokio::test]
    async fn admin_grant_requires_confirmation_when_open_ended_and_audits() {
        let mut m = mocks(true, t(5));
        m.entitlements
            .expect_upsert()
            .withf(|w| w.source == Source::Admin && w.ends_at == Some(t(20)))
            .times(1)
            .returning(|w| {
                Ok(EntitlementRow {
                    id: Uuid::new_v4(),
                    user_id: w.user_id,
                    source: w.source,
                    provider_ref: w.provider_ref,
                    campaign_id: None,
                    starts_at: w.starts_at,
                    ends_at: w.ends_at,
                    status: w.status,
                    revoked_at: None,
                    withdrawn_at: None,
                })
            });
        let s = service(m);
        assert!(matches!(
            s.grant_premium("u1", None, false, "adm", "r").await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            s.grant_premium("u1", Some(t(1)), false, "adm", "r").await,
            Err(AppError::InvalidArgument(_))
        ));
        let row = s
            .grant_premium("u1", Some(t(20)), false, "adm", "thanks")
            .await
            .unwrap();
        assert_eq!(row.source, Source::Admin);
    }

    #[tokio::test]
    async fn store_rows_are_not_revocable_but_admin_rows_are() {
        let mut m = mocks(true, t(5));
        let apple = row(Source::Apple, Some(t(20)));
        let admin = row(Source::Admin, Some(t(20)));
        let (aid, mid) = (apple.id, admin.id);
        let apple2 = apple.clone();
        let admin2 = admin.clone();
        m.entitlements.expect_get().returning(move |id| {
            Ok(if id == aid {
                Some(apple2.clone())
            } else if id == mid {
                Some(admin2.clone())
            } else {
                None
            })
        });
        m.entitlements
            .expect_revoke()
            .with(eq(mid), always())
            .times(1)
            .returning(|_, _| Ok(()));
        // after revoking the only row, the user lapses → withdrawal
        let mut revoked = admin.clone();
        revoked.revoked_at = Some(t(5));
        m.entitlements
            .expect_list_for_user()
            .returning(move |_| Ok(vec![revoked.clone()]));
        m.entitlements
            .expect_mark_withdrawn()
            .times(1)
            .returning(|_, _| Ok(1));
        m.rotator.expect_rotate().times(1).returning(|_| Ok(()));
        let s = service(m);
        assert!(matches!(
            s.revoke_entitlement(aid, "adm", "r").await,
            Err(AppError::FailedPrecondition(_))
        ));
        s.revoke_entitlement(mid, "adm", "r").await.unwrap();
        assert!(matches!(
            s.revoke_entitlement(Uuid::new_v4(), "adm", "r").await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn campaign_creation_validates_key_and_duration() {
        let mut m = mocks(true, t(5));
        m.campaigns.expect_create().returning(|n| {
            Ok(Campaign {
                id: Uuid::new_v4(),
                key: n.key,
                name: n.name,
                kind: n.kind,
                enrollment_closes_at: None,
                closed_at: None,
                created_by: n.created_by,
                created_at: t(5),
            })
        });
        let s = service(m);
        assert!(matches!(
            s.create_campaign("Bad Key!", "n", CampaignKind::Feature, "adm")
                .await,
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            s.create_campaign(
                "t",
                "n",
                CampaignKind::PremiumTrial { duration_days: 0 },
                "adm"
            )
            .await,
            Err(AppError::InvalidArgument(_))
        ));
        let c = s
            .create_campaign("Midi-Drums", " MIDI drums ", CampaignKind::Feature, "adm")
            .await
            .unwrap();
        assert_eq!(c.key, "midi-drums");
        assert_eq!(c.name, "MIDI drums");
    }

    #[tokio::test]
    async fn mint_codes_are_hashed_single_use_and_refused_when_closed() {
        let mut m = mocks(true, t(5));
        let mut campaign = trial_campaign();
        let open = campaign.clone();
        m.campaigns
            .expect_get_by_key()
            .with(eq("beta-premium"))
            .returning(move |_| Ok(Some(open.clone())));
        campaign.key = "closed".into();
        campaign.closed_at = Some(t(2));
        let closed = campaign.clone();
        m.campaigns
            .expect_get_by_key()
            .with(eq("closed"))
            .returning(move |_| Ok(Some(closed.clone())));
        m.codes
            .expect_insert()
            .withf(|_, hash, by, _, max| hash.len() == 64 && by == "bo" && *max == 1)
            .times(3)
            .returning(|cid, _, by, hint, max| {
                Ok(AccessCode {
                    id: Uuid::new_v4(),
                    campaign_id: cid,
                    issued_by: by.into(),
                    issued_to_hint: hint,
                    max_uses: max,
                    uses: 0,
                    revoked_at: None,
                    created_at: t(5),
                })
            });
        let s = service(m);
        let codes = s.mint_codes("beta-premium", 2, "bo", None).await.unwrap();
        assert_eq!(codes.len(), 2);
        assert_ne!(codes[0], codes[1]);
        let minted = s
            .mint("beta-premium", "bo", Some("hint".into()))
            .await
            .unwrap();
        assert_eq!(minted.campaign_key, "beta-premium");
        assert!(matches!(
            s.mint_codes("closed", 1, "bo", None).await,
            Err(AppError::FailedPrecondition(_))
        ));
        assert!(matches!(
            s.mint_codes("beta-premium", 0, "bo", None).await,
            Err(AppError::InvalidArgument(_))
        ));
    }

    #[tokio::test]
    async fn sweep_visits_candidates_and_counts_withdrawals() {
        let mut m = mocks(true, t(15));
        m.entitlements
            .expect_users_with_unwithdrawn_ended_rows()
            .returning(|_| Ok(vec!["a".into(), "b".into()]));
        m.entitlements.expect_list_for_user().returning(|u| {
            Ok(vec![if u == "a" {
                // still premium via a later row → nothing
                row(Source::Web, Some(t(30)))
            } else {
                row(Source::Code, Some(t(10)))
            }])
        });
        m.entitlements
            .expect_mark_withdrawn()
            .times(1)
            .returning(|_, _| Ok(1));
        m.rotator
            .expect_rotate()
            .with(eq("b"))
            .times(1)
            .returning(|_| Ok(()));
        let s = service(m);
        assert_eq!(s.sweep_withdrawals().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn account_ids_intersects_plan_and_beta_filters() {
        let mut m = mocks(true, t(5));
        m.entitlements
            .expect_active_user_ids()
            .with(always(), eq(3), eq(true))
            .returning(|_, _, _| Ok(vec!["a".into(), "b".into()]));
        let c = Campaign {
            kind: CampaignKind::Feature,
            key: "midi-drums".into(),
            ..trial_campaign()
        };
        let cid = c.id;
        m.campaigns
            .expect_get_by_key()
            .returning(move |_| Ok(Some(c.clone())));
        m.memberships
            .expect_active_member_ids()
            .with(eq(cid), always())
            .returning(|_, _| Ok(vec!["b".into(), "c".into()]));
        let s = service(m);
        assert_eq!(
            s.account_ids(PlanFilter::Trial, Some("midi-drums"))
                .await
                .unwrap(),
            vec!["b".to_string()]
        );
        assert_eq!(
            s.account_ids(PlanFilter::Any, Some("midi-drums"))
                .await
                .unwrap(),
            vec!["b".to_string(), "c".to_string()]
        );
        assert!(
            s.account_ids(PlanFilter::Any, None)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn apply_provider_write_then_checks_lapse() {
        let mut m = mocks(true, t(15));
        let refunded = EntitlementRow {
            status: EntitlementStatus::Refunded,
            ..row(Source::Apple, Some(t(30)))
        };
        let r2 = refunded.clone();
        m.entitlements
            .expect_upsert()
            .returning(move |_| Ok(r2.clone()));
        let r3 = refunded.clone();
        m.entitlements
            .expect_list_for_user()
            .returning(move |_| Ok(vec![r3.clone()]));
        m.entitlements
            .expect_mark_withdrawn()
            .times(1)
            .returning(|_, _| Ok(1));
        m.rotator.expect_rotate().times(1).returning(|_| Ok(()));
        m.billing
            .expect_record_if_new()
            .returning(|_, _, _, _| Ok(true));
        m.billing.expect_mark_applied().returning(|_, _| Ok(()));
        let s = service(m);
        assert!(
            s.record_event(Source::Apple, "e1", Some("u1"), "p")
                .await
                .unwrap()
        );
        s.apply(EntitlementWrite {
            user_id: "u1".into(),
            source: Source::Apple,
            provider_ref: "otx".into(),
            campaign_id: None,
            starts_at: t(1),
            ends_at: Some(t(30)),
            status: EntitlementStatus::Refunded,
        })
        .await
        .unwrap();
        s.mark_event_applied(Source::Apple, "e1").await.unwrap();
    }
}
