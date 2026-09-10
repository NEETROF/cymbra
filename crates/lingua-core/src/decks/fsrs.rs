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

//! FSRS-5 review scheduling (design D2).
//!
//! The scheduling half of FSRS — forgetting curve, stability/difficulty
//! updates, next interval at a target retention — reimplemented here with no
//! dependency, using the published FSRS-5 default weights and the ts-fsrs /
//! py-fsrs v5 formulas. The 19-weight vector plus request-retention and
//! max-interval are stored on [`FsrsParams`], so a future *optimiser* (the
//! `fsrs` crate, run server-side outside the WASM core) can refit them per
//! user with no schema change. Parameter optimisation is deliberately NOT in
//! this crate.
//!
//! The clock is injected (callers pass `now`), so scheduling is deterministic
//! and host-testable, and identical native vs WASM.

use serde::{Deserialize, Serialize};

const SECONDS_PER_DAY: f64 = 86_400.0;
/// FSRS-5 forgetting-curve decay.
const DECAY: f64 = -0.5;
/// Minimum stability, in days, so a lapse never collapses to zero.
const MIN_STABILITY: f64 = 0.01;

/// A grade given during review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rating {
    /// Forgotten.
    Again,
    /// Recalled with serious difficulty.
    Hard,
    /// Recalled.
    Good,
    /// Recalled easily.
    Easy,
}

impl Rating {
    /// The 1..=4 value used in the formulas.
    fn value(self) -> f64 {
        match self {
            Rating::Again => 1.0,
            Rating::Hard => 2.0,
            Rating::Good => 3.0,
            Rating::Easy => 4.0,
        }
    }
}

/// The FSRS scheduler parameters. Stored on the state so they can be refit by
/// an external optimiser later without a schema change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FsrsParams {
    /// The 19 FSRS-5 weights.
    pub w: [f64; 19],
    /// Target retention for interval computation (default 0.9).
    pub request_retention: f64,
    /// Interval ceiling in days (default ~100 years).
    pub maximum_interval_days: i64,
}

impl Default for FsrsParams {
    fn default() -> Self {
        Self {
            // Published FSRS-5 default weights.
            w: [
                0.40255, 1.18385, 3.173, 15.69105, 7.1949, 0.5345, 1.4604, 0.0046, 1.54575, 0.1192,
                1.01925, 1.9395, 0.11, 0.29605, 2.2698, 0.2315, 2.9898, 0.51655, 0.6621,
            ],
            request_retention: 0.9,
            maximum_interval_days: 36_500,
        }
    }
}

impl FsrsParams {
    fn factor(&self) -> f64 {
        0.9_f64.powf(1.0 / DECAY) - 1.0
    }

    /// Retrievability after `elapsed_days` at a given stability (0..=1).
    pub fn retrievability(&self, elapsed_days: f64, stability: f64) -> f64 {
        (1.0 + self.factor() * elapsed_days.max(0.0) / stability).powf(DECAY)
    }

    /// The next interval, in whole days (≥ 1, ≤ the ceiling), for a stability.
    /// At the default 0.9 retention this rounds to the stability itself.
    pub fn interval_days(&self, stability: f64) -> i64 {
        let raw = (stability / self.factor()) * (self.request_retention.powf(1.0 / DECAY) - 1.0);
        (raw.round() as i64).clamp(1, self.maximum_interval_days)
    }

    fn init_stability(&self, rating: Rating) -> f64 {
        self.w[(rating.value() as usize) - 1].max(MIN_STABILITY)
    }

    fn init_difficulty(&self, rating: Rating) -> f64 {
        (self.w[4] - (self.w[5] * (rating.value() - 1.0)).exp() + 1.0).clamp(1.0, 10.0)
    }

