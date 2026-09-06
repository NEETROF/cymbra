//! Integration tests for the Postgres adapters (tasks 2.2 / 2.3): forward-only
//! upserts, transactional enrolment (single-use code, one membership per
//! account per campaign, trial row), close semantics, withdrawal claim, purge.
//! Requires the dev infra up (`backend/docker-compose.yml`) with the roles
//! bootstrapped: `CYMBRA_PLANS_DATABASE_URL` must point at `plans_svc`.
//!
//! Run: `cargo test -p cymbra-plans --test pg_repos -- --ignored`

use chrono::{Duration, SubsecRound, Utc};
use cymbra_plans::pg::{
    PgAccessCodeRepo, PgAuditRepo, PgBillingEventRepo, PgCampaignRepo, PgEntitlementRepo,
    PgMembershipRepo,
};
use cymbra_plans::{
    AccessCodeRepo, AuditEntry, AuditRepo, BillingEventRepo, CampaignKind, CampaignRepo, Enrolment,
    EntitlementRepo, EntitlementStatus, EntitlementWrite, EventProvider, MembershipRepo,
    MembershipSource, NewCampaign, Source, codes,
};
use cymbra_platform::AppError;
use sqlx::PgPool;
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("CYMBRA_PLANS_DATABASE_URL")
        .expect("CYMBRA_PLANS_DATABASE_URL must be set (plans_svc)");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect as plans_svc");
    cymbra_plans::MIGRATOR
        .run(&pool)
        .await
        .expect("migrate plans");
    pool
}

