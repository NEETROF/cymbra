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

//! Curation-rewards orchestration (change: add-curation-rewards).
//!
//! [`CurationRewardsModule`] composes the pure math ([`crate::curation_rewards_core`])
//! over the storage port ([`CurationRewardsRepo`]) into the feature's behaviour:
//! record engagement + award coverage on rating (design D1/D4), settle honesty on
//! a moderator decision and on a community-consensus freeze (D2, idempotent + no
//! self-settlement + never-negative floor + moderator-outweighs-consensus), derive
//! lifetime/balance/level + curator metrics (D5/D7), grant milestone badges, and
//! run the reward shop (list/redeem, charged once, never negative). It implements
//! [`CurationRewardsSink`] so the score module drives it without knowing the
//! internals. All I/O is behind the repo, so this whole file is host-tested with
//! the in-memory fake.

use std::sync::Arc;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};

use crate::badges_core::{BadgeFamily, earned_curation_badges, family_badges};
use crate::curation_rewards::{
    CurationRewardsRepo, CurationRewardsSink, CuratorMetrics, GrantKind, LedgerEntry,
    SettleOutcome, ShopItem,
};
use crate::curation_rewards_core::{
    AwardKind, RewardConfig, SettlementSource, Truth, consensus_truth, coverage_award,
    coverage_base, honesty_award, level_progress, moderator_truth,
};
use crate::play_rewards_core::{PlayRewardConfig, meets_floor, performance_award, practice_award};

/// A user's full curation-rewards standing for the profile RPC.
#[derive(Debug, Clone, PartialEq)]
pub struct CuratorRewards {
    pub lifetime_points: i64,
    pub spendable_balance: i64,
    pub level: i64,
    /// Lifetime that started the current level (the progress-bar floor).
    pub level_floor: i64,
    /// Lifetime needed for the next level (the progress-bar ceiling).
    pub next_level_at: i64,
    pub metrics: CuratorMetrics,
    /// The CURATION badge keys the user has earned. Deprecated on the wire (design
    /// D8): the full cross-domain grid is read through `GetAchievements`, and this
    /// stays populated only so an app version already in users' hands keeps
    /// rendering the grid it knows.
    pub earned_badges: Vec<String>,
    /// The most recent ledger entries (the activity feed).
    pub recent: Vec<LedgerEntry>,
}

/// The result of a redemption attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedeemResult {
    /// The reward key that was (or already was) granted.
    pub key: String,
    /// The caller's spendable balance after the attempt.
    pub new_balance: i64,
    /// Whether the reward is now owned (always true on success, incl. idempotent re-redeem).
    pub owned: bool,
}

/// How many recent ledger entries the profile activity feed carries.
const RECENT_LIMIT: i64 = 20;

/// The curation-rewards service over the ledger + settlement repo.
pub struct CurationRewardsModule {
    repo: Arc<dyn CurationRewardsRepo>,
    config: RewardConfig,
    play_config: PlayRewardConfig,
}

impl CurationRewardsModule {
    /// Build with the design's default configuration.
    pub fn new(repo: Arc<dyn CurationRewardsRepo>) -> Self {
        Self {
            repo,
            config: RewardConfig::default(),
            play_config: PlayRewardConfig::default(),
        }
    }

    /// Override the reward configuration (server tuning / tests).
    pub fn with_config(mut self, config: RewardConfig) -> Self {
        self.config = config;
        self
    }

    /// Override the PLAY reward configuration (change: add-play-rewards). Separate
    /// from [`Self::with_config`] because the two economies answer different
    /// questions and must be retunable without one being a regression risk for the
    /// other (design D1).
    pub fn with_play_config(mut self, config: PlayRewardConfig) -> Self {
        self.play_config = config;
        self
    }

    /// The active configuration (for the module's own tests / callers that surface costs).
    pub fn config(&self) -> &RewardConfig {
        &self.config
    }

    /// The active play-reward configuration.
    pub fn play_config(&self) -> &PlayRewardConfig {
        &self.play_config
    }

    // --- profile reads -----------------------------------------------------

