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

//! Entitlement decision for the raw SoundFont bytes (change:
//! add-soundfont-entitlement-previews).
//!
//! A pure, host-testable function decides whether a caller may download a font's
//! `.sf2` bytes, mirroring the existing moderation split ([`crate::soundfont`]'s
//! `decide`) so both are unit-tested without a DB or HTTP. The delivery route
//! applies the **moderation-visibility** gate first (a caller who can't see the font
//! at all is refused before entitlement is considered), then supplies this function's
//! inputs — the font row, whether a redemption grant exists, and whether the caller is
//! a music-scope moderator/admin.

use crate::soundfont::FontEntry;

/// The outcome of the entitlement decision. `Deny` maps to the same not-found
/// response as a missing font (no existence oracle for costed fonts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Allow,
    Deny,
}

/// Decide whether `caller` is entitled to the raw bytes of `font`.
///
/// `Allow` iff **any** of:
/// - the font is **free** (`point_cost == 0`);
/// - the font is the caller's **own import** (`uploaded_by == caller`);
/// - the caller **owns** it (`has_grant` — a `music.curation_grants` row);
/// - the caller's **effective plan grants the SoundFont library unlock**
///   (`plan_unlocks_library` — premium, whatever its source; change:
///   add-premium-subscription); the route passes `false` while `plans.enabled` is off;
/// - the caller is a **music-scope moderator/admin** (`is_music_mod_admin`, exempt).
///
/// Otherwise `Deny`. `redeemable` is deliberately *not* consulted: it is a catalog
/// display flag ("offered in the shop" / "included in premium"), so a non-redeemable
/// costed font is still gated for a free caller.
pub fn entitlement(
    caller: &str,
    font: &FontEntry,
    has_grant: bool,
    plan_unlocks_library: bool,
    is_music_mod_admin: bool,
) -> Access {
    let entitled = font.point_cost == 0
        || font.uploaded_by.as_deref() == Some(caller)
        || has_grant
        || plan_unlocks_library
        || is_music_mod_admin;
    if entitled {
        Access::Allow
    } else {
        Access::Deny
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soundfont::sha256_hex;

    /// A costed font uploaded by `owner` (or nobody when `owner` is `None`).
    fn costed(owner: Option<&str>) -> FontEntry {
        FontEntry {
            id: "grand".into(),
            label: "Grand".into(),
            object_key: "grand.sf2".into(),
            instrument: "piano".into(),
            license: "CC0-1.0".into(),
            attribution: None,
            size_bytes: None,
            moderation_status: "accepted".into(),
            reviewed_by: None,
            reviewed_at: None,
            uploaded_by: owner.map(str::to_string),
            content_sha256: Some(sha256_hex(b"grand")),
            point_cost: 500,
            redeemable: true,
            review_reason: None,
            resubmission_note: None,
        }
    }

    fn free() -> FontEntry {
        FontEntry {
            point_cost: 0,
            ..costed(None)
        }
    }

    #[test]
    fn free_font_is_allowed_for_anyone() {
        assert_eq!(
            entitlement("u", &free(), false, false, false),
            Access::Allow
        );
    }

    #[test]
    fn own_import_is_allowed() {
        // The caller uploaded this costed font: allowed without a grant or role.
        assert_eq!(
            entitlement("owner", &costed(Some("owner")), false, false, false),
            Access::Allow
        );
    }

    #[test]
    fn owned_via_grant_is_allowed() {
        assert_eq!(
            entitlement("u", &costed(None), true, false, false),
            Access::Allow
        );
    }

    #[test]
    fn plan_unlock_is_allowed_without_grant() {
        // Premium (any source) unlocks the whole library — no grant row needed.
        assert_eq!(
            entitlement("u", &costed(None), false, true, false),
            Access::Allow
        );
        // A non-redeemable ("included in premium") font too.
        let mut f = costed(None);
        f.redeemable = false;
        assert_eq!(entitlement("u", &f, false, true, false), Access::Allow);
    }

    #[test]
    fn lapsed_plan_is_denied_like_any_locked_font() {
        assert_eq!(
            entitlement("u", &costed(None), false, false, false),
            Access::Deny
        );
    }

    #[test]
    fn music_moderator_admin_is_exempt() {
        assert_eq!(
            entitlement("m", &costed(None), false, false, true),
            Access::Allow
        );
    }

    #[test]
    fn locked_costed_font_is_denied() {
        // No grant, not the uploader, not a moderator/admin → refused.
        assert_eq!(
            entitlement(
                "intruder",
                &costed(Some("someone-else")),
                false,
                false,
                false
            ),
            Access::Deny
        );
    }

    #[test]
    fn redeemable_flag_does_not_grant_access() {
        // A non-redeemable costed font is still gated; a redeemable one is still gated
        // absent a grant — `redeemable` is display-only, never an entitlement.
        let mut f = costed(None);
        f.redeemable = false;
        assert_eq!(entitlement("u", &f, false, false, false), Access::Deny);
        f.redeemable = true;
        assert_eq!(entitlement("u", &f, false, false, false), Access::Deny);
    }
}