fn write(
    user: Uuid,
    source: Source,
    provider_ref: &str,
    days: i64,
    status: EntitlementStatus,
) -> EntitlementWrite {
    let now = Utc::now();
    EntitlementWrite {
        user_id: user.to_string(),
        source,
        provider_ref: provider_ref.into(),
        campaign_id: None,
        starts_at: now - Duration::days(1),
        ends_at: Some(now + Duration::days(days)),
        status,
    }
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) up with per-module roles"]
async fn upsert_is_forward_only_unless_terminal_and_withdrawal_claims_once() {
    let pool = pool().await;
    let repo = PgEntitlementRepo::new(pool.clone());
    let user = Uuid::now_v7();
    let r = repo
        .upsert(write(
            user,
            Source::Apple,
            "otx-1",
            30,
            EntitlementStatus::Active,
        ))
        .await
        .unwrap();
    // A stale event with an earlier end does not move ends_at backwards.
    let stale = repo
        .upsert(write(
            user,
            Source::Apple,
            "otx-1",
            10,
            EntitlementStatus::Cancelled,
        ))
        .await
        .unwrap();
    assert_eq!(stale.id, r.id, "same (source, provider_ref) ⇒ one row");
    assert_eq!(stale.ends_at, r.ends_at);
    assert_eq!(stale.status, EntitlementStatus::Cancelled);
    // A refund ends the row now regardless of ends_at.
    let refunded = repo
        .upsert(write(
            user,
            Source::Apple,
            "otx-1",
            0,
            EntitlementStatus::Refunded,
        ))
        .await
        .unwrap();
    assert!(refunded.status.is_terminal());
    assert_eq!(
        repo.list_for_user(&user.to_string()).await.unwrap().len(),
        1
    );

    // Withdrawal claim: first stamps 1, second stamps 0.
    let now = Utc::now();
    assert_eq!(repo.mark_withdrawn(&[r.id], now).await.unwrap(), 1);
    assert_eq!(repo.mark_withdrawn(&[r.id], now).await.unwrap(), 0);
    let candidates = repo.users_with_unwithdrawn_ended_rows(now).await.unwrap();
    assert!(!candidates.contains(&user.to_string()));

    // Resubscribing reuses this row (the key is stable per user+product), so a
    // write that leaves it LIVE must clear the stamp — otherwise the next lapse
    // is skipped and the offline cache secret never rotates again.
    let back = repo
        .upsert(write(
            user,
            Source::Apple,
            "otx-1",
            30,
            EntitlementStatus::Active,
        ))
        .await
        .unwrap();
    assert_eq!(back.id, r.id);
    assert!(back.withdrawn_at.is_none(), "reactivation clears the stamp");
    // Lapsing again (terminal, so `ends_at` moves back to now) makes the user a
    // withdrawal candidate once more — the regression this guards against.
    repo.upsert(write(
        user,
        Source::Apple,
        "otx-1",
        -1,
        EntitlementStatus::Refunded,
    ))
    .await
    .unwrap();
    assert!(
        repo.users_with_unwithdrawn_ended_rows(Utc::now())
            .await
            .unwrap()
            .contains(&user.to_string())
    );
    // Stamp that second lapse, then replay a non-terminal `ended` write on the
    // already-lapsed row: it does NOT clear, so the sweep rotates only once.
    assert_eq!(repo.mark_withdrawn(&[r.id], Utc::now()).await.unwrap(), 1);
    repo.upsert(write(
        user,
        Source::Apple,
        "otx-1",
        -1,
        EntitlementStatus::Ended,
    ))
    .await
    .unwrap();
    assert!(
        !repo
            .users_with_unwithdrawn_ended_rows(Utc::now())
            .await
            .unwrap()
            .contains(&user.to_string())
    );

    repo.purge_user(&user.to_string()).await.unwrap();
    assert!(
        repo.list_for_user(&user.to_string())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) up with per-module roles"]
async fn enrolment_is_transactional_single_use_and_one_per_account_per_campaign() {
    let pool = pool().await;
    let campaigns = PgCampaignRepo::new(pool.clone());
    let memberships = PgMembershipRepo::new(pool.clone());
    let code_repo = PgAccessCodeRepo::new(pool.clone());
    let entitlements = PgEntitlementRepo::new(pool.clone());
    let admin = Uuid::now_v7();
    let key = format!("it-trial-{}", &Uuid::now_v7().simple().to_string()[24..]);
    let campaign = campaigns
        .create(NewCampaign {
            key: key.clone(),
            name: "IT trial".into(),
            kind: CampaignKind::PremiumTrial { duration_days: 90 },
            created_by: admin.to_string(),
        })
        .await
        .unwrap();
    assert!(matches!(
        campaigns
            .create(NewCampaign {
                key: key.clone(),
                name: "dup".into(),
                kind: CampaignKind::Feature,
                created_by: admin.to_string(),
            })
            .await,
        Err(AppError::AlreadyExists(_))
    ));

    let clear = codes::generate_code();
    let code = code_repo
        .insert(campaign.id, &codes::hash_code(&clear), "it", None, 1)
        .await
        .unwrap();
    assert!(
        code_repo
            .find_by_hash(&codes::hash_code(&clear))
            .await
            .unwrap()
            .is_some()
    );

    let user = Uuid::now_v7();
    let now = Utc::now();
    let enrol = |u: Uuid, code_id: Option<Uuid>| Enrolment {
        campaign_id: campaign.id,
        user_id: u.to_string(),
        enrolled_at: now,
        ends_at: Some(now + Duration::days(90)),
        source: MembershipSource::Code,
        code_id,
        trial_row: Some(EntitlementWrite {
            user_id: u.to_string(),
            source: Source::Code,
            provider_ref: format!("{}:{}", campaign.id, u),
            campaign_id: Some(campaign.id),
            starts_at: now,
            ends_at: Some(now + Duration::days(90)),
            status: EntitlementStatus::Active,
        }),
    };
    memberships.enrol(enrol(user, Some(code.id))).await.unwrap();
    // The code is spent: a second account cannot use it, and nothing was written for it.
    let other = Uuid::now_v7();
    assert!(matches!(
        memberships.enrol(enrol(other, Some(code.id))).await,
        Err(AppError::NotFound(_))
    ));
    assert!(
        memberships
            .list_for_user(&other.to_string())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        entitlements
            .list_for_user(&other.to_string())
            .await
            .unwrap()
            .is_empty()
    );
    // One membership per account per campaign.
    assert!(matches!(
        memberships.enrol(enrol(user, None)).await,
        Err(AppError::AlreadyExists(_))
    ));

    let ms = memberships.list_for_user(&user.to_string()).await.unwrap();
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0].campaign.key, key);
    let rows = entitlements.list_for_user(&user.to_string()).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].campaign_id, Some(campaign.id));
    assert_eq!(code_repo.redemptions(campaign.id).await.unwrap().len(), 1);
    assert_eq!(
        memberships
            .active_member_ids(campaign.id, now)
            .await
            .unwrap(),
        vec![user.to_string()]
    );
    assert!(
        entitlements
            .active_user_ids(now, 3, true)
            .await
            .unwrap()
            .contains(&user.to_string())
    );

    // Closing enrolment leaves the trial running; closing the campaign ends membership.
    campaigns.close_enrollment(campaign.id, now).await.unwrap();
    let c = campaigns.get(campaign.id).await.unwrap().unwrap();
    assert!(!c.accepts_enrolment(now + Duration::seconds(1)));
    assert!(c.closed_at.is_none());
    campaigns.close(campaign.id, now).await.unwrap();
    assert!(
        memberships
            .active_member_ids(campaign.id, now + Duration::seconds(1))
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        campaigns
            .list(false)
            .await
            .unwrap()
            .iter()
            .all(|c| c.key != key)
    );
    assert!(
        campaigns
            .list(true)
            .await
            .unwrap()
            .iter()
            .any(|c| c.key == key)
    );

    // Revoke, then re-enrol: the row is REVIVED (one row per account per campaign), so
    // revoking is not a one-way door out of a campaign.
    memberships
        .revoke(campaign.id, &user.to_string(), now)
        .await
        .unwrap();
    // Truncated to the microsecond the column actually keeps: comparing a nanosecond
    // `Utc::now()` against what Postgres stored is a coin toss, not a test.
    let later = (now + Duration::days(1)).trunc_subsecs(6);
    memberships
        .enrol(Enrolment {
            campaign_id: campaign.id,
            user_id: user.to_string(),
            enrolled_at: later,
            ends_at: None,
            source: MembershipSource::Admin,
            code_id: None,
            trial_row: None,
        })
        .await
        .unwrap();
    let ms = memberships.list_for_user(&user.to_string()).await.unwrap();
    assert_eq!(ms.len(), 1, "revival updates the row, never adds a second");
    assert!(ms[0].row.revoked_at.is_none());
    assert_eq!(ms[0].row.enrolled_at, later);
    assert_eq!(ms[0].row.source, MembershipSource::Admin);
    // A LIVE membership is still a conflict.
    assert!(matches!(
        memberships
            .enrol(Enrolment {
                campaign_id: campaign.id,
                user_id: user.to_string(),
                enrolled_at: later,
                ends_at: None,
                source: MembershipSource::Admin,
                code_id: None,
                trial_row: None,
            })
            .await,
        Err(AppError::AlreadyExists(_))
    ));
    memberships
        .revoke(campaign.id, &user.to_string(), later)
        .await
        .unwrap();

    // Purge.
    assert_eq!(
        code_repo.revoke_campaign(campaign.id, now).await.unwrap(),
        0
    ); // spent already
    code_repo.purge_user(&user.to_string()).await.unwrap();
    memberships.purge_user(&user.to_string()).await.unwrap();
    entitlements.purge_user(&user.to_string()).await.unwrap();
    assert!(
        memberships
            .list_for_user(&user.to_string())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) up with per-module roles"]