    /// The signed-in user's full standing: lifetime/balance/level + progress,
    /// curator metrics, earned badges, and the recent-activity feed. Grants any
    /// newly-earned badges first (idempotent) so the grid is always current.
    pub async fn get_rewards(&self, user_id: &str) -> Result<CuratorRewards> {
        let metrics = self.metrics(user_id).await?;
        self.grant_due_badges(user_id, &metrics).await?;

        let lifetime = self.repo.lifetime_points(user_id).await?;
        let balance = self.repo.spendable_balance(user_id).await?;
        let (level, level_floor, next_level_at) = level_progress(lifetime, &self.config);
        let granted = self.repo.granted_keys(user_id).await?;
        let earned_badges = family_badges(BadgeFamily::Curation)
            .map(|b| b.key)
            .filter(|k| granted.contains(*k))
            .map(str::to_string)
            .collect();
        let recent = self.repo.recent_awards(user_id, RECENT_LIMIT).await?;
        Ok(CuratorRewards {
            lifetime_points: lifetime,
            spendable_balance: balance,
            level,
            level_floor,
            next_level_at,
            metrics,
            earned_badges,
            recent,
        })
    }

    /// Per-curator metrics (self-view + the back-office reliability indicator).
    pub async fn metrics(&self, user_id: &str) -> Result<CuratorMetrics> {
        self.repo
            .curator_metrics(
                user_id,
                self.config.honesty_floor,
                coverage_base(0, &self.config),
            )
            .await
    }

    // --- reward shop -------------------------------------------------------

    /// The reward-shop catalogue for the caller (priced/coming-soon SoundFonts with
    /// ownership resolved).
    pub async fn list_shop(&self, user_id: &str) -> Result<Vec<ShopItem>> {
        self.repo.shop_items(user_id).await
    }

    /// Redeem a reward by key: refuse a coming-soon item or an unaffordable one;
    /// otherwise grant it once and deduct its cost. Idempotent — a retried redeem
    /// (already owned) charges nothing and reports success. Never goes negative.
    pub async fn redeem(&self, user_id: &str, key: &str) -> Result<RedeemResult> {
        let item = self
            .repo
            .shop_item(user_id, key)
            .await?
            .ok_or_else(|| AppError::NotFound("reward not found".into()))?;
        if !item.redeemable {
            return Err(AppError::FailedPrecondition(
                "this reward is not available yet".into(),
            ));
        }
        // Already owned (a prior grant, or a free/default font): idempotent success,
        // no charge.
        if item.owned {
            let balance = self.repo.spendable_balance(user_id).await?;
            return Ok(RedeemResult {
                key: key.to_string(),
                new_balance: balance,
                owned: true,
            });
        }
        let balance = self.repo.spendable_balance(user_id).await?;
        if balance < item.point_cost {
            return Err(AppError::FailedPrecondition(format!(
                "not enough points: {} needed, {} available",
                item.point_cost, balance
            )));
        }
        // Grant-once is the charge guard: only the call that inserts the grant
        // appends the (negative) redeem event, so a concurrent double-submit charges
        // exactly once.
        let newly = self
            .repo
            .insert_grant(user_id, key, GrantKind::Reward)
            .await?;
        if newly && item.point_cost > 0 {
            self.repo
                .append_redeem(user_id, key, item.point_cost)
                .await?;
        }
        let new_balance = self.repo.spendable_balance(user_id).await?;
        Ok(RedeemResult {
            key: key.to_string(),
            new_balance,
            owned: true,
        })
    }

    // --- consensus settlement sweep (worker) -------------------------------

    /// Settle honesty by community consensus (design D2 / task 3.2): for every score
    /// past the consensus minimum that is not yet frozen, freeze its aggregate as
    /// truth (idempotent) and settle each still-unsettled rating against it. A
    /// moderator settlement already made is untouched (consensus never overrides a
    /// moderator). Idempotent + at-least-once safe. Returns ratings settled.
    pub async fn run_consensus_sweep(&self) -> Result<u64> {
        let candidates = self
            .repo
            .consensus_candidates(self.config.consensus_min_raters)
            .await?;
        let mut settled = 0u64;
        for c in candidates {
            let truth = consensus_truth(c.avg_effective, &self.config);
            let truth_positive = match truth {
                Truth::Positive => Some(true),
                Truth::Negative => Some(false),
                Truth::Ambiguous => None,
            };
            let newly = self
                .repo
                .freeze_consensus(
                    &c.catalog_score_id,
                    truth_positive,
                    c.avg_effective,
                    c.rater_count,
                )
                .await?;
            if !newly {
                continue; // another sweep froze it; its ratings are (being) settled.
            }
            let ratings = self
                .repo
                .ratings_for_consensus_settlement(&c.catalog_score_id)
                .await?;
            for r in ratings {
                if matches!(
                    self.repo
                        .mark_settled(&r.user_id, &c.catalog_score_id, SettlementSource::Consensus)
                        .await?,
                    SettleOutcome::Fresh
                ) {
                    let amount = honesty_award(
                        r.effective,
                        truth,
                        SettlementSource::Consensus,
                        &self.config,
                    );
                    if amount > 0 {
                        self.repo
                            .append_award(
                                &r.user_id,
                                &c.catalog_score_id,
                                amount,
                                AwardKind::Honesty,
                            )
                            .await?;
                    }
                    self.grant_due_badges_for(&r.user_id).await?;
                    settled += 1;
                }
            }
        }
        Ok(settled)
    }

