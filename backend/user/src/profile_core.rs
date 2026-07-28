//! Pure, host-testable logic for the public-profile minor safeguard (change:
//! add-play-activity-profile, design D6). The direct adapter ([`crate::module`])
//! calls these; no I/O, no clock — the current date is injected so eligibility is
//! deterministic and testable.
//!
//! The invariants:
//! * We store only a **derived eligibility date** (`share_eligible_from = DOB +
//!   min_public_sharing_age years`) — never the date of birth.
//! * Eligibility is a **UTC date comparison with a one-day safety margin**
//!   (`today > share_eligible_from`), so a timezone difference can never grant it
//!   early; being eligible one day late is harmless and self-correcting.

use chrono::{Datelike, NaiveDate};

/// Derive the eligibility date: the day the user reaches `min_age`, i.e.
/// `date_of_birth + min_age years`. Feb-29 births in a non-leap target year roll
/// **forward** to Mar 1 (the conservative choice for a safety gate — eligibility a
/// day later, never earlier).
pub fn derive_eligible_from(date_of_birth: NaiveDate, min_age: u32) -> NaiveDate {
    let target_year = date_of_birth.year() + min_age as i32;
    NaiveDate::from_ymd_opt(target_year, date_of_birth.month(), date_of_birth.day()).unwrap_or_else(
        || {
            // Only Feb 29 → a non-leap year lacks the day; roll to Mar 1.
            NaiveDate::from_ymd_opt(target_year, 3, 1).expect("Mar 1 is always valid")
        },
    )
}

/// Whether the user is old enough **today** (UTC) to make their profile public.
/// Strict `>` gives the one-day margin (design D6): on the birthday itself the
/// user is not yet eligible; the following day they are.
pub fn is_eligible(today: NaiveDate, eligible_from: NaiveDate) -> bool {
    today > eligible_from
}

/// A supplied date of birth is plausible only if it is not in the future
/// (relative to the injected `today`). Guards against a client sending a future
/// DOB to fabricate an eligibility date that never passes (harmless, but rejected
/// early with a clear error).
pub fn dob_is_plausible(date_of_birth: NaiveDate, today: NaiveDate) -> bool {
    date_of_birth <= today
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn eligible_from_is_dob_plus_min_age() {
        // Born 2008-06-15, min age 16 → eligible date 2024-06-15.
        assert_eq!(derive_eligible_from(ymd(2008, 6, 15), 16), ymd(2024, 6, 15));
    }

    #[test]
    fn feb_29_birth_rolls_forward_to_mar_1_in_non_leap_year() {
        // 2008 is a leap year; 2008+16 = 2024 IS a leap year → stays Feb 29.
        assert_eq!(derive_eligible_from(ymd(2008, 2, 29), 16), ymd(2024, 2, 29));
        // 2007+16 = 2023 is NOT leap → rolls to Mar 1 (conservative, a day later).
        // (2007-02-29 is itself invalid, so use a leap birth whose +N lands non-leap.)
        assert_eq!(derive_eligible_from(ymd(2004, 2, 29), 3), ymd(2007, 3, 1));
    }

    #[test]
    fn one_day_margin_before_on_and_after_the_boundary() {
        let elig = ymd(2024, 6, 15);
        // The eligibility date itself → NOT yet eligible (strict >).
        assert!(!is_eligible(ymd(2024, 6, 15), elig));
        // The day after → eligible.
        assert!(is_eligible(ymd(2024, 6, 16), elig));
        // Well before → not eligible.
        assert!(!is_eligible(ymd(2024, 6, 1), elig));
    }

    #[test]
    fn future_dob_is_rejected_but_past_and_today_ok() {
        let today = ymd(2026, 7, 28);
        assert!(dob_is_plausible(ymd(2008, 1, 1), today));
        assert!(dob_is_plausible(today, today));
        assert!(!dob_is_plausible(ymd(2027, 1, 1), today));
    }
}
