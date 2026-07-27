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
    /// S3-compatible object store for user-uploaded scores. `None` disables the
    /// score-upload feature (backend ships inert until configured); a *partial*
    /// S3 config is a hard error (fail-fast), not a silent disable.
    pub score_storage: Option<ScoreStorageConfig>,
    /// Postgres URL for the `music` schema (role `music_svc`). Required to wire the
    /// score-upload service; `None` leaves it unwired (the feature stays inert).
    pub music_database_url: Option<String>,
    /// Local warm-cache root the score reads serve from (crawler corpus + pulled
    /// uploads); the S3 fallback populates it on a miss.
    pub score_local_root: String,
    /// Max contributed scores a user may create within `upload_quota_window`.
    pub upload_quota_max: u32,
    /// Rolling window (days) for `upload_quota_max`.
    pub upload_quota_window_days: u32,
    /// Hard per-upload size cap (bytes) enforced server-side before storage.
    pub upload_max_bytes: usize,
    /// Allowed browser origins for the gRPC-web back office (change: add-moderation-
    /// back-office). The `CorsLayer` permits only these; empty (the default) allows
    /// no cross-origin browser access, so the native gRPC app is unaffected and the
    /// browser surface stays closed until an operator opts in an origin. Each entry
    /// is a full origin, e.g. `https://bo.cymbra.app`. Also used as the credentialed
    /// allow-list for the web-auth cookie surface (change: add-web-auth-cookies).
    pub back_office_origins: Vec<String>,
    /// `Domain` attribute for the web-auth refresh cookie. `None` (the default)
    /// scopes the cookie to the exact API host; set it to the shared registrable
    /// parent (e.g. `cymbra.app`) so `api.` and `bo.` share the first-party cookie.
    pub web_auth_cookie_domain: Option<String>,
    /// `Secure` attribute for the web-auth refresh cookie. Defaults to `true`
    /// (fail-closed for prod); set `false` only for plain-HTTP `localhost` dev.
    pub web_auth_cookie_secure: bool,
}

/// S3-compatible object-store connection for user scores. Maps to
/// `cymbra_storage::S3Params` in the composition root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreStorageConfig {
    pub bucket: String,
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    /// Allow plain HTTP (local MinIO in dev); keep false in prod.
    pub allow_http: bool,
}

