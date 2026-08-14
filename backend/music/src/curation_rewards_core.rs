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

//! Pure curation-rewards logic + configuration (change: add-curation-rewards).
//!
//! No I/O, no gRPC — just the "money math" the module orchestrates over a repo:
//! the diminishing coverage curve + daily cap, the honesty bonus sizing (aligned
//! vs the never-negative floor, moderator vs consensus), the consensus/moderator
//! ground-truth resolution, the lifetime→level derivation, and the earned-badge
//! milestones. All values are [`RewardConfig`] (design "Starting Configuration"),
//! tunable without a migration; the pure functions here are exhaustively unit
//! tested so the settlement rules are proven without a database.

/// The kind of a ledger entry (mirrors the `award_kind` CHECK in migrations
/// 0016 and 0025). `Coverage`/`Honesty` are positive RATING awards; `Adjustment` is a
/// positive correction appended when a late moderator decision raises a consensus
/// settlement; `Performance`/`Practice` are the positive PLAY awards (change:
/// add-play-rewards) — a scored run past the quality floor, and the once-a-day
/// showing-up award for a scoreless practice run; `Redeem` is the negative shop
/// spend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwardKind {
    Coverage,
    Honesty,
    Adjustment,
    Performance,
    Practice,
    Redeem,
}

impl AwardKind {
    /// The persisted `award_kind` string.
    pub fn as_str(self) -> &'static str {
        match self {
            AwardKind::Coverage => "coverage",
            AwardKind::Honesty => "honesty",
            AwardKind::Adjustment => "adjustment",
            AwardKind::Performance => "performance",
            AwardKind::Practice => "practice",
            AwardKind::Redeem => "redeem",
        }
    }
}

/// Where a honesty settlement's ground truth came from (design D2). A `Moderator`
/// decision is the expert truth, weighted above `Consensus` and worth more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementSource {
    Consensus,
    Moderator,
}

impl SettlementSource {
    /// The persisted `settled_source` string on `score_ratings`.
    pub fn as_str(self) -> &'static str {
        match self {
            SettlementSource::Consensus => "consensus",
            SettlementSource::Moderator => "moderator",
        }
    }
}

/// The frozen ground truth for a score at settlement (design D2). `Ambiguous` (a
/// consensus average inside the neutral band around the midpoint) settles every
/// rater at the floor — nobody's taste is "right".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Truth {
    Positive,
    Negative,
    Ambiguous,
}

/// Tunable curation-rewards configuration (task 1.4). Defaults are the design's
/// straw-man "Starting Configuration"; all are config so they retune without a
/// migration. Held on the module and overridable in tests / by the server.
#[derive(Debug, Clone, PartialEq)]
pub struct RewardConfig {
    /// Coverage bands as `(max_existing_ratings_inclusive, points)`, ascending by
    /// bound. A rating of a score with `r` existing ratings earns the points of
    /// the first band whose bound `r` does not exceed; beyond every bound → 0.
    /// Default: `[(0,10),(4,6),(19,3),(49,1)]` then 0 (≥ 50).
    pub coverage_bands: Vec<(i64, i64)>,
    /// Per-user daily coverage-points cap (default 60).
    pub daily_cap: i64,
    /// Honesty bonus when a rating aligns with a MODERATOR (expert) truth (default 8).
    pub honesty_moderator: i64,
    /// Honesty bonus when a rating aligns with a CONSENSUS truth (default 5).
    pub honesty_consensus: i64,
    /// Honesty floor for a misaligned rating — never negative (default 1).
    pub honesty_floor: i64,
    /// Alignment midpoint on the 1–5 scale (default 3.0): a rating above it is
    /// positive, below it negative.
    pub alignment_midpoint: f64,
    /// Half-width of the neutral band around the midpoint (default 0.25): a
    /// consensus average within ± this is `Ambiguous`.
    pub ambiguous_band: f64,
    /// Distinct raters a score needs before its consensus is frozen (default 8).
    pub consensus_min_raters: i64,
    /// Cumulative LIFETIME-point thresholds for levels 1.. (default
    /// `[50,150,350,700,1200,2000,3000]`). Beyond the last, each further level
    /// costs `level_step` more.
    pub level_thresholds: Vec<i64>,
    /// Lifetime points per level beyond the explicit `level_thresholds` (default 1200).
    pub level_step: i64,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            coverage_bands: vec![(0, 10), (4, 6), (19, 3), (49, 1)],
            daily_cap: 60,
            honesty_moderator: 8,
            honesty_consensus: 5,
            honesty_floor: 1,
            alignment_midpoint: 3.0,
            ambiguous_band: 0.25,
            consensus_min_raters: 8,
            level_thresholds: vec![50, 150, 350, 700, 1200, 2000, 3000],
            level_step: 1200,
        }
    }
}