    // --- badges ------------------------------------------------------------

    /// Grant every CURATION milestone badge the user has newly earned
    /// (idempotent). The definitions come from the badge registry, not from this
    /// module — curation contributes counters, it no longer owns badges. The other
    /// families are granted on their own read (`BadgesModule`, design D2).
    async fn grant_due_badges(&self, user_id: &str, metrics: &CuratorMetrics) -> Result<()> {
        for key in earned_curation_badges(&metrics.badge_counts()) {
            self.repo
                .insert_grant(user_id, key, GrantKind::Badge)
                .await?;
        }
        Ok(())
    }

    /// Re-read metrics and grant due badges (used after a settlement award changes
    /// the aligned count).
    async fn grant_due_badges_for(&self, user_id: &str) -> Result<()> {
        let metrics = self.metrics(user_id).await?;
        self.grant_due_badges(user_id, &metrics).await
    }
}

#[async_trait]
impl CurationRewardsSink for CurationRewardsModule {
    async fn record_engagement(&self, user_id: &str, catalog_score_id: &str) -> Result<()> {
        self.repo.record_engagement(user_id, catalog_score_id).await
    }

    async fn award_coverage(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        existing_ratings: i64,
    ) -> Result<i64> {
        // Engagement gate: no preview, no coverage points (design D4).
        if !self.repo.has_engagement(user_id, catalog_score_id).await? {
            return Ok(0);
        }
        let already_today = self.repo.coverage_today(user_id).await?;
        let amount = coverage_award(existing_ratings, already_today, &self.config);
        if amount <= 0 {
            return Ok(0);
        }
        // Once per (user, score): a re-rating never re-awards coverage.
        let inserted = self
            .repo
            .append_coverage(user_id, catalog_score_id, amount)
            .await?;
        if !inserted {
            return Ok(0);
        }
        // A first rating can newly earn a coverage/First-Note badge.
        self.grant_due_badges_for(user_id).await?;
        Ok(amount)
    }

    async fn settle_by_moderator(
        &self,
        catalog_score_id: &str,
        decider_user_id: &str,
        accepted: bool,
    ) -> Result<()> {
        let truth = moderator_truth(accepted);
        let ratings = self
            .repo
            .ratings_for_moderator_settlement(catalog_score_id, decider_user_id)
            .await?;
        for r in ratings {
            let outcome = self
                .repo
                .mark_settled(&r.user_id, catalog_score_id, SettlementSource::Moderator)
                .await?;
            let target = honesty_award(
                r.effective,
                truth,
                SettlementSource::Moderator,
                &self.config,
            );
            match outcome {
                SettleOutcome::Fresh => {
                    if target > 0 {
                        self.repo
                            .append_award(&r.user_id, catalog_score_id, target, AwardKind::Honesty)
                            .await?;
                    }
                }
                SettleOutcome::FromConsensus => {
                    // A moderator only ever tops up over the prior consensus award
                    // (never claws back → lifetime never falls); the delta is a
                    // correcting `adjustment` entry.
                    let delta = target - r.prior_honesty;
                    if delta > 0 {
                        self.repo
                            .append_award(
                                &r.user_id,
                                catalog_score_id,
                                delta,
                                AwardKind::Adjustment,
                            )
                            .await?;
                    }
                }
                SettleOutcome::Already => continue,
            }
            self.grant_due_badges_for(&r.user_id).await?;
        }
        Ok(())
    }

