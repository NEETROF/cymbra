//! Worker configuration (task 7.3). [`core::parse`] is pure and host-testable
//! over a key/value map; [`WorkerConfig::from_env`] just collects the process
//! environment into that map. Mirrors `cymbra-platform`'s config style.

use std::collections::HashMap;
use std::time::Duration;

/// Fully-resolved worker configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerConfig {
    /// `worker_svc` connection — the queue, scheduler, and dead-letter sweep.
    pub worker_database_url: String,
    /// `user_svc` connection — used by the orphan-reaper job handler.
    pub user_database_url: String,
    /// `auth_svc` connection — used by the session-reaper job handler.
    pub auth_database_url: String,
    /// `admin_svc` connection — used by the `purge_user` job to erase a deleted
    /// user's data across the `user_account` and `auth` schemas in one atomic
    /// transaction. Worker-only: `admin_svc` crosses schema isolation (D0) and
    /// MUST NEVER be wired into an application service.
    pub admin_database_url: String,
    pub smtp_url: String,
    pub smtp_from: String,
    /// Health/readiness HTTP surface.
    pub http_addr: String,
    /// sqlxmq runner concurrency bounds (design D7 — the operational tunable).
    pub concurrency_min: usize,
    pub concurrency_max: usize,
    /// How often the recurring scheduler evaluates `jobs.schedules`.
    pub scheduler_interval: Duration,
    /// How often the dead-letter sweep runs.
    pub dlq_sweep_interval: Duration,
    /// Grace before a handle-less account is reaped (reused by the reaper job).
    pub orphan_reap_grace: Duration,
    /// OTLP export endpoint (traces/metrics/logs); ignored when disabled.
    pub otlp_endpoint: Option<String>,
    /// Whether to export OpenTelemetry over OTLP (off for tests/offline runs).
    pub otlp_enabled: bool,
    /// Object store for the `purge_score_object` job. `None` when unconfigured
    /// (the score-upload feature is off); a *partial* S3 config is a hard error.
    pub score_storage: Option<cymbra_platform::config::ScoreStorageConfig>,
    /// Local warm-cache root (mirrors the server's `CYMBRA_SCORE_LOCAL_ROOT`).
    pub score_local_root: String,
    /// SoundFont object store (change: add-score-daily-access-rewards): the
    /// `score_preview_render` job reads the configured preview font's bytes from
    /// it. `None` leaves that job dormant. Mirrors the server's
    /// `CYMBRA_SOUNDFONT_S3_*` keys.
    pub soundfont_storage: Option<cymbra_platform::config::SoundfontStorageConfig>,
    /// Retention window (days) for the heavy per-session play detail before the
    /// `play_detail_prune` job NULLs it (change: add-play-activity-profile, D7).
    /// Mirrors the server's `CYMBRA_PLAY_DETAIL_RETENTION_DAYS`. Default 90.
    pub play_detail_retention_days: usize,
    /// `flags_svc` connection for the shared feature-flag service (change: add-
    /// feature-usage-analytics). `None` runs the flag service in defaults-only mode,
    /// so the usage-purge job uses the code-default retention window. Mirrors the
    /// server's `CYMBRA_FLAGS_DATABASE_URL`.
    pub flags_database_url: Option<String>,
    /// `plans_svc` connection for the plans module (change: add-premium-
    /// subscription): the `plans_reconcile` / `plans_withdraw` sweeps and the
    /// erasure's web-subscription cancellation. `None` leaves those jobs inert.
    /// Mirrors the server's `CYMBRA_PLANS_DATABASE_URL`.
    pub plans_database_url: Option<String>,
    /// Firebase service-account key JSON for the FCM sender (change: add-push-
    /// notifications). `None` leaves the `push_dispatch` job inert (it logs and
    /// completes) so a deployment without a Firebase project simply sends nothing.
    pub fcm_service_account_json: Option<String>,
}

impl WorkerConfig {
    pub fn from_env() -> Result<Self, String> {
        let map: HashMap<String, String> = std::env::vars().collect();
        core::parse(&map)
    }
}

/// Pure parsing/validation over a key/value map.
pub mod core {
    use super::{Duration, HashMap, WorkerConfig};

