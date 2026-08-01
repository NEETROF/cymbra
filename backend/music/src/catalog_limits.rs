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

//! Per-user catalog access guardrail (change: add-catalog-access-limits).
//!
//! Stops an authenticated token from being used to scrape the whole catalog while
//! never penalising a user whose downloads track their play. Two guardrails, keyed
//! on the caller's `AuthIdentity.user_id`, enforced at the gRPC layer before any
//! storage read:
//!
//! - **Download** (`GetCatalogScoreBytes` / `GetRatingPreviewBytes`): a short-window
//!   **burst** cap (pure rate) plus a **play-aware volume** allowance
//!   `min(hard_ceiling, base_floor + per_play * plays_in_window)` — so downloads
//!   proportional to play are never blocked, and only a download-heavy/play-light
//!   profile falls back to the floor.
//! - **Enumeration** (`SearchCatalog` / `GetCatalogScore` / `ListRatingDeck`): a
//!   request-rate cap to slow programmatic catalog walk-through.
//!
//! A **music-scope admin** (a `music/admin` or the `global/admin` break-glass) is
//! exempt; moderators and admins scoped only to another domain (e.g. `live`) are
//! not. `enabled=false` is the kill-switch. Counters ride the shared `Cache` via
//! `cymbra_platform::ratelimit::check`, so limits hold across instances.

use std::sync::Arc;
use std::time::Duration;

use cymbra_platform::cache::Cache;
use cymbra_platform::config::CatalogLimitsConfig;
use cymbra_platform::{AppError, AuthIdentity, Result, ratelimit};

use crate::play::PlayRepo;

/// TTL for the cached per-user in-window play count. The allowance only needs to be
/// approximately fresh, so a short TTL keeps the hot download path off `PlayRepo`.
const PLAYS_CACHE_TTL: Duration = Duration::from_secs(60);

/// Pure, host-testable allowance math (no I/O, no clock).
pub mod catalog_limits_core {
    /// The play-aware download allowance:
    /// `min(hard_ceiling, base_floor + per_play * plays)`.
    pub fn effective_allowance(
        base_floor: u32,
        per_play: u32,
        plays: u64,
        hard_ceiling: u32,
    ) -> u32 {
        let earned = (base_floor as u64).saturating_add((per_play as u64).saturating_mul(plays));
        earned.min(hard_ceiling as u64) as u32
    }

    /// Count play-session timestamps that fall within `[now_ms - window_ms, now_ms]`.
    pub fn plays_in_window<I: IntoIterator<Item = i64>>(
        points_ms: I,
        now_ms: i64,
        window_ms: i64,
    ) -> u64 {
        let cutoff = now_ms.saturating_sub(window_ms);
        points_ms.into_iter().filter(|&ms| ms >= cutoff).count() as u64
    }
}

/// Enforces the per-user catalog access limits over the shared cache and play port.
pub struct CatalogAccessLimiter {
    cache: Arc<dyn Cache>,
    plays: Arc<dyn PlayRepo>,
    cfg: CatalogLimitsConfig,
}

impl CatalogAccessLimiter {
    pub fn new(cache: Arc<dyn Cache>, plays: Arc<dyn PlayRepo>, cfg: CatalogLimitsConfig) -> Self {
        Self { cache, plays, cfg }
    }

    /// Who bypasses the guardrail. Exempt:
    /// - the **back-office** audience — a trusted, CORS-gated curator console (a
    ///   different audience than the music app). The scrape threat model is the
    ///   music-app token, and the play-aware allowance is meaningless for a console
    ///   whose users never play (they'd sit permanently at the base floor); and
    /// - a **`music`-scope admin** (a `music/admin` or the `global/admin`
    ///   break-glass), for legitimate bulk operations.
    ///
    /// Every other caller — music-app moderators, other-scope admins, regular users
    /// — is subject.
    fn exempt(&self, id: &AuthIdentity) -> bool {
        id.audience == cymbra_platform::BACKOFFICE_AUDIENCE
            || id.has_role_in_scope("music", "admin")
    }

    /// Burst cap + play-aware volume allowance on raw-bytes egress. `Ok` when the
    /// guard is disabled, the caller is exempt, or both tiers pass; `ResourceExhausted`
    /// on breach (checked before any storage read, so a reject never egresses bytes).
    pub async fn check_download(&self, id: &AuthIdentity) -> Result<()> {
        if !self.cfg.enabled || self.exempt(id) {
            return Ok(());
        }
        let subject = &id.user_id;
        // Tier 1 — burst (pure rate), independent of play.
        log_reject(
            ratelimit::check(
                self.cache.as_ref(),
                "cat_dl_burst",
                subject,
                self.cfg.download_burst_max,
                self.cfg.download_burst_window,
            )
            .await,
            subject,
            "download-burst",
        )?;
        // Tier 2 — play-aware volume allowance.
        let plays = self.plays_in_window(subject).await;
        let effective = catalog_limits_core::effective_allowance(
            self.cfg.volume_base_floor,
            self.cfg.volume_per_play,
            plays,
            self.cfg.volume_hard_ceiling,
        );
        log_reject(
            ratelimit::check(
                self.cache.as_ref(),
                "cat_dl_vol",
                subject,
                effective,
                self.cfg.volume_window,
            )
            .await,
            subject,
            "download-volume",
        )
    }

