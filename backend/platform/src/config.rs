//! Typed configuration with fail-fast validation (task 2.1).
//!
//! [`config_core`] is the pure, host-testable parser over a key/value map;
//! [`Config::from_env`] just collects the process environment into that map. A
//! missing or malformed required value fails fast with a clear [`AppError::Config`].

use crate::error::{AppError, Result};
use std::collections::HashMap;
use std::time::Duration;

/// Fully-resolved backend configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub grpc_addr: String,
    pub http_addr: String,
    pub auth_database_url: String,
    pub user_database_url: String,
    pub redis_url: String,
    /// App audiences a sign-in may target (one login per app).
    pub allowed_audiences: Vec<String>,
    pub token: TokenConfig,
    pub password_min_length: usize,
    pub signin_max_attempts: u32,
    pub signin_lockout: Duration,
    pub smtp_url: String,
    pub smtp_from: String,
    /// Max verification/reset emails per window.
    pub email_max: u32,
    pub email_window: Duration,
    pub verify_ttl: Duration,
    pub reset_ttl: Duration,
    /// Trusted OIDC providers (Google/Apple) — empty entries omitted.
    pub oidc_providers: Vec<OidcProvider>,
    pub otlp_endpoint: Option<String>,
    pub otlp_enabled: bool,
}

/// A trusted external OIDC provider.
#[derive(Debug, Clone)]
pub struct OidcProvider {
    pub provider: String,
    pub issuer: String,
    pub audience: String,
    pub jwks_uri: String,
}

/// Internal-token signing parameters (asymmetric; public key served via JWKS).
#[derive(Debug, Clone)]
pub struct TokenConfig {
    /// Ed25519 private key (PKCS#8 PEM) used to sign access tokens.
    pub signing_key_pem: String,
    /// Ed25519 public key (SPKI PEM) advertised at the JWKS endpoint.
    pub public_key_pem: String,
    /// Key id advertised in the token header and the JWKS.
    pub kid: String,
    pub access_ttl: Duration,
    pub refresh_ttl: Duration,
}

impl Config {
    /// Collect the process environment and parse it.
    pub fn from_env() -> Result<Self> {
        let map: HashMap<String, String> = std::env::vars().collect();
        config_core::parse(&map)
    }
}

/// Pure, host-testable parsing/validation over a key/value map.
pub mod config_core {
    use super::*;

    pub fn parse(m: &HashMap<String, String>) -> Result<Config> {
        Ok(Config {
            grpc_addr: opt(m, "CYMBRA_GRPC_ADDR", "0.0.0.0:50051"),
            http_addr: opt(m, "CYMBRA_HTTP_ADDR", "0.0.0.0:8081"),
            auth_database_url: req(m, "CYMBRA_AUTH_DATABASE_URL")?,
            user_database_url: req(m, "CYMBRA_USER_DATABASE_URL")?,
            redis_url: req(m, "CYMBRA_REDIS_URL")?,
            allowed_audiences: list(m, "CYMBRA_ALLOWED_AUDIENCES")?,
            token: TokenConfig {
                signing_key_pem: req(m, "CYMBRA_TOKEN_SIGNING_KEY_PEM")?,
                public_key_pem: req(m, "CYMBRA_TOKEN_PUBLIC_KEY_PEM")?,
                kid: opt(m, "CYMBRA_TOKEN_SIGNING_KID", "k1"),
                access_ttl: dur(m, "CYMBRA_ACCESS_TOKEN_TTL", "15m")?,
                refresh_ttl: dur(m, "CYMBRA_REFRESH_TOKEN_TTL", "30d")?,
            },
            password_min_length: num(m, "CYMBRA_PASSWORD_MIN_LENGTH", 12)?,
            signin_max_attempts: num(m, "CYMBRA_SIGNIN_MAX_ATTEMPTS", 5)?,
            signin_lockout: dur(m, "CYMBRA_SIGNIN_LOCKOUT", "15m")?,
            smtp_url: req(m, "CYMBRA_SMTP_URL")?,
            smtp_from: opt(m, "CYMBRA_SMTP_FROM", "no-reply@cymbra.dev"),
            email_max: email_rate(m)?.0,
            email_window: email_rate(m)?.1,
            verify_ttl: dur(m, "CYMBRA_VERIFY_TOKEN_TTL", "24h")?,
            reset_ttl: dur(m, "CYMBRA_RESET_TOKEN_TTL", "1h")?,
            oidc_providers: oidc_providers(m),
            otlp_endpoint: m.get("CYMBRA_OTLP_ENDPOINT").cloned(),
            otlp_enabled: flag(m, "CYMBRA_OTLP_ENABLED", false),
        })
    }

