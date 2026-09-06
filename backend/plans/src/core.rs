//! Pure, host-testable decision core (design D2, D4, D13): which rows are
//! active, which plan is effective, which memberships are active, whether an
//! account may enrol, and whether a lapse still needs withdrawing.
//!
//! Every function takes `now` explicitly so the invariants are testable without
//! a clock; the service layer supplies the clock and the repos.

use crate::model::{
    BetaInfo, Campaign, CampaignKind, EntitlementRow, EntitlementStatus, Membership, Plan,
    PlanSnapshot, Source, TrialInfo,
};
use chrono::{DateTime, Duration, Utc};

/// The end past which a row stops being active, given the grace period. `None`
/// = open-ended. Billing retry is the only state that borrows the grace period.
pub fn row_effective_end(row: &EntitlementRow, grace: Duration) -> Option<DateTime<Utc>> {
    if let Some(r) = row.revoked_at {
        return Some(r);
    }
    match (row.ends_at, row.status) {
        (None, _) => None,
        (Some(e), EntitlementStatus::BillingRetry) => Some(e + grace),
        (Some(e), _) => Some(e),
    }
}

/// Active now: started, not terminal, not revoked, and before its effective end.
pub fn row_is_active(row: &EntitlementRow, now: DateTime<Utc>, grace: Duration) -> bool {
    if row.status.is_terminal() || row.revoked_at.is_some() || row.starts_at > now {
        return false;
    }
    match row_effective_end(row, grace) {
        None => true,
        Some(end) => now < end,
    }
}

/// The row that governs the effective plan: among active rows, the one with the
/// latest effective end (an open-ended row wins outright).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoverningRow<'a> {
    pub row: &'a EntitlementRow,
    /// `None` = open-ended.
    pub effective_end: Option<DateTime<Utc>>,
}

pub fn governing_row<'a>(
    rows: &'a [EntitlementRow],
    now: DateTime<Utc>,
    grace: Duration,
) -> Option<GoverningRow<'a>> {
    let mut best: Option<GoverningRow<'a>> = None;
    for row in rows.iter().filter(|r| row_is_active(r, now, grace)) {
        let end = row_effective_end(row, grace);
        let better = match (&best, end) {
            (None, _) => true,
            (Some(b), _) if b.effective_end.is_none() => false,
            (Some(_), None) => true,
            (Some(b), Some(e)) => Some(e) > b.effective_end,
        };
        if better {
            best = Some(GoverningRow {
                row,
                effective_end: end,
            });
        }
    }
    best
}

/// The active **paid-channel** row (`apple` / `google` / `web`) with the latest
/// effective end, if any — independent of which row governs the plan end. Drives
/// "managed on", the cross-channel purchase refusal and the web portal.
pub fn active_paid_row<'a>(
    rows: &'a [EntitlementRow],
    now: DateTime<Utc>,
    grace: Duration,
) -> Option<GoverningRow<'a>> {
    let paid: Vec<EntitlementRow> = rows
        .iter()
        .filter(|r| r.source.is_paid_channel())
        .cloned()
        .collect();
    let g = governing_row(&paid, now, grace)?;
    // Map back to the caller's slice (same id) so the borrow outlives `paid`.
    let row = rows.iter().find(|r| r.id == g.row.id)?;
    Some(GoverningRow {
        row,
        effective_end: g.effective_end,
    })
}

/// `Premium` iff any row is active now (spec: "premium while any row is active").
pub fn effective_plan(rows: &[EntitlementRow], now: DateTime<Utc>, grace: Duration) -> Plan {
    if governing_row(rows, now, grace).is_some() {
        Plan::Premium
    } else {
        Plan::Free
    }
}

/// A membership is active iff its campaign is not closed, its own end (if any)
/// is in the future, and it is not revoked.
pub fn membership_is_active(m: &Membership, now: DateTime<Utc>) -> bool {
    m.row.revoked_at.is_none()
        && m.campaign.closed_at.is_none_or(|c| now < c)
        && m.row.ends_at.is_none_or(|e| now < e)
}

pub fn active_memberships(memberships: &[Membership], now: DateTime<Utc>) -> Vec<&Membership> {
    memberships
        .iter()
        .filter(|m| membership_is_active(m, now))
        .collect()
}

