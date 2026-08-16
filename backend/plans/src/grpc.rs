//! gRPC adapter (`cymbra.plans.v1.PlanService`) over [`PlanService`]: the app
//! surface (my plan, redeem, store purchases, web checkout) and the music-admin
//! console surface. Thin: auth + rate limit + proto mapping; every decision is in
//! the service / core. Coverage-excluded like the other `grpc.rs` adapters.

// `tonic::Status` is large by design; every adapter in the workspace allows this.
#![allow(clippy::result_large_err)]

use crate::model::{
    Campaign, CampaignKind, EntitlementRow, Membership, MembershipRow, PlanSnapshot, Source,
};
use crate::ports::{
    Channel, HandleResolver, PaywallConfigSource, Platform, StorePurchaseVerifier,
    WebBillingProvider,
};
use crate::proto::plan_service_server::{PlanService as PlanServiceTrait, PlanServiceServer};
use crate::proto::{self};
use crate::service::{PlanFilter, PlanService};
use chrono::{DateTime, Utc};
use cymbra_platform::cache::Cache;
use cymbra_platform::identity::AuthIdentity;
use cymbra_platform::{AppError, guard, ratelimit};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// Redemption throttle: attempts per account and per address per window.
const REDEEM_MAX_PER_USER: u32 = 10;
const REDEEM_MAX_PER_ADDR: u32 = 30;
const REDEEM_WINDOW: Duration = Duration::from_secs(600);

/// The store-build audience: never offered the web channel or the redeem field.
const APP_AUDIENCE: &str = "music";

pub struct PlanGrpc {
    svc: Arc<PlanService>,
    paywall: Arc<dyn PaywallConfigSource>,
    cache: Arc<dyn Cache>,
    handles: Option<Arc<dyn HandleResolver>>,
    verifiers: HashMap<Channel, Arc<dyn StorePurchaseVerifier>>,
    web: Option<Arc<dyn WebBillingProvider>>,
}

impl PlanGrpc {
    pub fn new(
        svc: Arc<PlanService>,
        paywall: Arc<dyn PaywallConfigSource>,
        cache: Arc<dyn Cache>,
    ) -> Self {
        Self {
            svc,
            paywall,
            cache,
            handles: None,
            verifiers: HashMap::new(),
            web: None,
        }
    }

    pub fn with_handles(mut self, handles: Arc<dyn HandleResolver>) -> Self {
        self.handles = Some(handles);
        self
    }

    pub fn with_verifier(mut self, channel: Channel, v: Arc<dyn StorePurchaseVerifier>) -> Self {
        self.verifiers.insert(channel, v);
        self
    }

    pub fn with_web(mut self, web: Arc<dyn WebBillingProvider>) -> Self {
        self.web = Some(web);
        self
    }

    pub fn into_server(self) -> PlanServiceServer<Self> {
        PlanServiceServer::new(self)
    }

    fn admin<T>(&self, req: &Request<T>) -> Result<AuthIdentity, Status> {
        let id = identity(req)?;
        guard::require_admin_in_scope(&id, "music").map_err(|e| e.to_status())?;
        Ok(id)
    }

    /// Resolve `user_id` | `handle` to an account id (admin RPCs).
    async fn target(&self, user_id: &str, handle: &str) -> Result<String, Status> {
        if !user_id.is_empty() {
            return Ok(user_id.to_string());
        }
        let handle = handle.trim();
        if handle.is_empty() {
            return Err(Status::invalid_argument("user_id or handle required"));
        }
        let resolver = self
            .handles
            .as_ref()
            .ok_or_else(|| Status::unimplemented("handle lookup not wired"))?;
        resolver
            .user_id_for_handle(handle)
            .await
            .map_err(|e| e.to_status())?
            .ok_or_else(|| Status::not_found("no account with that handle"))
    }

    /// The paywall part of the plan answer: managed-on + can-purchase-here.
    fn purchase_view(
        &self,
        snapshot: &PlanSnapshot,
        platform: Option<Platform>,
    ) -> (proto::Channel, bool, proto::Channel) {
        let managed_on = snapshot.source.and_then(Channel::from_source);
        let channel = platform.map(Platform::channel);
        let can_purchase = match (managed_on, channel) {
            (Some(_), _) => false,
            (None, Some(c)) => self.paywall.channel_enabled(c) && self.svc.enabled(),
            (None, None) => false,
        };
        (
            managed_on.map(channel_to_proto).unwrap_or_default(),
            can_purchase,
            channel
                .filter(|_| can_purchase)
                .map(channel_to_proto)
                .unwrap_or_default(),
        )
    }

