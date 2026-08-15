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

//! Postgres-backed [`CurationRewardsRepo`] (change: add-curation-rewards) — thin
//! I/O glue (excluded from the coverage gate; the award/settlement semantics are
//! proven against the in-memory fake). The coverage-once, settlement-precedence,
//! and grant-once guarantees are mirrored in SQL: coverage inserts key off the
//! partial unique index (`ON CONFLICT … DO NOTHING`); settlement flips off the
//! `settled_at`/`settled_source` state; the effective value folds stars/verdict the
//! same way `score_rating.rs` does. Table names are fully qualified so it works
//! regardless of the connecting role's `search_path`.

use std::collections::HashSet;

use async_trait::async_trait;
use cymbra_platform::{AppError, Result};
use sqlx::{PgPool, Row};

use crate::curation_rewards::{
    ConsensusCandidate, CurationRewardsRepo, CuratorMetrics, GrantKind, LedgerEntry, SettleOutcome,
    SettleableRating, ShopItem,
};
use crate::curation_rewards_core::{AwardKind, SettlementSource};

/// The SQL fragment that folds a rating's verdict/stars into its effective value
/// on the 1–5 scale — the exact mirror of `score_rating::effective_value`.
const EFFECTIVE_SQL: &str = "COALESCE(stars::float8, \
     CASE verdict WHEN 'dislike' THEN 1.5 WHEN 'like' THEN 3.5 WHEN 'love' THEN 5.0 END)";

fn internal(e: sqlx::Error) -> AppError {
    AppError::Internal(anyhow::anyhow!("curation rewards db: {e}"))
}

fn uid(user_id: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(user_id).map_err(|_| AppError::InvalidArgument("invalid user id".into()))
}

fn sid(score_id: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(score_id)
        .map_err(|_| AppError::InvalidArgument("invalid catalog score id".into()))
}

/// Postgres implementation over the `music_svc` pool (search_path = `music`).
pub struct PgCurationRewardsRepo {
    pool: PgPool,
}

impl PgCurationRewardsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn parse_kind(s: &str) -> AwardKind {
    match s {
        "coverage" => AwardKind::Coverage,
        "adjustment" => AwardKind::Adjustment,
        "performance" => AwardKind::Performance,
        "practice" => AwardKind::Practice,
        "redeem" => AwardKind::Redeem,
        _ => AwardKind::Honesty,
    }
}

fn parse_source(s: Option<String>) -> Option<SettlementSource> {
    match s.as_deref() {
        Some("consensus") => Some(SettlementSource::Consensus),
        Some("moderator") => Some(SettlementSource::Moderator),
        _ => None,
    }
}