async fn billing_events_are_idempotent() {
    let pool = pool().await;
    let repo = PgBillingEventRepo::new(pool);
    let id = format!("evt-{}", Uuid::now_v7());
    assert!(
        repo.record_if_new(EventProvider::Revenuecat, &id, None, "sha")
            .await
            .unwrap()
    );
    assert!(
        !repo
            .record_if_new(EventProvider::Revenuecat, &id, None, "sha")
            .await
            .unwrap()
    );
    repo.mark_applied(EventProvider::Revenuecat, &id)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "needs docker compose (Postgres) up with per-module roles"]
async fn audit_entries_are_read_back_for_one_account_most_recent_first() {
    // The console asks for a reason on every plan mutation; an audit that cannot be read
    // back is a field the operator fills in for nothing.
    let pool = pool().await;
    let repo = PgAuditRepo::new(pool);
    let user = Uuid::now_v7().to_string();
    let other = Uuid::now_v7().to_string();
    let actor = Uuid::now_v7().to_string();

    for (action, target_ref, reason) in [
        ("enrol", Some("spring-trial"), "beta thanks"),
        ("revoke_membership", Some("spring-trial"), "wrong person"),
    ] {
        repo.record(AuditEntry {
            actor: actor.clone(),
            action: action.into(),
            target_user: Some(user.clone()),
            target_ref: target_ref.map(str::to_string),
            reason: reason.into(),
        })
        .await
        .unwrap();
    }
    repo.record(AuditEntry {
        actor: actor.clone(),
        action: "grant".into(),
        target_user: Some(other.clone()),
        target_ref: None,
        reason: "someone else".into(),
    })
    .await
    .unwrap();

    let entries = repo.list_for_user(&user, 50).await.unwrap();
    assert_eq!(entries.len(), 2, "only this account's entries");
    // Most recent first, and the reason survives the round trip.
    assert_eq!(entries[0].action, "revoke_membership");
    assert_eq!(entries[0].reason, "wrong person");
    assert_eq!(entries[0].target_ref.as_deref(), Some("spring-trial"));
    assert_eq!(entries[0].actor, actor);
    assert_eq!(entries[1].action, "enrol");
    assert!(entries.iter().all(|e| e.reason != "someone else"));

    // The window is honoured.
    assert_eq!(repo.list_for_user(&user, 1).await.unwrap().len(), 1);
    // An account with nothing audited is an empty list, not an error.
    assert!(
        repo.list_for_user(&Uuid::now_v7().to_string(), 50)
            .await
            .unwrap()
            .is_empty()
    );
}