    pub fn parse(m: &HashMap<String, String>) -> Result<WorkerConfig, String> {
        let concurrency_min = num(m, "CYMBRA_WORKER_CONCURRENCY_MIN", 1)?;
        let concurrency_max = num(m, "CYMBRA_WORKER_CONCURRENCY_MAX", 16)?;
        if concurrency_min > concurrency_max {
            return Err(format!(
                "CYMBRA_WORKER_CONCURRENCY_MIN ({concurrency_min}) > MAX ({concurrency_max})"
            ));
        }
        Ok(WorkerConfig {
            worker_database_url: req(m, "CYMBRA_WORKER_DATABASE_URL")?,
            user_database_url: req(m, "CYMBRA_USER_DATABASE_URL")?,
            auth_database_url: req(m, "CYMBRA_AUTH_DATABASE_URL")?,
            admin_database_url: req(m, "CYMBRA_ADMIN_DATABASE_URL")?,
            smtp_url: req(m, "CYMBRA_SMTP_URL")?,
            smtp_from: opt(m, "CYMBRA_SMTP_FROM", "no-reply@cymbra.dev"),
            http_addr: opt(m, "CYMBRA_WORKER_HTTP_ADDR", "0.0.0.0:8082"),
            concurrency_min,
            concurrency_max,
            scheduler_interval: dur(m, "CYMBRA_WORKER_SCHEDULER_INTERVAL", "30s")?,
            dlq_sweep_interval: dur(m, "CYMBRA_WORKER_DLQ_SWEEP_INTERVAL", "60s")?,
            orphan_reap_grace: dur(m, "CYMBRA_ORPHAN_REAP_GRACE", "24h")?,
            otlp_endpoint: m
                .get("CYMBRA_OTLP_ENDPOINT")
                .filter(|s| !s.is_empty())
                .cloned(),
            otlp_enabled: flag(m, "CYMBRA_OTLP_ENABLED", false),
            score_storage: score_storage(m)?,
            score_local_root: opt(m, "CYMBRA_SCORE_LOCAL_ROOT", "/srv/cymbra/scores"),
            soundfont_storage: soundfont_storage(m)?,
            play_detail_retention_days: num(m, "CYMBRA_PLAY_DETAIL_RETENTION_DAYS", 90)?,
            flags_database_url: m
                .get("CYMBRA_FLAGS_DATABASE_URL")
                .filter(|v| !v.is_empty())
                .cloned(),
            plans_database_url: m
                .get("CYMBRA_PLANS_DATABASE_URL")
                .filter(|v| !v.is_empty())
                .cloned(),
            fcm_service_account_json: m
                .get("CYMBRA_FCM_SERVICE_ACCOUNT_JSON")
                .filter(|v| !v.is_empty())
                .cloned(),
        })
    }

    /// Score object-store config, enabled when `CYMBRA_SCORE_S3_BUCKET` is set
    /// (then the rest is required — a partial config fails fast). Mirrors the
    /// server's parsing so both read the same keys.
    fn score_storage(
        m: &HashMap<String, String>,
    ) -> Result<Option<cymbra_platform::config::ScoreStorageConfig>, String> {
        if m.get("CYMBRA_SCORE_S3_BUCKET")
            .filter(|v| !v.is_empty())
            .is_none()
        {
            return Ok(None);
        }
        Ok(Some(cymbra_platform::config::ScoreStorageConfig {
            bucket: req(m, "CYMBRA_SCORE_S3_BUCKET")?,
            endpoint: req(m, "CYMBRA_SCORE_S3_ENDPOINT")?,
            region: req(m, "CYMBRA_SCORE_S3_REGION")?,
            access_key: req(m, "CYMBRA_SCORE_S3_ACCESS_KEY")?,
            secret_key: req(m, "CYMBRA_SCORE_S3_SECRET_KEY")?,
            allow_http: flag(m, "CYMBRA_SCORE_S3_ALLOW_HTTP", false),
        }))
    }

    /// SoundFont object-store config, enabled when `CYMBRA_SOUNDFONT_S3_BUCKET` is
    /// set (then the rest is required). Mirrors the server's parsing.
    fn soundfont_storage(
        m: &HashMap<String, String>,
    ) -> Result<Option<cymbra_platform::config::SoundfontStorageConfig>, String> {
        if m.get("CYMBRA_SOUNDFONT_S3_BUCKET")
            .filter(|v| !v.is_empty())
            .is_none()
        {
            return Ok(None);
        }
        Ok(Some(cymbra_platform::config::SoundfontStorageConfig {
            bucket: req(m, "CYMBRA_SOUNDFONT_S3_BUCKET")?,
            endpoint: req(m, "CYMBRA_SOUNDFONT_S3_ENDPOINT")?,
            region: req(m, "CYMBRA_SOUNDFONT_S3_REGION")?,
            access_key: req(m, "CYMBRA_SOUNDFONT_S3_ACCESS_KEY")?,
            secret_key: req(m, "CYMBRA_SOUNDFONT_S3_SECRET_KEY")?,
            allow_http: flag(m, "CYMBRA_SOUNDFONT_S3_ALLOW_HTTP", false),
            local_root: opt(m, "CYMBRA_SOUNDFONT_LOCAL_ROOT", "/srv/cymbra/soundfonts"),
        }))
    }

    fn flag(m: &HashMap<String, String>, k: &str, default: bool) -> bool {
        m.get(k)
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
            .unwrap_or(default)
    }

    fn req(m: &HashMap<String, String>, k: &str) -> Result<String, String> {
        m.get(k)
            .filter(|v| !v.is_empty())
            .cloned()
            .ok_or_else(|| format!("missing required key {k}"))
    }