    async fn award_performance(
        &self,
        user_id: &str,
        piece_id: &str,
        accuracy_pct: f32,
        level: Option<String>,
        session_id: &str,
    ) -> Result<i64> {
        // Quality gate first, so a mashed or abandoned run costs no reads at all
        // on the ingest's hot path.
        if !meets_floor(accuracy_pct, &self.play_config) {
            return Ok(0);
        }
        let times_already_paid = self
            .repo
            .performance_count_for_piece(user_id, piece_id)
            .await?;
        let already_today = self.repo.performance_today(user_id).await?;
        let amount = performance_award(
            accuracy_pct,
            times_already_paid,
            level.as_deref(),
            already_today,
            &self.play_config,
        );
        if amount <= 0 {
            return Ok(0);
        }
        // Keyed on the client's session id — the ingest's own idempotency key — so
        // a re-delivered session is a no-op HERE, independently of whether the
        // session row itself was newly stored (design D4).
        let inserted = self
            .repo
            .append_play_award(
                user_id,
                AwardKind::Performance,
                amount,
                Some(piece_id),
                &performance_award_key(session_id),
            )
            .await?;
        if !inserted {
            return Ok(0);
        }
        self.grant_due_badges_for(user_id).await?;
        Ok(amount)
    }

    async fn award_practice(&self, user_id: &str, local_day: &str) -> Result<i64> {
        let amount = practice_award(false, &self.play_config);
        if amount <= 0 {
            return Ok(0);
        }
        // The player's LOCAL day is the key, so the award is once per day whatever
        // the number of sessions or laps, and a retried delivery cannot pay twice
        // (design D3/D4).
        let inserted = self
            .repo
            .append_play_award(
                user_id,
                AwardKind::Practice,
                amount,
                None,
                &practice_award_key(local_day),
            )
            .await?;
        if !inserted {
            // Already awarded for this local day.
            return Ok(practice_award(true, &self.play_config));
        }
        self.grant_due_badges_for(user_id).await?;
        Ok(amount)
    }
}

/// The ledger idempotency key for a performance award: the client's session id,
/// namespaced so it can never collide with a practice day.
fn performance_award_key(session_id: &str) -> String {
    format!("performance:{session_id}")
}

