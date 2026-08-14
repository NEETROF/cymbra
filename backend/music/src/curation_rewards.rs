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

//! Curation-rewards data ports (change: add-curation-rewards).
//!
//! Two seams over the append-only points ledger + settlement state:
//!
//! * [`CurationRewardsRepo`] — the storage port the [`crate::CurationRewardsModule`]
//!   composes its logic over: engagement signal, the ledger (coverage/honesty/
//!   adjustment/redeem), per-rating + per-score settlement state, durable grants
//!   (redeemed rewards + earned badges), curator metrics, and the reward-shop item
//!   read. A stateful in-memory [`FakeCurationRewardsRepo`] mirrors the Postgres
//!   adapter's semantics so the whole award/settlement flow is host-testable.
//! * [`CurationRewardsSink`] — the narrow producer seam the [`crate::ScoreModule`]
//!   depends on (like `LeaderboardSink`): the rating path records engagement and
//!   awards coverage, and the moderation path settles honesty — without the score
//!   module knowing the rewards internals. `None` in the score module leaves
//!   ratings/moderation fully functional (design Rollback).

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use async_trait::async_trait;
use cymbra_platform::Result;

use crate::badges_core::BadgeCounters;
use crate::curation_rewards_core::{AwardKind, SettlementSource};

/// A durable grant's kind (mirrors the `grant_kind` CHECK in migration 0016): a
/// redeemed shop `Reward` (piano/SoundFont) or an earned `Badge`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantKind {
    Reward,
    Badge,
}

impl GrantKind {
    pub fn as_str(self) -> &'static str {
        match self {
            GrantKind::Reward => "reward",
            GrantKind::Badge => "badge",
        }
    }
}

/// The outcome of flipping one rating's settlement state ([`CurationRewardsRepo::mark_settled`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettleOutcome {
    /// Was unsettled; now settled against the given source (award the full bonus).
    Fresh,
    /// Was `consensus`-settled; upgraded to `moderator` (append the top-up adjustment).
    FromConsensus,
    /// Already settled at this (or a higher-precedence) source; nothing to do.
    Already,
}

/// One rating eligible for settlement: its rater, effective value (1–5), and the
/// honesty already awarded for it (0 if unsettled) — enough for the module to size
/// a fresh award or a moderator top-up over a prior consensus award.
#[derive(Debug, Clone, PartialEq)]
pub struct SettleableRating {
    pub user_id: String,
    pub effective: f64,
    pub prior_honesty: i64,
}

/// A score that has crossed the consensus minimum and is not yet frozen: its id,
/// the aggregate value, and the distinct-rater count (the frozen truth inputs).
#[derive(Debug, Clone, PartialEq)]
pub struct ConsensusCandidate {
    pub catalog_score_id: String,
    pub avg_effective: f64,
    pub rater_count: i64,
}

/// One append-only ledger entry, for the recent-activity feed.
#[derive(Debug, Clone, PartialEq)]
pub struct LedgerEntry {
    pub kind: AwardKind,
    pub amount: i64,
    pub catalog_score_id: Option<String>,
    pub reward_key: Option<String>,
    pub source: Option<SettlementSource>,
    pub created_at_ms: i64,
}

/// Per-curator metrics for the profile self-view + the back-office reliability
/// indicator (design D7). `alignment_rate` is derived by the module as
/// `aligned / settled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CuratorMetrics {
    /// Total ratings the user has recorded.
    pub total_ratings: i64,
    /// Ratings that earned coverage points (i.e. of under-covered scores).
    pub coverage_contribution: i64,
    /// Settled ratings that ALIGNED with the ground truth (honesty > floor).
    pub aligned_count: i64,
    /// Settled ratings (the alignment-rate denominator).
    pub settled_count: i64,
    /// Ratings where the user was the first rater (top coverage band).
    pub first_rater_count: i64,
}

impl CuratorMetrics {
    /// The share of settled ratings that matched the ground truth (0 when none settled).
    pub fn alignment_rate(&self) -> f64 {
        if self.settled_count > 0 {
            self.aligned_count as f64 / self.settled_count as f64
        } else {
            0.0
        }
    }

    /// Curation's CONTRIBUTION to the badge registry: the three milestone counters
    /// this domain owns, in the registry's counter shape. Every other field stays
    /// zero — curation knows nothing about play, ranking or learning, and only the
    /// curation subset of the registry is ever evaluated against this
    /// (`badges_core::earned_curation_badges`).
    pub fn badge_counts(&self) -> BadgeCounters {
        BadgeCounters {
            rating_count: self.total_ratings,
            aligned_count: self.aligned_count,
            first_rater_count: self.first_rater_count,
            ..Default::default()
        }
    }
}

