// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Catalog daily-access gate: ports + module (change: add-score-daily-access-
//! rewards, design D1/D2/D4/D6).
//!
//! [`CatalogDailyAccess`] composes the pure rules of
//! [`crate::catalog_daily_access_core`] over three seams:
//! - [`CatalogDayAccessRepo`] — today's opened/paid set per user, the idempotent
//!   open record, the spendable balance, and — as ONE atomic operation — the
//!   day-slot charge (ledger debit under an idempotency key + the paid row);
//! - [`DailyAccessConfigSource`] — the flags in force, read per call so the quota,
//!   the cost and the kill-switch hot-reload (the music crate stays flag-free;
//!   the server implements it over the flag service like `StreakConfigSource`);
//! - [`SubscriptionSource`] — the billing seam; [`NoSubscriptions`] until then.
//!
//! The day-slot charge lives *in the port* (like the streak freeze) because the
//! debit and the paid row must both happen or neither; the in-memory fake mirrors
//! the same all-or-nothing + charge-once semantics so the module's tests prove
//! the rule without a database.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use cymbra_platform::{AppError, Result};

use crate::catalog_daily_access_core::{
    CallerKind, DailyAccessConfig, DaySlot, DayState, Open, day_slot_award_key, day_slot_decision,
    decide_open, records_open, resets_at_ms, server_day,
};

pub use crate::catalog_daily_access_core::DAY_SLOT_REWARD_KEY;

/// Where the gate's flags come from at **call** time. `staff` is whether the
/// caller holds a staff role (admin/moderator) — a `staff_only` rollout of the
/// gate reaches them first.
pub trait DailyAccessConfigSource: Send + Sync {
    fn daily_access_config(&self, staff: bool) -> DailyAccessConfig;
}

/// A fixed configuration (tests, or a deployment with no flag store).
pub struct FixedDailyAccessConfig(pub DailyAccessConfig);

impl DailyAccessConfigSource for FixedDailyAccessConfig {
    fn daily_access_config(&self, _staff: bool) -> DailyAccessConfig {
        self.0
    }
}

/// The subscription seam (design D6): a subscriber has unlimited opens. No
/// billing exists yet — [`NoSubscriptions`] answers `false`; the future billing
/// work implements this trait and nothing else in the gate changes.
#[async_trait]
pub trait SubscriptionSource: Send + Sync {
    async fn has_active_subscription(&self, user_id: &str) -> bool;
}

/// The stub in force until a billing system exists.
#[derive(Default)]
pub struct NoSubscriptions;

#[async_trait]
impl SubscriptionSource for NoSubscriptions {
    async fn has_active_subscription(&self, _user_id: &str) -> bool {
        false
    }
}

/// Storage surface of the gate. Keyed by the plain `user_id` string (no
/// cross-schema FK), like every other music-module port. `day` is always the
/// server (UTC) day the module computed.
#[async_trait]
pub trait CatalogDayAccessRepo: Send + Sync {
    /// The user's opened/paid set for `day`. No rows = an empty state, not an error.
    async fn day_state(&self, user_id: &str, day: NaiveDate) -> Result<DayState>;

    /// Record a served FREE open of `catalog_id` for `day` (idempotent: an existing
    /// row — free or paid — is left untouched, so a re-open never downgrades a paid
    /// slot and never consumes quota twice).
    async fn record_open(&self, user_id: &str, catalog_id: &str, day: NaiveDate) -> Result<()>;

    /// The user's spendable points balance (the curation-rewards ledger sum).
    async fn spendable_balance(&self, user_id: &str) -> Result<i64>;

    /// Charge `cost` points under `award_key` and record `catalog_id` as a PAID
    /// slot for `day`, atomically. Returns `false` (and writes NOTHING) when the
    /// balance no longer covers the cost. Charging is **idempotent on
    /// `award_key`**: a second call for the same key records the slot but does not
    /// debit again, so two confirmations racing each other cost one slot, not two.
    async fn spend_day_slot(
        &self,
        user_id: &str,
        catalog_id: &str,
        day: NaiveDate,
        cost: i64,
        award_key: &str,
    ) -> Result<bool>;

