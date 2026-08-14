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

//! Pure play-rewards logic + configuration (change: add-play-rewards).
//!
//! The "money math" for **playing**, mirroring [`crate::curation_rewards_core`]'s
//! split for rating (design D1): the quality gate, the per-piece diminishing
//! curve, the difficulty weighting and the daily cap — no I/O, so the anti-grind
//! rules are proven without a database.
//!
//! The four brakes are **independent and multiplicative** (design D2), because any
//! one alone is farmable:
//!
//! | Brake | Closes |
//! |---|---|
//! | [`meets_floor`] — a run below the accuracy floor earns 0 | mashing keys, walking away mid-piece |
//! | the per-piece bands — the Nth paid run of a piece pays less | replaying one easy piece all evening |
//! | the difficulty weight | farming the shortest trivial piece |
//! | the daily cap | sheer volume |
//!
//! Practice is deliberately outside all of that ([`practice_award`]): a scoreless
//! run carries no quality signal by construction, so the only thing it evidences
//! is *you sat down today* — a small flat amount, at most once per player-local
//! day (design D3).

use crate::global_leaderboard_core::{GlobalConfig, difficulty_weight_of};

/// Tunable play-rewards configuration (design "Play award values are
/// configuration"). Every value retunes without a migration; a value lowered
/// later applies only to FUTURE awards — the ledger is append-only and nothing is
/// ever clawed back.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayRewardConfig {
    /// Overall accuracy (0..100) a run must REACH to earn anything (default 60).
    /// Below it, the session is recorded as activity and pays zero.
    pub accuracy_floor: f32,
    /// Per-piece diminishing bands as `(max_times_already_paid_inclusive, points)`,
    /// ascending by bound — the [`crate::curation_rewards_core::RewardConfig::coverage_bands`]
    /// shape re-aimed from "how many ratings does this score have" to "how many
    /// times have YOU already been paid for this piece". Beyond every bound → 0.
    ///
    /// Default `[(0,8),(1,3),(3,1)]` then 0: one piece can ever pay `8+3+1+1 = 13`,
    /// so replaying it all evening is worth **barely more than playing it once**
    /// (the proposal's target) and cannot on its own reach the daily cap — even at
    /// the top difficulty weight.
    pub piece_bands: Vec<(i64, i64)>,
    /// Difficulty weight per catalog `level`. Seeded from
    /// [`GlobalConfig::level_weights`] so play awards and the global season boards
    /// rank difficulty on ONE scale, not two (task 1.4).
    pub level_weights: Vec<(String, f64)>,
    /// Weight for a piece with no (or an unknown) level — NEUTRAL, never zero: a
    /// missing catalog level is a metadata gap, not a reason to tell a player their
    /// practice was worthless (design D7). Default 1.0.
    pub unleveled_weight: f64,
    /// Per-user daily cap on what PLAY can pay (default 40). Deliberately smaller
    /// than the curation cap: the four brakes are multiplicative and the starting
    /// amounts are small next to the coverage curve.
    pub daily_cap: i64,
    /// The flat showing-up award for a scoreless practice day (default 3).
    pub practice_award: i64,
}

impl Default for PlayRewardConfig {
    fn default() -> Self {
        let global = GlobalConfig::default();
        Self {
            accuracy_floor: 60.0,
            piece_bands: vec![(0, 8), (1, 3), (3, 1)],
            // Reused, not re-invented: the leaderboard's difficulty scale.
            level_weights: global.level_weights,
            unleveled_weight: global.unleveled_weight,
            daily_cap: 40,
            practice_award: 3,
        }
    }
}

impl PlayRewardConfig {
    /// The difficulty weight of a piece with catalog `level` — the same function
    /// the global season boards weigh with. An absent or unrecognised level is
    /// neutral, never zero (design D7).
    pub fn difficulty_weight(&self, level: Option<&str>) -> f64 {
        difficulty_weight_of(level, &self.level_weights, self.unleveled_weight)
    }
}

/// Whether a run's overall accuracy (0..100) reaches the quality floor. The gate
/// is inclusive: exactly at the floor pays.
pub fn meets_floor(accuracy_pct: f32, cfg: &PlayRewardConfig) -> bool {
    accuracy_pct >= cfg.accuracy_floor
}