    fn plan_response(
        &self,
        snapshot: &PlanSnapshot,
        platform: Option<Platform>,
    ) -> proto::GetMyPlanResponse {
        let (managed_on, can_purchase_here, purchase_channel) =
            self.purchase_view(snapshot, platform);
        proto::GetMyPlanResponse {
            plan: snapshot.plan.as_str().to_string(),
            source: snapshot.source.map(|s| s.as_str().to_string()),
            ends_at: snapshot.ends_at.map(rfc3339),
            ends_without_renewal: snapshot.ends_without_renewal,
            trial_campaign_key: snapshot.trial.as_ref().map(|t| t.campaign_key.clone()),
            trial_campaign_name: snapshot.trial.as_ref().map(|t| t.campaign_name.clone()),
            trial_ends_at: snapshot.trial.as_ref().map(|t| rfc3339(t.ends_at)),
            betas: snapshot
                .betas
                .iter()
                .map(|b| proto::BetaMembership {
                    campaign_key: b.campaign_key.clone(),
                    campaign_name: b.campaign_name.clone(),
                    kind: b.kind.as_str().to_string(),
                    joined_at: rfc3339(b.joined_at),
                    ends_at: b.ends_at.map(rfc3339),
                })
                .collect(),
            managed_on: managed_on.into(),
            can_purchase_here,
            purchase_channel: purchase_channel.into(),
            products: if can_purchase_here {
                self.paywall.products()
            } else {
                Vec::new()
            },
            unlocks: crate::model::PREMIUM_UNLOCKS
                .iter()
                .filter(|u| snapshot.grants(**u))
                .map(|u| u.key().to_string())
                .collect(),
        }
    }
}

fn identity<T>(req: &Request<T>) -> Result<AuthIdentity, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

fn rfc3339(t: DateTime<Utc>) -> String {
    t.to_rfc3339()
}

fn parse_rfc3339(s: &str) -> Result<DateTime<Utc>, Status> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| Status::invalid_argument(format!("invalid RFC 3339 timestamp: {s}")))
}

fn channel_to_proto(c: Channel) -> proto::Channel {
    match c {
        Channel::Apple => proto::Channel::Apple,
        Channel::Google => proto::Channel::Google,
        Channel::Web => proto::Channel::Web,
    }
}

fn platform_from_proto(p: i32) -> Option<Platform> {
    match proto::Platform::try_from(p).ok()? {
        proto::Platform::Ios => Some(Platform::Ios),
        proto::Platform::Macos => Some(Platform::Macos),
        proto::Platform::Android => Some(Platform::Android),
        proto::Platform::Linux => Some(Platform::Linux),
        proto::Platform::Windows => Some(Platform::Windows),
        proto::Platform::Web => Some(Platform::Web),
        proto::Platform::Unspecified => None,
    }
}

fn row_msg(r: &EntitlementRow) -> proto::EntitlementRowMsg {
    proto::EntitlementRowMsg {
        id: r.id.to_string(),
        source: r.source.as_str().to_string(),
        provider_ref: r.provider_ref.clone(),
        campaign_id: r.campaign_id.map(|c| c.to_string()),
        starts_at: rfc3339(r.starts_at),
        ends_at: r.ends_at.map(rfc3339),
        status: r.status.as_str().to_string(),
        revoked_at: r.revoked_at.map(rfc3339),
        withdrawn_at: r.withdrawn_at.map(rfc3339),
    }
}

fn membership_msg(m: &Membership) -> proto::MembershipMsg {
    membership_row_msg(&m.row, &m.campaign)
}

fn membership_row_msg(row: &MembershipRow, c: &Campaign) -> proto::MembershipMsg {
    proto::MembershipMsg {
        campaign_key: c.key.clone(),
        campaign_name: c.name.clone(),
        kind: c.kind.as_str().to_string(),
        user_id: row.user_id.clone(),
        enrolled_at: rfc3339(row.enrolled_at),
        ends_at: row.ends_at.map(rfc3339),
        revoked_at: row.revoked_at.map(rfc3339),
        source: row.source.as_str().to_string(),
    }
}