    /// Delete every row older than `before` (retention prune). Returns the count.
    async fn prune_before(&self, before: NaiveDate) -> Result<u64>;
}

/// The caller's access state for the current server day — what the bytes RPC
/// carries when a piece is locked and what the state read returns (design D3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessState {
    /// The gate is on for this caller.
    pub enabled: bool,
    /// The piece the caller asked for is refused (only meaningful on a bytes read).
    pub locked: bool,
    pub free_quota: u32,
    pub free_used: u32,
    /// Next server-day rollover, epoch ms.
    pub resets_at_ms: i64,
    pub day_slot_cost: i64,
    pub spendable_balance: i64,
    pub subscriber: bool,
    /// The moment the future subscription offer would be shown (design D6).
    pub upsell: bool,
    /// Every piece open for the caller today (free + paid).
    pub opened_today: Vec<String>,
    /// The subset bought with points.
    pub paid_today: Vec<String>,
}

/// What a bytes read decided (the module tells the caller which; the caller
/// serves or refuses).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenDecision {
    /// Serve the bytes (state attached so the client can refresh its chip).
    Serve(AccessState),
    /// Refuse the bytes; `state.locked` is `true`.
    Locked(AccessState),
}

impl OpenDecision {
    pub fn state(&self) -> &AccessState {
        match self {
            OpenDecision::Serve(s) | OpenDecision::Locked(s) => s,
        }
    }
}

/// Clock seam so tests pin the server day.
pub type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

/// The gate module.
pub struct CatalogDailyAccess {
    repo: Arc<dyn CatalogDayAccessRepo>,
    config: Arc<dyn DailyAccessConfigSource>,
    subscriptions: Arc<dyn SubscriptionSource>,
    clock: Clock,
}

impl CatalogDailyAccess {
    /// A gate over `repo` with the design's default configuration (gate OFF),
    /// no subscriptions and the real clock. Wire the flag-backed source with
    /// [`Self::with_config_source`].
    pub fn new(repo: Arc<dyn CatalogDayAccessRepo>) -> Self {
        Self {
            repo,
            config: Arc::new(FixedDailyAccessConfig(DailyAccessConfig::default())),
            subscriptions: Arc::new(NoSubscriptions),
            clock: Arc::new(Utc::now),
        }
    }

    /// Fix the configuration (tests, or a deployment with no flag store).
    pub fn with_config(self, cfg: DailyAccessConfig) -> Self {
        self.with_config_source(Arc::new(FixedDailyAccessConfig(cfg)))
    }

    /// Resolve the configuration per call (hot-reloadable flags).
    pub fn with_config_source(mut self, source: Arc<dyn DailyAccessConfigSource>) -> Self {
        self.config = source;
        self
    }

    /// Wire the billing seam.
    pub fn with_subscriptions(mut self, source: Arc<dyn SubscriptionSource>) -> Self {
        self.subscriptions = source;
        self
    }

    /// Pin the clock (tests).
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    fn now(&self) -> DateTime<Utc> {
        (self.clock)()
    }

    /// Classify the caller for one piece: `exempt` (back-office audience or
    /// music-scope moderator/admin) wins, then the piece's contributor, then a
    /// subscriber, else a regular player.
    pub async fn caller_kind(
        &self,
        user_id: &str,
        exempt: bool,
        proposed_by: Option<&str>,
    ) -> CallerKind {
        if exempt {
            return CallerKind::Exempt;
        }
        if proposed_by.is_some_and(|p| p == user_id) {
            return CallerKind::Contributor;
        }
        if self.subscriptions.has_active_subscription(user_id).await {
            return CallerKind::Subscriber;
        }
        CallerKind::Regular
    }