/// One reward-shop item: a priced SoundFont from the catalog, plus whether the
/// caller already owns it (a grant, or a free/default font).
#[derive(Debug, Clone, PartialEq)]
pub struct ShopItem {
    pub key: String,
    pub label: String,
    pub instrument: String,
    pub license: String,
    pub attribution: Option<String>,
    pub point_cost: i64,
    pub redeemable: bool,
    pub owned: bool,
}

/// Storage port for the curation-rewards economy. All ops are keyed by the plain
/// `user_id` string (no cross-schema FK). Implemented by the Postgres adapter
/// ([`crate::PgCurationRewardsRepo`], coverage-excluded) and the in-memory fake.
#[async_trait]
pub trait CurationRewardsRepo: Send + Sync {
    // --- engagement signal (coverage eligibility, design D4) ---------------

    /// Record that `user` engaged with (previewed/opened) `score`; idempotent upsert.
    async fn record_engagement(&self, user_id: &str, catalog_score_id: &str) -> Result<()>;

    /// Whether `user` has an engagement row for `score`.
    async fn has_engagement(&self, user_id: &str, catalog_score_id: &str) -> Result<bool>;

    // --- coverage award ----------------------------------------------------

    /// Coverage points `user` has earned so far today (their daily-cap headroom).
    async fn coverage_today(&self, user_id: &str) -> Result<i64>;

    /// Append a coverage award once per (user, score) — `ON CONFLICT DO NOTHING`
    /// against the partial unique index. Returns `true` iff a row was inserted
    /// (`false` = already awarded, so re-rating never re-awards).
    async fn append_coverage(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        amount: i64,
    ) -> Result<bool>;

    // --- honesty settlement ------------------------------------------------

    /// Ratings of `score` a MODERATOR decision may settle — unsettled OR already
    /// `consensus`-settled (upgradeable once) — EXCLUDING `exclude_user` (the
    /// no-self-settlement guard, design D2).
    async fn ratings_for_moderator_settlement(
        &self,
        catalog_score_id: &str,
        exclude_user: &str,
    ) -> Result<Vec<SettleableRating>>;

    /// Unsettled ratings of `score` a CONSENSUS freeze settles (settled_at IS NULL).
    async fn ratings_for_consensus_settlement(
        &self,
        catalog_score_id: &str,
    ) -> Result<Vec<SettleableRating>>;

    /// Flip a rating's settlement state to `source`, returning the [`SettleOutcome`].
    /// A `moderator` source settles an unsettled rating (`Fresh`) or upgrades a
    /// `consensus` one (`FromConsensus`); a `consensus` source only settles an
    /// unsettled rating (`Fresh`), never touching a settled one — the award-once +
    /// single-override guarantee (design D2/3.3).
    async fn mark_settled(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        source: SettlementSource,
    ) -> Result<SettleOutcome>;

    /// Append a honesty or adjustment award to the ledger.
    async fn append_award(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        amount: i64,
        kind: AwardKind,
    ) -> Result<()>;

    // --- consensus freeze --------------------------------------------------

    /// Scores with at least `min_raters` distinct raters that are NOT yet frozen —
    /// the consensus sweep's work list.
    async fn consensus_candidates(&self, min_raters: i64) -> Result<Vec<ConsensusCandidate>>;

    /// Freeze `score`'s community truth (idempotent by PK). Returns `true` iff this
    /// call inserted the freeze (so the caller settles its ratings exactly once).
    async fn freeze_consensus(
        &self,
        catalog_score_id: &str,
        truth_positive: Option<bool>,
        avg_effective: f64,
        rater_count: i64,
    ) -> Result<bool>;

    // --- balances / activity ----------------------------------------------

    /// Lifetime earned points: the sum of all NON-`redeem` ledger amounts (only rises).
    async fn lifetime_points(&self, user_id: &str) -> Result<i64>;

    /// Spendable balance: the sum of ALL ledger amounts (awards − redemptions).
    async fn spendable_balance(&self, user_id: &str) -> Result<i64>;

    /// The user's most recent ledger entries, newest first (the activity feed).
    async fn recent_awards(&self, user_id: &str, limit: i64) -> Result<Vec<LedgerEntry>>;