    fn next_difficulty(&self, difficulty: f64, rating: Rating) -> f64 {
        let delta = -self.w[6] * (rating.value() - 3.0);
        let damped = difficulty + delta * (10.0 - difficulty) / 9.0;
        let reverted = self.w[7] * self.init_difficulty(Rating::Easy) + (1.0 - self.w[7]) * damped;
        reverted.clamp(1.0, 10.0)
    }

    fn next_stability(
        &self,
        difficulty: f64,
        stability: f64,
        retrievability: f64,
        rating: Rating,
    ) -> f64 {
        let next = if rating == Rating::Again {
            let forget = self.w[11]
                * difficulty.powf(-self.w[12])
                * ((stability + 1.0).powf(self.w[13]) - 1.0)
                * (self.w[14] * (1.0 - retrievability)).exp();
            forget.min(stability)
        } else {
            let hard_penalty = if rating == Rating::Hard {
                self.w[15]
            } else {
                1.0
            };
            let easy_bonus = if rating == Rating::Easy {
                self.w[16]
            } else {
                1.0
            };
            stability
                * (1.0
                    + self.w[8].exp()
                        * (11.0 - difficulty)
                        * stability.powf(-self.w[9])
                        * ((self.w[10] * (1.0 - retrievability)).exp() - 1.0)
                        * hard_penalty
                        * easy_bonus)
        };
        next.max(MIN_STABILITY)
    }
}

/// The learned memory of a card: FSRS stability (days) and difficulty (1..10).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    /// Stability in days — the interval at which retention drops to the
    /// request-retention target.
    pub stability: f64,
    /// Difficulty, 1 (easy) .. 10 (hard).
    pub difficulty: f64,
}

/// A card's review state: its memory, when it is next due, and history
/// counters. A never-reviewed card has no memory and is due immediately.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewState {
    /// The FSRS memory, once first graded.
    pub memory: Option<Memory>,
    /// Unix-epoch seconds when the card is next due (`None` = due now / new).
    pub due: Option<i64>,
    /// Unix-epoch seconds of the last review.
    pub last_review: Option<i64>,
    /// Number of reviews.
    pub reps: u32,
    /// Number of lapses (`Again` on a reviewed card).
    pub lapses: u32,
}

impl ReviewState {
    /// A fresh, never-reviewed state (due immediately).
    pub fn new() -> Self {
        Self {
            memory: None,
            due: None,
            last_review: None,
            reps: 0,
            lapses: 0,
        }
    }

    /// Whether the card is due at `now` (a never-reviewed card always is).
    pub fn is_due(&self, now: i64) -> bool {
        match self.due {
            Some(due) => now >= due,
            None => true,
        }
    }

    /// Retires the card from review: it keeps its memory and history but is
    /// never due again (used by "I know this"). The lemma's `known` status
    /// lives in the knowledge model, not here.
    pub fn retire(&mut self, _now: i64) {
        self.due = Some(i64::MAX);
    }

    /// Applies a grade at `now`, updating memory, due date and counters.
    pub fn grade(&mut self, params: &FsrsParams, rating: Rating, now: i64) {
        let memory = match self.memory {
            None => Memory {
                stability: params.init_stability(rating),
                difficulty: params.init_difficulty(rating),
            },
            Some(prev) => {
                let elapsed = self
                    .last_review
                    .map(|t| ((now - t).max(0)) as f64 / SECONDS_PER_DAY)
                    .unwrap_or(0.0);
                let r = params.retrievability(elapsed, prev.stability);
                let difficulty = params.next_difficulty(prev.difficulty, rating);
                let stability = params.next_stability(difficulty, prev.stability, r, rating);
                if rating == Rating::Again {
                    self.lapses += 1;
                }
                Memory {
                    stability,
                    difficulty,
                }
            }
        };
        self.reps += 1;
        self.last_review = Some(now);
        self.due = Some(now + params.interval_days(memory.stability) * SECONDS_PER_DAY as i64);
        self.memory = Some(memory);
    }
}