fn campaign_msg(c: &Campaign, now: DateTime<Utc>) -> proto::CampaignMsg {
    proto::CampaignMsg {
        id: c.id.to_string(),
        key: c.key.clone(),
        name: c.name.clone(),
        kind: c.kind.as_str().to_string(),
        duration_days: match c.kind {
            CampaignKind::PremiumTrial { duration_days } => Some(duration_days),
            CampaignKind::Feature => None,
        },
        enrollment_closes_at: c.enrollment_closes_at.map(rfc3339),
        closed_at: c.closed_at.map(rfc3339),
        created_by: c.created_by.clone(),
        created_at: rfc3339(c.created_at),
        accepts_enrolment: c.accepts_enrolment(now),
    }
}

fn parse_kind(kind: &str, duration_days: Option<u32>) -> Result<CampaignKind, Status> {
    match kind {
        "premium_trial" => Ok(CampaignKind::PremiumTrial {
            duration_days: duration_days.unwrap_or(CampaignKind::DEFAULT_TRIAL_DAYS),
        }),
        "feature" => Ok(CampaignKind::Feature),
        other => Err(Status::invalid_argument(format!(
            "unknown campaign kind {other}"
        ))),
    }
}

fn parse_filter(plan: &str) -> Result<PlanFilter, Status> {
    match plan {
        "" | "any" => Ok(PlanFilter::Any),
        "premium" => Ok(PlanFilter::Premium),
        "trial" => Ok(PlanFilter::Trial),
        other => Err(Status::invalid_argument(format!(
            "unknown plan filter {other} (any | premium | trial)"
        ))),
    }
}