    // --- grants / redemption ----------------------------------------------

    /// The keys the user has been granted (redeemed rewards + earned badges).
    async fn granted_keys(&self, user_id: &str) -> Result<HashSet<String>>;

    /// Insert a grant once — `ON CONFLICT (user_id, key) DO NOTHING`. Returns `true`
    /// iff newly granted (the redemption/badge idempotency guard).
    async fn insert_grant(&self, user_id: &str, key: &str, kind: GrantKind) -> Result<bool>;

    /// Append a `redeem` ledger event (negative `cost`) for a granted reward.
    async fn append_redeem(&self, user_id: &str, key: &str, cost: i64) -> Result<()>;

    // --- metrics / shop ----------------------------------------------------

    /// Per-curator metrics. `honesty_floor` classifies aligned (award > floor) vs
    /// misaligned settled ratings; `first_rater_points` (the top coverage band)
    /// identifies first-rater coverage.
    async fn curator_metrics(
        &self,
        user_id: &str,
        honesty_floor: i64,
        first_rater_points: i64,
    ) -> Result<CuratorMetrics>;

    /// The reward-shop catalogue for `user`: every priced/coming-soon SoundFont
    /// with the caller's ownership resolved. Free (cost 0) fonts are `owned` for
    /// everyone (available as today); costed fonts are `owned` iff granted.
    async fn shop_items(&self, user_id: &str) -> Result<Vec<ShopItem>>;

    /// One shop item by key, with the caller's ownership resolved (`None` = unknown key).
    async fn shop_item(&self, user_id: &str, key: &str) -> Result<Option<ShopItem>>;
}

/// The narrow producer seam the score module depends on. The rewards module
/// implements it; the score module calls it on the rating + moderation paths.
#[async_trait]
pub trait CurationRewardsSink: Send + Sync {
    /// Record that `user` previewed/opened `score` (the coverage engagement gate).
    async fn record_engagement(&self, user_id: &str, catalog_score_id: &str) -> Result<()>;

    /// Award coverage points for `user`'s fresh rating of `score`, given the score's
    /// `existing_ratings` count BEFORE this rating. Diminishing + daily-capped +
    /// engagement-gated + once-per-(user, score). Returns the points actually
    /// awarded (0 = gated out / capped / already awarded).
    async fn award_coverage(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        existing_ratings: i64,
    ) -> Result<i64>;

    /// Settle honesty for every OTHER rater of `score` against a moderator's
    /// accept/reject decision (skips `decider_user_id` — no self-settlement).
    async fn settle_by_moderator(
        &self,
        catalog_score_id: &str,
        decider_user_id: &str,
        accepted: bool,
    ) -> Result<()>;
}

// -------------------------------------------------------------------------
// In-memory fake (stateful) for host tests.
// -------------------------------------------------------------------------

#[derive(Clone)]
struct FakeRating {
    user_id: String,
    catalog_score_id: String,
    effective: f64,
    settled_source: Option<SettlementSource>,
    /// Whether this rating earned the top coverage band (was the first rater).
    first_rater: bool,
}

#[derive(Clone)]
struct FakeLedger {
    user_id: String,
    kind: AwardKind,
    amount: i64,
    catalog_score_id: Option<String>,
    reward_key: Option<String>,
    source: Option<SettlementSource>,
    seq: i64,
}

#[derive(Clone)]
struct FakeConsensus {
    catalog_score_id: String,
}

#[derive(Clone)]
struct FakeShopRow {
    key: String,
    label: String,
    instrument: String,
    license: String,
    attribution: Option<String>,
    point_cost: i64,
    redeemable: bool,
}

#[derive(Default)]
struct FakeState {
    engagements: HashSet<(String, String)>,
    ratings: Vec<FakeRating>,
    ledger: Vec<FakeLedger>,
    consensus: Vec<FakeConsensus>,
    grants: HashMap<(String, String), GrantKind>,
    shop: Vec<FakeShopRow>,
    seq: i64,
}

/// Stateful in-memory [`CurationRewardsRepo`] for module tests. Mirrors the
/// Postgres adapter's coverage-once, settlement-precedence, and grant-once
/// semantics so the award/settlement flow is proven without a database.
#[derive(Default)]
pub struct FakeCurationRewardsRepo {
    state: Mutex<FakeState>,
}