    /// Request-rate cap on catalog enumeration (search / browse / rating deck).
    pub async fn check_enumeration(&self, id: &AuthIdentity) -> Result<()> {
        if !self.cfg.enabled || self.exempt(id) {
            return Ok(());
        }
        log_reject(
            ratelimit::check(
                self.cache.as_ref(),
                "cat_enum",
                &id.user_id,
                self.cfg.enum_max,
                self.cfg.enum_window,
            )
            .await,
            &id.user_id,
            "enumeration",
        )
    }

    /// In-window play-session count, cached in the shared cache with a short TTL so
    /// the hot download path avoids a `PlayRepo` round-trip. Any cache/`PlayRepo`
    /// error fails safe to 0 plays — the caller then gets exactly the base floor.
    async fn plays_in_window(&self, user_id: &str) -> u64 {
        let key = format!("catplays:{user_id}");
        if let Ok(Some(v)) = self.cache.get(&key).await
            && let Ok(n) = v.parse::<u64>()
        {
            return n;
        }
        let now_ms = chrono::Utc::now().timestamp_millis();
        let window_ms = self.cfg.volume_window.as_millis() as i64;
        let n = match self.plays.session_points(user_id).await {
            Ok(points) => catalog_limits_core::plays_in_window(
                points.iter().map(|p| p.played_at_ms),
                now_ms,
                window_ms,
            ),
            Err(_) => 0, // fail-safe: unavailable play data ⇒ base floor
        };
        // Best-effort cache write; a failure just means we recompute next time.
        let _ = self
            .cache
            .set_ex(&key, &n.to_string(), PLAYS_CACHE_TTL)
            .await;
        n
    }
}

