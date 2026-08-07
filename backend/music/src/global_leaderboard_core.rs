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

//! Pure, host-testable GLOBAL leaderboard logic (change: add-global-leaderboard):
//! the season model (D3), the difficulty weights + best-N configuration (D1), the
//! difficulty-weighted best-N aggregation that turns a season's per-piece bests
//! into one **global season score** per player, and the ranking order / own-rank
//! computation. No I/O and no gRPC — so the aggregation rules (the part that
//! decides who is "the best player overall") are proven without a database.

use std::cmp::Ordering;
use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};

use crate::global_leaderboard::{GlobalScore, SeasonBestRow};

/// One season window: its stable id and its half-open UTC bounds
/// `[start_ms, end_ms)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Season {
    /// Stable id — the window's UTC start date in `YYYY-MM-DD` form, so ids sort
    /// chronologically as plain strings and read well in a season selector.
    pub id: String,
    pub start_ms: i64,
    pub end_ms: i64,
}

/// Tunable global-leaderboard configuration (tasks 1.3 + 2.1). Every value is
/// config — the aggregation curve and the season cadence retune without a
/// migration (design D1/D3). Held on the module and overridable in tests.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalConfig {
    /// How many of a player's strongest pieces count toward the season score.
    /// Beyond this, playing MORE pieces adds nothing — only replacing a weaker
    /// contribution does (design D1). Default 20.
    pub best_n: usize,
    /// Season length in days (UTC fixed-length windows). Default 30 ("about
    /// monthly"); fixed-length keeps every season the same size to compete in,
    /// which calendar months would not.
    pub season_length_days: i64,
    /// UTC instant the season grid is anchored on (unix ms). Season *k* spans
    /// `[anchor + k·length, anchor + (k+1)·length)`. Default 2026-01-01T00:00:00Z.
    pub season_anchor_ms: i64,
    /// Difficulty weight per catalog `level`, keyed by the stored level string.
    /// A harder piece multiplies its contribution, so easy-farming cannot out-earn
    /// playing hard pieces well (design D1).
    pub level_weights: Vec<(String, f64)>,
    /// Weight for a piece with no (or an unknown) level — a NEUTRAL default, so a
    /// missing `level` neither punishes nor rewards. Default 1.0.
    pub unleveled_weight: f64,
}

/// 2026-01-01T00:00:00Z in unix ms — the default season grid anchor.
const DEFAULT_SEASON_ANCHOR_MS: i64 = 1_767_225_600_000;

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            best_n: 20,
            season_length_days: 30,
            season_anchor_ms: DEFAULT_SEASON_ANCHOR_MS,
            level_weights: vec![
                ("beginner".to_string(), 1.0),
                ("intermediate".to_string(), 1.5),
                ("advanced".to_string(), 2.0),
            ],
            unleveled_weight: 1.0,
        }
    }
}

impl GlobalConfig {
    /// Season length in milliseconds (always ≥ one day, so a misconfigured 0 can
    /// never divide by zero).
    fn season_len_ms(&self) -> i64 {
        self.season_length_days.max(1) * 86_400_000
    }

    /// The difficulty weight of a piece with catalog `level`. An absent or
    /// unrecognised level falls back to [`Self::unleveled_weight`] (design D1:
    /// `level` is partly heuristic, so an unknown value must stay neutral).
    pub fn difficulty_weight(&self, level: Option<&str>) -> f64 {
        level
            .and_then(|l| {
                self.level_weights
                    .iter()
                    .find(|(name, _)| name == l)
                    .map(|(_, w)| *w)
            })
            .unwrap_or(self.unleveled_weight)
    }

    /// The season containing `ts_ms` (UTC boundaries, design D3). Timestamps
    /// before the anchor floor into earlier (negative-index) windows, so the
    /// derivation is total — a clock-skewed client never lands "outside" a season.
    pub fn season_at(&self, ts_ms: i64) -> Season {
        let len = self.season_len_ms();
        // Floor division (Rust's `/` truncates toward zero, which is wrong for
        // instants before the anchor).
        let index = (ts_ms - self.season_anchor_ms).div_euclid(len);
        let start_ms = self.season_anchor_ms + index * len;
        Season {
            id: season_id(start_ms),
            start_ms,
            end_ms: start_ms + len,
        }
    }