impl FakeCurationRewardsRepo {
    /// Seed a rating (as if `SubmitScoreRating` recorded it) for settlement tests.
    pub fn seed_rating(&self, user_id: &str, catalog_score_id: &str, effective: f64) {
        self.seed_rating_flagged(user_id, catalog_score_id, effective, false);
    }

    /// Seed a rating, marking whether the user was the first rater (top coverage band).
    pub fn seed_rating_flagged(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        effective: f64,
        first_rater: bool,
    ) {
        self.state
            .lock()
            .expect("rewards fake lock")
            .ratings
            .push(FakeRating {
                user_id: user_id.to_string(),
                catalog_score_id: catalog_score_id.to_string(),
                effective,
                settled_source: None,
                first_rater,
            });
    }

    /// Seed a reward-shop item (a priced/coming-soon SoundFont).
    pub fn seed_shop_item(&self, key: &str, label: &str, point_cost: i64, redeemable: bool) {
        self.state
            .lock()
            .expect("rewards fake lock")
            .shop
            .push(FakeShopRow {
                key: key.to_string(),
                label: label.to_string(),
                instrument: "piano".into(),
                license: "CC0-1.0".into(),
                attribution: None,
                point_cost,
                redeemable,
            });
    }

    /// Total ledger rows (test assertions on award count).
    pub fn ledger_len(&self) -> usize {
        self.state.lock().expect("rewards fake lock").ledger.len()
    }

    fn next_seq(st: &mut FakeState) -> i64 {
        st.seq += 1;
        st.seq
    }
}

#[async_trait]
impl CurationRewardsRepo for FakeCurationRewardsRepo {
    async fn record_engagement(&self, user_id: &str, catalog_score_id: &str) -> Result<()> {
        self.state
            .lock()
            .expect("rewards fake lock")
            .engagements
            .insert((user_id.to_string(), catalog_score_id.to_string()));
        Ok(())
    }

    async fn has_engagement(&self, user_id: &str, catalog_score_id: &str) -> Result<bool> {
        Ok(self
            .state
            .lock()
            .expect("rewards fake lock")
            .engagements
            .contains(&(user_id.to_string(), catalog_score_id.to_string())))
    }

    async fn coverage_today(&self, user_id: &str) -> Result<i64> {
        // The fake has no clock, so "today" is all of the user's coverage awards.
        let st = self.state.lock().expect("rewards fake lock");
        Ok(st
            .ledger
            .iter()
            .filter(|l| l.user_id == user_id && l.kind == AwardKind::Coverage)
            .map(|l| l.amount)
            .sum())
    }

    async fn append_coverage(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        amount: i64,
    ) -> Result<bool> {
        let mut st = self.state.lock().expect("rewards fake lock");
        let already = st.ledger.iter().any(|l| {
            l.user_id == user_id
                && l.kind == AwardKind::Coverage
                && l.catalog_score_id.as_deref() == Some(catalog_score_id)
        });
        if already {
            return Ok(false);
        }
        let seq = Self::next_seq(&mut st);
        st.ledger.push(FakeLedger {
            user_id: user_id.to_string(),
            kind: AwardKind::Coverage,
            amount,
            catalog_score_id: Some(catalog_score_id.to_string()),
            reward_key: None,
            source: None,
            seq,
        });
        Ok(true)
    }

    async fn ratings_for_moderator_settlement(
        &self,
        catalog_score_id: &str,
        exclude_user: &str,
    ) -> Result<Vec<SettleableRating>> {
        let st = self.state.lock().expect("rewards fake lock");
        Ok(st
            .ratings
            .iter()
            .filter(|r| {
                r.catalog_score_id == catalog_score_id
                    && r.user_id != exclude_user
                    // unsettled OR consensus-settled (upgradeable)
                    && !matches!(r.settled_source, Some(SettlementSource::Moderator))
            })
            .map(|r| SettleableRating {
                user_id: r.user_id.clone(),
                effective: r.effective,
                prior_honesty: honesty_for(&st, &r.user_id, catalog_score_id),
            })
            .collect())
    }

    async fn ratings_for_consensus_settlement(
        &self,
        catalog_score_id: &str,
    ) -> Result<Vec<SettleableRating>> {
        let st = self.state.lock().expect("rewards fake lock");
        Ok(st
            .ratings
            .iter()
            .filter(|r| r.catalog_score_id == catalog_score_id && r.settled_source.is_none())
            .map(|r| SettleableRating {
                user_id: r.user_id.clone(),
                effective: r.effective,
                prior_honesty: 0,
            })
            .collect())
    }