/// Emit an operator-visible warning when a limit rejects, then pass the result
/// through unchanged (observability task: a scrape shows as sustained rejections).
fn log_reject(r: Result<()>, subject: &str, which: &str) -> Result<()> {
    if let Err(AppError::ResourceExhausted(_)) = &r {
        tracing::warn!(user_id = %subject, limit = which, "catalog access limit reached");
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::{FakePlayRepo, PlaySession};
    use cymbra_platform::cache::FakeCache;
    use std::collections::BTreeMap;

    fn cfg() -> CatalogLimitsConfig {
        CatalogLimitsConfig {
            enabled: true,
            download_burst_max: 5,
            download_burst_window: Duration::from_secs(60),
            volume_window: Duration::from_secs(24 * 3600),
            volume_base_floor: 3,
            volume_per_play: 2,
            volume_hard_ceiling: 20,
            enum_max: 4,
            enum_window: Duration::from_secs(60),
        }
    }

    fn user(user_id: &str, roles_by_scope: &[(&str, &[&str])]) -> AuthIdentity {
        let map: BTreeMap<String, Vec<String>> = roles_by_scope
            .iter()
            .map(|(s, rs)| (s.to_string(), rs.iter().map(|r| r.to_string()).collect()))
            .collect();
        let roles = map.values().flatten().cloned().collect();
        AuthIdentity {
            user_id: user_id.into(),
            audience: "music".into(),
            roles,
            roles_by_scope: map,
        }
    }

    /// A regular music user (no elevated role).
    fn regular(user_id: &str) -> AuthIdentity {
        user(user_id, &[("global", &["user"])])
    }

    fn limiter(plays: Arc<FakePlayRepo>) -> CatalogAccessLimiter {
        CatalogAccessLimiter::new(Arc::new(FakeCache::default()), plays, cfg())
    }

    /// Seed `n` recent play sessions for `user_id` (all inside the volume window).
    async fn seed_plays(repo: &FakePlayRepo, user_id: &str, n: usize) {
        let now = chrono::Utc::now().timestamp_millis();
        for i in 0..n {
            repo.record(&PlaySession {
                session_id: format!("{user_id}-{i}"),
                user_id: user_id.into(),
                score_id: None,
                played_at_ms: now,
                tz_offset_minutes: 0,
                overall_sync_pct: 50.0,
                session_result_json: String::new(),
            })
            .await
            .unwrap();
        }
    }

    #[test]
    fn allowance_is_floor_plus_play_capped_by_ceiling() {
        // base 3 + 2*plays, capped at 20.
        assert_eq!(catalog_limits_core::effective_allowance(3, 2, 0, 20), 3);
        assert_eq!(catalog_limits_core::effective_allowance(3, 2, 4, 20), 11);
        assert_eq!(catalog_limits_core::effective_allowance(3, 2, 100, 20), 20);
    }

    #[test]
    fn plays_in_window_counts_only_recent() {
        // window 1000ms ending at now=10_000 ⇒ cutoff 9_000.
        let pts = [9_500i64, 9_000, 8_999, 10_000];
        assert_eq!(catalog_limits_core::plays_in_window(pts, 10_000, 1_000), 3);
    }

    #[tokio::test]
    async fn burst_cap_rejects_a_fast_flood() {
        // Seed enough plays that the volume allowance (min(20, 3+2*9)=20) far exceeds
        // the burst cap (5), so the burst tier is the binding limit.
        let plays = Arc::new(FakePlayRepo::default());
        seed_plays(&plays, "u1", 9).await;
        let l = limiter(plays);
        let id = regular("u1");
        for _ in 0..5 {
            l.check_download(&id).await.expect("within burst cap");
        }
        assert!(matches!(
            l.check_download(&id).await,
            Err(AppError::ResourceExhausted(_))
        ));
    }

    #[tokio::test]
    async fn downloads_proportional_to_play_are_not_blocked() {
        let plays = Arc::new(FakePlayRepo::default());
        seed_plays(&plays, "u1", 8).await; // allowance = min(20, 3 + 2*8) = 19
        let l = limiter(plays);
        let id = regular("u1");
        // Downloads 4 and 5 are above the base floor (3) yet still succeed — the
        // volume tier tracks play. (Kept within the burst cap of 5, which is a pure
        // rate limit that applies to everyone and is exercised separately.)
        for _ in 0..5 {
            l.check_download(&id)
                .await
                .expect("engaged user not blocked");
        }
    }

    #[tokio::test]
    async fn download_heavy_play_light_falls_back_to_floor() {
        let l = limiter(Arc::new(FakePlayRepo::default())); // 0 plays ⇒ allowance = floor 3
        let id = regular("u1");
        for _ in 0..3 {
            l.check_download(&id).await.expect("up to the floor");
        }
        assert!(matches!(
            l.check_download(&id).await,
            Err(AppError::ResourceExhausted(_))
        ));
    }

    #[tokio::test]
    async fn music_admin_and_global_admin_bypass() {
        let l = limiter(Arc::new(FakePlayRepo::default()));
        let music_admin = user("a1", &[("global", &["user"]), ("music", &["admin"])]);
        let global_admin = user("a2", &[("global", &["admin"])]);
        for _ in 0..50 {
            l.check_download(&music_admin)
                .await
                .expect("music admin exempt");
            l.check_download(&global_admin)
                .await
                .expect("global admin exempt");
            l.check_enumeration(&music_admin).await.expect("exempt");
        }
    }

    #[tokio::test]
    async fn back_office_audience_is_exempt_even_as_moderator() {
        // The curator console reuses GetCatalogScoreBytes but is a trusted, CORS-gated
        // audience with no play activity — it must never be limited, even for a
        // moderator (who would otherwise sit at the base floor).
        let l = limiter(Arc::new(FakePlayRepo::default()));
        let mut bo_mod = user("bo1", &[("global", &["user"]), ("music", &["moderator"])]);
        bo_mod.audience = cymbra_platform::BACKOFFICE_AUDIENCE.into();
        for _ in 0..50 {
            l.check_download(&bo_mod).await.expect("back-office exempt");
            l.check_enumeration(&bo_mod).await.expect("back-office exempt");
        }
    }

    #[tokio::test]
    async fn moderator_and_other_scope_admin_are_subject() {
        let l = limiter(Arc::new(FakePlayRepo::default()));
        let moderator = user("m1", &[("global", &["user"]), ("music", &["moderator"])]);
        let live_admin = user("l1", &[("global", &["user"]), ("live", &["admin"])]);
        for id in [&moderator, &live_admin] {
            let mut rejected = false;
            for _ in 0..8 {
                if l.check_download(id).await.is_err() {
                    rejected = true;
                    break;
                }
            }
            assert!(rejected, "non-music-admin must be subject to the limit");
        }
    }

    #[tokio::test]
    async fn enumeration_cap_rejects_rapid_browsing() {
        let l = limiter(Arc::new(FakePlayRepo::default()));
        let id = regular("u1");
        for _ in 0..4 {
            l.check_enumeration(&id).await.expect("within enum cap");
        }
        assert!(matches!(
            l.check_enumeration(&id).await,
            Err(AppError::ResourceExhausted(_))
        ));
    }

    #[tokio::test]
    async fn kill_switch_disables_enforcement() {
        let mut c = cfg();
        c.enabled = false;
        let l = CatalogAccessLimiter::new(
            Arc::new(FakeCache::default()),
            Arc::new(FakePlayRepo::default()),
            c,
        );
        let id = regular("u1");
        for _ in 0..100 {
            l.check_download(&id).await.expect("disabled ⇒ no limit");
            l.check_enumeration(&id).await.expect("disabled ⇒ no limit");
        }
    }

    #[tokio::test]
    async fn limits_are_isolated_per_user() {
        let l = limiter(Arc::new(FakePlayRepo::default()));
        let a = regular("a");
        let b = regular("b");
        // Exhaust a's floor (3).
        for _ in 0..3 {
            l.check_download(&a).await.unwrap();
        }
        assert!(l.check_download(&a).await.is_err());
        // b is unaffected.
        l.check_download(&b).await.expect("other user unaffected");
    }
}