/// The base points for a good run of a piece that has already paid this user
/// `times_already_paid` times, before the difficulty weight and the daily cap —
/// the diminishing curve. Full for a piece not yet paid for, approaching (then
/// reaching) zero as the same piece is replayed.
pub fn piece_base(times_already_paid: i64, cfg: &PlayRewardConfig) -> i64 {
    for &(bound, points) in &cfg.piece_bands {
        if times_already_paid <= bound {
            return points.max(0);
        }
    }
    0
}

/// The performance points to ACTUALLY award for one scored run: zero below the
/// quality floor, else the per-piece band ([`piece_base`]) scaled by the piece's
/// difficulty weight and clamped by the user's remaining daily headroom
/// (`cap − already_today`). Never negative; `0` means "award nothing".
pub fn performance_award(
    accuracy_pct: f32,
    times_already_paid: i64,
    level: Option<&str>,
    already_today: i64,
    cfg: &PlayRewardConfig,
) -> i64 {
    if !meets_floor(accuracy_pct, cfg) {
        return 0;
    }
    let base = piece_base(times_already_paid, cfg);
    // A negative weight would be a misconfiguration; clamp rather than pay a
    // negative award (the ledger only ever rises).
    let weight = cfg.difficulty_weight(level).max(0.0);
    let weighted = ((base as f64) * weight).round() as i64;
    let remaining = (cfg.daily_cap - already_today).max(0);
    weighted.clamp(0, remaining)
}