    async fn mark_settled(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        source: SettlementSource,
    ) -> Result<SettleOutcome> {
        let mut st = self.state.lock().expect("rewards fake lock");
        let Some(r) = st
            .ratings
            .iter_mut()
            .find(|r| r.user_id == user_id && r.catalog_score_id == catalog_score_id)
        else {
            return Ok(SettleOutcome::Already);
        };
        match (r.settled_source, source) {
            (None, s) => {
                r.settled_source = Some(s);
                Ok(SettleOutcome::Fresh)
            }
            (Some(SettlementSource::Consensus), SettlementSource::Moderator) => {
                r.settled_source = Some(SettlementSource::Moderator);
                Ok(SettleOutcome::FromConsensus)
            }
            _ => Ok(SettleOutcome::Already),
        }
    }

    async fn append_award(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        amount: i64,
        kind: AwardKind,
    ) -> Result<()> {
        let mut st = self.state.lock().expect("rewards fake lock");
        // The settlement source, for the activity feed, mirrors the rating's state.
        let source = st
            .ratings
            .iter()
            .find(|r| r.user_id == user_id && r.catalog_score_id == catalog_score_id)
            .and_then(|r| r.settled_source);
        let seq = Self::next_seq(&mut st);
        st.ledger.push(FakeLedger {
            user_id: user_id.to_string(),
            kind,
            amount,
            catalog_score_id: Some(catalog_score_id.to_string()),
            reward_key: None,
            source,
            seq,
        });
        Ok(())
    }

    async fn consensus_candidates(&self, min_raters: i64) -> Result<Vec<ConsensusCandidate>> {
        let st = self.state.lock().expect("rewards fake lock");
        // Group ratings by score.
        let mut by_score: HashMap<String, Vec<f64>> = HashMap::new();
        for r in &st.ratings {
            by_score
                .entry(r.catalog_score_id.clone())
                .or_default()
                .push(r.effective);
        }
        let frozen: HashSet<&str> = st
            .consensus
            .iter()
            .map(|c| c.catalog_score_id.as_str())
            .collect();
        let mut out: Vec<ConsensusCandidate> = by_score
            .into_iter()
            .filter(|(id, vals)| vals.len() as i64 >= min_raters && !frozen.contains(id.as_str()))
            .map(|(id, vals)| {
                let avg = vals.iter().sum::<f64>() / vals.len() as f64;
                ConsensusCandidate {
                    catalog_score_id: id,
                    avg_effective: avg,
                    rater_count: vals.len() as i64,
                }
            })
            .collect();
        out.sort_by(|a, b| a.catalog_score_id.cmp(&b.catalog_score_id));
        Ok(out)
    }

    async fn freeze_consensus(
        &self,
        catalog_score_id: &str,
        _truth_positive: Option<bool>,
        _avg_effective: f64,
        _rater_count: i64,
    ) -> Result<bool> {
        let mut st = self.state.lock().expect("rewards fake lock");
        if st
            .consensus
            .iter()
            .any(|c| c.catalog_score_id == catalog_score_id)
        {
            return Ok(false);
        }
        st.consensus.push(FakeConsensus {
            catalog_score_id: catalog_score_id.to_string(),
        });
        Ok(true)
    }

    async fn lifetime_points(&self, user_id: &str) -> Result<i64> {
        let st = self.state.lock().expect("rewards fake lock");
        Ok(st
            .ledger
            .iter()
            .filter(|l| l.user_id == user_id && l.kind != AwardKind::Redeem)
            .map(|l| l.amount)
            .sum())
    }

    async fn spendable_balance(&self, user_id: &str) -> Result<i64> {
        let st = self.state.lock().expect("rewards fake lock");
        Ok(st
            .ledger
            .iter()
            .filter(|l| l.user_id == user_id)
            .map(|l| l.amount)
            .sum())
    }

    async fn recent_awards(&self, user_id: &str, limit: i64) -> Result<Vec<LedgerEntry>> {
        let st = self.state.lock().expect("rewards fake lock");
        let mut rows: Vec<&FakeLedger> =
            st.ledger.iter().filter(|l| l.user_id == user_id).collect();
        rows.sort_by_key(|l| std::cmp::Reverse(l.seq));
        Ok(rows
            .into_iter()
            .take(limit.max(0) as usize)
            .map(|l| LedgerEntry {
                kind: l.kind,
                amount: l.amount,
                catalog_score_id: l.catalog_score_id.clone(),
                reward_key: l.reward_key.clone(),
                source: l.source,
                created_at_ms: l.seq,
            })
            .collect())
    }

