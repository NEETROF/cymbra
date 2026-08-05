//! Pure, host-testable core for usage ingestion (change: add-feature-usage-
//! analytics, tasks 4.1 + 4.2). No I/O — so the period-salted bucketing and the
//! per-event validation are fully unit-tested. The gRPC handler ([`crate::grpc`])
//! is a thin adapter that decodes the proto batch, runs each event through
//! [`validate`] + [`user_bucket`], and hands the valid rows to the repo.

use chrono::{DateTime, Datelike, TimeZone, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// The agreed shape rule for `action` / `variant`: `^[a-z][a-z0-9_]{0,63}$`
/// (lower `snake_case`, 1..=64 chars). Governance is the client-owned registry +
/// code review (design D7); the server validates only the *shape* — it accepts any
/// well-formed action, even one it has never seen, and rejects only malformed ones.
const MAX_TAXONOMY_LEN: usize = 64;
/// `subject_id` is an opaque bounded reference (e.g. a score UUID); we only bound
/// its length so a malformed client cannot bloat the raw store.
const MAX_SUBJECT_LEN: usize = 128;
const MAX_APP_VERSION_LEN: usize = 32;
const MAX_LOCALE_LEN: usize = 35;

/// Allowed `platform` values (the six shipped targets).
pub const PLATFORMS: [&str; 6] = ["ios", "android", "macos", "windows", "linux", "web"];
/// Allowed `device_class` values.
pub const DEVICE_CLASSES: [&str; 3] = ["phone", "tablet", "desktop"];

/// Clock-skew tolerance: an `occurred_at` more than this far in the future (vs the
/// server `received_at`) is implausible and clamped back to `received_at`.
const FUTURE_SKEW: chrono::Duration = chrono::Duration::hours(24);
/// Floor for a plausible `occurred_at`: anything before this is clamped to
/// `received_at` (a client with a badly-wrong clock, not a genuinely old buffered
/// event — the offline buffer never holds events for years).
fn floor() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).single().unwrap()
}

/// A raw, proto-agnostic event as received from the client (before validation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEvent {
    pub action: String,
    pub variant: Option<String>,
    pub subject_id: Option<String>,
    pub platform: String,
    pub device_class: String,
    pub app_version: String,
    pub locale: String,
    /// Client wall clock, unix epoch milliseconds.
    pub occurred_at_ms: i64,
}

/// A validated event ready to persist. `occurred_at` is clamped to a plausible
/// range; the `user_bucket` is computed separately by [`user_bucket`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidEvent {
    pub action: String,
    pub variant: Option<String>,
    pub subject_id: Option<String>,
    pub platform: String,
    pub device_class: String,
    pub app_version: String,
    pub locale: String,
    pub occurred_at: DateTime<Utc>,
}

/// Why an event was rejected (skipped without failing the batch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invalid {
    Action,
    Variant,
    Subject,
    Platform,
    DeviceClass,
    AppVersion,
    Locale,
    Timestamp,
}