#[async_trait]
impl CurationRewardsRepo for PgCurationRewardsRepo {
    async fn record_engagement(&self, user_id: &str, catalog_score_id: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO music.score_engagements (user_id, catalog_score_id) VALUES ($1, $2) \
             ON CONFLICT (user_id, catalog_score_id) DO UPDATE SET engaged_at = now()",
        )
        .bind(uid(user_id)?)
        .bind(sid(catalog_score_id)?)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn has_engagement(&self, user_id: &str, catalog_score_id: &str) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM music.score_engagements \
             WHERE user_id = $1 AND catalog_score_id = $2)",
        )
        .bind(uid(user_id)?)
        .bind(sid(catalog_score_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(exists)
    }

    async fn coverage_today(&self, user_id: &str) -> Result<i64> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0)::bigint FROM music.curation_points \
             WHERE user_id = $1 AND award_kind = 'coverage' \
             AND created_at >= date_trunc('day', now())",
        )
        .bind(uid(user_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(total)
    }

    async fn append_coverage(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        amount: i64,
    ) -> Result<bool> {
        let res = sqlx::query(
            "INSERT INTO music.curation_points (user_id, award_kind, amount, catalog_score_id) \
             VALUES ($1, 'coverage', $2, $3) \
             ON CONFLICT (user_id, catalog_score_id) WHERE award_kind = 'coverage' DO NOTHING",
        )
        .bind(uid(user_id)?)
        .bind(amount as i32)
        .bind(sid(catalog_score_id)?)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(res.rows_affected() > 0)
    }

    async fn ratings_for_moderator_settlement(
        &self,
        catalog_score_id: &str,
        exclude_user: &str,
    ) -> Result<Vec<SettleableRating>> {
        let query = format!(
            "SELECT r.user_id::text AS user_id, {EFFECTIVE_SQL} AS effective, \
             (SELECT COALESCE(SUM(amount), 0)::bigint FROM music.curation_points p \
              WHERE p.user_id = r.user_id AND p.catalog_score_id = r.catalog_score_id \
              AND p.award_kind IN ('honesty', 'adjustment')) AS prior \
             FROM music.score_ratings r \
             WHERE r.catalog_score_id = $1 AND r.user_id <> $2 \
             AND (r.settled_at IS NULL OR r.settled_source = 'consensus')"
        );
        let rows = sqlx::query(&query)
            .bind(sid(catalog_score_id)?)
            .bind(uid(exclude_user)?)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| SettleableRating {
                user_id: r.get::<String, _>("user_id"),
                effective: r.get::<f64, _>("effective"),
                prior_honesty: r.get::<i64, _>("prior"),
            })
            .collect())
    }

    async fn ratings_for_consensus_settlement(
        &self,
        catalog_score_id: &str,
    ) -> Result<Vec<SettleableRating>> {
        let query = format!(
            "SELECT r.user_id::text AS user_id, {EFFECTIVE_SQL} AS effective \
             FROM music.score_ratings r \
             WHERE r.catalog_score_id = $1 AND r.settled_at IS NULL"
        );
        let rows = sqlx::query(&query)
            .bind(sid(catalog_score_id)?)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| SettleableRating {
                user_id: r.get::<String, _>("user_id"),
                effective: r.get::<f64, _>("effective"),
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
        // One statement, race-safe: capture the prior source under a row lock, then
        // conditionally update. A `consensus` source only settles an unsettled row;
        // a `moderator` source also upgrades a `consensus` one (the single override).
        let row = sqlx::query(
            "WITH prev AS ( \
                 SELECT settled_source FROM music.score_ratings \
                 WHERE user_id = $1 AND catalog_score_id = $2 FOR UPDATE \
             ), upd AS ( \
                 UPDATE music.score_ratings SET settled_source = $3, settled_at = now() \
                 WHERE user_id = $1 AND catalog_score_id = $2 \
                 AND (settled_at IS NULL OR (settled_source = 'consensus' AND $3 = 'moderator')) \
                 RETURNING 1 \
             ) \
             SELECT (SELECT settled_source FROM prev) AS prev_source, \
                    EXISTS(SELECT 1 FROM upd) AS updated",
        )
        .bind(uid(user_id)?)
        .bind(sid(catalog_score_id)?)
        .bind(source.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        let updated: bool = row.get("updated");
        let prev: Option<String> = row.get("prev_source");
        Ok(if !updated {
            SettleOutcome::Already
        } else if prev.as_deref() == Some("consensus") {
            SettleOutcome::FromConsensus
        } else {
            SettleOutcome::Fresh
        })
    }

    async fn append_award(
        &self,
        user_id: &str,
        catalog_score_id: &str,
        amount: i64,
        kind: AwardKind,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO music.curation_points (user_id, award_kind, amount, catalog_score_id) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(uid(user_id)?)
        .bind(kind.as_str())
        .bind(amount as i32)
        .bind(sid(catalog_score_id)?)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn append_play_award(
        &self,
        user_id: &str,
        kind: AwardKind,
        amount: i64,
        piece_id: Option<&str>,
        award_key: &str,
    ) -> Result<bool> {
        // Exactly-once against `curation_points_award_key_once_idx` — a retried
        // ingest re-attempts the award and this turns it into a no-op (design D4).
        let res = sqlx::query(
            "INSERT INTO music.curation_points \
             (user_id, award_kind, amount, piece_id, award_key) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (user_id, award_key) WHERE award_key IS NOT NULL DO NOTHING",
        )
        .bind(uid(user_id)?)
        .bind(kind.as_str())
        .bind(amount as i32)
        .bind(piece_id)
        .bind(award_key)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(res.rows_affected() > 0)
    }

    async fn performance_count_for_piece(&self, user_id: &str, piece_id: &str) -> Result<i64> {
        // Served by `curation_points_piece_paid_idx (user_id, piece_id) WHERE
        // award_kind = 'performance'`.
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM music.curation_points \
             WHERE user_id = $1 AND piece_id = $2 AND award_kind = 'performance'",
        )
        .bind(uid(user_id)?)
        .bind(piece_id)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(n)
    }

    async fn performance_today(&self, user_id: &str) -> Result<i64> {
        // Server-local day, exactly like `coverage_today` — deliberately NOT the
        // player's local day: a tz-keyed cap would let a farmer reset their
        // allowance by changing the device clock's offset. The player-local day
        // matters for the PRACTICE award (which is about their day, and is keyed
        // on it durably), not for the anti-farming ceiling. Served by
        // `curation_points_user_idx (user_id, created_at DESC)`.
        let total: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0)::bigint FROM music.curation_points \
             WHERE user_id = $1 AND award_kind = 'performance' \
             AND created_at >= date_trunc('day', now())",
        )
        .bind(uid(user_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(total)
    }

    async fn consensus_candidates(&self, min_raters: i64) -> Result<Vec<ConsensusCandidate>> {
        let query = format!(
            "SELECT catalog_score_id::text AS id, \
             AVG({EFFECTIVE_SQL}) AS avg_effective, COUNT(*)::bigint AS rater_count \
             FROM music.score_ratings \
             GROUP BY catalog_score_id \
             HAVING COUNT(*) >= $1 \
             AND NOT EXISTS (SELECT 1 FROM music.score_consensus_settlements c \
                             WHERE c.catalog_score_id = score_ratings.catalog_score_id) \
             ORDER BY catalog_score_id"
        );
        let rows = sqlx::query(&query)
            .bind(min_raters)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| ConsensusCandidate {
                catalog_score_id: r.get::<String, _>("id"),
                avg_effective: r.get::<f64, _>("avg_effective"),
                rater_count: r.get::<i64, _>("rater_count"),
            })
            .collect())
    }

    async fn freeze_consensus(
        &self,
        catalog_score_id: &str,
        truth_positive: Option<bool>,
        avg_effective: f64,
        rater_count: i64,
    ) -> Result<bool> {
        let res = sqlx::query(
            "INSERT INTO music.score_consensus_settlements \
             (catalog_score_id, truth_positive, avg_effective, rater_count) \
             VALUES ($1, $2, $3, $4) ON CONFLICT (catalog_score_id) DO NOTHING",
        )
        .bind(sid(catalog_score_id)?)
        .bind(truth_positive)
        .bind(avg_effective)
        .bind(rater_count)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(res.rows_affected() > 0)
    }

    async fn lifetime_points(&self, user_id: &str) -> Result<i64> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0)::bigint FROM music.curation_points \
             WHERE user_id = $1 AND award_kind <> 'redeem'",
        )
        .bind(uid(user_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(total)
    }

    async fn spendable_balance(&self, user_id: &str) -> Result<i64> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0)::bigint FROM music.curation_points WHERE user_id = $1",
        )
        .bind(uid(user_id)?)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(total)
    }

    async fn recent_awards(&self, user_id: &str, limit: i64) -> Result<Vec<LedgerEntry>> {
        let rows = sqlx::query(
            "SELECT p.award_kind, p.amount, p.catalog_score_id::text AS score, p.reward_key, \
             (extract(epoch FROM p.created_at) * 1000)::bigint AS ms, r.settled_source \
             FROM music.curation_points p \
             LEFT JOIN music.score_ratings r \
               ON r.user_id = p.user_id AND r.catalog_score_id = p.catalog_score_id \
             WHERE p.user_id = $1 \
             ORDER BY p.created_at DESC, p.id DESC LIMIT $2",
        )
        .bind(uid(user_id)?)
        .bind(limit.max(0))
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let kind = parse_kind(&r.get::<String, _>("award_kind"));
                let source = match kind {
                    AwardKind::Honesty | AwardKind::Adjustment => {
                        parse_source(r.get::<Option<String>, _>("settled_source"))
                    }
                    _ => None,
                };
                LedgerEntry {
                    kind,
                    amount: r.get::<i32, _>("amount") as i64,
                    catalog_score_id: r.get::<Option<String>, _>("score"),
                    reward_key: r.get::<Option<String>, _>("reward_key"),
                    source,
                    created_at_ms: r.get::<i64, _>("ms"),
                }
            })
            .collect())
    }

    async fn granted_keys(&self, user_id: &str) -> Result<HashSet<String>> {
        let rows: Vec<String> =
            sqlx::query_scalar("SELECT key FROM music.curation_grants WHERE user_id = $1")
                .bind(uid(user_id)?)
                .fetch_all(&self.pool)
                .await
                .map_err(internal)?;
        Ok(rows.into_iter().collect())
    }

    async fn insert_grant(&self, user_id: &str, key: &str, kind: GrantKind) -> Result<bool> {
        let res = sqlx::query(
            "INSERT INTO music.curation_grants (user_id, grant_kind, key) VALUES ($1, $2, $3) \
             ON CONFLICT (user_id, key) DO NOTHING",
        )
        .bind(uid(user_id)?)
        .bind(kind.as_str())
        .bind(key)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(res.rows_affected() > 0)
    }

    async fn append_redeem(&self, user_id: &str, key: &str, cost: i64) -> Result<()> {
        sqlx::query(
            "INSERT INTO music.curation_points (user_id, award_kind, amount, reward_key) \
             VALUES ($1, 'redeem', $2, $3)",
        )
        .bind(uid(user_id)?)
        .bind(-(cost as i32))
        .bind(key)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(())
    }

    async fn curator_metrics(
        &self,
        user_id: &str,
        honesty_floor: i64,
        first_rater_points: i64,
    ) -> Result<CuratorMetrics> {
        let row = sqlx::query(
            "SELECT \
             (SELECT COUNT(*)::bigint FROM music.score_ratings WHERE user_id = $1) AS total_ratings, \
             (SELECT COUNT(*)::bigint FROM music.curation_points \
              WHERE user_id = $1 AND award_kind = 'coverage') AS coverage_contribution, \
             (SELECT COUNT(*)::bigint FROM music.curation_points \
              WHERE user_id = $1 AND award_kind = 'coverage' AND amount >= $3) AS first_rater_count, \
             (SELECT COUNT(*)::bigint FROM music.score_ratings \
              WHERE user_id = $1 AND settled_at IS NOT NULL) AS settled_count, \
             (SELECT COUNT(*)::bigint FROM music.score_ratings r \
              WHERE r.user_id = $1 AND r.settled_at IS NOT NULL \
              AND (SELECT COALESCE(SUM(amount), 0) FROM music.curation_points p \
                   WHERE p.user_id = r.user_id AND p.catalog_score_id = r.catalog_score_id \
                   AND p.award_kind IN ('honesty', 'adjustment')) > $2) AS aligned_count",
        )
        .bind(uid(user_id)?)
        .bind(honesty_floor)
        .bind(first_rater_points as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(internal)?;
        Ok(CuratorMetrics {
            total_ratings: row.get::<i64, _>("total_ratings"),
            coverage_contribution: row.get::<i64, _>("coverage_contribution"),
            aligned_count: row.get::<i64, _>("aligned_count"),
            settled_count: row.get::<i64, _>("settled_count"),
            first_rater_count: row.get::<i64, _>("first_rater_count"),
        })
    }

    async fn shop_items(&self, user_id: &str) -> Result<Vec<ShopItem>> {
        let rows = sqlx::query(
            "SELECT s.id AS key, s.label, s.instrument, s.license, s.attribution, \
             s.point_cost, s.redeemable, \
             (s.point_cost = 0 OR EXISTS(SELECT 1 FROM music.curation_grants g \
               WHERE g.user_id = $1 AND g.key = s.id)) AS owned \
             FROM music.soundfonts s \
             WHERE s.moderation_status = 'accepted' \
               AND (s.point_cost > 0 OR s.redeemable = FALSE) \
             ORDER BY s.point_cost, s.label",
        )
        .bind(uid(user_id)?)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        Ok(rows.into_iter().map(row_to_shop_item).collect())
    }

    async fn shop_item(&self, user_id: &str, key: &str) -> Result<Option<ShopItem>> {
        let row = sqlx::query(
            "SELECT s.id AS key, s.label, s.instrument, s.license, s.attribution, \
             s.point_cost, s.redeemable, \
             (s.point_cost = 0 OR EXISTS(SELECT 1 FROM music.curation_grants g \
               WHERE g.user_id = $1 AND g.key = s.id)) AS owned \
             FROM music.soundfonts s \
             WHERE s.id = $2 AND s.moderation_status = 'accepted'",
        )
        .bind(uid(user_id)?)
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        Ok(row.map(row_to_shop_item))
    }
}

fn row_to_shop_item(r: sqlx::postgres::PgRow) -> ShopItem {
    ShopItem {
        key: r.get::<String, _>("key"),
        label: r.get::<String, _>("label"),
        instrument: r.get::<String, _>("instrument"),
        license: r.get::<String, _>("license"),
        attribution: r.get::<Option<String>, _>("attribution"),
        point_cost: r.get::<i32, _>("point_cost") as i64,
        redeemable: r.get::<bool, _>("redeemable"),
        owned: r.get::<bool, _>("owned"),
    }
}