    async fn granted_keys(&self, user_id: &str) -> Result<HashSet<String>> {
        let st = self.state.lock().expect("rewards fake lock");
        Ok(st
            .grants
            .keys()
            .filter(|(u, _)| u == user_id)
            .map(|(_, k)| k.clone())
            .collect())
    }

    async fn insert_grant(&self, user_id: &str, key: &str, kind: GrantKind) -> Result<bool> {
        let mut st = self.state.lock().expect("rewards fake lock");
        let k = (user_id.to_string(), key.to_string());
        if st.grants.contains_key(&k) {
            return Ok(false);
        }
        st.grants.insert(k, kind);
        Ok(true)
    }

    async fn append_redeem(&self, user_id: &str, key: &str, cost: i64) -> Result<()> {
        let mut st = self.state.lock().expect("rewards fake lock");
        let seq = Self::next_seq(&mut st);
        st.ledger.push(FakeLedger {
            user_id: user_id.to_string(),
            kind: AwardKind::Redeem,
            amount: -cost,
            catalog_score_id: None,
            reward_key: Some(key.to_string()),
            source: None,
            seq,
        });
        Ok(())
    }

    async fn curator_metrics(
        &self,
        user_id: &str,
        honesty_floor: i64,
        _first_rater_points: i64,
    ) -> Result<CuratorMetrics> {
        let st = self.state.lock().expect("rewards fake lock");
        let mine: Vec<&FakeRating> = st.ratings.iter().filter(|r| r.user_id == user_id).collect();
        let total_ratings = mine.len() as i64;
        let first_rater_count = mine.iter().filter(|r| r.first_rater).count() as i64;
        let settled_count = mine.iter().filter(|r| r.settled_source.is_some()).count() as i64;
        let aligned_count = mine
            .iter()
            .filter(|r| {
                r.settled_source.is_some()
                    && honesty_for(&st, &r.user_id, &r.catalog_score_id) > honesty_floor
            })
            .count() as i64;
        let coverage_contribution = st
            .ledger
            .iter()
            .filter(|l| l.user_id == user_id && l.kind == AwardKind::Coverage)
            .count() as i64;
        Ok(CuratorMetrics {
            total_ratings,
            coverage_contribution,
            aligned_count,
            settled_count,
            first_rater_count,
        })
    }

    async fn shop_items(&self, user_id: &str) -> Result<Vec<ShopItem>> {
        let st = self.state.lock().expect("rewards fake lock");
        Ok(st
            .shop
            .iter()
            .map(|s| ShopItem {
                key: s.key.clone(),
                label: s.label.clone(),
                instrument: s.instrument.clone(),
                license: s.license.clone(),
                attribution: s.attribution.clone(),
                point_cost: s.point_cost,
                redeemable: s.redeemable,
                owned: is_owned(&st, user_id, s),
            })
            .collect())
    }

    async fn shop_item(&self, user_id: &str, key: &str) -> Result<Option<ShopItem>> {
        let st = self.state.lock().expect("rewards fake lock");
        Ok(st.shop.iter().find(|s| s.key == key).map(|s| ShopItem {
            key: s.key.clone(),
            label: s.label.clone(),
            instrument: s.instrument.clone(),
            license: s.license.clone(),
            attribution: s.attribution.clone(),
            point_cost: s.point_cost,
            redeemable: s.redeemable,
            owned: is_owned(&st, user_id, s),
        }))
    }
}

/// Honesty already awarded for one (user, score) in the fake ledger.
fn honesty_for(st: &FakeState, user_id: &str, catalog_score_id: &str) -> i64 {
    st.ledger
        .iter()
        .filter(|l| {
            l.user_id == user_id
                && l.catalog_score_id.as_deref() == Some(catalog_score_id)
                && matches!(l.kind, AwardKind::Honesty | AwardKind::Adjustment)
        })
        .map(|l| l.amount)
        .sum()
}

/// A font is owned when it is free (cost 0, available to all) or the user holds a grant.
fn is_owned(st: &FakeState, user_id: &str, s: &FakeShopRow) -> bool {
    s.point_cost == 0
        || st
            .grants
            .contains_key(&(user_id.to_string(), s.key.clone()))
}