/// The ledger idempotency key for a practice award: the player's local day.
fn practice_award_key(local_day: &str) -> String {
    format!("practice:{local_day}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curation_rewards::FakeCurationRewardsRepo;
    use crate::curation_rewards_core::level_for;

    fn module() -> (CurationRewardsModule, Arc<FakeCurationRewardsRepo>) {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        (CurationRewardsModule::new(repo.clone()), repo)
    }

    // --- coverage ----------------------------------------------------------

    #[tokio::test]
    async fn coverage_requires_engagement() {
        let (m, _repo) = module();
        // No preview first → no coverage points, even for a first-rater score.
        assert_eq!(m.award_coverage("u1", "s1", 0).await.unwrap(), 0);
        // After engaging → the full first-rater band.
        m.record_engagement("u1", "s1").await.unwrap();
        assert_eq!(m.award_coverage("u1", "s1", 0).await.unwrap(), 10);
    }

    #[tokio::test]
    async fn coverage_diminishes_and_is_awarded_once() {
        let (m, repo) = module();
        m.record_engagement("u1", "s1").await.unwrap();
        // Under-covered → 10; well-covered elsewhere → less.
        assert_eq!(m.award_coverage("u1", "s1", 0).await.unwrap(), 10);
        // Re-rating the same score never re-awards (coverage-once).
        assert_eq!(m.award_coverage("u1", "s1", 0).await.unwrap(), 0);
        m.record_engagement("u1", "s2").await.unwrap();
        assert_eq!(m.award_coverage("u1", "s2", 30).await.unwrap(), 1); // 20–49 band
        assert_eq!(repo.lifetime_points("u1").await.unwrap(), 11);
    }

    #[tokio::test]
    async fn coverage_daily_cap_clamps() {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        // A tiny cap makes the clamp observable.
        let cfg = RewardConfig {
            daily_cap: 12,
            ..RewardConfig::default()
        };
        let m = CurationRewardsModule::new(repo.clone()).with_config(cfg);
        for i in 0..3 {
            let s = format!("s{i}");
            m.record_engagement("u1", &s).await.unwrap();
            m.award_coverage("u1", &s, 0).await.unwrap(); // 10, then 2 (clamped), then 0
        }
        assert_eq!(repo.coverage_today("u1").await.unwrap(), 12); // never exceeds the cap
    }

    // --- honesty: moderator settlement ------------------------------------

    #[tokio::test]
    async fn moderator_settlement_aligned_and_floor_never_negative() {
        let (m, repo) = module();
        repo.seed_rating("aligned", "s1", 4.5); // liked
        repo.seed_rating("misaligned", "s1", 1.5); // disliked
        // Moderator accepts (positive truth).
        m.settle_by_moderator("s1", "modself", true).await.unwrap();
        assert_eq!(repo.lifetime_points("aligned").await.unwrap(), 8); // full moderator bonus
        assert_eq!(repo.lifetime_points("misaligned").await.unwrap(), 1); // floor, never negative
        // Award-once: settling again does nothing.
        m.settle_by_moderator("s1", "modself", true).await.unwrap();
        assert_eq!(repo.lifetime_points("aligned").await.unwrap(), 8);
        assert_eq!(repo.ledger_len(), 2);
    }

    #[tokio::test]
    async fn moderator_never_self_settles_own_rating() {
        let (m, repo) = module();
        repo.seed_rating("mod", "s1", 5.0); // the moderator also rated it (loved)
        repo.seed_rating("other", "s1", 5.0);
        // The moderator decides; their own rating is skipped.
        m.settle_by_moderator("s1", "mod", true).await.unwrap();
        assert_eq!(repo.lifetime_points("mod").await.unwrap(), 0); // no self-settlement
        assert_eq!(repo.lifetime_points("other").await.unwrap(), 8);
    }

    // --- honesty: consensus sweep -----------------------------------------

    #[tokio::test]
    async fn consensus_sweep_settles_once_and_is_idempotent() {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        let cfg = RewardConfig {
            consensus_min_raters: 3,
            ..RewardConfig::default()
        };
        let m = CurationRewardsModule::new(repo.clone()).with_config(cfg);
        // Three raters, aggregate clearly positive.
        repo.seed_rating("a", "s1", 5.0);
        repo.seed_rating("b", "s1", 4.0);
        repo.seed_rating("c", "s1", 1.5); // the dissenter
        let n = m.run_consensus_sweep().await.unwrap();
        assert_eq!(n, 3);
        assert_eq!(repo.lifetime_points("a").await.unwrap(), 5); // aligned consensus bonus
        assert_eq!(repo.lifetime_points("b").await.unwrap(), 5);
        assert_eq!(repo.lifetime_points("c").await.unwrap(), 1); // misaligned floor
        // Re-running settles nothing new (frozen + all settled).
        assert_eq!(m.run_consensus_sweep().await.unwrap(), 0);
        assert_eq!(repo.ledger_len(), 3);
    }

    #[tokio::test]
    async fn consensus_below_minimum_does_not_settle() {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        let m = CurationRewardsModule::new(repo.clone()); // default min 8
        repo.seed_rating("a", "s1", 5.0);
        repo.seed_rating("b", "s1", 5.0);
        assert_eq!(m.run_consensus_sweep().await.unwrap(), 0);
        assert_eq!(repo.lifetime_points("a").await.unwrap(), 0);
    }

    #[tokio::test]
    async fn moderator_outweighs_and_tops_up_a_consensus_settlement() {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        let cfg = RewardConfig {
            consensus_min_raters: 2,
            ..RewardConfig::default()
        };
        let m = CurationRewardsModule::new(repo.clone()).with_config(cfg);
        repo.seed_rating("u", "s1", 4.5); // liked
        repo.seed_rating("v", "s1", 4.5);
        // Consensus settles first (positive): each aligned rater gets 5.
        m.run_consensus_sweep().await.unwrap();
        assert_eq!(repo.lifetime_points("u").await.unwrap(), 5);
        // A moderator later ACCEPTS: aligned raters top up 5 → 8 via a +3 adjustment.
        m.settle_by_moderator("s1", "mod", true).await.unwrap();
        assert_eq!(repo.lifetime_points("u").await.unwrap(), 8);
        assert_eq!(repo.lifetime_points("v").await.unwrap(), 8);
        // A second moderator decision does not re-settle (already moderator).
        m.settle_by_moderator("s1", "mod", true).await.unwrap();
        assert_eq!(repo.lifetime_points("u").await.unwrap(), 8);
    }

    #[tokio::test]
    async fn moderator_override_never_claws_back() {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        let cfg = RewardConfig {
            consensus_min_raters: 2,
            ..RewardConfig::default()
        };
        let m = CurationRewardsModule::new(repo.clone()).with_config(cfg);
        repo.seed_rating("u", "s1", 4.5); // liked
        repo.seed_rating("v", "s1", 4.5);
        m.run_consensus_sweep().await.unwrap(); // positive consensus → 5 each
        // Moderator REJECTS (negative truth): the liked raters are now misaligned
        // (target floor 1 < prior 5) → no clawback, lifetime stays 5.
        m.settle_by_moderator("s1", "mod", false).await.unwrap();
        assert_eq!(repo.lifetime_points("u").await.unwrap(), 5);
    }

    // --- reward shop -------------------------------------------------------

    #[tokio::test]
    async fn redeem_affordable_then_refuse_unaffordable() {
        let (m, repo) = module();
        repo.seed_shop_item("grand", "Grand Piano", 50, true);
        repo.seed_shop_item("premium", "Premium (soon)", 500, false);
        // Give the user 60 lifetime points (via coverage) to spend.
        repo.record_engagement("u1", "s1").await.unwrap();
        m.award_coverage("u1", "s1", 0).await.unwrap(); // +10
        repo.append_award("u1", "s2", 50, AwardKind::Honesty)
            .await
            .unwrap(); // +50 → balance 60
        // Redeem the grand: charged 50, balance 10.
        let r = m.redeem("u1", "grand").await.unwrap();
        assert!(r.owned);
        assert_eq!(r.new_balance, 10);
        // Redeeming again is idempotent (already owned, no charge).
        let again = m.redeem("u1", "grand").await.unwrap();
        assert_eq!(again.new_balance, 10);
        // The premium item cannot be redeemed yet.
        assert!(matches!(
            m.redeem("u1", "premium").await,
            Err(AppError::FailedPrecondition(_))
        ));
        // An unaffordable font is refused, balance unchanged.
        repo.seed_shop_item("concert", "Concert Grand", 300, true);
        assert!(matches!(
            m.redeem("u1", "concert").await,
            Err(AppError::FailedPrecondition(_))
        ));
        assert_eq!(repo.spendable_balance("u1").await.unwrap(), 10);
    }

    #[tokio::test]
    async fn spending_does_not_lower_level() {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        let m = CurationRewardsModule::new(repo.clone());
        repo.seed_shop_item("grand", "Grand", 60, true);
        // Earn 200 lifetime → level 2.
        repo.append_award("u1", "s1", 200, AwardKind::Honesty)
            .await
            .unwrap();
        let before = m.get_rewards("u1").await.unwrap();
        assert_eq!(before.level, level_for(200, m.config()));
        assert!(before.level >= 2);
        // Redeem: balance drops, lifetime + level unchanged.
        m.redeem("u1", "grand").await.unwrap();
        let after = m.get_rewards("u1").await.unwrap();
        assert_eq!(after.lifetime_points, 200);
        assert_eq!(after.level, before.level);
        assert_eq!(after.spendable_balance, 140);
    }

    /// Pricing is allowed before acceptance (an admin may set a font's price while it is
    /// still in review — change: add-soundfont-reward-pricing), but an unvalidated font
    /// must never reach the app: the shop is the one read path onto `music.soundfonts`
    /// that does not go through the moderation-visibility gate.
    #[tokio::test]
    async fn an_unaccepted_font_is_never_offered_or_redeemable() {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        let m = CurationRewardsModule::new(repo.clone());
        repo.seed_shop_item("grand", "Grand", 50, true); // accepted
        repo.seed_shop_item_status("draft", "Priced in review", 50, true, "pending");
        repo.seed_shop_item_status("nope", "Priced then rejected", 50, true, "rejected");
        repo.append_award("u1", "s1", 500, AwardKind::Honesty)
            .await
            .unwrap();

        // Only the accepted font is listed, however it is priced.
        let keys: Vec<String> = m
            .list_shop("u1")
            .await
            .unwrap()
            .into_iter()
            .map(|i| i.key)
            .collect();
        assert_eq!(keys, vec!["grand".to_string()]);

        // And it cannot be redeemed by key either — the redeem lookup applies the same
        // rule, so a crafted call is not-found rather than a grant.
        for key in ["draft", "nope"] {
            assert!(matches!(
                m.redeem("u1", key).await,
                Err(AppError::NotFound(_))
            ));
        }
        // Nothing was granted and nothing was charged.
        assert_eq!(repo.spendable_balance("u1").await.unwrap(), 500);
    }

    #[tokio::test]
    async fn unknown_reward_is_not_found() {
        let (m, _repo) = module();
        assert!(matches!(
            m.redeem("u1", "nope").await,
            Err(AppError::NotFound(_))
        ));
    }

    // --- badges + reliability ---------------------------------------------

    #[tokio::test]
    async fn badges_granted_at_milestones() {
        let (m, repo) = module();
        // First rating → First Note.
        repo.seed_rating("u1", "s1", 4.0);
        repo.record_engagement("u1", "s1").await.unwrap();
        m.award_coverage("u1", "s1", 0).await.unwrap();
        let rewards = m.get_rewards("u1").await.unwrap();
        assert!(rewards.earned_badges.contains(&"first_note".to_string()));
        assert!(!rewards.earned_badges.contains(&"curator_1".to_string()));
    }

    // --- play awards (change: add-play-rewards) ----------------------------

    /// A run comfortably above the default accuracy floor.
    const GOOD: f32 = 85.0;

    #[tokio::test]
    async fn a_good_run_pays_and_a_sub_floor_run_pays_nothing() {
        let (m, repo) = module();
        assert_eq!(
            m.award_performance("u1", "p1", GOOD, Some("beginner".into()), "s1")
                .await
                .unwrap(),
            8
        );
        // Below the floor: recorded as activity by the ingest, but worth nothing.
        assert_eq!(
            m.award_performance("u1", "p2", 20.0, Some("advanced".into()), "s2")
                .await
                .unwrap(),
            0
        );
        assert_eq!(repo.lifetime_points("u1").await.unwrap(), 8);
        assert_eq!(repo.ledger_len(), 1);
    }

    #[tokio::test]
    async fn a_replayed_session_pays_exactly_once() {
        // The client's outbox retries until acked, so the SAME session id arrives
        // more than once. The durable award key makes the second one a no-op.
        let (m, repo) = module();
        assert_eq!(
            m.award_performance("u1", "p1", GOOD, None, "s1")
                .await
                .unwrap(),
            8
        );
        assert_eq!(
            m.award_performance("u1", "p1", GOOD, None, "s1")
                .await
                .unwrap(),
            0
        );
        assert_eq!(repo.lifetime_points("u1").await.unwrap(), 8);
        assert_eq!(repo.ledger_len(), 1);
    }

    #[tokio::test]
    async fn the_same_piece_pays_less_each_time_and_eventually_nothing() {
        let (m, repo) = module();
        let mut paid = Vec::new();
        for n in 0..6 {
            paid.push(
                m.award_performance("u1", "p1", GOOD, Some("beginner".into()), &format!("s{n}"))
                    .await
                    .unwrap(),
            );
        }
        assert_eq!(paid, vec![8, 3, 1, 1, 0, 0]);
        // A piece the user has NOT been paid for is worth the full amount — the
        // curve is per piece, not per player.
        assert_eq!(
            m.award_performance("u1", "p2", GOOD, Some("beginner".into()), "other")
                .await
                .unwrap(),
            8
        );
        assert_eq!(repo.lifetime_points("u1").await.unwrap(), 21);
    }

    #[tokio::test]
    async fn a_harder_piece_pays_more_and_an_unleveled_one_is_not_penalised() {
        let (m, _repo) = module();
        let beginner = m
            .award_performance("u1", "easy", GOOD, Some("beginner".into()), "s1")
            .await
            .unwrap();
        let advanced = m
            .award_performance("u1", "hard", GOOD, Some("advanced".into()), "s2")
            .await
            .unwrap();
        assert!(advanced > beginner, "{advanced} should beat {beginner}");
        // A user's own upload carries no catalog level: neutral, never zero.
        let upload = m
            .award_performance("u1", "mine", GOOD, None, "s3")
            .await
            .unwrap();
        assert_eq!(upload, beginner);
        assert!(upload > 0);
    }

    #[tokio::test]
    async fn the_daily_cap_clamps_and_the_next_day_restores_the_allowance() {
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        // A tiny cap makes the clamp observable in a couple of runs.
        let cfg = PlayRewardConfig {
            daily_cap: 12,
            ..PlayRewardConfig::default()
        };
        let m = CurationRewardsModule::new(repo.clone()).with_play_config(cfg);
        // 8 for the first piece, then only the 4 that remain, then nothing.
        assert_eq!(
            m.award_performance("u1", "p1", GOOD, None, "s1")
                .await
                .unwrap(),
            8
        );
        assert_eq!(
            m.award_performance("u1", "p2", GOOD, None, "s2")
                .await
                .unwrap(),
            4
        );
        assert_eq!(
            m.award_performance("u1", "p3", GOOD, None, "s3")
                .await
                .unwrap(),
            0
        );
        assert_eq!(repo.performance_today("u1").await.unwrap(), 12); // never above the cap

        // A new day restores the allowance. The fake has no clock, so a fresh repo
        // (nothing "today") models tomorrow; the amount is the full first band
        // again because the cap, not the piece, was what stopped p3.
        let tomorrow = Arc::new(FakeCurationRewardsRepo::default());
        let m2 = CurationRewardsModule::new(tomorrow.clone()).with_play_config(PlayRewardConfig {
            daily_cap: 12,
            ..PlayRewardConfig::default()
        });
        assert_eq!(
            m2.award_performance("u1", "p3", GOOD, None, "s4")
                .await
                .unwrap(),
            8
        );
    }

    #[tokio::test]
    async fn practice_pays_once_per_local_day() {
        let (m, repo) = module();
        assert_eq!(m.award_practice("u1", "2026-08-14").await.unwrap(), 3);
        // Further practice the same local day is recorded as activity and pays
        // nothing more — whatever the number of sessions or laps.
        assert_eq!(m.award_practice("u1", "2026-08-14").await.unwrap(), 0);
        assert_eq!(m.award_practice("u1", "2026-08-14").await.unwrap(), 0);
        // The next local day pays again.
        assert_eq!(m.award_practice("u1", "2026-08-15").await.unwrap(), 3);
        // Another player's day is their own.
        assert_eq!(m.award_practice("u2", "2026-08-14").await.unwrap(), 3);
        assert_eq!(repo.lifetime_points("u1").await.unwrap(), 6);
        assert_eq!(repo.ledger_len(), 3);
    }

    #[tokio::test]
    async fn play_awards_raise_lifetime_level_and_balance_like_any_other() {
        // The point of the change: a player who never rates still progresses.
        let repo = Arc::new(FakeCurationRewardsRepo::default());
        let m = CurationRewardsModule::new(repo.clone());
        for n in 0..8 {
            m.award_performance(
                "u1",
                &format!("p{n}"),
                GOOD,
                Some("advanced".into()),
                &format!("s{n}"),
            )
            .await
            .unwrap();
        }
        m.award_practice("u1", "2026-08-14").await.unwrap();
        let rewards = m.get_rewards("u1").await.unwrap();
        assert_eq!(rewards.lifetime_points, 43); // 40 capped play + 3 practice
        assert_eq!(rewards.spendable_balance, 43);
        assert_eq!(rewards.level, level_for(43, m.config()));
        assert!(rewards.metrics.total_ratings == 0, "no rating was needed");
    }

    #[tokio::test]
    async fn reliability_alignment_rate_over_settled_only() {
        let (m, repo) = module();
        // Two settled (one aligned, one not) + one unsettled → rate 1/2.
        repo.seed_rating("u1", "s1", 4.5);
        repo.seed_rating("u1", "s2", 1.5);
        repo.seed_rating("u1", "s3", 4.5); // stays unsettled
        m.settle_by_moderator("s1", "mod", true).await.unwrap(); // aligned → 8
        m.settle_by_moderator("s2", "mod", true).await.unwrap(); // misaligned → floor 1
        let metrics = m.metrics("u1").await.unwrap();
        assert_eq!(metrics.total_ratings, 3);
        assert_eq!(metrics.settled_count, 2);
        assert_eq!(metrics.aligned_count, 1);
        assert!((metrics.alignment_rate() - 0.5).abs() < 1e-9);
    }
}