    /// The season immediately BEFORE the one containing `ts_ms` — the window the
    /// end-of-season snapshot job freezes once it has closed (task 2.2).
    pub fn previous_season(&self, ts_ms: i64) -> Season {
        let current = self.season_at(ts_ms);
        self.season_at(current.start_ms - 1)
    }
}

/// Render a season id from its UTC start instant: the start date, `YYYY-MM-DD`.
fn season_id(start_ms: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(start_ms)
        .map(|d| d.date_naive())
        // Unreachable for any plausible anchor/length; a stable fallback keeps the
        // derivation total rather than panicking on a pathological config.
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch date"))
        .format("%Y-%m-%d")
        .to_string()
}

/// The difficulty-weighted contribution of one season best: its sub-score scaled
/// to `0..1` times the piece's difficulty weight (design D1).
fn contribution(subscore: f32, weight: f64) -> f64 {
    (subscore as f64 / 100.0) * weight
}

/// Aggregate one season+mode's per-piece bests into the players' **global season
/// scores**, in ranking order (task 3.1, design D1).
///
/// Per player: take the difficulty-weighted contribution of each of their season
/// bests, keep only the **best N**, and sum them. So:
/// * playing MORE than N pieces adds nothing unless it displaces a weaker
///   contribution — volume alone never raises the score;
/// * the same sub-score on a harder piece contributes more.
///
/// `contributing_pieces` is how many bests actually made the cut (≤ N) and
/// `reached_at_ms` is the LATEST achievement among them — the moment the player's
/// current score was reached, which is the final tie-break (earliest wins).
///
/// Returns the standings sorted by [`rank_cmp`]. Duplicate `(user, piece)` rows
/// cannot occur (the store's primary key), so no de-duplication is needed here.
pub fn aggregate(rows: &[SeasonBestRow], cfg: &GlobalConfig) -> Vec<GlobalScore> {
    // Per player: (contribution, achieved_at) for each of their season bests.
    let mut by_user: HashMap<&str, Vec<(f64, i64)>> = HashMap::new();
    for row in rows {
        let weight = cfg.difficulty_weight(row.level.as_deref());
        by_user
            .entry(row.user_id.as_str())
            .or_default()
            .push((contribution(row.subscore, weight), row.achieved_at_ms));
    }

    let mut out: Vec<GlobalScore> = by_user
        .into_iter()
        .filter_map(|(user_id, mut items)| {
            // Best-N by contribution (descending), so only the strongest count.
            items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
            let kept = &items[..items.len().min(cfg.best_n.max(1))];
            if kept.is_empty() {
                return None;
            }
            Some(GlobalScore {
                user_id: user_id.to_string(),
                score: kept.iter().map(|(c, _)| *c).sum(),
                contributing_pieces: kept.len() as i32,
                // The current score exists only once its LAST contribution landed.
                reached_at_ms: kept.iter().map(|(_, at)| *at).max().unwrap_or(0),
                // A live aggregation carries no frozen consent — the live gate
                // decides. The snapshot writer stamps the real value.
                was_listable: true,
            })
        })
        .collect();
    out.sort_by(rank_cmp);
    out
}

/// Ranking comparator between two global standings: the one that sorts **`Less`
/// ranks higher** (rank 1 first). Higher global season score first; ties by MORE
/// contributing pieces; then by the EARLIER moment the score was reached.
pub fn rank_cmp(a: &GlobalScore, b: &GlobalScore) -> Ordering {
    b.score
        .partial_cmp(&a.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| b.contributing_pieces.cmp(&a.contributing_pieces))
        .then_with(|| a.reached_at_ms.cmp(&b.reached_at_ms))
}

