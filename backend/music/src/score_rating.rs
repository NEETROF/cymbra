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

//! Per-user score ratings (change: add-app-score-rating).
//!
//! A signed-in user rates an `accepted` catalog score with a swipe [`Verdict`]
//! (dislike / like / love) and an optional 1–5 star value; there is at most one
//! rating per (user, score) — re-rating **upserts** the row. This file holds the
//! owner-scoped data-access port ([`ScoreRatingRepo`]) with its in-memory fake,
//! plus the pure rating math (the effective value of one rating, the per-score
//! [`RatingAggregate`], and the hybrid re-review flag) so the whole computation is
//! host-testable without a database — the Postgres adapter (coverage-excluded)
//! mirrors the same arithmetic in SQL.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};

/// A swipe verdict: the coarse "how much do you like this" signal. Left = dislike
/// (a negative verdict), right = like, up = love.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Dislike,
    Like,
    Love,
}

impl Verdict {
    /// Parse the wire string; an unknown value is an `InvalidArgument` (rejected
    /// before any write).
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "dislike" => Ok(Verdict::Dislike),
            "like" => Ok(Verdict::Like),
            "love" => Ok(Verdict::Love),
            other => Err(AppError::InvalidArgument(format!(
                "unknown verdict {other:?}"
            ))),
        }
    }

    /// The wire string (also the persisted `verdict` value).
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Dislike => "dislike",
            Verdict::Like => "like",
            Verdict::Love => "love",
        }
    }

    /// The verdict's implied value on the 1–5 scale, used for the aggregate when a
    /// rating carries no explicit stars (design D2): a `dislike` pulls the average
    /// down (the negative signal the re-review flag needs), `like` is mid-high,
    /// `love` the maximum.
    pub fn implied_value(self) -> f64 {
        match self {
            Verdict::Dislike => 1.5,
            Verdict::Like => 3.5,
            Verdict::Love => 5.0,
        }
    }
}

/// The effective numeric value of one rating on the 1–5 scale (design D2): the
/// explicit `stars` when present, else the verdict's implied value. This is the
/// single comparable scale the aggregate averages over.
pub fn effective_value(verdict: Verdict, stars: Option<i16>) -> f64 {
    match stars {
        Some(s) => f64::from(s),
        None => verdict.implied_value(),
    }
}

/// Tunable thresholds for the hybrid re-review flag (design D4). Config, not
/// schema — tunable without a migration. Defaults: **N = 5 ratings, T = 2.0** on
/// the 1–5 scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RatingConfig {
    /// Minimum ratings before a low aggregate can flag a score (`N`).
    pub min_count: i64,
    /// Aggregate at or below which a score with ≥ `min_count` ratings is flagged
    /// for re-review (`T`, on the 1–5 scale).
    pub review_threshold: f64,
}

impl Default for RatingConfig {
    fn default() -> Self {
        Self {
            min_count: 5,
            review_threshold: 2.0,
        }
    }
}

/// The per-score aggregate (design D3): the average effective value on the 1–5
/// scale, the number of ratings, and the verdict breakdown. `avg_effective` is
/// `0.0` when there are no ratings.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RatingAggregate {
    pub avg_effective: f64,
    pub count: i64,
    pub dislike: i64,
    pub like: i64,
    pub love: i64,
}

/// The hybrid re-review flag (design D4): a validated score is **eligible for
/// moderator re-review** once it has at least `min_count` ratings AND its average
/// effective value is at or below `review_threshold`. Below the minimum count it
/// is never flagged, however low the average — so a single harsh dislike can't
/// flag a score. The flag never changes `moderation_status`; a moderator decides.
pub fn is_flagged_for_review(agg: &RatingAggregate, cfg: &RatingConfig) -> bool {
    agg.count >= cfg.min_count && agg.avg_effective <= cfg.review_threshold
}

/// One user's own rating of one score, as read back (change:
/// add-post-play-rating-prompt). `stars` is `None` for a swipe-only rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserRating {
    pub verdict: Verdict,
    pub stars: Option<i16>,
}

/// Owner-scoped storage for one user's rating of one catalog score. Upsert
/// enforces the single-row-per-(user, score) invariant; the aggregate is read
/// per score.
#[async_trait]
pub trait ScoreRatingRepo: Send + Sync {
    /// Record or update the caller's rating of a score (one row per user+score):
    /// on conflict, overwrite `verdict`/`stars` and bump `updated_at`.
    async fn upsert(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        verdict: Verdict,
        stars: Option<i16>,
    ) -> Result<()>;