/// True when `s` matches the taxonomy shape rule `^[a-z][a-z0-9_]{0,63}$`.
pub fn is_valid_taxonomy(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_TAXONOMY_LEN {
        return false;
    }
    if !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes
        .iter()
        .all(|&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Validate one event and clamp its timestamp. Accepts any well-formed `action`
/// (even previously unseen); rejects only malformed shape / out-of-range dimensions
/// (design D7). `received_at` is the server ingest instant used to clamp the client
/// clock (design: risks / trade-offs).
pub fn validate(raw: &RawEvent, received_at: DateTime<Utc>) -> Result<ValidEvent, Invalid> {
    if !is_valid_taxonomy(&raw.action) {
        return Err(Invalid::Action);
    }
    if let Some(v) = &raw.variant
        && !is_valid_taxonomy(v)
    {
        return Err(Invalid::Variant);
    }
    if let Some(s) = &raw.subject_id
        && (s.is_empty() || s.len() > MAX_SUBJECT_LEN)
    {
        return Err(Invalid::Subject);
    }
    if !PLATFORMS.contains(&raw.platform.as_str()) {
        return Err(Invalid::Platform);
    }
    if !DEVICE_CLASSES.contains(&raw.device_class.as_str()) {
        return Err(Invalid::DeviceClass);
    }
    if raw.app_version.is_empty() || raw.app_version.len() > MAX_APP_VERSION_LEN {
        return Err(Invalid::AppVersion);
    }
    if raw.locale.is_empty() || raw.locale.len() > MAX_LOCALE_LEN {
        return Err(Invalid::Locale);
    }
    let occurred_at = clamp_occurred_at(raw.occurred_at_ms, received_at)?;
    Ok(ValidEvent {
        action: raw.action.clone(),
        variant: raw.variant.clone(),
        subject_id: raw.subject_id.clone(),
        platform: raw.platform.clone(),
        device_class: raw.device_class.clone(),
        app_version: raw.app_version.clone(),
        locale: raw.locale.clone(),
        occurred_at,
    })
}

/// Clamp a client-supplied epoch-ms timestamp into a plausible range: a value in
/// the far future (beyond a day of skew) or far past (before [`floor`]) is clamped
/// to `received_at`; a value that cannot even be represented is rejected.
fn clamp_occurred_at(
    occurred_at_ms: i64,
    received_at: DateTime<Utc>,
) -> Result<DateTime<Utc>, Invalid> {
    let ts = Utc
        .timestamp_millis_opt(occurred_at_ms)
        .single()
        .ok_or(Invalid::Timestamp)?;
    if ts > received_at + FUTURE_SKEW || ts < floor() {
        return Ok(received_at);
    }
    Ok(ts)
}

/// The monthly period key `"YYYY-MM"` used as the salt-rotation window (design D2).
pub fn period_key(occurred_at: DateTime<Utc>) -> String {
    format!("{:04}-{:02}", occurred_at.year(), occurred_at.month())
}

/// Compute the period-salted pseudonymous `user_bucket` (design D2, Option A):
/// `salt(month) = HMAC(master_secret, "YYYY-MM")`, then
/// `user_bucket = HMAC(salt, user_id)` hex-encoded. Nothing is stored; the value is
/// reproducible within a month but unlinkable across months from analytics data
/// alone (the salt for another month is a different HMAC of the master secret).
pub fn user_bucket(master_secret: &[u8], user_id: &str, occurred_at: DateTime<Utc>) -> String {
    let period = period_key(occurred_at);
    let mut salt_mac =
        HmacSha256::new_from_slice(master_secret).expect("HMAC accepts any key length");
    salt_mac.update(period.as_bytes());
    let salt = salt_mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&salt).expect("HMAC accepts any key length");
    mac.update(user_id.as_bytes());
    hex(&mac.finalize().into_bytes())
}

/// Lower-case hex encoding (no external crate).
fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw() -> RawEvent {
        RawEvent {
            action: "play_start".into(),
            variant: None,
            subject_id: Some("11111111-1111-7111-8111-111111111111".into()),
            platform: "ios".into(),
            device_class: "phone".into(),
            app_version: "1.17.0".into(),
            locale: "fr".into(),
            occurred_at_ms: Utc
                .with_ymd_and_hms(2026, 6, 15, 12, 0, 0)
                .single()
                .unwrap()
                .timestamp_millis(),
        }
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 15, 12, 5, 0)
            .single()
            .unwrap()
    }

    // --- taxonomy shape --------------------------------------------------------

    #[test]
    fn taxonomy_shape_rules() {
        assert!(is_valid_taxonomy("play_start"));
        assert!(is_valid_taxonomy("a"));
        assert!(is_valid_taxonomy("fall_note"));
        assert!(is_valid_taxonomy("piano_type"));
        assert!(is_valid_taxonomy(&"a".repeat(64)));
        // malformed
        assert!(!is_valid_taxonomy(""));
        assert!(!is_valid_taxonomy("Play")); // uppercase
        assert!(!is_valid_taxonomy("1play")); // leading digit
        assert!(!is_valid_taxonomy("play-start")); // hyphen
        assert!(!is_valid_taxonomy("play start")); // space
        assert!(!is_valid_taxonomy(&"a".repeat(65))); // too long
        assert!(!is_valid_taxonomy("_x")); // leading underscore
    }

    // --- validation ------------------------------------------------------------

    #[test]
    fn well_formed_event_is_accepted_even_when_action_is_novel() {
        let mut r = raw();
        r.action = "some_new_future_action".into();
        assert!(validate(&r, now()).is_ok());
    }

    #[test]
    fn malformed_action_rejected() {
        let mut r = raw();
        r.action = "NOPE!".into();
        assert_eq!(validate(&r, now()), Err(Invalid::Action));
    }

    #[test]
    fn bad_variant_rejected_but_absent_variant_ok() {
        let mut r = raw();
        r.variant = Some("Bad Value".into());
        assert_eq!(validate(&r, now()), Err(Invalid::Variant));
        r.variant = Some("fall_note".into());
        assert!(validate(&r, now()).is_ok());
    }

    #[test]
    fn out_of_range_platform_and_device_rejected() {
        let mut r = raw();
        r.platform = "playstation".into();
        assert_eq!(validate(&r, now()), Err(Invalid::Platform));
        let mut r = raw();
        r.device_class = "watch".into();
        assert_eq!(validate(&r, now()), Err(Invalid::DeviceClass));
    }

    #[test]
    fn subject_id_bounds() {
        let mut r = raw();
        r.subject_id = Some(String::new());
        assert_eq!(validate(&r, now()), Err(Invalid::Subject));
        r.subject_id = Some("x".repeat(129));
        assert_eq!(validate(&r, now()), Err(Invalid::Subject));
        r.subject_id = None;
        assert!(validate(&r, now()).is_ok());
    }

    #[test]
    fn app_version_and_locale_required_and_bounded() {
        let mut r = raw();
        r.app_version = String::new();
        assert_eq!(validate(&r, now()), Err(Invalid::AppVersion));
        let mut r = raw();
        r.locale = "x".repeat(36);
        assert_eq!(validate(&r, now()), Err(Invalid::Locale));
    }

    #[test]
    fn future_timestamp_is_clamped_to_received_at() {
        let mut r = raw();
        let n = now();
        r.occurred_at_ms = (n + chrono::Duration::days(3)).timestamp_millis();
        assert_eq!(validate(&r, n).unwrap().occurred_at, n);
    }

    #[test]
    fn far_past_timestamp_is_clamped() {
        let mut r = raw();
        let n = now();
        r.occurred_at_ms = Utc
            .with_ymd_and_hms(1999, 1, 1, 0, 0, 0)
            .single()
            .unwrap()
            .timestamp_millis();
        assert_eq!(validate(&r, n).unwrap().occurred_at, n);
    }

    #[test]
    fn plausible_timestamp_is_kept() {
        let r = raw();
        let got = validate(&r, now()).unwrap().occurred_at;
        assert_eq!(
            got,
            Utc.with_ymd_and_hms(2026, 6, 15, 12, 0, 0)
                .single()
                .unwrap()
        );
    }

    // --- bucketing (design D2) -------------------------------------------------

    #[test]
    fn period_key_is_year_month() {
        assert_eq!(
            period_key(Utc.with_ymd_and_hms(2026, 6, 15, 0, 0, 0).single().unwrap()),
            "2026-06"
        );
        assert_eq!(
            period_key(Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).single().unwrap()),
            "2026-12"
        );
    }

    #[test]
    fn same_user_same_month_maps_to_one_bucket() {
        let secret = b"master-secret";
        let d1 = Utc.with_ymd_and_hms(2026, 6, 3, 9, 0, 0).single().unwrap();
        let d2 = Utc
            .with_ymd_and_hms(2026, 6, 27, 22, 0, 0)
            .single()
            .unwrap();
        assert_eq!(
            user_bucket(secret, "user-1", d1),
            user_bucket(secret, "user-1", d2)
        );
    }

    #[test]
    fn same_user_across_months_is_unlinkable() {
        let secret = b"master-secret";
        let june = Utc.with_ymd_and_hms(2026, 6, 15, 0, 0, 0).single().unwrap();
        let july = Utc.with_ymd_and_hms(2026, 7, 15, 0, 0, 0).single().unwrap();
        assert_ne!(
            user_bucket(secret, "user-1", june),
            user_bucket(secret, "user-1", july)
        );
    }

    #[test]
    fn different_users_differ_within_a_period() {
        let secret = b"master-secret";
        let d = Utc.with_ymd_and_hms(2026, 6, 15, 0, 0, 0).single().unwrap();
        assert_ne!(
            user_bucket(secret, "user-1", d),
            user_bucket(secret, "user-2", d)
        );
    }

    #[test]
    fn bucket_is_hex_of_expected_length() {
        let secret = b"master-secret";
        let d = Utc.with_ymd_and_hms(2026, 6, 15, 0, 0, 0).single().unwrap();
        let b = user_bucket(secret, "user-1", d);
        assert_eq!(b.len(), 64); // 32-byte HMAC-SHA256, hex
        assert!(
            b.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }
}