    /// Decide a player-open of `catalog_id` for `user_id` and, when it consumes a
    /// free slot, record it. The returned state already reflects the recorded open.
    pub async fn open(
        &self,
        user_id: &str,
        catalog_id: &str,
        caller: CallerKind,
        staff: bool,
    ) -> Result<OpenDecision> {
        let cfg = self.config.daily_access_config(staff);
        let now = self.now();
        let day = server_day(now);
        // Outside the gate there is nothing to read: serve, and report a state that
        // says so (an exempt/subscriber caller sees no quota chip).
        if !cfg.enabled || caller != CallerKind::Regular {
            let state = self.state_for(user_id, caller, &cfg, now, None).await?;
            return Ok(OpenDecision::Serve(state));
        }
        let mut day_state = self.repo.day_state(user_id, day).await?;
        let open = decide_open(catalog_id, &day_state, &cfg, caller);
        if records_open(open) {
            self.repo.record_open(user_id, catalog_id, day).await?;
            day_state.opened.insert(catalog_id.to_string());
        }
        let locked = matches!(open, Open::Locked { .. });
        let state = self
            .state_for(user_id, caller, &cfg, now, Some(day_state))
            .await?
            .with_locked(locked);
        Ok(if locked {
            OpenDecision::Locked(state)
        } else {
            OpenDecision::Serve(state)
        })
    }

    /// The caller's current state (the hub/library read, design D3).
    pub async fn state(
        &self,
        user_id: &str,
        caller: CallerKind,
        staff: bool,
    ) -> Result<AccessState> {
        let cfg = self.config.daily_access_config(staff);
        self.state_for(user_id, caller, &cfg, self.now(), None)
            .await
    }

    /// Buy a day-slot for `catalog_id` after the user's explicit confirmation
    /// (design D4). Returns the state after the purchase; an unaffordable slot is
    /// a typed precondition failure and writes nothing.
    pub async fn unlock_for_today(
        &self,
        user_id: &str,
        catalog_id: &str,
        caller: CallerKind,
        staff: bool,
    ) -> Result<AccessState> {
        let cfg = self.config.daily_access_config(staff);
        let now = self.now();
        let day = server_day(now);
        let day_state = if cfg.enabled && caller == CallerKind::Regular {
            self.repo.day_state(user_id, day).await?
        } else {
            DayState::default()
        };
        let balance = self.repo.spendable_balance(user_id).await?;
        match day_slot_decision(catalog_id, &day_state, &cfg, caller, balance) {
            DaySlot::AlreadyOpen => {}
            DaySlot::Insufficient { cost, balance } => {
                return Err(AppError::FailedPrecondition(format!(
                    "insufficient points: {balance} < {cost}"
                )));
            }
            DaySlot::Charge { cost } => {
                let key = day_slot_award_key(catalog_id, day);
                let charged = self
                    .repo
                    .spend_day_slot(user_id, catalog_id, day, cost, &key)
                    .await?;
                if !charged {
                    // The balance moved under us (a concurrent spend): same typed
                    // refusal, nothing written.
                    return Err(AppError::FailedPrecondition("insufficient points".into()));
                }
            }
        }
        self.state_for(user_id, caller, &cfg, now, None).await
    }

    async fn state_for(
        &self,
        user_id: &str,
        caller: CallerKind,
        cfg: &DailyAccessConfig,
        now: DateTime<Utc>,
        day_state: Option<DayState>,
    ) -> Result<AccessState> {
        let gated = cfg.enabled && caller == CallerKind::Regular;
        let day_state = match day_state {
            Some(s) => s,
            None if gated => self.repo.day_state(user_id, server_day(now)).await?,
            None => DayState::default(),
        };
        let spendable_balance = if gated {
            self.repo.spendable_balance(user_id).await?
        } else {
            0
        };
        Ok(AccessState {
            enabled: gated,
            locked: false,
            free_quota: cfg.free_quota,
            free_used: day_state.free_used(),
            resets_at_ms: resets_at_ms(now),
            day_slot_cost: cfg.day_slot_cost,
            spendable_balance,
            subscriber: caller == CallerKind::Subscriber,
            upsell: gated,
            opened_today: day_state.opened.iter().cloned().collect(),
            paid_today: day_state.paid.iter().cloned().collect(),
        })
    }
}

impl AccessState {
    fn with_locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }
}

// --- In-memory fake (tests) -------------------------------------------------