/// The 1-based rank of `own` among the already-ranking-ordered public standings:
/// one plus the number of public players that rank strictly higher. Works whether
/// or not `own` is itself listed — a private or under-age caller still gets "your
/// position among the public players" (design D5).
pub fn own_rank(public_ranked: &[GlobalScore], own: &GlobalScore) -> i32 {
    let ahead = public_ranked
        .iter()
        .filter(|e| rank_cmp(e, own) == Ordering::Less)
        .count();
    (ahead as i32) + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leaderboard::Mode;

    fn row(user: &str, piece: &str, level: Option<&str>, sub: f32, at: i64) -> SeasonBestRow {
        SeasonBestRow {
            user_id: user.into(),
            catalog_score_id: piece.into(),
            mode: Mode::Tempo,
            level: level.map(str::to_string),
            subscore: sub,
            achieved_at_ms: at,
        }
    }

    fn score_of(standings: &[GlobalScore], user: &str) -> f64 {
        standings
            .iter()
            .find(|s| s.user_id == user)
            .map(|s| s.score)
            .expect("user ranked")
    }

    #[test]
    fn difficulty_weight_maps_levels_and_defaults_neutral() {
        let cfg = GlobalConfig::default();
        assert_eq!(cfg.difficulty_weight(Some("beginner")), 1.0);
        assert_eq!(cfg.difficulty_weight(Some("intermediate")), 1.5);
        assert_eq!(cfg.difficulty_weight(Some("advanced")), 2.0);
        // Unleveled or unknown → the neutral default, never 0.
        assert_eq!(cfg.difficulty_weight(None), 1.0);
        assert_eq!(cfg.difficulty_weight(Some("wizard")), 1.0);
    }

    #[test]
    fn seasons_are_fixed_utc_windows_derived_from_a_timestamp() {
        let cfg = GlobalConfig::default();
        let anchor = cfg.season_anchor_ms;
        let len = 30 * 86_400_000i64;
        // The anchor instant opens season 0.
        let s0 = cfg.season_at(anchor);
        assert_eq!(s0.start_ms, anchor);
        assert_eq!(s0.end_ms, anchor + len);
        assert_eq!(s0.id, "2026-01-01");
        // The last millisecond of the window is still the SAME season...
        assert_eq!(cfg.season_at(anchor + len - 1).id, s0.id);
        // ...and the next one opens a new season with a fresh id.
        let s1 = cfg.season_at(anchor + len);
        assert_ne!(s1.id, s0.id);
        assert_eq!(s1.start_ms, s0.end_ms);
        assert_eq!(s1.id, "2026-01-31");
        // An instant BEFORE the anchor floors into the preceding window (total).
        assert_eq!(cfg.season_at(anchor - 1).end_ms, anchor);
    }

    #[test]
    fn previous_season_is_the_window_that_just_closed() {
        let cfg = GlobalConfig::default();
        let len = 30 * 86_400_000i64;
        let now = cfg.season_anchor_ms + len + 5_000; // early in season 1
        let prev = cfg.previous_season(now);
        assert_eq!(prev.id, cfg.season_at(cfg.season_anchor_ms).id);
        assert_eq!(prev.end_ms, cfg.season_at(now).start_ms);
    }

    #[test]
    fn only_the_best_n_pieces_count() {
        let cfg = GlobalConfig {
            best_n: 2,
            ..GlobalConfig::default()
        };
        // Three beginner pieces at 90/80/70; only the two best count.
        let rows = vec![
            row("u", "p1", Some("beginner"), 90.0, 1),
            row("u", "p2", Some("beginner"), 80.0, 2),
            row("u", "p3", Some("beginner"), 70.0, 3),
        ];
        let out = aggregate(&rows, &cfg);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].contributing_pieces, 2);
        assert!((out[0].score - (0.9 + 0.8)).abs() < 1e-9);
        // `reached_at` is the latest CONTRIBUTING achievement (p2 at 2), not p3's.
        assert_eq!(out[0].reached_at_ms, 2);
    }

    #[test]
    fn harder_pieces_are_worth_more() {
        let cfg = GlobalConfig::default();
        // Same sub-score, different difficulty.
        let rows = vec![
            row("easy", "p1", Some("beginner"), 80.0, 1),
            row("hard", "p2", Some("advanced"), 80.0, 1),
        ];
        let out = aggregate(&rows, &cfg);
        assert!(score_of(&out, "hard") > score_of(&out, "easy"));
        // Ranked accordingly: the advanced player is first.
        assert_eq!(out[0].user_id, "hard");
    }

    #[test]
    fn volume_alone_does_not_raise_the_score() {
        let cfg = GlobalConfig {
            best_n: 2,
            ..GlobalConfig::default()
        };
        let base = vec![
            row("u", "p1", Some("beginner"), 90.0, 1),
            row("u", "p2", Some("beginner"), 80.0, 1),
        ];
        let before = score_of(&aggregate(&base, &cfg), "u");
        // Grind many more EASY pieces without beating the existing best N.
        let mut grinded = base.clone();
        for i in 0..20 {
            grinded.push(row("u", &format!("g{i}"), Some("beginner"), 50.0, 2));
        }
        let after = score_of(&aggregate(&grinded, &cfg), "u");
        assert_eq!(before, after);
        // Improving a contribution (a harder piece played well) DOES raise it.
        let mut improved = base.clone();
        improved.push(row("u", "hard", Some("advanced"), 95.0, 3));
        assert!(score_of(&aggregate(&improved, &cfg), "u") > before);
    }

    #[test]
    fn unleveled_pieces_use_the_neutral_weight() {
        let cfg = GlobalConfig::default();
        let rows = vec![
            row("a", "p1", None, 60.0, 1),
            row("b", "p2", Some("beginner"), 60.0, 1),
        ];
        let out = aggregate(&rows, &cfg);
        assert_eq!(score_of(&out, "a"), score_of(&out, "b"));
    }

    #[test]
    fn ranking_breaks_ties_by_pieces_then_earliest() {
        let cfg = GlobalConfig::default();
        // Equal totals (0.9 + 0.1 vs 1.0): "many" has 2 pieces, "one" has 1.
        let rows = vec![
            row("many", "a", Some("beginner"), 90.0, 5),
            row("many", "b", Some("beginner"), 10.0, 5),
            row("one", "c", Some("beginner"), 100.0, 5),
        ];
        let out = aggregate(&rows, &cfg);
        assert!((score_of(&out, "many") - score_of(&out, "one")).abs() < 1e-9);
        assert_eq!(out[0].user_id, "many"); // more contributing pieces wins
        // Fully equal (same total AND same piece count) → the earlier one wins.
        let early = GlobalScore {
            user_id: "early".into(),
            score: 1.0,
            contributing_pieces: 1,
            reached_at_ms: 10,
            was_listable: true,
        };
        let late = GlobalScore {
            user_id: "late".into(),
            score: 1.0,
            contributing_pieces: 1,
            reached_at_ms: 20,
            was_listable: true,
        };
        assert_eq!(rank_cmp(&early, &late), Ordering::Less);
    }

    #[test]
    fn empty_input_ranks_nobody() {
        assert!(aggregate(&[], &GlobalConfig::default()).is_empty());
    }

    #[test]
    fn own_rank_counts_public_players_ahead() {
        let s = |user: &str, score: f64| GlobalScore {
            user_id: user.into(),
            score,
            contributing_pieces: 1,
            reached_at_ms: 1,
            was_listable: true,
        };
        let public = vec![s("a", 9.0), s("b", 5.0), s("c", 2.0)];
        // A private caller at 4.0 slots between b and c → rank 3.
        assert_eq!(own_rank(&public, &s("me", 4.0)), 3);
        assert_eq!(own_rank(&public, &s("me", 99.0)), 1);
        assert_eq!(own_rank(&public, &s("me", 0.5)), 4);
    }
}