/// A trusted external OIDC provider.
#[derive(Debug, Clone)]
pub struct OidcProvider {
    pub provider: String,
    pub issuer: String,
    /// Accepted `aud` values — a token matching **any** is trusted. Google needs
    /// more than one when several client types sign in (web/Android/iOS +
    /// desktop-loopback each mint a token with their own client id as `aud`).
    pub audiences: Vec<String>,
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
    use super::{
        AppError, Config, Duration, HashMap, OidcProvider, Result, ScoreStorageConfig, TokenConfig,
    };

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
            score_storage: score_storage(m)?,
            music_database_url: m
                .get("CYMBRA_MUSIC_DATABASE_URL")
                .filter(|v| !v.is_empty())
                .cloned(),
            score_local_root: opt(m, "CYMBRA_SCORE_LOCAL_ROOT", "/srv/cymbra/scores"),
            upload_quota_max: num(m, "CYMBRA_SCORE_UPLOAD_QUOTA_MAX", 5)?,
            upload_quota_window_days: num(m, "CYMBRA_SCORE_UPLOAD_QUOTA_WINDOW_DAYS", 7)?,
            upload_max_bytes: num(m, "CYMBRA_SCORE_UPLOAD_MAX_BYTES", 8 * 1024 * 1024)?,
            // Optional; absent = no cross-origin browser access (gRPC-web stays closed).
            back_office_origins: csv(m, "CYMBRA_BACK_OFFICE_ORIGINS"),
            web_auth_cookie_domain: m
                .get("CYMBRA_WEB_AUTH_COOKIE_DOMAIN")
                .filter(|v| !v.is_empty())
                .cloned(),
            // Fail-closed: default Secure=true; a dev override sets it false for http localhost.
            web_auth_cookie_secure: flag(m, "CYMBRA_WEB_AUTH_COOKIE_SECURE", true),
        })
    }

    /// Build the score object-store config. Enabled when `CYMBRA_SCORE_S3_BUCKET`
    /// is set; then every other S3 key is required (a partial config fails fast).
    /// Absent bucket ⇒ `None` (the score-upload feature stays inert).
    fn score_storage(m: &HashMap<String, String>) -> Result<Option<ScoreStorageConfig>> {
        if m.get("CYMBRA_SCORE_S3_BUCKET")
            .filter(|v| !v.is_empty())
            .is_none()
        {
            return Ok(None);
        }
        Ok(Some(ScoreStorageConfig {
            bucket: req(m, "CYMBRA_SCORE_S3_BUCKET")?,
            endpoint: req(m, "CYMBRA_SCORE_S3_ENDPOINT")?,
            region: req(m, "CYMBRA_SCORE_S3_REGION")?,
            access_key: req(m, "CYMBRA_SCORE_S3_ACCESS_KEY")?,
            secret_key: req(m, "CYMBRA_SCORE_S3_SECRET_KEY")?,
            allow_http: flag(m, "CYMBRA_SCORE_S3_ALLOW_HTTP", false),
        }))
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

    /// Build the trusted OIDC provider list from Google/Apple env. Each audience
    /// var is a comma-separated set (`CYMBRA_GOOGLE_AUDIENCE=web-id,desktop-id`);
    /// a provider with no non-empty audience is omitted.
    fn oidc_providers(m: &HashMap<String, String>) -> Vec<OidcProvider> {
        let mut v = Vec::new();
        let google = csv(m, "CYMBRA_GOOGLE_AUDIENCE");
        if !google.is_empty() {
            v.push(OidcProvider {
                provider: "google".into(),
                issuer: opt(m, "CYMBRA_GOOGLE_ISSUER", "https://accounts.google.com"),
                audiences: google,
                jwks_uri: "https://www.googleapis.com/oauth2/v3/certs".into(),
            });
        }
        let apple = csv(m, "CYMBRA_APPLE_AUDIENCE");
        if !apple.is_empty() {
            v.push(OidcProvider {
                provider: "apple".into(),
                issuer: opt(m, "CYMBRA_APPLE_ISSUER", "https://appleid.apple.com"),
                audiences: apple,
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

    /// Optional comma-separated list (trimmed, empties dropped); `[]` when unset.
    /// Unlike `list`, an absent/empty key is not an error.
    fn csv(m: &HashMap<String, String>, k: &str) -> Vec<String> {
        m.get(k)
            .map(|raw| {
                raw.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Required comma-separated list: `csv` plus a non-empty guard (the key must
    /// be present and yield at least one value).
    fn list(m: &HashMap<String, String>, k: &str) -> Result<Vec<String>> {
        req(m, k)?; // presence check (distinct "missing key" error)
        let items = csv(m, k);
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
        // Web-auth cookie: no Domain by default, Secure fail-closed to true.
        assert!(c.web_auth_cookie_domain.is_none());
        assert!(c.web_auth_cookie_secure);
    }

    #[test]
    fn web_auth_cookie_overrides_parse() {
        let mut m = base();
        m.insert("CYMBRA_WEB_AUTH_COOKIE_DOMAIN".into(), "cymbra.app".into());
        m.insert("CYMBRA_WEB_AUTH_COOKIE_SECURE".into(), "false".into());
        let c = config_core::parse(&m).unwrap();
        assert_eq!(c.web_auth_cookie_domain.as_deref(), Some("cymbra.app"));
        assert!(!c.web_auth_cookie_secure);
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

    #[test]
    fn no_oidc_audience_yields_no_provider() {
        let c = config_core::parse(&base()).unwrap();
        assert!(c.oidc_providers.is_empty());
    }

    #[test]
    fn google_audience_accepts_a_comma_separated_set() {
        let mut m = base();
        // web client + desktop-loopback client both trusted for Google.
        m.insert(
            "CYMBRA_GOOGLE_AUDIENCE".into(),
            "web-client.example, desktop-client.example".into(),
        );
        let c = config_core::parse(&m).unwrap();
        let google = c
            .oidc_providers
            .iter()
            .find(|p| p.provider == "google")
            .expect("google provider present");
        assert_eq!(
            google.audiences,
            vec!["web-client.example", "desktop-client.example"],
        );
    }

    #[test]
    fn single_google_audience_still_works() {
        let mut m = base();
        m.insert("CYMBRA_GOOGLE_AUDIENCE".into(), "only-web.example".into());
        let c = config_core::parse(&m).unwrap();
        let google = c
            .oidc_providers
            .iter()
            .find(|p| p.provider == "google")
            .unwrap();
        assert_eq!(google.audiences, vec!["only-web.example"]);
    }

    #[test]
    fn score_storage_absent_disables_uploads_with_sane_defaults() {
        let c = config_core::parse(&base()).unwrap();
        assert!(c.score_storage.is_none());
        assert_eq!(c.score_local_root, "/srv/cymbra/scores");
        assert_eq!(c.upload_quota_max, 5);
        assert_eq!(c.upload_quota_window_days, 7);
        assert_eq!(c.upload_max_bytes, 8 * 1024 * 1024);
    }

    #[test]
    fn score_storage_present_parses_full_config() {
        let mut m = base();
        for (k, v) in [
            ("CYMBRA_SCORE_S3_BUCKET", "cymbra-scores"),
            ("CYMBRA_SCORE_S3_ENDPOINT", "http://minio:9000"),
            ("CYMBRA_SCORE_S3_REGION", "us-east-1"),
            ("CYMBRA_SCORE_S3_ACCESS_KEY", "ak"),
            ("CYMBRA_SCORE_S3_SECRET_KEY", "sk"),
            ("CYMBRA_SCORE_S3_ALLOW_HTTP", "true"),
            ("CYMBRA_SCORE_UPLOAD_QUOTA_WINDOW_DAYS", "14"),
        ] {
            m.insert(k.into(), v.into());
        }
        let c = config_core::parse(&m).unwrap();
        let s = c.score_storage.expect("storage configured");
        assert_eq!(s.bucket, "cymbra-scores");
        assert_eq!(s.endpoint, "http://minio:9000");
        assert!(s.allow_http);
        assert_eq!(c.upload_quota_window_days, 14);
    }

    #[test]
    fn partial_score_storage_fails_fast() {
        let mut m = base();
        // Bucket set but credentials missing → hard error, not a silent disable.
        m.insert("CYMBRA_SCORE_S3_BUCKET".into(), "cymbra-scores".into());
        let err = config_core::parse(&m).unwrap_err();
        assert!(matches!(err, AppError::Config(msg) if msg.contains("CYMBRA_SCORE_S3_ENDPOINT")));
    }
}
