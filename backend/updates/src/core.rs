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

//! Pure, host-testable feed logic (change: add-desktop-auto-update, task 2.8):
//! version ordering and the servable-release predicate. No Postgres, no HTTP —
//! the adapters in [`crate::repo`] and `backend/server/src/updates.rs` are thin
//! glue over what lives here.

use crate::Release;

/// Upper bounds each version component must stay under for [`version_order`] to
/// be injective. Generous enough that no real release approaches them.
const MINOR_SPAN: i64 = 10_000;
const PATCH_SPAN: i64 = 10_000;
const BUILD_SPAN: i64 = 100_000;

/// Project `major.minor.patch+build` onto a single sortable integer.
///
/// The whole point is that a text sort puts `1.10.0` *below* `1.9.0`. The order
/// is the triple then the build number, matching the app's own `AppVersion`
/// ordering — the two must agree or a client would be offered an update it then
/// refuses as non-newer.
///
/// Returns `None` for anything malformed or out of range, which the ingest
/// handler turns into a rejection rather than storing an unsortable row.
pub fn version_order(version: &str) -> Option<i64> {
    let (triple, build) = match version.split_once('+') {
        Some((t, b)) => (t, b.parse::<i64>().ok()?),
        None => (version, 0),
    };
    let mut parts = triple.split('.');
    let major = parts.next()?.parse::<i64>().ok()?;
    let minor = parts.next()?.parse::<i64>().ok()?;
    let patch = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if !(0..MINOR_SPAN).contains(&minor)
        || !(0..PATCH_SPAN).contains(&patch)
        || !(0..BUILD_SPAN).contains(&build)
        || !(0..100_000).contains(&major)
    {
        return None;
    }
    Some(((major * MINOR_SPAN + minor) * PATCH_SPAN + patch) * BUILD_SPAN + build)
}

/// Whether a stored release may be served at all. Paused and rollout-0 are the
/// two ops levers, and rollout-0 is the kill-switch.
pub fn is_servable(release: &Release) -> bool {
    !release.paused && release.rollout_percent > 0
}

/// The release the public endpoint answers with: the highest **servable**
/// version, or `None` (a `204`) when there is nothing to offer.
///
/// Highest, not most recently ingested: re-ingesting an older version must never
/// roll clients back — the client refuses a non-newer version anyway (design D6),
/// so serving one would just make every check a silent no-op.
pub fn select_servable(candidates: &[Release]) -> Option<&Release> {
    candidates
        .iter()
        .filter(|r| is_servable(r))
        .max_by_key(|r| r.version_order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(version: &str, rollout: i16, paused: bool) -> Release {
        Release {
            product: "music".into(),
            channel: "stable".into(),
            version: version.into(),
            version_order: version_order(version).unwrap(),
            manifest: "bWFuaWZlc3Q=".into(),
            signature: "c2ln".into(),
            key_id: "2026-08-a".into(),
            rollout_percent: rollout,
            paused,
        }
    }

    #[test]
    fn version_order_is_numeric_not_lexical() {
        assert!(version_order("1.10.0+40").unwrap() > version_order("1.9.0+39").unwrap());
        assert!(version_order("2.0.0+1").unwrap() > version_order("1.99.99+99999").unwrap());
    }

    #[test]
    fn build_number_breaks_the_tie() {
        assert!(version_order("1.25.0+34").unwrap() > version_order("1.25.0+33").unwrap());
        assert_eq!(version_order("1.25.0"), version_order("1.25.0+0"));
    }

    #[test]
    fn malformed_versions_have_no_order() {
        for bad in [
            "",
            "1",
            "1.2",
            "1.2.3.4",
            "1.2.x",
            "v1.2.3",
            "1.2.3+",
            "1.2.3+abc",
            "-1.2.3",
        ] {
            assert!(version_order(bad).is_none(), "{bad} should not parse");
        }
    }

    #[test]
    fn out_of_range_components_have_no_order() {
        assert!(version_order("1.10000.0").is_none());
        assert!(version_order("1.0.10000").is_none());
        assert!(version_order("1.0.0+100000").is_none());
    }

    #[test]
    fn a_paused_release_is_not_servable() {
        assert!(!is_servable(&release("1.25.0+34", 100, true)));
    }

    #[test]
    fn rollout_zero_is_the_kill_switch() {
        assert!(!is_servable(&release("1.25.0+34", 0, false)));
    }

    #[test]
    fn selects_the_highest_servable_version() {
        let rows = vec![
            release("1.24.0+32", 100, false),
            release("1.25.0+34", 100, false),
            release("1.23.0+30", 100, false),
        ];
        assert_eq!(select_servable(&rows).unwrap().version, "1.25.0+34");
    }

    #[test]
    fn skips_the_paused_head_and_offers_the_one_below() {
        let rows = vec![
            release("1.24.0+32", 100, false),
            release("1.25.0+34", 100, true),
        ];
        assert_eq!(select_servable(&rows).unwrap().version, "1.24.0+32");
    }

    #[test]
    fn nothing_servable_means_nothing_to_offer() {
        let rows = vec![
            release("1.25.0+34", 0, false),
            release("1.24.0+32", 100, true),
        ];
        assert!(select_servable(&rows).is_none());
        assert!(select_servable(&[]).is_none());
    }
}