/// End of a membership created now in `campaign` (trials: enrolled + duration).
pub fn membership_end(kind: CampaignKind, enrolled_at: DateTime<Utc>) -> Option<DateTime<Utc>> {
    match kind {
        CampaignKind::PremiumTrial { duration_days } => {
            Some(enrolled_at + Duration::days(i64::from(duration_days)))
        }
        CampaignKind::Feature => None,
    }
}

/// Why an enrolment is refused (spec `music-access-codes`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrolRefusal {
    /// Campaign closed, or its enrolment window closed.
    CampaignClosed,
    /// The account already holds a membership of this campaign.
    AlreadyMember,
    /// The account has another active premium trial.
    TrialActive,
}

/// May `user`'s existing memberships accept a new one in `campaign` now?
pub fn can_enrol(
    campaign: &Campaign,
    existing: &[Membership],
    now: DateTime<Utc>,
) -> Result<(), EnrolRefusal> {
    if !campaign.accepts_enrolment(now) {
        return Err(EnrolRefusal::CampaignClosed);
    }
    // Only a LIVE membership blocks. A revoked one must not: revoking is an admin act,
    // and with no way back, revoking would ban the account from that campaign for good —
    // recoverable only with a hand-written UPDATE. Re-enrolling someone by name is
    // explicit and audited, which is the opposite of reopening a campaign, where an
    // individually revoked member deliberately stays out.
    if existing
        .iter()
        .any(|m| m.campaign.id == campaign.id && m.row.revoked_at.is_none())
    {
        return Err(EnrolRefusal::AlreadyMember);
    }
    if campaign.kind.is_trial()
        && existing
            .iter()
            .any(|m| m.campaign.kind.is_trial() && membership_is_active(m, now))
    {
        return Err(EnrolRefusal::TrialActive);
    }
    Ok(())
}

/// Rows whose lapse still needs withdrawing (design D13): only when **no** row is
/// active, every non-open-ended row that has ended (past grace) or been revoked
/// and is not yet stamped. Empty while a row is active — including grace.
pub fn withdrawal_pending(
    rows: &[EntitlementRow],
    now: DateTime<Utc>,
    grace: Duration,
) -> Vec<&EntitlementRow> {
    if governing_row(rows, now, grace).is_some() {
        return Vec::new();
    }
    rows.iter()
        .filter(|r| r.withdrawn_at.is_none())
        .filter(|r| {
            // A refunded/revoked row has ended whatever its `ends_at` says.
            r.status.is_terminal()
                || match row_effective_end(r, grace) {
                    Some(end) => end <= now,
                    None => false,
                }
        })
        .collect()
}

/// Whether the plan of a lapsed user still needs the one-time withdrawal.
pub fn needs_withdrawal(rows: &[EntitlementRow], now: DateTime<Utc>, grace: Duration) -> bool {
    !withdrawal_pending(rows, now, grace).is_empty()
}

