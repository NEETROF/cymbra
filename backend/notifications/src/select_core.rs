//! Recipient selection — the host-testable core (change: add-push-notifications,
//! task 2.3, design D5).
//!
//! **All** of the "who receives this send" logic lives here: the global
//! kill-switch, the per-category enable flag, the user's opt-out, the local-hour
//! match and token presence. It is a pure function of its inputs — no DB, no FCM —
//! so every gate is unit-tested. Getting this wrong means messaging someone who
//! opted out, so it is deliberately kept out of the coverage-excluded glue.

use chrono::{DateTime, Timelike, Utc};
use chrono_tz::Tz;
use cymbra_platform::{AppError, Result};

use crate::repo::Candidate;

/// The timezone a send falls back to when a user has never reported one
/// (design D3) — a defined default rather than an arbitrary hour.
pub const DEFAULT_TIMEZONE: &str = "Europe/Paris";

/// Everything selection needs beyond the candidate rows themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionFlags {
    /// The category this send is for (opaque, owned by the declaring feature).
    pub category: String,
    /// Global kill-switch: `false` halts **every** category (design D4).
    pub kill_switch_on: bool,
    /// Per-category enable flag: `false` halts this category regardless of any
    /// user preference.
    pub category_enabled: bool,
    /// The local hour (0–23) this category is scheduled for. `None` means the
    /// send is event-triggered, so no hour gate applies.
    pub schedule_hour: Option<u32>,
    /// What an absent user preference means for this category — the product
    /// default declared by the feature that owns it.
    pub default_pref: bool,
    /// Fallback timezone for users with none stored.
    pub default_timezone: String,
}

impl SelectionFlags {
    /// A scheduled category with sane platform defaults; callers override the
    /// fields their flags resolve.
    pub fn new(category: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            kill_switch_on: false,
            category_enabled: false,
            schedule_hour: None,
            default_pref: true,
            default_timezone: DEFAULT_TIMEZONE.to_string(),
        }
    }
}

/// Validate an IANA timezone name, returning it normalized.
///
/// Used both on the write path (`SetTimezone`) and as the guard around a stored
/// value, so a zone that stops existing can never misfire a send.
pub fn validate_timezone(name: &str) -> Result<String> {
    name.parse::<Tz>()
        .map(|tz| tz.name().to_string())
        .map_err(|_| AppError::InvalidArgument(format!("unknown timezone {name:?}")))
}

/// The user's local hour at `now`, falling back to `default_timezone` when the
/// stored zone is missing or no longer parses, and to [`DEFAULT_TIMEZONE`] when
/// even the configured default is bad (so selection can never panic on config).
fn local_hour(stored: Option<&str>, default_timezone: &str, now: DateTime<Utc>) -> u32 {
    let tz = stored
        .and_then(|name| name.parse::<Tz>().ok())
        .or_else(|| default_timezone.parse::<Tz>().ok())
        .unwrap_or_else(|| DEFAULT_TIMEZONE.parse::<Tz>().expect("built-in zone"));
    now.with_timezone(&tz).hour()
}