/// The base coverage points for rating a score that already has `existing`
/// ratings, before the daily cap (the diminishing curve). More for an under-rated
/// score, approaching zero (then exactly zero) as coverage grows.
pub fn coverage_base(existing: i64, cfg: &RewardConfig) -> i64 {
    for &(bound, points) in &cfg.coverage_bands {
        if existing <= bound {
            return points;
        }
    }
    0
}

/// Coverage points to ACTUALLY award for this rating: the diminishing base
/// ([`coverage_base`]) clamped by the user's remaining daily headroom
/// (`cap − already_today`). Never negative; `0` means "award nothing" (cap
/// reached, or a fully-covered score).
pub fn coverage_award(existing: i64, already_today: i64, cfg: &RewardConfig) -> i64 {
    let base = coverage_base(existing, cfg);
    let remaining = (cfg.daily_cap - already_today).max(0);
    base.min(remaining)
}

/// Whether a rating with effective value `effective` aligns with `truth`
/// (design D2). An `Ambiguous` truth aligns with nobody; a value exactly at the
/// midpoint is treated as not aligned (neither positive nor negative).
pub fn is_aligned(effective: f64, truth: Truth, cfg: &RewardConfig) -> bool {
    match truth {
        Truth::Ambiguous => false,
        Truth::Positive => effective > cfg.alignment_midpoint,
        Truth::Negative => effective < cfg.alignment_midpoint,
    }
}

/// The honesty bonus for a rating of effective value `effective`, settled against
/// `truth` from `source`: the full source-sized bonus when aligned, else the
/// never-negative floor. This is the target TOTAL honesty for the rating; the
/// module appends the delta over anything already awarded (so a moderator
/// override only ever tops up, never claws back — lifetime never falls).
pub fn honesty_award(
    effective: f64,
    truth: Truth,
    source: SettlementSource,
    cfg: &RewardConfig,
) -> i64 {
    if is_aligned(effective, truth, cfg) {
        match source {
            SettlementSource::Moderator => cfg.honesty_moderator,
            SettlementSource::Consensus => cfg.honesty_consensus,
        }
    } else {
        cfg.honesty_floor
    }
}

/// The frozen consensus truth for a score whose ratings average `avg` on the 1–5
/// scale: `Ambiguous` within the neutral band around the midpoint, else positive
/// above / negative below.
pub fn consensus_truth(avg: f64, cfg: &RewardConfig) -> Truth {
    if (avg - cfg.alignment_midpoint).abs() <= cfg.ambiguous_band {
        Truth::Ambiguous
    } else if avg > cfg.alignment_midpoint {
        Truth::Positive
    } else {
        Truth::Negative
    }
}

/// A moderator accept/reject decision as a ground truth (accept → positive,
/// reject → negative). Only accept/reject settle honesty (a re-queue to `pending`
/// is not a truth).
pub fn moderator_truth(accepted: bool) -> Truth {
    if accepted {
        Truth::Positive
    } else {
        Truth::Negative
    }
}

/// The level for a given LIFETIME points total (design D5, monotonic): the number
/// of crossed `level_thresholds`, then one extra level per `level_step` beyond the
/// last threshold. Level 0 below the first threshold.
pub fn level_for(lifetime: i64, cfg: &RewardConfig) -> i64 {
    let mut level = 0i64;
    let mut last = 0i64;
    for &t in &cfg.level_thresholds {
        if lifetime >= t {
            level += 1;
            last = t;
        } else {
            return level;
        }
    }
    if cfg.level_step > 0 {
        level += (lifetime - last) / cfg.level_step;
    }
    level
}