    /// The per-score aggregate over every user's rating of `catalog_score_id`
    /// (average effective value + count + verdict breakdown). A score with no
    /// ratings yields the default (count 0, avg 0.0).
    async fn aggregate(&self, catalog_score_id: &str) -> Result<RatingAggregate>;

    /// The caller's OWN rating of one score (change: add-post-play-rating-prompt),
    /// or `None` when they have not rated it. Owner-scoped by construction — it
    /// filters on `user_id`, so it can never surface another user's rating.
    async fn find_by_user(
        &self,
        user_id: &str,
        catalog_score_id: &str,
    ) -> Result<Option<UserRating>>;

    /// How many ratings `user_id` has recorded within the last `window` (by
    /// `updated_at`). Feeds the catalog access limiter's engagement allowance
    /// (change: add-catalog-access-limits): rating a score in the swipe deck is
    /// genuine engagement, so it earns download headroom just like playing.
    async fn count_recent_by_user(&self, user_id: &str, window: Duration) -> Result<u64>;
}

/// In-memory [`ScoreRatingRepo`] for unit tests. Computes the aggregate in Rust
/// with [`effective_value`], mirroring the Pg adapter's SQL average, so the module
/// tests exercise the real rating math without a database.
#[derive(Default)]
pub struct FakeScoreRatingRepo {
    rows: Mutex<Vec<Rating>>,
}

#[derive(Clone)]
struct Rating {
    user_id: String,
    catalog_score_id: String,
    verdict: Verdict,
    stars: Option<i16>,
}

impl FakeScoreRatingRepo {
    /// Total stored rows (test assertions — asserts upsert never duplicates).
    pub fn len(&self) -> usize {
        self.rows.lock().expect("score_rating fake lock").len()
    }

    /// Whether the repo holds no ratings.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The catalog-score ids `user_id` has already rated — the deck-sourcing
    /// exclusion set (mirrors the Pg `LEFT JOIN … r.user_id IS NULL`).
    pub fn rated_ids(&self, user_id: &str) -> std::collections::HashSet<String> {
        self.rows
            .lock()
            .expect("score_rating fake lock")
            .iter()
            .filter(|r| r.user_id == user_id)
            .map(|r| r.catalog_score_id.clone())
            .collect()
    }

    /// How many ratings a score has (the deck orders least-rated first).
    pub fn rating_count(&self, catalog_score_id: &str) -> i64 {
        self.rows
            .lock()
            .expect("score_rating fake lock")
            .iter()
            .filter(|r| r.catalog_score_id == catalog_score_id)
            .count() as i64
    }
}

#[async_trait]
impl ScoreRatingRepo for FakeScoreRatingRepo {
    async fn upsert(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        verdict: Verdict,
        stars: Option<i16>,
    ) -> Result<()> {
        let mut rows = self.rows.lock().expect("score_rating fake lock");
        match rows
            .iter_mut()
            .find(|r| r.user_id == user_id && r.catalog_score_id == catalog_score_id)
        {
            Some(existing) => {
                existing.verdict = verdict;
                existing.stars = stars;
            }
            None => rows.push(Rating {
                user_id: user_id.to_string(),
                catalog_score_id: catalog_score_id.to_string(),
                verdict,
                stars,
            }),
        }
        Ok(())
    }

    async fn aggregate(&self, catalog_score_id: &str) -> Result<RatingAggregate> {
        let rows = self.rows.lock().expect("score_rating fake lock");
        let mine = rows
            .iter()
            .filter(|r| r.catalog_score_id == catalog_score_id);
        let mut agg = RatingAggregate::default();
        let mut sum = 0.0;
        for r in mine {
            agg.count += 1;
            sum += effective_value(r.verdict, r.stars);
            match r.verdict {
                Verdict::Dislike => agg.dislike += 1,
                Verdict::Like => agg.like += 1,
                Verdict::Love => agg.love += 1,
            }
        }
        agg.avg_effective = if agg.count == 0 {
            0.0
        } else {
            sum / agg.count as f64
        };
        Ok(agg)
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        catalog_score_id: &str,
    ) -> Result<Option<UserRating>> {
        let rows = self.rows.lock().expect("score_rating fake lock");
        Ok(rows
            .iter()
            .find(|r| r.user_id == user_id && r.catalog_score_id == catalog_score_id)
            .map(|r| UserRating {
                verdict: r.verdict,
                stars: r.stars,
            }))
    }