#[derive(Default)]
struct FakeState {
    /// `(user_id, day)` → opened set / paid set.
    days: HashMap<(String, NaiveDate), (BTreeSet<String>, BTreeSet<String>)>,
    balances: HashMap<String, i64>,
    /// Debits appended, newest last: `(user_id, cost, award_key)`.
    debits: Vec<(String, i64, String)>,
    /// Award keys already charged — the fake's stand-in for the ledger's partial
    /// unique index on `(user_id, award_key)`.
    charged: HashSet<(String, String)>,
}

/// In-memory [`CatalogDayAccessRepo`] for module tests (no Postgres). Mirrors the
/// adapter's atomic spend semantics: an unaffordable slot writes neither the
/// debit nor the row, and a repeated award key records without charging twice.
#[derive(Default)]
pub struct FakeCatalogDayAccessRepo {
    state: Mutex<FakeState>,
}

impl FakeCatalogDayAccessRepo {
    /// Seed a user's spendable points balance.
    pub fn seed_balance(&self, user_id: &str, balance: i64) {
        self.state
            .lock()
            .expect("day access fake lock")
            .balances
            .insert(user_id.to_string(), balance);
    }

    /// Test helper: the day-slot debits recorded so far, as `(user_id, cost, key)`.
    /// Empty proves the "no silent debit" rule held.
    pub fn debits(&self) -> Vec<(String, i64, String)> {
        self.state
            .lock()
            .expect("day access fake lock")
            .debits
            .clone()
    }
}

#[async_trait]
impl CatalogDayAccessRepo for FakeCatalogDayAccessRepo {
    async fn day_state(&self, user_id: &str, day: NaiveDate) -> Result<DayState> {
        let s = self.state.lock().expect("day access fake lock");
        Ok(s.days
            .get(&(user_id.to_string(), day))
            .map(|(opened, paid)| DayState {
                opened: opened.clone(),
                paid: paid.clone(),
            })
            .unwrap_or_default())
    }

    async fn record_open(&self, user_id: &str, catalog_id: &str, day: NaiveDate) -> Result<()> {
        let mut s = self.state.lock().expect("day access fake lock");
        let entry = s.days.entry((user_id.to_string(), day)).or_default();
        entry.0.insert(catalog_id.to_string());
        Ok(())
    }

    async fn spendable_balance(&self, user_id: &str) -> Result<i64> {
        Ok(self
            .state
            .lock()
            .expect("day access fake lock")
            .balances
            .get(user_id)
            .copied()
            .unwrap_or(0))
    }

    async fn spend_day_slot(
        &self,
        user_id: &str,
        catalog_id: &str,
        day: NaiveDate,
        cost: i64,
        award_key: &str,
    ) -> Result<bool> {
        let mut s = self.state.lock().expect("day access fake lock");
        let balance = s.balances.get(user_id).copied().unwrap_or(0);
        if balance < cost {
            return Ok(false);
        }
        let key = (user_id.to_string(), award_key.to_string());
        if s.charged.insert(key) {
            *s.balances.entry(user_id.to_string()).or_insert(0) -= cost;
            s.debits
                .push((user_id.to_string(), cost, award_key.to_string()));
        }
        let entry = s.days.entry((user_id.to_string(), day)).or_default();
        entry.0.insert(catalog_id.to_string());
        entry.1.insert(catalog_id.to_string());
        Ok(true)
    }