/// The lifetime-points threshold that STARTS `level` (the entry cost), extending
/// past the explicit list by `level_step`. `threshold_for_level(0)` is 0.
pub fn threshold_for_level(level: i64, cfg: &RewardConfig) -> i64 {
    if level <= 0 {
        return 0;
    }
    let n = cfg.level_thresholds.len() as i64;
    if level <= n {
        cfg.level_thresholds[(level - 1) as usize]
    } else {
        let last = *cfg.level_thresholds.last().unwrap_or(&0);
        last + (level - n) * cfg.level_step
    }
}

/// Progress toward the next level for a lifetime total: `(current_level,
/// current_floor, next_threshold)`. `current_floor` is the lifetime that started
/// the current level and `next_threshold` the lifetime needed for the next, so a
/// UI can render `(lifetime − floor) / (next − floor)`.
pub fn level_progress(lifetime: i64, cfg: &RewardConfig) -> (i64, i64, i64) {
    let level = level_for(lifetime, cfg);
    (
        level,
        threshold_for_level(level, cfg),
        threshold_for_level(level + 1, cfg),
    )
}

// Badges no longer live here (change: add-achievement-badges). They are defined
// once in [`crate::badges_core::REGISTRY`], which spans every family — play,
// consistency, ranking, contribution, curation and learning — instead of only the
// three curation counters this module owns. Curation still CONTRIBUTES its
// counters (see [`crate::curation_rewards::CuratorMetrics::badge_counts`]) and
// still grants the curation subset on its own write paths, through
// [`crate::badges_core::earned_curation_badges`]; it just no longer defines,
// evaluates or renders badges itself.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_curve_diminishes_then_zeroes() {
        let cfg = RewardConfig::default();
        assert_eq!(coverage_base(0, &cfg), 10); // first rater
        assert_eq!(coverage_base(1, &cfg), 6);
        assert_eq!(coverage_base(4, &cfg), 6);
        assert_eq!(coverage_base(5, &cfg), 3);
        assert_eq!(coverage_base(19, &cfg), 3);
        assert_eq!(coverage_base(20, &cfg), 1);
        assert_eq!(coverage_base(49, &cfg), 1);
        assert_eq!(coverage_base(50, &cfg), 0); // well covered → nothing
        assert_eq!(coverage_base(1000, &cfg), 0);
    }

    #[test]
    fn coverage_award_respects_daily_cap() {
        let cfg = RewardConfig::default(); // cap 60
        // Plenty of headroom → full base.
        assert_eq!(coverage_award(0, 0, &cfg), 10);
        // Near the cap → clamped to what remains.
        assert_eq!(coverage_award(0, 55, &cfg), 5);
        // At the cap → nothing more.
        assert_eq!(coverage_award(0, 60, &cfg), 0);
        // Past the cap (defensive) → still non-negative.
        assert_eq!(coverage_award(0, 100, &cfg), 0);
        // A fully-covered score is 0 regardless of headroom.
        assert_eq!(coverage_award(50, 0, &cfg), 0);
    }

    #[test]
    fn alignment_and_honesty_sizing() {
        let cfg = RewardConfig::default(); // midpoint 3.0, mod 8 / cons 5 / floor 1
        // Aligned with a positive truth.
        assert!(is_aligned(3.5, Truth::Positive, &cfg));
        assert_eq!(
            honesty_award(3.5, Truth::Positive, SettlementSource::Moderator, &cfg),
            8
        );
        assert_eq!(
            honesty_award(3.5, Truth::Positive, SettlementSource::Consensus, &cfg),
            5
        );
        // Misaligned → the floor, from either source, never negative.
        assert!(!is_aligned(1.5, Truth::Positive, &cfg));
        assert_eq!(
            honesty_award(1.5, Truth::Positive, SettlementSource::Moderator, &cfg),
            1
        );
        assert_eq!(
            honesty_award(1.5, Truth::Positive, SettlementSource::Consensus, &cfg),
            1
        );
        // Aligned with a negative truth (a dislike matching a rejected/hated score).
        assert!(is_aligned(1.5, Truth::Negative, &cfg));
        assert_eq!(
            honesty_award(1.5, Truth::Negative, SettlementSource::Moderator, &cfg),
            8
        );
        // Ambiguous truth → everyone gets the floor.
        assert!(!is_aligned(5.0, Truth::Ambiguous, &cfg));
        assert_eq!(
            honesty_award(5.0, Truth::Ambiguous, SettlementSource::Consensus, &cfg),
            1
        );
        // Exactly at the midpoint is not aligned either way.
        assert!(!is_aligned(3.0, Truth::Positive, &cfg));
        assert!(!is_aligned(3.0, Truth::Negative, &cfg));
    }

    #[test]
    fn consensus_truth_neutral_band() {
        let cfg = RewardConfig::default(); // midpoint 3.0, band 0.25
        assert_eq!(consensus_truth(4.0, &cfg), Truth::Positive);
        assert_eq!(consensus_truth(2.0, &cfg), Truth::Negative);
        assert_eq!(consensus_truth(3.0, &cfg), Truth::Ambiguous);
        assert_eq!(consensus_truth(3.2, &cfg), Truth::Ambiguous); // within ±0.25
        assert_eq!(consensus_truth(2.75, &cfg), Truth::Ambiguous); // boundary inclusive
        assert_eq!(consensus_truth(3.26, &cfg), Truth::Positive); // just outside
    }

    #[test]
    fn moderator_truth_maps_decision() {
        assert_eq!(moderator_truth(true), Truth::Positive);
        assert_eq!(moderator_truth(false), Truth::Negative);
    }

    #[test]
    fn levels_are_monotonic_and_extend_past_the_list() {
        let cfg = RewardConfig::default(); // [50,150,350,700,1200,2000,3000] step 1200
        assert_eq!(level_for(0, &cfg), 0);
        assert_eq!(level_for(49, &cfg), 0);
        assert_eq!(level_for(50, &cfg), 1); // first threshold
        assert_eq!(level_for(149, &cfg), 1);
        assert_eq!(level_for(150, &cfg), 2);
        assert_eq!(level_for(3000, &cfg), 7); // last explicit threshold
        assert_eq!(level_for(4199, &cfg), 7);
        assert_eq!(level_for(4200, &cfg), 8); // 3000 + 1200
        assert_eq!(level_for(5400, &cfg), 9);
        // Monotonic: more lifetime never lowers the level.
        let mut prev = 0;
        for lp in (0..7000).step_by(37) {
            let l = level_for(lp, &cfg);
            assert!(l >= prev);
            prev = l;
        }
    }

    #[test]
    fn level_progress_bounds() {
        let cfg = RewardConfig::default();
        // At 200 lifetime → level 2, floor 150, next 350.
        assert_eq!(level_progress(200, &cfg), (2, 150, 350));
        // Below the first threshold → level 0, floor 0, next 50.
        assert_eq!(level_progress(10, &cfg), (0, 0, 50));
        // Past the list → extended by the step.
        assert_eq!(level_progress(4200, &cfg), (8, 4200, 5400));
    }

    #[test]
    fn award_kind_and_source_strings() {
        assert_eq!(AwardKind::Coverage.as_str(), "coverage");
        assert_eq!(AwardKind::Honesty.as_str(), "honesty");
        assert_eq!(AwardKind::Adjustment.as_str(), "adjustment");
        assert_eq!(AwardKind::Performance.as_str(), "performance");
        assert_eq!(AwardKind::Practice.as_str(), "practice");
        assert_eq!(AwardKind::Redeem.as_str(), "redeem");
        assert_eq!(SettlementSource::Consensus.as_str(), "consensus");
        assert_eq!(SettlementSource::Moderator.as_str(), "moderator");
    }
}