/// The tokens that should actually receive this category send.
///
/// Order is the candidate order; a token appearing on several candidate rows is
/// returned once (a user can register the same install twice across sessions).
pub fn select_recipients(
    candidates: &[Candidate],
    flags: &SelectionFlags,
    now: DateTime<Utc>,
) -> Vec<String> {
    // Hard global gates first — neither depends on a candidate, so a disabled
    // category never even looks at the rows.
    if !flags.kill_switch_on || !flags.category_enabled {
        return Vec::new();
    }

    let mut out: Vec<String> = Vec::new();
    for c in candidates {
        let token = c.token.trim();
        if token.is_empty() {
            continue;
        }
        // Absent preference = the category's product default (design D4).
        if !c.pref.unwrap_or(flags.default_pref) {
            continue;
        }
        if let Some(hour) = flags.schedule_hour
            && local_hour(c.timezone.as_deref(), &flags.default_timezone, now) != hour
        {
            continue;
        }
        if !out.iter().any(|t| t == token) {
            out.push(token.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn candidate(user: &str, token: &str, tz: Option<&str>, pref: Option<bool>) -> Candidate {
        Candidate {
            user_id: user.into(),
            token: token.into(),
            platform: "ios".into(),
            timezone: tz.map(str::to_string),
            pref,
        }
    }

    /// A live, scheduled 20:00 category whose product default is opt-in.
    fn flags_at_20h() -> SelectionFlags {
        SelectionFlags {
            kill_switch_on: true,
            category_enabled: true,
            schedule_hour: Some(20),
            ..SelectionFlags::new("practice_streak")
        }
    }

    #[test]
    fn kill_switch_off_halts_every_send() {
        let flags = SelectionFlags {
            kill_switch_on: false,
            ..flags_at_20h()
        };
        let c = [candidate("u1", "t1", Some("Europe/Paris"), Some(true))];
        assert!(select_recipients(&c, &flags, utc("2026-08-06T18:00:00Z")).is_empty());
    }

    #[test]
    fn category_disabled_halts_that_category() {
        let flags = SelectionFlags {
            category_enabled: false,
            ..flags_at_20h()
        };
        // Even an explicitly opted-in user gets nothing.
        let c = [candidate("u1", "t1", Some("Europe/Paris"), Some(true))];
        assert!(select_recipients(&c, &flags, utc("2026-08-06T18:00:00Z")).is_empty());
    }

    #[test]
    fn opted_out_user_is_skipped_and_absent_pref_uses_the_default() {
        let now = utc("2026-08-06T18:00:00Z"); // 20:00 in Paris (CEST)
        let c = [
            candidate("out", "t-out", Some("Europe/Paris"), Some(false)),
            candidate("in", "t-in", Some("Europe/Paris"), Some(true)),
            candidate("unset", "t-unset", Some("Europe/Paris"), None),
        ];
        // default_pref = true → the never-chose user is included.
        assert_eq!(
            select_recipients(&c, &flags_at_20h(), now),
            vec!["t-in".to_string(), "t-unset".to_string()]
        );

        // default_pref = false → only the explicit opt-in remains.
        let opt_in_only = SelectionFlags {
            default_pref: false,
            ..flags_at_20h()
        };
        assert_eq!(
            select_recipients(&c, &opt_in_only, now),
            vec!["t-in".to_string()]
        );
    }

    #[test]
    fn local_hour_gate_targets_each_timezone_at_its_own_20h() {
        // 18:00Z = 20:00 Paris (CEST), 14:00 New York (EDT), 03:00 (+1d) Tokyo.
        let now = utc("2026-08-06T18:00:00Z");
        let c = [
            candidate("paris", "t-paris", Some("Europe/Paris"), None),
            candidate("ny", "t-ny", Some("America/New_York"), None),
            candidate("tokyo", "t-tokyo", Some("Asia/Tokyo"), None),
        ];
        assert_eq!(
            select_recipients(&c, &flags_at_20h(), now),
            vec!["t-paris".to_string()]
        );

        // Six hours later it is 20:00 in New York.
        assert_eq!(
            select_recipients(&c, &flags_at_20h(), utc("2026-08-07T00:00:00Z")),
            vec!["t-ny".to_string()]
        );
    }

    #[test]
    fn unknown_timezone_falls_back_to_the_configured_default() {
        let now = utc("2026-08-06T18:00:00Z"); // 20:00 in Paris
        // No stored zone, and a stored zone that no longer parses: both fall back.
        let c = [
            candidate("none", "t-none", None, None),
            candidate("bad", "t-bad", Some("Mars/Olympus"), None),
        ];
        assert_eq!(
            select_recipients(&c, &flags_at_20h(), now),
            vec!["t-none".to_string(), "t-bad".to_string()]
        );

        // A different default moves them: 18:00Z is 14:00 in New York, not 20:00.
        let ny_default = SelectionFlags {
            default_timezone: "America/New_York".into(),
            ..flags_at_20h()
        };
        assert!(select_recipients(&c, &ny_default, now).is_empty());

        // A nonsense default degrades to the built-in one rather than panicking.
        let bad_default = SelectionFlags {
            default_timezone: "Nowhere/At/All".into(),
            ..flags_at_20h()
        };
        assert_eq!(select_recipients(&c, &bad_default, now).len(), 2);
    }

    #[test]
    fn event_triggered_sends_have_no_hour_gate() {
        let flags = SelectionFlags {
            schedule_hour: None,
            ..flags_at_20h()
        };
        let c = [
            candidate("paris", "t-paris", Some("Europe/Paris"), None),
            candidate("tokyo", "t-tokyo", Some("Asia/Tokyo"), None),
        ];
        assert_eq!(
            select_recipients(&c, &flags, utc("2026-08-06T09:13:00Z")).len(),
            2
        );
    }

    #[test]
    fn blank_tokens_are_skipped_and_duplicates_collapse() {
        let now = utc("2026-08-06T18:00:00Z");
        let c = [
            candidate("u1", "   ", Some("Europe/Paris"), None),
            candidate("u1", " t-dup ", Some("Europe/Paris"), None),
            candidate("u2", "t-dup", Some("Europe/Paris"), None),
        ];
        assert_eq!(
            select_recipients(&c, &flags_at_20h(), now),
            vec!["t-dup".to_string()]
        );
    }

    #[test]
    fn empty_candidate_set_selects_nothing() {
        assert!(select_recipients(&[], &flags_at_20h(), utc("2026-08-06T18:00:00Z")).is_empty());
    }

    #[test]
    fn timezone_validation_accepts_iana_names_only() {
        assert_eq!(validate_timezone("Europe/Paris").unwrap(), "Europe/Paris");
        assert_eq!(validate_timezone("UTC").unwrap(), "UTC");
        assert!(matches!(
            validate_timezone("Mars/Olympus").unwrap_err(),
            AppError::InvalidArgument(_)
        ));
        assert!(validate_timezone("").is_err());
        // A UTC offset is not an IANA zone name — rejected so the stored value is
        // always resolvable to real local time (DST included).
        assert!(validate_timezone("+02:00").is_err());
    }

    #[test]
    fn selection_flags_default_to_the_safe_disabled_state() {
        let f = SelectionFlags::new("x");
        assert_eq!(f.category, "x");
        assert!(!f.kill_switch_on);
        assert!(!f.category_enabled);
        assert_eq!(f.schedule_hour, None);
        assert_eq!(f.default_timezone, DEFAULT_TIMEZONE);
        // With the safe defaults nothing is ever selected.
        let c = [candidate("u", "t", None, Some(true))];
        assert!(select_recipients(&c, &f, utc("2026-08-06T18:00:00Z")).is_empty());
    }
}
