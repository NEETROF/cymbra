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
        })
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
            ("CYMBRA_SMTP_URL", "smtp://s"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    #[test]
    fn parses_with_defaults() {
        let c = core::parse(&base()).unwrap();
        assert_eq!(c.concurrency_min, 1);
        assert_eq!(c.concurrency_max, 16);
        assert_eq!(c.scheduler_interval, Duration::from_secs(30));
        assert_eq!(c.dlq_sweep_interval, Duration::from_secs(60));
        assert_eq!(c.http_addr, "0.0.0.0:8082");
        assert_eq!(c.smtp_from, "no-reply@cymbra.dev");
        assert!(!c.otlp_enabled);
        assert_eq!(c.otlp_endpoint, None);
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