/// The showing-up award for a scoreless practice run: the flat amount the first
/// time on the player's local day, zero thereafter (design D3). The durable
/// per-(user, local day) award key on the ledger is what ENFORCES this in
/// production; this function states the rule.
pub fn practice_award(already_awarded_today: bool, cfg: &PlayRewardConfig) -> i64 {
    if already_awarded_today {
        0
    } else {
        cfg.practice_award.max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run comfortably above the default floor.
    const GOOD: f32 = 85.0;

    #[test]
    fn the_floor_is_inclusive_and_gates_everything_below_it() {
        let cfg = PlayRewardConfig::default(); // floor 60
        // Just below → nothing, however fresh the piece and however hard it is.
        assert!(!meets_floor(59.9, &cfg));
        assert_eq!(performance_award(59.9, 0, Some("advanced"), 0, &cfg), 0);
        assert_eq!(performance_award(0.0, 0, Some("advanced"), 0, &cfg), 0);
        // Exactly at the floor → paid.
        assert!(meets_floor(60.0, &cfg));
        assert_eq!(performance_award(60.0, 0, Some("beginner"), 0, &cfg), 8);
    }

    #[test]
    fn the_per_piece_curve_diminishes_then_zeroes() {
        let cfg = PlayRewardConfig::default(); // [(0,8),(1,3),(3,1)]
        assert_eq!(piece_base(0, &cfg), 8); // never paid for this piece
        assert_eq!(piece_base(1, &cfg), 3); // band edge
        assert_eq!(piece_base(2, &cfg), 1);
        assert_eq!(piece_base(3, &cfg), 1); // band edge
        assert_eq!(piece_base(4, &cfg), 0); // ground out
        assert_eq!(piece_base(1_000, &cfg), 0);
        // Everything one piece can EVER pay, at the top difficulty weight, stays
        // under the daily cap: no single piece is a well.
        let ever: i64 = (0..64).map(|n| piece_base(n, &cfg)).sum();
        assert_eq!(ever, 13);
        assert!((ever as f64 * cfg.difficulty_weight(Some("advanced"))) < cfg.daily_cap as f64);
        // Monotonically non-increasing: replaying is never worth MORE.
        let mut prev = i64::MAX;
        for n in 0..40 {
            let v = piece_base(n, &cfg);
            assert!(v <= prev, "band rose at {n}");
            prev = v;
        }
    }

    #[test]
    fn difficulty_scales_the_award_and_an_unknown_level_is_neutral() {
        let cfg = PlayRewardConfig::default(); // 1.0 / 1.5 / 2.0, unleveled 1.0
        assert_eq!(performance_award(GOOD, 0, Some("beginner"), 0, &cfg), 8);
        assert_eq!(
            performance_award(GOOD, 0, Some("intermediate"), 0, &cfg),
            12
        );
        assert_eq!(performance_award(GOOD, 0, Some("advanced"), 0, &cfg), 16);
        // A user upload (no level) and a level this build does not know both weigh
        // NEUTRALLY — they still pay, they are never zeroed (design D7).
        assert_eq!(performance_award(GOOD, 0, None, 0, &cfg), 8);
        assert_eq!(performance_award(GOOD, 0, Some("virtuoso"), 0, &cfg), 8);
        assert_eq!(cfg.difficulty_weight(None), 1.0);
        assert_eq!(cfg.difficulty_weight(Some("nope")), 1.0);
        // The scale IS the leaderboard's — not a second one.
        let global = GlobalConfig::default();
        for level in ["beginner", "intermediate", "advanced"] {
            assert_eq!(
                cfg.difficulty_weight(Some(level)),
                global.difficulty_weight(Some(level)),
                "{level} weighs differently from the season boards"
            );
        }
    }

    #[test]
    fn the_daily_cap_clamps_and_never_goes_negative() {
        let cfg = PlayRewardConfig::default(); // cap 40
        // Plenty of headroom → the full weighted award.
        assert_eq!(performance_award(GOOD, 0, Some("advanced"), 0, &cfg), 16);
        // Headroom smaller than the award → clamped to what remains.
        assert_eq!(performance_award(GOOD, 0, Some("advanced"), 30, &cfg), 10);
        assert_eq!(performance_award(GOOD, 0, Some("advanced"), 39, &cfg), 1);
        // Exactly at the cap → nothing more today.
        assert_eq!(performance_award(GOOD, 0, Some("advanced"), 40, &cfg), 0);
        // Past the cap (defensive; a retuned-down cap can leave a user over it) →
        // still zero, never negative.
        assert_eq!(performance_award(GOOD, 0, Some("advanced"), 999, &cfg), 0);
    }

    #[test]
    fn no_input_combination_yields_a_negative_award() {
        let cfg = PlayRewardConfig::default();
        for &acc in &[-5.0f32, 0.0, 59.9, 60.0, 100.0, 250.0] {
            for paid in [-3i64, 0, 1, 12, 500] {
                for level in [None, Some("beginner"), Some("advanced"), Some("???")] {
                    for today in [-10i64, 0, 39, 40, 10_000] {
                        let a = performance_award(acc, paid, level, today, &cfg);
                        assert!(a >= 0, "negative award for {acc}/{paid}/{level:?}/{today}");
                        assert!(a <= cfg.daily_cap.max(0), "award above the cap");
                    }
                }
            }
        }
        // A pathological config cannot produce a negative award either.
        let broken = PlayRewardConfig {
            piece_bands: vec![(0, -50)],
            unleveled_weight: -2.0,
            practice_award: -7,
            ..PlayRewardConfig::default()
        };
        assert_eq!(performance_award(GOOD, 0, None, 0, &broken), 0);
        assert_eq!(practice_award(false, &broken), 0);
    }

    #[test]
    fn practice_pays_the_flat_amount_once() {
        let cfg = PlayRewardConfig::default();
        assert_eq!(practice_award(false, &cfg), 3);
        assert_eq!(practice_award(true, &cfg), 0);
    }

    #[test]
    fn grinding_one_piece_earns_strictly_less_than_playing_many() {
        // The anti-grind property, stated directly (task 1.6): four good runs of
        // ONE piece are worth strictly less than four good runs of four DIFFERENT
        // pieces of the same difficulty — and neither can pass the daily cap.
        let cfg = PlayRewardConfig::default();
        let runs = 4;

        let mut grinding = 0i64;
        for n in 0..runs {
            grinding += performance_award(GOOD, n, Some("beginner"), grinding, &cfg);
        }
        let mut varied = 0i64;
        for _ in 0..runs {
            // Each piece is fresh: nothing has paid for it yet.
            varied += performance_award(GOOD, 0, Some("beginner"), varied, &cfg);
        }
        assert!(
            grinding < varied,
            "grinding ({grinding}) must earn strictly less than variety ({varied})"
        );
        assert!(grinding <= cfg.daily_cap && varied <= cfg.daily_cap);

        // Pushed all day, grinding one piece grounds out well under the cap while
        // the cap itself remains the ceiling for ANY strategy. This holds at the
        // TOP difficulty weight, so no piece is farmable by being hard either.
        let mut all_evening = 0i64;
        for n in 0..500 {
            all_evening += performance_award(GOOD, n, Some("advanced"), all_evening, &cfg);
        }
        assert!(
            all_evening < cfg.daily_cap,
            "one piece must not reach the cap"
        );
        let mut varied_all_day = 0i64;
        for _ in 0..500 {
            varied_all_day += performance_award(GOOD, 0, Some("advanced"), varied_all_day, &cfg);
        }
        assert_eq!(varied_all_day, cfg.daily_cap, "the cap is the hard ceiling");
    }
}