    /// The in-memory fake carries no timestamps, so it counts **all** of the user's
    /// ratings (the `window` is ignored). Tests control the count by seeding rows.
    async fn count_recent_by_user(&self, user_id: &str, _window: Duration) -> Result<u64> {
        let rows = self.rows.lock().expect("score_rating fake lock");
        Ok(rows.iter().filter(|r| r.user_id == user_id).count() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_parse_and_implied_values() {
        assert_eq!(Verdict::parse("dislike").unwrap(), Verdict::Dislike);
        assert_eq!(Verdict::parse("like").unwrap(), Verdict::Like);
        assert_eq!(Verdict::parse("love").unwrap(), Verdict::Love);
        assert!(matches!(
            Verdict::parse("meh"),
            Err(AppError::InvalidArgument(_))
        ));
        // A dislike pulls down, love tops out (the ordering the flag relies on).
        assert!(Verdict::Dislike.implied_value() < Verdict::Like.implied_value());
        assert!(Verdict::Like.implied_value() < Verdict::Love.implied_value());
    }

    #[test]
    fn effective_value_prefers_explicit_stars() {
        // Explicit stars win over the verdict's implied value…
        assert_eq!(effective_value(Verdict::Dislike, Some(4)), 4.0);
        // …and a verdict-only rating uses the implied value.
        assert_eq!(effective_value(Verdict::Love, None), 5.0);
        assert_eq!(effective_value(Verdict::Dislike, None), 1.5);
    }

    #[tokio::test]
    async fn upsert_keeps_one_row_per_user_and_score() {
        let repo = FakeScoreRatingRepo::default();
        repo.upsert("u1", "s1", Verdict::Like, None).await.unwrap();
        // Re-rating the same score overwrites (no duplicate row).
        repo.upsert("u1", "s1", Verdict::Love, Some(5))
            .await
            .unwrap();
        // A different user, same score, is a separate row.
        repo.upsert("u2", "s1", Verdict::Dislike, None)
            .await
            .unwrap();
        assert_eq!(repo.len(), 2);
        let agg = repo.aggregate("s1").await.unwrap();
        assert_eq!(agg.count, 2);
        assert_eq!(agg.love, 1); // u1's upserted verdict
        assert_eq!(agg.dislike, 1); // u2's verdict
        assert_eq!(agg.like, 0); // u1's original like was overwritten
    }

    #[tokio::test]
    async fn aggregate_folds_stars_and_verdicts_onto_one_scale() {
        let repo = FakeScoreRatingRepo::default();
        // Explicit 2 stars, verdict-only dislike (1.5), verdict-only love (5.0).
        repo.upsert("u1", "s1", Verdict::Like, Some(2))
            .await
            .unwrap();
        repo.upsert("u2", "s1", Verdict::Dislike, None)
            .await
            .unwrap();
        repo.upsert("u3", "s1", Verdict::Love, None).await.unwrap();
        let agg = repo.aggregate("s1").await.unwrap();
        assert_eq!(agg.count, 3);
        // (2.0 + 1.5 + 5.0) / 3 = 2.8333…
        assert!((agg.avg_effective - (8.5 / 3.0)).abs() < 1e-9);
        // Another score with no ratings aggregates to the zero default.
        let empty = repo.aggregate("s2").await.unwrap();
        assert_eq!(empty, RatingAggregate::default());
        assert_eq!(empty.avg_effective, 0.0);
    }

    #[test]
    fn flag_requires_both_min_count_and_low_average() {
        let cfg = RatingConfig::default(); // N = 5, T = 2.0
        // Enough votes AND a low average → flagged.
        let low = RatingAggregate {
            avg_effective: 1.8,
            count: 5,
            dislike: 5,
            ..Default::default()
        };
        assert!(is_flagged_for_review(&low, &cfg));
        // Low average but too few votes → NOT flagged (a harsh minority can't flag).
        let few = RatingAggregate {
            avg_effective: 1.5,
            count: 4,
            dislike: 4,
            ..Default::default()
        };
        assert!(!is_flagged_for_review(&few, &cfg));
        // Enough votes but a healthy average → NOT flagged.
        let healthy = RatingAggregate {
            avg_effective: 4.2,
            count: 20,
            ..Default::default()
        };
        assert!(!is_flagged_for_review(&healthy, &cfg));
        // Exactly at both bounds (count == N, avg == T) → flagged (inclusive).
        let boundary = RatingAggregate {
            avg_effective: 2.0,
            count: 5,
            ..Default::default()
        };
        assert!(is_flagged_for_review(&boundary, &cfg));
    }
}