    /// Parse `CYMBRA_EMAIL_SEND_RATE` of the form `N/<duration>` (e.g. `3/1h`).
    fn email_rate(m: &HashMap<String, String>) -> Result<(u32, Duration)> {
        let raw = opt(m, "CYMBRA_EMAIL_SEND_RATE", "3/1h");
        let (n, win) = raw
            .split_once('/')
            .ok_or_else(|| AppError::Config(format!("CYMBRA_EMAIL_SEND_RATE invalid: {raw:?}")))?;
        let max = n.trim().parse::<u32>().map_err(|_| {
            AppError::Config(format!("CYMBRA_EMAIL_SEND_RATE count invalid: {n:?}"))
        })?;
        let window = humantime::parse_duration(win.trim())
            .map_err(|e| AppError::Config(format!("CYMBRA_EMAIL_SEND_RATE window invalid: {e}")))?;
        Ok((max, window))
    }

    /// Build the trusted OIDC provider list from Google/Apple env (audience set).
    fn oidc_providers(m: &HashMap<String, String>) -> Vec<OidcProvider> {
        let mut v = Vec::new();
        if let Some(aud) = m.get("CYMBRA_GOOGLE_AUDIENCE").filter(|s| !s.is_empty()) {
            v.push(OidcProvider {
                provider: "google".into(),
                issuer: opt(m, "CYMBRA_GOOGLE_ISSUER", "https://accounts.google.com"),
                audience: aud.clone(),
                jwks_uri: "https://www.googleapis.com/oauth2/v3/certs".into(),
            });
        }
        if let Some(aud) = m.get("CYMBRA_APPLE_AUDIENCE").filter(|s| !s.is_empty()) {
            v.push(OidcProvider {
                provider: "apple".into(),
                issuer: opt(m, "CYMBRA_APPLE_ISSUER", "https://appleid.apple.com"),
                audience: aud.clone(),
                jwks_uri: "https://appleid.apple.com/auth/keys".into(),
            });
        }
        v
    }

    fn req(m: &HashMap<String, String>, k: &str) -> Result<String> {
        m.get(k)
            .filter(|v| !v.is_empty())
            .cloned()
            .ok_or_else(|| AppError::Config(format!("missing required key {k}")))
    }

    fn opt(m: &HashMap<String, String>, k: &str, default: &str) -> String {
        m.get(k)
            .filter(|v| !v.is_empty())
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    fn list(m: &HashMap<String, String>, k: &str) -> Result<Vec<String>> {
        let raw = req(m, k)?;
        let items: Vec<String> = raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if items.is_empty() {
            return Err(AppError::Config(format!(
                "{k} must list at least one value"
            )));
        }
        Ok(items)
    }

    fn dur(m: &HashMap<String, String>, k: &str, default: &str) -> Result<Duration> {
        let raw = opt(m, k, default);
        humantime::parse_duration(&raw)
            .map_err(|e| AppError::Config(format!("{k} is not a duration ({raw:?}): {e}")))
    }

    fn num<T: std::str::FromStr>(m: &HashMap<String, String>, k: &str, default: T) -> Result<T> {
        match m.get(k).filter(|v| !v.is_empty()) {
            None => Ok(default),
            Some(v) => v
                .parse::<T>()
                .map_err(|_| AppError::Config(format!("{k} is not a valid number ({v:?})"))),
        }
    }

    fn flag(m: &HashMap<String, String>, k: &str, default: bool) -> bool {
        m.get(k)
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
            .unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> HashMap<String, String> {
        [
            ("CYMBRA_AUTH_DATABASE_URL", "postgres://a"),
            ("CYMBRA_USER_DATABASE_URL", "postgres://u"),
            ("CYMBRA_REDIS_URL", "redis://r"),
            ("CYMBRA_ALLOWED_AUDIENCES", "music, live"),
            ("CYMBRA_TOKEN_SIGNING_KEY_PEM", "PEM"),
            ("CYMBRA_TOKEN_PUBLIC_KEY_PEM", "PUBPEM"),
            ("CYMBRA_SMTP_URL", "smtp://s"),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    #[test]
    fn parses_with_defaults() {
        let c = config_core::parse(&base()).unwrap();
        assert_eq!(c.allowed_audiences, vec!["music", "live"]);
        assert_eq!(c.token.access_ttl, Duration::from_secs(15 * 60));
        assert_eq!(c.token.refresh_ttl, Duration::from_secs(30 * 24 * 3600));
        assert_eq!(c.password_min_length, 12);
        assert!(!c.otlp_enabled);
    }

    #[test]
    fn missing_required_key_fails() {
        let mut m = base();
        m.remove("CYMBRA_REDIS_URL");
        let err = config_core::parse(&m).unwrap_err();
        assert!(matches!(err, AppError::Config(msg) if msg.contains("CYMBRA_REDIS_URL")));
    }

    #[test]
    fn bad_duration_fails() {
        let mut m = base();
        m.insert("CYMBRA_ACCESS_TOKEN_TTL".into(), "soon".into());
        assert!(matches!(config_core::parse(&m), Err(AppError::Config(_))));
    }

    #[test]
    fn empty_audience_list_fails() {
        let mut m = base();
        m.insert("CYMBRA_ALLOWED_AUDIENCES".into(), " , ".into());
        assert!(matches!(config_core::parse(&m), Err(AppError::Config(_))));
    }
}