    async fn prune_before(&self, before: NaiveDate) -> Result<u64> {
        let mut s = self.state.lock().expect("day access fake lock");
        let n = s.days.len();
        s.days.retain(|(_, d), _| *d >= before);
        Ok((n - s.days.len()) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const P1: &str = "11111111-1111-7111-8111-111111111111";
    const P2: &str = "22222222-2222-7222-8222-222222222222";
    const P3: &str = "33333333-3333-7333-8333-333333333333";
    const P4: &str = "44444444-4444-7444-8444-444444444444";

    fn on(quota: u32, cost: i64) -> DailyAccessConfig {
        DailyAccessConfig {
            enabled: true,
            free_quota: quota,
            day_slot_cost: cost,
        }
    }

    fn at(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
    }

    fn gate(repo: Arc<FakeCatalogDayAccessRepo>, cfg: DailyAccessConfig) -> CatalogDailyAccess {
        CatalogDailyAccess::new(repo)
            .with_config(cfg)
            .with_clock(Arc::new(|| at(2026, 8, 15, 10)))
    }

    #[tokio::test]
    async fn quota_then_lock_then_reopen_free() {
        let repo = Arc::new(FakeCatalogDayAccessRepo::default());
        let g = gate(repo.clone(), on(2, 20));
        for p in [P1, P2] {
            let d = g.open("u1", p, CallerKind::Regular, false).await.unwrap();
            assert!(matches!(d, OpenDecision::Serve(_)), "{p} should serve");
        }
        let d = g.open("u1", P3, CallerKind::Regular, false).await.unwrap();
        let OpenDecision::Locked(state) = d else {
            panic!("third distinct piece must be locked")
        };
        assert!(state.locked);
        assert_eq!(state.free_used, 2);
        assert_eq!(state.free_quota, 2);
        assert_eq!(state.day_slot_cost, 20);
        assert!(state.upsell);
        assert_eq!(state.opened_today, vec![P1.to_string(), P2.to_string()]);
        // Re-opening an already-open piece stays free.
        let d = g.open("u1", P1, CallerKind::Regular, false).await.unwrap();
        assert!(matches!(d, OpenDecision::Serve(s) if s.free_used == 2));
        assert!(repo.debits().is_empty(), "no silent debit");
    }

    #[tokio::test]
    async fn gate_off_and_outside_callers_serve_without_reading_quota() {
        let repo = Arc::new(FakeCatalogDayAccessRepo::default());
        let g = gate(repo.clone(), on(0, 20));
        for k in [
            CallerKind::Exempt,
            CallerKind::Subscriber,
            CallerKind::Contributor,
        ] {
            let d = g.open("u1", P1, k, false).await.unwrap();
            let OpenDecision::Serve(s) = d else { panic!() };
            assert!(!s.enabled);
            assert!(!s.locked);
        }
        let off = CatalogDailyAccess::new(repo.clone()).with_config(DailyAccessConfig {
            enabled: false,
            ..on(0, 20)
        });
        let d = off
            .open("u1", P1, CallerKind::Regular, false)
            .await
            .unwrap();
        assert!(matches!(d, OpenDecision::Serve(s) if !s.enabled));
        // Nothing was recorded for those opens.
        assert!(
            repo.day_state("u1", NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
                .await
                .unwrap()
                .opened
                .is_empty()
        );
    }

    #[tokio::test]
    async fn unlock_charges_once_and_reopens_free_today() {
        let repo = Arc::new(FakeCatalogDayAccessRepo::default());
        repo.seed_balance("u1", 25);
        let g = gate(repo.clone(), on(0, 20));
        assert!(matches!(
            g.open("u1", P1, CallerKind::Regular, false).await.unwrap(),
            OpenDecision::Locked(_)
        ));
        let s = g
            .unlock_for_today("u1", P1, CallerKind::Regular, false)
            .await
            .unwrap();
        assert_eq!(s.spendable_balance, 5);
        assert_eq!(s.paid_today, vec![P1.to_string()]);
        assert_eq!(s.free_used, 0, "a paid slot consumes no free quota");
        // Second confirmation for the same piece + day: no second debit.
        let s2 = g
            .unlock_for_today("u1", P1, CallerKind::Regular, false)
            .await
            .unwrap();
        assert_eq!(s2.spendable_balance, 5);
        assert_eq!(repo.debits().len(), 1);
        assert_eq!(
            repo.debits()[0].2,
            format!("score_day_slot:{P1}:2026-08-15")
        );
        // And the piece now serves for the rest of the day.
        assert!(matches!(
            g.open("u1", P1, CallerKind::Regular, false).await.unwrap(),
            OpenDecision::Serve(_)
        ));
    }

    #[tokio::test]
    async fn insufficient_balance_is_a_typed_refusal_writing_nothing() {
        let repo = Arc::new(FakeCatalogDayAccessRepo::default());
        repo.seed_balance("u1", 19);
        let g = gate(repo.clone(), on(0, 20));
        let err = g
            .unlock_for_today("u1", P1, CallerKind::Regular, false)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::FailedPrecondition(_)), "{err:?}");
        assert!(repo.debits().is_empty());
        assert!(matches!(
            g.open("u1", P1, CallerKind::Regular, false).await.unwrap(),
            OpenDecision::Locked(_)
        ));
    }

    #[tokio::test]
    async fn the_same_piece_costs_again_the_next_day() {
        let repo = Arc::new(FakeCatalogDayAccessRepo::default());
        repo.seed_balance("u1", 100);
        let today = gate(repo.clone(), on(0, 20));
        today
            .unlock_for_today("u1", P4, CallerKind::Regular, false)
            .await
            .unwrap();
        let tomorrow = CatalogDailyAccess::new(repo.clone())
            .with_config(on(0, 20))
            .with_clock(Arc::new(|| at(2026, 8, 16, 1)));
        assert!(matches!(
            tomorrow
                .open("u1", P4, CallerKind::Regular, false)
                .await
                .unwrap(),
            OpenDecision::Locked(_)
        ));
        tomorrow
            .unlock_for_today("u1", P4, CallerKind::Regular, false)
            .await
            .unwrap();
        assert_eq!(repo.debits().len(), 2, "a new day, a new charge");
    }

    #[tokio::test]
    async fn state_read_reports_quota_reset_and_today_ids() {
        let repo = Arc::new(FakeCatalogDayAccessRepo::default());
        repo.seed_balance("u1", 7);
        let g = gate(repo.clone(), on(3, 20));
        g.open("u1", P2, CallerKind::Regular, false).await.unwrap();
        let s = g.state("u1", CallerKind::Regular, false).await.unwrap();
        assert!(s.enabled && !s.locked);
        assert_eq!((s.free_quota, s.free_used), (3, 1));
        assert_eq!(s.spendable_balance, 7);
        assert_eq!(s.opened_today, vec![P2.to_string()]);
        assert_eq!(s.resets_at_ms, at(2026, 8, 16, 0).timestamp_millis());
    }

    #[tokio::test]
    async fn caller_kind_precedence() {
        let repo = Arc::new(FakeCatalogDayAccessRepo::default());
        let g = gate(repo, on(3, 20));
        assert_eq!(
            g.caller_kind("u1", true, Some("u1")).await,
            CallerKind::Exempt
        );
        assert_eq!(
            g.caller_kind("u1", false, Some("u1")).await,
            CallerKind::Contributor
        );
        assert_eq!(
            g.caller_kind("u1", false, Some("u2")).await,
            CallerKind::Regular
        );
        assert_eq!(g.caller_kind("u1", false, None).await, CallerKind::Regular);
    }

    struct EveryoneSubscribed;
    #[async_trait]
    impl SubscriptionSource for EveryoneSubscribed {
        async fn has_active_subscription(&self, _user_id: &str) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn subscription_seam_lifts_the_quota() {
        let repo = Arc::new(FakeCatalogDayAccessRepo::default());
        let g = gate(repo, on(0, 20)).with_subscriptions(Arc::new(EveryoneSubscribed));
        let kind = g.caller_kind("u1", false, None).await;
        assert_eq!(kind, CallerKind::Subscriber);
        let d = g.open("u1", P1, kind, false).await.unwrap();
        assert!(matches!(d, OpenDecision::Serve(s) if s.subscriber && !s.enabled));
    }

    #[tokio::test]
    async fn prune_drops_old_days_only() {
        let repo = FakeCatalogDayAccessRepo::default();
        let d1 = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
        repo.record_open("u1", P1, d1).await.unwrap();
        repo.record_open("u1", P1, d2).await.unwrap();
        assert_eq!(repo.prune_before(d2).await.unwrap(), 1);
        assert_eq!(repo.day_state("u1", d2).await.unwrap().opened.len(), 1);
    }
}