/// Build the app-facing snapshot from the raw rows and memberships.
pub fn snapshot(
    rows: &[EntitlementRow],
    memberships: &[Membership],
    now: DateTime<Utc>,
    grace: Duration,
) -> PlanSnapshot {
    let governing = governing_row(rows, now, grace);
    let paid_source = active_paid_row(rows, now, grace).map(|g| g.row.source);
    let active = active_memberships(memberships, now);

    let trial = active
        .iter()
        .filter(|m| m.campaign.kind.is_trial())
        .filter_map(|m| {
            m.row.ends_at.map(|e| TrialInfo {
                campaign_key: m.campaign.key.clone(),
                campaign_name: m.campaign.name.clone(),
                ends_at: e,
            })
        })
        .next();

    let betas = active
        .iter()
        .map(|m| BetaInfo {
            campaign_key: m.campaign.key.clone(),
            campaign_name: m.campaign.name.clone(),
            kind: m.campaign.kind,
            joined_at: m.row.enrolled_at,
            ends_at: m.row.ends_at,
        })
        .collect();

    match governing {
        None => PlanSnapshot {
            plan: Plan::Free,
            source: None,
            ends_at: None,
            ends_without_renewal: false,
            paid_source: None,
            trial,
            betas,
        },
        Some(g) => {
            let ends_without_renewal = match g.row.source {
                Source::Code | Source::Admin => g.effective_end.is_some(),
                Source::Apple | Source::Google | Source::Web => {
                    matches!(g.row.status, EntitlementStatus::Cancelled)
                }
            };
            // The trial info comes from the membership regardless of which row
            // governs (a paying trial tester's Apple row may be the governing one).
            PlanSnapshot {
                plan: Plan::Premium,
                source: Some(g.row.source),
                ends_at: g.effective_end,
                ends_without_renewal,
                paid_source,
                trial,
                betas,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MembershipRow, MembershipSource};
    use chrono::TimeZone;
    use uuid::Uuid;

    fn t(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, 12, 0, 0).unwrap()
    }
    const GRACE: Duration = Duration::days(3);

    fn row(source: Source, ends: Option<u32>, status: EntitlementStatus) -> EntitlementRow {
        EntitlementRow {
            id: Uuid::new_v4(),
            user_id: "u".into(),
            source,
            provider_ref: Uuid::new_v4().to_string(),
            campaign_id: None,
            starts_at: t(1),
            ends_at: ends.map(t),
            status,
            revoked_at: None,
            withdrawn_at: None,
        }
    }

    fn campaign(kind: CampaignKind, key: &str) -> Campaign {
        Campaign {
            id: Uuid::new_v4(),
            key: key.into(),
            name: key.to_uppercase(),
            kind,
            enrollment_closes_at: None,
            closed_at: None,
            created_by: "admin".into(),
            created_at: t(1),
        }
    }

    fn member(c: &Campaign, enrolled: u32) -> Membership {
        Membership {
            row: MembershipRow {
                campaign_id: c.id,
                user_id: "u".into(),
                enrolled_at: t(enrolled),
                ends_at: membership_end(c.kind, t(enrolled)),
                revoked_at: None,
                source: MembershipSource::Code,
            },
            campaign: c.clone(),
        }
    }

    #[test]
    fn no_row_is_free() {
        assert_eq!(effective_plan(&[], t(5), GRACE), Plan::Free);
        assert_eq!(snapshot(&[], &[], t(5), GRACE), PlanSnapshot::free());
    }

    #[test]
    fn any_active_row_is_premium_and_latest_end_governs() {
        let trial = row(Source::Code, Some(10), EntitlementStatus::Active);
        let apple = row(Source::Apple, Some(20), EntitlementStatus::Active);
        let rows = vec![trial.clone(), apple.clone()];
        let g = governing_row(&rows, t(5), GRACE).unwrap();
        assert_eq!(g.row.source, Source::Apple);
        assert_eq!(g.effective_end, Some(t(20)));
        // trial ended → still premium via apple, unchanged
        assert_eq!(effective_plan(&rows, t(15), GRACE), Plan::Premium);
        let s = snapshot(&rows, &[], t(15), GRACE);
        assert_eq!(s.source, Some(Source::Apple));
        assert!(!s.ends_without_renewal);
    }

    #[test]
    fn a_trial_outlasting_a_subscription_keeps_the_paid_source() {
        // Task 11.9 precedence: the trial governs the end date, but the paywall
        // must still know the account is subscribed on the App Store.
        let apple = row(Source::Apple, Some(10), EntitlementStatus::Active);
        let trial = row(Source::Code, Some(20), EntitlementStatus::Active);
        let rows = vec![apple.clone(), trial.clone()];
        let s = snapshot(&rows, &[], t(5), GRACE);
        assert_eq!(s.source, Some(Source::Code));
        assert_eq!(s.ends_at, Some(t(20)));
        assert_eq!(s.paid_source, Some(Source::Apple));
        // Once the subscription lapses, no paid source (rights carry on via the trial).
        let s = snapshot(&rows, &[], t(15), GRACE);
        assert_eq!(s.paid_source, None);
        assert_eq!(s.plan, Plan::Premium);
        // No paid row at all → None.
        assert_eq!(snapshot(&[trial], &[], t(5), GRACE).paid_source, None);
    }

    #[test]
    fn open_ended_admin_row_wins_and_never_ends() {
        let rows = vec![
            row(Source::Apple, Some(20), EntitlementStatus::Active),
            row(Source::Admin, None, EntitlementStatus::Active),
        ];
        let g = governing_row(&rows, t(25), GRACE).unwrap();
        assert_eq!(g.row.source, Source::Admin);
        assert_eq!(g.effective_end, None);
        assert!(withdrawal_pending(&rows, t(25), GRACE).is_empty());
    }

    #[test]
    fn grace_only_for_billing_retry() {
        let mut r = row(Source::Google, Some(10), EntitlementStatus::BillingRetry);
        assert!(row_is_active(&r, t(12), GRACE));
        assert!(!row_is_active(&r, t(13), GRACE));
        r.status = EntitlementStatus::Cancelled;
        assert!(!row_is_active(&r, t(11), GRACE));
        r.status = EntitlementStatus::Active;
        assert!(row_is_active(&r, t(9), GRACE));
        assert!(!row_is_active(&r, t(10), GRACE));
    }

    #[test]
    fn terminal_and_revoked_and_future_rows_are_inactive() {
        let mut r = row(Source::Web, Some(30), EntitlementStatus::Refunded);
        assert!(!row_is_active(&r, t(5), GRACE));
        r.status = EntitlementStatus::Active;
        r.revoked_at = Some(t(4));
        assert!(!row_is_active(&r, t(5), GRACE));
        assert_eq!(row_effective_end(&r, GRACE), Some(t(4)));
        r.revoked_at = None;
        r.starts_at = t(6);
        assert!(!row_is_active(&r, t(5), GRACE));
    }

    #[test]
    fn snapshot_flags_ends_without_renewal() {
        let trial = row(Source::Code, Some(10), EntitlementStatus::Active);
        assert!(snapshot(&[trial], &[], t(5), GRACE).ends_without_renewal);
        let cancelled = row(Source::Apple, Some(10), EntitlementStatus::Cancelled);
        assert!(snapshot(&[cancelled], &[], t(5), GRACE).ends_without_renewal);
        let renewing = row(Source::Apple, Some(10), EntitlementStatus::Active);
        assert!(!snapshot(&[renewing], &[], t(5), GRACE).ends_without_renewal);
        let comp = row(Source::Admin, Some(10), EntitlementStatus::Active);
        assert!(snapshot(&[comp], &[], t(5), GRACE).ends_without_renewal);
    }

    #[test]
    fn membership_activity_and_trial_end() {
        let trial = campaign(CampaignKind::PremiumTrial { duration_days: 90 }, "trial");
        let m = member(&trial, 1);
        assert_eq!(m.row.ends_at, Some(t(1) + Duration::days(90)));
        assert!(membership_is_active(&m, t(30)));
        assert!(!membership_is_active(&m, t(1) + Duration::days(90)));

        let mut feat = campaign(CampaignKind::Feature, "midi-drums");
        let mut fm = member(&feat, 2);
        assert_eq!(fm.row.ends_at, None);
        assert!(membership_is_active(&fm, t(31)));
        feat.closed_at = Some(t(20));
        fm.campaign = feat.clone();
        assert!(!membership_is_active(&fm, t(21)));
        fm.campaign.closed_at = None;
        fm.row.revoked_at = Some(t(3));
        assert!(!membership_is_active(&fm, t(4)));
    }

    #[test]
    fn enrolment_rules() {
        let mut trial = campaign(CampaignKind::PremiumTrial { duration_days: 90 }, "t1");
        let trial2 = campaign(CampaignKind::PremiumTrial { duration_days: 30 }, "t2");
        let feat = campaign(CampaignKind::Feature, "f");
        assert_eq!(can_enrol(&trial, &[], t(2)), Ok(()));
        let m = member(&trial, 2);
        assert_eq!(
            can_enrol(&trial, std::slice::from_ref(&m), t(3)),
            Err(EnrolRefusal::AlreadyMember)
        );
        // A revoked membership does not bar re-enrolment: an admin may put the account
        // back in by name (revoking must not be a one-way door).
        let mut revoked = m.clone();
        revoked.row.revoked_at = Some(t(3));
        assert_eq!(
            can_enrol(&trial, std::slice::from_ref(&revoked), t(4)),
            Ok(())
        );
        // ...and it no longer counts as a running trial either.
        assert_eq!(
            can_enrol(&trial2, std::slice::from_ref(&revoked), t(4)),
            Ok(())
        );
        assert_eq!(
            can_enrol(&trial2, std::slice::from_ref(&m), t(3)),
            Err(EnrolRefusal::TrialActive)
        );
        // a feature beta is fine alongside a running trial
        assert_eq!(can_enrol(&feat, std::slice::from_ref(&m), t(3)), Ok(()));
        // once the trial is over, another trial is allowed
        assert_eq!(
            can_enrol(&trial2, std::slice::from_ref(&m), t(2) + Duration::days(91)),
            Ok(())
        );
        trial.enrollment_closes_at = Some(t(2));
        assert_eq!(
            can_enrol(&trial, &[], t(3)),
            Err(EnrolRefusal::CampaignClosed)
        );
        let mut closed = feat.clone();
        closed.closed_at = Some(t(1));
        assert_eq!(
            can_enrol(&closed, &[], t(3)),
            Err(EnrolRefusal::CampaignClosed)
        );
    }

    #[test]
    fn withdrawal_only_after_lapse_past_grace_and_once() {
        let mut r = row(Source::Code, Some(10), EntitlementStatus::Active);
        // running: nothing
        assert!(withdrawal_pending(std::slice::from_ref(&r), t(5), GRACE).is_empty());
        // ended: pending
        assert!(needs_withdrawal(std::slice::from_ref(&r), t(11), GRACE));
        // stamped: not pending any more
        r.withdrawn_at = Some(t(11));
        assert!(!needs_withdrawal(std::slice::from_ref(&r), t(12), GRACE));

        // billing retry within grace: nothing; past grace: pending
        let g = row(Source::Google, Some(10), EntitlementStatus::BillingRetry);
        assert!(!needs_withdrawal(std::slice::from_ref(&g), t(12), GRACE));
        assert!(needs_withdrawal(std::slice::from_ref(&g), t(14), GRACE));

        // ended trial but a paid row is active: nothing (paid row untouched)
        let trial = row(Source::Code, Some(10), EntitlementStatus::Active);
        let apple = row(Source::Apple, Some(20), EntitlementStatus::Active);
        assert!(withdrawal_pending(&[trial, apple], t(15), GRACE).is_empty());

        // revoked admin grant: pending immediately
        let mut adm = row(Source::Admin, None, EntitlementStatus::Revoked);
        adm.revoked_at = Some(t(8));
        assert!(needs_withdrawal(std::slice::from_ref(&adm), t(9), GRACE));
        // refunded store row: pending even if its ends_at is in the future
        let refunded = row(Source::Apple, Some(30), EntitlementStatus::Refunded);
        assert!(needs_withdrawal(
            std::slice::from_ref(&refunded),
            t(9),
            GRACE
        ));
    }

    #[test]
    fn snapshot_carries_trial_and_betas() {
        let trial_c = campaign(CampaignKind::PremiumTrial { duration_days: 90 }, "trial");
        let feat_c = campaign(CampaignKind::Feature, "midi-drums");
        let mut trial_row = row(Source::Code, None, EntitlementStatus::Active);
        trial_row.ends_at = Some(t(1) + Duration::days(90));
        trial_row.campaign_id = Some(trial_c.id);
        let ms = vec![member(&trial_c, 1), member(&feat_c, 2)];
        let s = snapshot(std::slice::from_ref(&trial_row), &ms, t(5), GRACE);
        assert_eq!(s.plan, Plan::Premium);
        assert_eq!(s.source, Some(Source::Code));
        assert!(s.ends_without_renewal);
        assert_eq!(s.trial.as_ref().unwrap().campaign_key, "trial");
        assert_eq!(
            s.beta_keys(),
            vec!["trial".to_string(), "midi-drums".to_string()]
        );
        // feature-beta member on free stays free but keeps the membership
        let s2 = snapshot(&[], &ms[1..], t(5), GRACE);
        assert_eq!(s2.plan, Plan::Free);
        assert_eq!(s2.beta_keys(), vec!["midi-drums".to_string()]);
        assert!(s2.trial.is_none());
    }
}
