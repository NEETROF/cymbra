//! Handle policy + normalization (task 1.2) — pure and host-tested.
//!
//! A handle is 1–15 Unicode letters/digits. Uniqueness is **case-insensitive**
//! and NFC-insensitive: the display form is stored as entered, while comparisons
//! use a normalized key (Unicode NFC + case-fold) so handles that differ only by
//! letter case or by NFC-equivalent code points collide. The normalization lives
//! here, in one place, so a future confusables guard can layer on top (design D5).

use cymbra_platform::{AppError, Result};
use unicode_normalization::UnicodeNormalization;

/// Maximum handle length, in Unicode scalar values.
pub const MAX_LEN: usize = 15;

/// Validate the handle policy: 1–15 characters, Unicode letters/numbers only
/// (no spaces, punctuation, or symbols). Runs identically client- and server-side.
pub fn validate(handle: &str) -> Result<()> {
    let len = handle.chars().count();
    if len == 0 {
        return Err(AppError::InvalidArgument("handle must not be empty".into()));
    }
    if len > MAX_LEN {
        return Err(AppError::InvalidArgument(format!(
            "handle must be at most {MAX_LEN} characters"
        )));
    }
    if !handle.chars().all(char::is_alphanumeric) {
        return Err(AppError::InvalidArgument(
            "handle may contain only letters and numbers".into(),
        ));
    }
    Ok(())
}

/// Normalized uniqueness key for a handle: Unicode **NFC** then case-fold
/// (lowercase). Two handles share a key iff they collide under the uniqueness
/// rule. Callers SHOULD [`validate`] first; this only folds.
pub fn normalize(handle: &str) -> String {
    handle.nfc().collect::<String>().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_letters_and_numbers_within_length() {
        assert!(validate("alice").is_ok());
        assert!(validate("Alice99").is_ok());
        assert!(validate("a").is_ok());
        assert!(validate("123456789012345").is_ok()); // exactly 15
        assert!(validate("café").is_ok()); // Unicode letters allowed
    }

    #[test]
    fn rejects_empty_too_long_and_non_alphanumeric() {
        assert!(matches!(validate(""), Err(AppError::InvalidArgument(_))));
        assert!(matches!(
            validate("1234567890123456"), // 16
            Err(AppError::InvalidArgument(_))
        ));
        for bad in ["has space", "dash-y", "under_score", "dot.", "emoji😀"] {
            assert!(
                matches!(validate(bad), Err(AppError::InvalidArgument(_))),
                "expected reject: {bad:?}"
            );
        }
    }

    #[test]
    fn normalize_folds_case_and_nfc() {
        // Case-insensitive
        assert_eq!(normalize("Alice"), normalize("alice"));
        assert_eq!(normalize("ABC"), "abc");
        // NFC-equivalent forms collide: "é" as one code point vs "e" + combining acute.
        let composed = "caf\u{00e9}"; // café (NFC)
        let decomposed = "cafe\u{0301}"; // cafe + U+0301
        assert_eq!(normalize(composed), normalize(decomposed));
    }

    #[test]
    fn distinct_handles_keep_distinct_keys() {
        assert_ne!(normalize("alice"), normalize("alice2"));
        assert_ne!(normalize("bob"), normalize("bobby"));
    }
}
