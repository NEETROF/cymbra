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

//! Authorization + validation for pricing a catalog SoundFont (change:
//! add-soundfont-reward-pricing).
//!
//! A pure, host-testable decision the `SetSoundFontPricing` handler runs before it
//! writes, mirroring [`crate::soundfont_access`]'s entitlement split so both sides of
//! the economy — who may *set* a price, who may *download* a costed font — are unit
//! tested without a DB or HTTP.
//!
//! Pricing is deliberately **admin**-only, strictly stronger than the
//! moderator-or-admin gate on metadata editing and moderation: a moderator may fix a
//! typo, but what a font *costs* is a product decision.

use cymbra_platform::error::{AppError, Result};
use cymbra_platform::guard;
use cymbra_platform::identity::AuthIdentity;

/// Authorize `caller` to price a font and validate `point_cost`, returning the cost to
/// persist.
///
/// - a non-admin (including a music-scope moderator) → `PermissionDenied`;
/// - a negative cost → `InvalidArgument`;
/// - otherwise the cost, widened to the repo's `i64`.
///
/// `redeemable` needs no validation: both values are meaningful (`false` lists the font
/// as "coming later"), and the entitlement gate keys on the cost, not on this flag.
/// Whether the font *exists* is not decided here — the repo write reports that, so an
/// unauthorized caller never learns which ids exist.
pub fn decide(caller: &AuthIdentity, point_cost: i32) -> Result<i64> {
    guard::require_admin(caller)?;
    if point_cost < 0 {
        return Err(AppError::InvalidArgument(
            "point_cost must be greater than or equal to 0".into(),
        ));
    }
    Ok(point_cost as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(roles: &[&str]) -> AuthIdentity {
        AuthIdentity {
            user_id: "u".into(),
            audience: "back-office".into(),
            roles: roles.iter().map(|s| s.to_string()).collect(),
            roles_by_scope: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn an_admin_may_price_a_font() {
        assert_eq!(decide(&id(&["user", "admin"]), 250).unwrap(), 250);
        // Free is a valid price — it is how a costed font reverts to free.
        assert_eq!(decide(&id(&["admin"]), 0).unwrap(), 0);
    }

    #[test]
    fn a_moderator_may_not_price_a_font() {
        // Metadata editing is moderator-or-admin; pricing is strictly admin.
        assert!(matches!(
            decide(&id(&["user", "moderator"]), 250),
            Err(AppError::PermissionDenied(_))
        ));
        assert!(matches!(
            decide(&id(&["user"]), 250),
            Err(AppError::PermissionDenied(_))
        ));
    }

    #[test]
    fn a_negative_cost_is_invalid() {
        assert!(matches!(
            decide(&id(&["admin"]), -1),
            Err(AppError::InvalidArgument(_))
        ));
    }

    /// The authorization is checked *before* the value, so a non-admin sending a bogus
    /// cost is refused as denied — never told their input was the problem.
    #[test]
    fn authorization_precedes_validation() {
        assert!(matches!(
            decide(&id(&["moderator"]), -1),
            Err(AppError::PermissionDenied(_))
        ));
    }
}