    fn opt(m: &HashMap<String, String>, k: &str, default: &str) -> String {
        m.get(k)
            .filter(|v| !v.is_empty())
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    fn dur(m: &HashMap<String, String>, k: &str, default: &str) -> Result<Duration, String> {
        let raw = opt(m, k, default);
        humantime::parse_duration(&raw).map_err(|e| format!("{k} is not a duration ({raw:?}): {e}"))
    }

    fn num(m: &HashMap<String, String>, k: &str, default: usize) -> Result<usize, String> {
        match m.get(k).filter(|v| !v.is_empty()) {
            None => Ok(default),
            Some(v) => v
                .parse::<usize>()
                .map_err(|_| format!("{k} is not a valid number ({v:?})")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> HashMap<String, String> {
        [
            ("CYMBRA_WORKER_DATABASE_URL", "postgres://w"),
            ("CYMBRA_USER_DATABASE_URL", "postgres://u"),
            ("CYMBRA_AUTH_DATABASE_URL", "postgres://a"),
            ("CYMBRA_ADMIN_DATABASE_URL", "postgres://admin"),
            ("CYMBRA_SMTP_URL", "smtp://s"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    #[test]
    fn parses_with_defaults() {
        let c = core::parse(&base()).unwrap();
        assert_eq!(c.admin_database_url, "postgres://admin");
        assert_eq!(c.concurrency_min, 1);
        assert_eq!(c.concurrency_max, 16);
        assert_eq!(c.scheduler_interval, Duration::from_secs(30));
        assert_eq!(c.dlq_sweep_interval, Duration::from_secs(60));
        assert_eq!(c.http_addr, "0.0.0.0:8082");
        assert_eq!(c.smtp_from, "no-reply@cymbra.dev");
        assert!(!c.otlp_enabled);
        assert_eq!(c.otlp_endpoint, None);
        // Default play-detail retention (change: add-play-activity-profile).
        assert_eq!(c.play_detail_retention_days, 90);
        // No Firebase credentials ⇒ push dispatch is inert, not a startup failure
        // (change: add-push-notifications).
        assert_eq!(c.fcm_service_account_json, None);
    }

    #[test]
    fn fcm_service_account_is_optional_but_read_when_set() {
        let mut m = base();
        m.insert(
            "CYMBRA_FCM_SERVICE_ACCOUNT_JSON".into(),
            r#"{"project_id":"cymbra"}"#.into(),
        );
        assert_eq!(
            core::parse(&m).unwrap().fcm_service_account_json.as_deref(),
            Some(r#"{"project_id":"cymbra"}"#)
        );
        // An empty value is the same as unset.
        m.insert("CYMBRA_FCM_SERVICE_ACCOUNT_JSON".into(), String::new());
        assert_eq!(core::parse(&m).unwrap().fcm_service_account_json, None);
    }

    #[test]
    fn play_detail_retention_override() {
        let mut m = base();
        m.insert("CYMBRA_PLAY_DETAIL_RETENTION_DAYS".into(), "30".into());
        assert_eq!(core::parse(&m).unwrap().play_detail_retention_days, 30);
    }

    #[test]
    fn otlp_can_be_enabled() {
        let mut m = base();
        m.insert("CYMBRA_OTLP_ENABLED".into(), "true".into());
        m.insert(
            "CYMBRA_OTLP_ENDPOINT".into(),
            "http://collector:4317".into(),
        );
        let c = core::parse(&m).unwrap();
        assert!(c.otlp_enabled);
        assert_eq!(c.otlp_endpoint.as_deref(), Some("http://collector:4317"));
    }

    #[test]
    fn missing_required_key_fails() {
        let mut m = base();
        m.remove("CYMBRA_WORKER_DATABASE_URL");
        assert!(
            core::parse(&m)
                .unwrap_err()
                .contains("CYMBRA_WORKER_DATABASE_URL")
        );
    }

    #[test]
    fn admin_database_url_is_required() {
        let mut m = base();
        m.remove("CYMBRA_ADMIN_DATABASE_URL");
        assert!(
            core::parse(&m)
                .unwrap_err()
                .contains("CYMBRA_ADMIN_DATABASE_URL")
        );
    }

    #[test]
    fn concurrency_min_above_max_fails() {
        let mut m = base();
        m.insert("CYMBRA_WORKER_CONCURRENCY_MIN".into(), "20".into());
        m.insert("CYMBRA_WORKER_CONCURRENCY_MAX".into(), "10".into());
        assert!(core::parse(&m).unwrap_err().contains("MIN"));
    }

    #[test]
    fn bad_duration_and_number_fail() {
        let mut m = base();
        m.insert("CYMBRA_WORKER_SCHEDULER_INTERVAL".into(), "soon".into());
        assert!(core::parse(&m).is_err());
        let mut m = base();
        m.insert("CYMBRA_WORKER_CONCURRENCY_MAX".into(), "lots".into());
        assert!(core::parse(&m).is_err());
    }
}