#[tonic::async_trait]
impl PlanServiceTrait for PlanGrpc {
    async fn get_my_plan(
        &self,
        req: Request<proto::GetMyPlanRequest>,
    ) -> Result<Response<proto::GetMyPlanResponse>, Status> {
        let id = identity(&req)?;
        let platform = platform_from_proto(req.into_inner().platform);
        let snapshot = self
            .svc
            .snapshot(&id.user_id)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(self.plan_response(&snapshot, platform)))
    }

    async fn redeem_access_code(
        &self,
        req: Request<proto::RedeemAccessCodeRequest>,
    ) -> Result<Response<proto::RedeemAccessCodeResponse>, Status> {
        let id = identity(&req)?;
        // Web-only: the store builds carry no code entry (Apple 3.1.1).
        if id.audience == APP_AUDIENCE {
            return Err(Status::failed_precondition("codes are redeemed on the web"));
        }
        let addr = req
            .remote_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_else(|| "unknown".into());
        // Throttle BEFORE any lookup (per account and per address).
        ratelimit::check(
            self.cache.as_ref(),
            "redeem_code_user",
            &id.user_id,
            REDEEM_MAX_PER_USER,
            REDEEM_WINDOW,
        )
        .await
        .map_err(|e| e.to_status())?;
        ratelimit::check(
            self.cache.as_ref(),
            "redeem_code_addr",
            &addr,
            REDEEM_MAX_PER_ADDR,
            REDEEM_WINDOW,
        )
        .await
        .map_err(|e| e.to_status())?;
        let code = req.into_inner().code;
        let out = self
            .svc
            .redeem(&id.user_id, &code)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::RedeemAccessCodeResponse {
            campaign_key: out.campaign_key,
            campaign_name: out.campaign_name,
            kind: out.kind.as_str().to_string(),
            ends_at: out.ends_at.map(rfc3339),
        }))
    }

    async fn report_store_purchase(
        &self,
        req: Request<proto::ReportStorePurchaseRequest>,
    ) -> Result<Response<proto::ReportStorePurchaseResponse>, Status> {
        let id = identity(&req)?;
        let r = req.into_inner();
        let channel = match proto::Channel::try_from(r.channel) {
            Ok(proto::Channel::Apple) => Channel::Apple,
            Ok(proto::Channel::Google) => Channel::Google,
            _ => return Err(Status::invalid_argument("channel must be apple or google")),
        };
        if !self.paywall.channel_enabled(channel) {
            return Err(Status::failed_precondition("channel disabled"));
        }
        let verifier = self
            .verifiers
            .get(&channel)
            .ok_or_else(|| Status::unimplemented("purchase channel not configured"))?;
        let verified = verifier
            .verify(&id.user_id, &r.payload, &r.product_id)
            .await
            .map_err(|e| e.to_status())?;
        if verified.write.user_id != id.user_id {
            return Err(Status::permission_denied(
                "purchase belongs to another account",
            ));
        }
        self.svc
            .apply(verified.write)
            .await
            .map_err(|e| e.to_status())?;
        let snapshot = self
            .svc
            .snapshot(&id.user_id)
            .await
            .map_err(|e| e.to_status())?;
        let platform = match channel {
            Channel::Apple => Some(Platform::Ios),
            Channel::Google => Some(Platform::Android),
            Channel::Web => Some(Platform::Web),
        };
        Ok(Response::new(proto::ReportStorePurchaseResponse {
            plan: Some(self.plan_response(&snapshot, platform)),
        }))
    }

    async fn create_web_checkout(
        &self,
        req: Request<proto::CreateWebCheckoutRequest>,
    ) -> Result<Response<proto::CreateWebCheckoutResponse>, Status> {
        let id = identity(&req)?;
        if !self.svc.enabled() || !self.paywall.channel_enabled(Channel::Web) {
            return Err(Status::failed_precondition("web channel disabled"));
        }
        let web = self
            .web
            .as_ref()
            .ok_or_else(|| Status::unimplemented("web billing not configured"))?;
        let snapshot = self
            .svc
            .snapshot(&id.user_id)
            .await
            .map_err(|e| e.to_status())?;
        if snapshot.source.is_some_and(|s| s.is_paid_channel()) {
            return Err(Status::failed_precondition(
                "already subscribed on another channel",
            ));
        }
        let product = req.into_inner().product_id;
        if !self.paywall.products().contains(&product) {
            return Err(Status::invalid_argument("unknown product"));
        }
        let url = web
            .create_checkout(&id.user_id, &product)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::CreateWebCheckoutResponse {
            checkout_url: url,
        }))
    }

    // ------------------------------------------------------------- admin

    async fn lookup_account_plan(
        &self,
        req: Request<proto::LookupAccountPlanRequest>,
    ) -> Result<Response<proto::LookupAccountPlanResponse>, Status> {
        self.admin(&req)?;
        let r = req.into_inner();
        let user_id = self.target(&r.user_id, &r.handle).await?;
        let ap = self
            .svc
            .account_plan(&user_id)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::LookupAccountPlanResponse {
            user_id,
            snapshot: Some(self.plan_response(&ap.snapshot, None)),
            rows: ap.rows.iter().map(row_msg).collect(),
            memberships: ap.memberships.iter().map(membership_msg).collect(),
        }))
    }

    async fn get_plans_for_accounts(
        &self,
        req: Request<proto::GetPlansForAccountsRequest>,
    ) -> Result<Response<proto::GetPlansForAccountsResponse>, Status> {
        self.admin(&req)?;
        let ids = req.into_inner().user_ids;
        if ids.len() > 200 {
            return Err(Status::invalid_argument("at most 200 ids per call"));
        }
        let mut badges = Vec::with_capacity(ids.len());
        for uid in ids {
            let s = self.svc.snapshot(&uid).await.map_err(|e| e.to_status())?;
            badges.push(proto::AccountPlanBadge {
                user_id: uid,
                plan: s.plan.as_str().to_string(),
                trial: s.trial.is_some() && s.source == Some(Source::Code),
                ends_at: s.ends_at.map(rfc3339),
                beta_keys: s.beta_keys(),
            });
        }
        Ok(Response::new(proto::GetPlansForAccountsResponse { badges }))
    }

    async fn list_account_ids_by_plan(
        &self,
        req: Request<proto::ListAccountIdsByPlanRequest>,
    ) -> Result<Response<proto::ListAccountIdsByPlanResponse>, Status> {
        self.admin(&req)?;
        let r = req.into_inner();
        let filter = parse_filter(&r.plan)?;
        let beta = (!r.beta_campaign_key.is_empty()).then_some(r.beta_campaign_key.as_str());
        let user_ids = self
            .svc
            .account_ids(filter, beta)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::ListAccountIdsByPlanResponse {
            user_ids,
        }))
    }

    async fn grant_premium(
        &self,
        req: Request<proto::GrantPremiumRequest>,
    ) -> Result<Response<proto::GrantPremiumResponse>, Status> {
        let admin = self.admin(&req)?;
        let r = req.into_inner();
        let user_id = self.target(&r.user_id, &r.handle).await?;
        let ends_at = r.ends_at.as_deref().map(parse_rfc3339).transpose()?;
        let row = self
            .svc
            .grant_premium(
                &user_id,
                ends_at,
                r.confirm_open_ended,
                &admin.user_id,
                &r.reason,
            )
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::GrantPremiumResponse {
            row: Some(row_msg(&row)),
        }))
    }

    async fn revoke_entitlement(
        &self,
        req: Request<proto::RevokeEntitlementRequest>,
    ) -> Result<Response<proto::RevokeEntitlementResponse>, Status> {
        let admin = self.admin(&req)?;
        let r = req.into_inner();
        let id = Uuid::parse_str(&r.entitlement_id)
            .map_err(|_| Status::invalid_argument("invalid entitlement id"))?;
        self.svc
            .revoke_entitlement(id, &admin.user_id, &r.reason)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::RevokeEntitlementResponse {}))
    }

    async fn enrol_handle(
        &self,
        req: Request<proto::EnrolHandleRequest>,
    ) -> Result<Response<proto::EnrolHandleResponse>, Status> {
        let admin = self.admin(&req)?;
        let r = req.into_inner();
        let user_id = self.target(&r.user_id, &r.handle).await?;
        let out = self
            .svc
            .enrol_by_admin(&user_id, &r.campaign_key, &admin.user_id, &r.reason)
            .await
            .map_err(|e| e.to_status())?;
        let campaign = self
            .svc
            .campaign_by_key(&r.campaign_key)
            .await
            .map_err(|e| e.to_status())?;
        let row = MembershipRow {
            campaign_id: campaign.id,
            user_id,
            enrolled_at: Utc::now(),
            ends_at: out.ends_at,
            revoked_at: None,
            source: crate::model::MembershipSource::Admin,
        };
        Ok(Response::new(proto::EnrolHandleResponse {
            membership: Some(membership_row_msg(&row, &campaign)),
        }))
    }

    async fn revoke_membership(
        &self,
        req: Request<proto::RevokeMembershipRequest>,
    ) -> Result<Response<proto::RevokeMembershipResponse>, Status> {
        let admin = self.admin(&req)?;
        let r = req.into_inner();
        let user_id = self.target(&r.user_id, &r.handle).await?;
        self.svc
            .revoke_membership(&r.campaign_key, &user_id, &admin.user_id, &r.reason)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::RevokeMembershipResponse {}))
    }

    async fn create_campaign(
        &self,
        req: Request<proto::CreateCampaignRequest>,
    ) -> Result<Response<proto::CreateCampaignResponse>, Status> {
        let admin = self.admin(&req)?;
        let r = req.into_inner();
        let kind = parse_kind(&r.kind, r.duration_days)?;
        let c = self
            .svc
            .create_campaign(&r.key, &r.name, kind, &admin.user_id)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::CreateCampaignResponse {
            campaign: Some(campaign_msg(&c, Utc::now())),
        }))
    }

    async fn list_campaigns(
        &self,
        req: Request<proto::ListCampaignsRequest>,
    ) -> Result<Response<proto::ListCampaignsResponse>, Status> {
        self.admin(&req)?;
        let include_closed = req.into_inner().include_closed;
        let now = Utc::now();
        let campaigns = self
            .svc
            .list_campaigns(include_closed)
            .await
            .map_err(|e| e.to_status())?
            .iter()
            .map(|c| campaign_msg(c, now))
            .collect();
        Ok(Response::new(proto::ListCampaignsResponse { campaigns }))
    }

    async fn close_enrollment(
        &self,
        req: Request<proto::CloseEnrollmentRequest>,
    ) -> Result<Response<proto::CloseEnrollmentResponse>, Status> {
        let admin = self.admin(&req)?;
        let key = req.into_inner().campaign_key;
        self.svc
            .close_enrollment(&key, &admin.user_id)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::CloseEnrollmentResponse {}))
    }

    async fn close_campaign(
        &self,
        req: Request<proto::CloseCampaignRequest>,
    ) -> Result<Response<proto::CloseCampaignResponse>, Status> {
        let admin = self.admin(&req)?;
        let key = req.into_inner().campaign_key;
        self.svc
            .close_campaign(&key, &admin.user_id)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::CloseCampaignResponse {}))
    }

    async fn mint_codes(
        &self,
        req: Request<proto::MintCodesRequest>,
    ) -> Result<Response<proto::MintCodesResponse>, Status> {
        let admin = self.admin(&req)?;
        let r = req.into_inner();
        let hint =
            (!r.issued_to_hint.trim().is_empty()).then(|| r.issued_to_hint.trim().to_string());
        let codes = self
            .svc
            .mint_codes(&r.campaign_key, r.count, &admin.user_id, hint)
            .await
            .map_err(|e| e.to_status())?;
        Ok(Response::new(proto::MintCodesResponse { codes }))
    }

    async fn revoke_codes(
        &self,
        req: Request<proto::RevokeCodesRequest>,
    ) -> Result<Response<proto::RevokeCodesResponse>, Status> {
        let admin = self.admin(&req)?;
        let r = req.into_inner();
        let mut revoked: u32 = 0;
        if !r.code_ids.is_empty() {
            for id in &r.code_ids {
                let id =
                    Uuid::parse_str(id).map_err(|_| Status::invalid_argument("invalid code id"))?;
                self.svc
                    .revoke_code(id, &admin.user_id)
                    .await
                    .map_err(|e| e.to_status())?;
                revoked += 1;
            }
        } else if !r.campaign_key.is_empty() {
            revoked = self
                .svc
                .revoke_campaign_codes(&r.campaign_key, &admin.user_id)
                .await
                .map_err(|e| e.to_status())?
                .min(u64::from(u32::MAX)) as u32;
        } else {
            return Err(Status::invalid_argument(
                "campaign_key or code_ids required",
            ));
        }
        Ok(Response::new(proto::RevokeCodesResponse { revoked }))
    }

    async fn list_members(
        &self,
        req: Request<proto::ListMembersRequest>,
    ) -> Result<Response<proto::ListMembersResponse>, Status> {
        self.admin(&req)?;
        let key = req.into_inner().campaign_key;
        let campaign = self
            .svc
            .campaign_by_key(&key)
            .await
            .map_err(|e| e.to_status())?;
        let members = self
            .svc
            .list_members(&key)
            .await
            .map_err(|e| e.to_status())?
            .iter()
            .map(|m| membership_row_msg(m, &campaign))
            .collect();
        Ok(Response::new(proto::ListMembersResponse { members }))
    }
}