impl Default for ReviewState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400;

    #[test]
    fn interval_equals_stability_at_default_retention() {
        let p = FsrsParams::default();
        // At r = 0.9 the interval rounds to the stability itself.
        assert_eq!(p.interval_days(3.173), 3);
        assert_eq!(p.interval_days(15.69105), 16);
        assert_eq!(p.interval_days(0.4), 1); // clamped to a minimum of 1 day
    }

    #[test]
    fn retrievability_starts_at_one_and_decays() {
        let p = FsrsParams::default();
        assert!((p.retrievability(0.0, 5.0) - 1.0).abs() < 1e-9);
        let early = p.retrievability(1.0, 5.0);
        let late = p.retrievability(10.0, 5.0);
        assert!(
            early > late,
            "retrievability must fall as time passes ({early} !> {late})"
        );
        assert!((0.0..=1.0).contains(&late));
    }

    #[test]
    fn spec_a_good_answer_pushes_the_due_date_out() {
        let p = FsrsParams::default();
        let mut state = ReviewState::new();
        state.grade(&p, Rating::Good, 1_000);
        let due = state.due.expect("scheduled");
        assert!(due > 1_000, "due must be strictly later than now");
        assert!(state.memory.is_some());
        assert_eq!(state.reps, 1);
    }

    #[test]
    fn first_good_review_schedules_three_days() {
        let p = FsrsParams::default();
        let mut state = ReviewState::new();
        state.grade(&p, Rating::Good, 0);
        // init stability for Good = w[2] = 3.173 → 3-day interval.
        assert_eq!(state.due, Some(3 * DAY));
    }

    #[test]
    fn easy_beats_good_beats_hard_on_stability_gain() {
        let p = FsrsParams::default();
        let base = Memory {
            stability: 5.0,
            difficulty: 5.0,
        };
        let r = p.retrievability(5.0, 5.0);
        let hard = p.next_stability(p.next_difficulty(5.0, Rating::Hard), 5.0, r, Rating::Hard);
        let good = p.next_stability(p.next_difficulty(5.0, Rating::Good), 5.0, r, Rating::Good);
        let easy = p.next_stability(p.next_difficulty(5.0, Rating::Easy), 5.0, r, Rating::Easy);
        let _ = base;
        assert!(
            hard < good && good < easy,
            "hard {hard} < good {good} < easy {easy}"
        );
        assert!(good > 5.0, "a successful recall must not shrink stability");
    }

    #[test]
    fn again_is_a_lapse_that_shortens_the_interval() {
        let p = FsrsParams::default();
        let mut state = ReviewState::new();
        state.grade(&p, Rating::Good, 0);
        let good_due = state.due.unwrap();
        // A month later the card comes up; the user forgets.
        state.grade(&p, Rating::Again, 30 * DAY);
        assert_eq!(state.lapses, 1);
        let again_due = state.due.unwrap();
        assert!(
            again_due - 30 * DAY < good_due,
            "a lapse must schedule sooner than the prior success did"
        );
        assert!(state.memory.unwrap().stability >= MIN_STABILITY);
    }

    #[test]
    fn difficulty_stays_within_bounds() {
        let p = FsrsParams::default();
        for rating in [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy] {
            let d = p.next_difficulty(1.0, rating);
            assert!((1.0..=10.0).contains(&d), "{rating:?} → {d}");
            let d2 = p.next_difficulty(10.0, rating);
            assert!((1.0..=10.0).contains(&d2), "{rating:?} → {d2}");
        }
    }

    #[test]
    fn scheduling_is_deterministic() {
        let p = FsrsParams::default();
        let run = || {
            let mut s = ReviewState::new();
            s.grade(&p, Rating::Good, 0);
            s.grade(&p, Rating::Hard, 5 * DAY);
            s.grade(&p, Rating::Easy, 20 * DAY);
            s
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn new_card_is_due_now() {
        let s = ReviewState::new();
        assert!(s.is_due(0));
        assert!(s.is_due(i64::MAX));
    }
}