/// Map an [`AppError`] for callers that pre-empt the service (kept for symmetry).
#[allow(dead_code)]
fn status(e: AppError) -> Status {
    e.to_status()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Plan;

    #[test]
    fn platform_channel_mapping_and_proto_round_trip() {
        assert_eq!(Platform::Ios.channel(), Channel::Apple);
        assert_eq!(Platform::Macos.channel(), Channel::Apple);
        assert_eq!(Platform::Android.channel(), Channel::Google);
        assert_eq!(Platform::Linux.channel(), Channel::Web);
        assert_eq!(Platform::Windows.channel(), Channel::Web);
        assert_eq!(
            platform_from_proto(proto::Platform::Ios as i32),
            Some(Platform::Ios)
        );
        assert_eq!(platform_from_proto(0), None);
        assert_eq!(parse_filter("trial").unwrap(), PlanFilter::Trial);
        assert!(parse_filter("gold").is_err());
        assert_eq!(
            parse_kind("premium_trial", None).unwrap(),
            CampaignKind::PremiumTrial { duration_days: 90 }
        );
        assert!(parse_kind("nope", None).is_err());
    }

    #[test]
    fn unlock_keys_follow_the_plan() {
        let free = PlanSnapshot::free();
        let premium = PlanSnapshot {
            plan: Plan::Premium,
            ..PlanSnapshot::free()
        };
        let keys = |s: &PlanSnapshot| -> Vec<&'static str> {
            crate::model::PREMIUM_UNLOCKS
                .iter()
                .filter(|u| s.grants(**u))
                .map(|u| u.key())
                .collect()
        };
        assert!(keys(&free).is_empty());
        assert!(keys(&premium).contains(&crate::model::Unlock::OfflineCache.key()));
    }
}
