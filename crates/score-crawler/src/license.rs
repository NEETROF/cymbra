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

//! The legally-critical licence engine.
//!
//! [`normalize`] turns a raw licence signal (free-text label, Creative Commons
//! URL, SPDX id, or source status string) into a canonical [`LicenseOutcome`]
//! — a pure function, no I/O, exhaustively unit-tested. [`is_redistributable`]
//! then decides whether that outcome may be kept, applying the whitelist:
//! **CC0 / confirmed Public Domain / CC-BY (any version) / CC-BY-SA (any
//! version)**. Everything else — NC, ND, all-rights-reserved, unknown, or
//! ambiguous — is rejected. Self-declared public domain is accepted only as
//! low-confidence (`unverified`), never mixed into the safe corpus.

use serde::{Deserialize, Serialize};

/// A raw licence signal observed for an item, gathered *before* any heavy
/// download so the gate can run license-first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLicense {
    /// The observed text: a label ("CC BY-SA 4.0"), a CC deed URL, an SPDX id,
    /// or a source-specific status ("Public Domain").
    pub signal: String,
    /// True when the redistributable status is asserted by an uploader/user
    /// without independent verification (e.g. musetrainer) — drives the
    /// `unverified` confidence classification.
    pub self_declared: bool,
}

impl RawLicense {
    /// A verified (source-authoritative) signal.
    pub fn verified(signal: impl Into<String>) -> Self {
        Self {
            signal: signal.into(),
            self_declared: false,
        }
    }

    /// A user/uploader-declared signal (low confidence).
    pub fn declared(signal: impl Into<String>) -> Self {
        Self {
            signal: signal.into(),
            self_declared: true,
        }
    }
}

/// The licence family a signal normalises to. Only the first four are
/// redistributable; the rest are always rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseFamily {
    /// CC0 / public-domain dedication.
    Cc0,
    /// Confirmed public domain (Public Domain Mark or source PD status).
    PublicDomain,
    /// CC-BY, any version.
    CcBy,
    /// CC-BY-SA, any version.
    CcBySa,
    /// CC-BY-NC (non-commercial) — rejected.
    CcByNc,
    /// CC-BY-ND (no-derivatives) — rejected.
    CcByNd,
    /// CC-BY-NC-SA — rejected.
    CcByNcSa,
    /// CC-BY-NC-ND — rejected.
    CcByNcNd,
    /// Explicit "all rights reserved" / copyright — rejected.
    AllRightsReserved,
    /// Unrecognised, empty, or ambiguous (conflicting signals) — rejected.
    Unknown,
}

impl LicenseFamily {
    /// Whether this family is on the redistributable whitelist.
    pub fn is_whitelisted(self) -> bool {
        matches!(
            self,
            LicenseFamily::Cc0
                | LicenseFamily::PublicDomain
                | LicenseFamily::CcBy
                | LicenseFamily::CcBySa
        )
    }
}

/// Confidence in the redistributable classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Source-authoritative (a real licence tag or an authoritative PD status).
    Verified,
    /// User/uploader-declared without independent verification.
    Unverified,
}

/// The normalised outcome of a raw licence signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseOutcome {
    /// Canonical code, e.g. `CC-BY-SA-4.0`, `CC-BY`, `CC0-1.0`, `PublicDomain`,
    /// `CC-BY-NC-4.0`, `AllRightsReserved`, `Unknown`.
    pub code: String,
    /// Canonical licence/deed URL when one applies.
    pub url: Option<String>,
    pub family: LicenseFamily,
    pub confidence: Confidence,
}

/// The gate decision for an outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Whitelisted and verified — safe corpus.
    Accept,
    /// Whitelisted but unverified — low-confidence corpus.
    LowConfidence,
    /// Not redistributable — journal and skip.
    Reject { reason: String },
}

/// The CC clause components parsed from a signal.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
struct CcClauses {
    by: bool,
    nc: bool,
    nd: bool,
    sa: bool,
}

/// Normalises a raw signal into a canonical [`LicenseOutcome`]. Pure; never
/// panics. Unknown, empty, and contradictory signals normalise to
/// [`LicenseFamily::Unknown`] (rejected) rather than being optimistically
/// mapped to any redistributable code.
pub fn normalize(raw: &RawLicense) -> LicenseOutcome {
    // Canonicalise: lowercase, unify separators so "CC BY-SA 4.0",
    // "cc_by_sa_4.0" and the deed URL all reduce to comparable tokens.
    let s = raw.signal.trim().to_lowercase();
    let flat = s.replace(['_', ' ', '/'], "-");

    let confidence = if raw.self_declared {
        Confidence::Unverified
    } else {
        Confidence::Verified
    };

    if s.is_empty() {
        return outcome("Unknown", None, LicenseFamily::Unknown, confidence);
    }

    // Detect each independent signal; more than one distinct family ⇒ ambiguous.
    let is_cc0 =
        flat.contains("cc0") || flat.contains("publicdomain-zero") || flat.contains("commons-zero");
    let is_pd = !is_cc0
        && (flat.contains("public-domain")
            || flat.contains("publicdomain")
            || flat.contains("public-domain-mark")
            || flat.contains("-pdm")
            || s == "pd"
            || s == "public domain");
    let is_arr = flat.contains("all-rights-reserved")
        || flat.contains("tous-droits-reserves")
        || flat.contains("copyright")
        || flat.contains('©');
    let cc = parse_cc(&flat);

    // Count distinct families present to catch contradictions.
    let mut families: Vec<LicenseFamily> = Vec::new();
    if is_cc0 {
        families.push(LicenseFamily::Cc0);
    }
    if is_pd {
        families.push(LicenseFamily::PublicDomain);
    }
    if let Some((clauses, _)) = cc {
        families.push(cc_family(clauses));
    }
    if is_arr {
        families.push(LicenseFamily::AllRightsReserved);
    }

    if families.len() > 1 {
        // e.g. a page tagged both "public domain" and "all rights reserved".
        return outcome("Ambiguous", None, LicenseFamily::Unknown, confidence);
    }

    match families.first().copied() {
        Some(LicenseFamily::Cc0) => outcome(
            "CC0-1.0",
            Some("https://creativecommons.org/publicdomain/zero/1.0/".into()),
            LicenseFamily::Cc0,
            confidence,
        ),
        Some(LicenseFamily::PublicDomain) => outcome(
            "PublicDomain",
            Some("https://creativecommons.org/publicdomain/mark/1.0/".into()),
            LicenseFamily::PublicDomain,
            confidence,
        ),
        Some(LicenseFamily::AllRightsReserved) => outcome(
            "AllRightsReserved",
            None,
            LicenseFamily::AllRightsReserved,
            confidence,
        ),
        Some(fam) => {
            // A CC licence family: build the canonical code + deed URL.
            let (clauses, version) = cc.expect("cc family implies parsed clauses");
            let (code, url) = cc_code_and_url(clauses, version.as_deref());
            outcome(&code, url, fam, confidence)
        }
        None => outcome("Unknown", None, LicenseFamily::Unknown, confidence),
    }
}

/// Applies the whitelist to a normalised outcome.
pub fn is_redistributable(outcome: &LicenseOutcome) -> Decision {
    if outcome.family.is_whitelisted() {
        match outcome.confidence {
            Confidence::Verified => Decision::Accept,
            Confidence::Unverified => Decision::LowConfidence,
        }
    } else {
        Decision::Reject {
            reason: format!("licence {} is not redistributable", outcome.code),
        }
    }
}

/// Convenience: normalise then gate in one call.
pub fn evaluate(raw: &RawLicense) -> (LicenseOutcome, Decision) {
    let outcome = normalize(raw);
    let decision = is_redistributable(&outcome);
    (outcome, decision)
}

fn outcome(
    code: &str,
    url: Option<String>,
    family: LicenseFamily,
    confidence: Confidence,
) -> LicenseOutcome {
    LicenseOutcome {
        code: code.to_string(),
        url,
        family,
        confidence,
    }
}

/// Parses CC clauses + version from a flattened signal, or `None` if it is not a
/// recognisable CC *licence* (CC0/PD are handled separately). Requires the `by`
/// clause, since every redistributable-or-not CC *licence* includes attribution.
fn parse_cc(flat: &str) -> Option<(CcClauses, Option<String>)> {
    // Recognise both the short token form ("cc-by-sa-4.0", the deed URL) and the
    // spelled-out form libraries like Mutopia use ("creative commons
    // attribution-sharealike 4.0"). Both require the attribution (`by`) clause.
    let short = flat.contains("cc-by") || flat.contains("licenses-by");
    let long = (flat.contains("creative-commons") || flat.contains("creativecommons"))
        && (flat.contains("attribution") || flat.contains("-by"));
    if !short && !long {
        return None;
    }
    let clauses = CcClauses {
        by: true,
        nc: flat.contains("-nc") || flat.contains("noncommercial"),
        nd: flat.contains("-nd")
            || flat.contains("noderiv")
            || flat.contains("no-derivatives")
            || flat.contains("noderivatives"),
        sa: flat.contains("-sa") || flat.contains("sharealike") || flat.contains("share-alike"),
    };
    Some((clauses, extract_version(flat)))
}

/// Extracts a CC version token like `4.0`, `3.0`, `2.5`, `1.0` if present.
fn extract_version(flat: &str) -> Option<String> {
    const VERSIONS: [&str; 6] = ["4.0", "3.0", "2.5", "2.1", "2.0", "1.0"];
    VERSIONS
        .iter()
        .find(|v| flat.contains(**v))
        .map(|v| v.to_string())
}

fn cc_family(c: CcClauses) -> LicenseFamily {
    match (c.nc, c.nd, c.sa) {
        (false, false, false) => LicenseFamily::CcBy,
        (false, false, true) => LicenseFamily::CcBySa,
        (true, false, false) => LicenseFamily::CcByNc,
        (false, true, false) => LicenseFamily::CcByNd,
        (true, false, true) => LicenseFamily::CcByNcSa,
        (true, true, false) => LicenseFamily::CcByNcNd,
        // ND+SA is not a real CC combination; treat as unknown-ish ND (rejected).
        _ => LicenseFamily::CcByNd,
    }
}

fn cc_code_and_url(c: CcClauses, version: Option<&str>) -> (String, Option<String>) {
    let mut parts = vec!["by"];
    if c.nc {
        parts.push("nc");
    }
    if c.nd {
        parts.push("nd");
    }
    if c.sa {
        parts.push("sa");
    }
    let path = parts.join("-");
    let code = match version {
        Some(v) => format!("CC-{}-{v}", path.to_uppercase()),
        None => format!("CC-{}", path.to_uppercase()),
    };
    let url = version
        .map(|v| format!("https://creativecommons.org/licenses/{path}/{v}/"))
        .or_else(|| Some(format!("https://creativecommons.org/licenses/{path}/")));
    (code, url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fam(signal: &str) -> LicenseFamily {
        normalize(&RawLicense::verified(signal)).family
    }

    fn decide(signal: &str) -> Decision {
        is_redistributable(&normalize(&RawLicense::verified(signal)))
    }

    #[test]
    fn accepts_cc0() {
        let o = normalize(&RawLicense::verified("CC0 1.0"));
        assert_eq!(o.family, LicenseFamily::Cc0);
        assert_eq!(o.code, "CC0-1.0");
        assert_eq!(decide("CC0 1.0"), Decision::Accept);
        assert_eq!(
            fam("https://creativecommons.org/publicdomain/zero/1.0/"),
            LicenseFamily::Cc0
        );
    }

    #[test]
    fn accepts_public_domain() {
        assert_eq!(fam("Public Domain"), LicenseFamily::PublicDomain);
        assert_eq!(fam("public-domain-mark"), LicenseFamily::PublicDomain);
        assert_eq!(fam("PD"), LicenseFamily::PublicDomain);
        assert_eq!(decide("Public Domain"), Decision::Accept);
    }

    #[test]
    fn accepts_cc_by_all_versions() {
        for s in ["CC BY 4.0", "cc-by-3.0", "CC-BY-2.0", "cc by"] {
            assert_eq!(fam(s), LicenseFamily::CcBy, "signal: {s}");
            assert_eq!(decide(s), Decision::Accept, "signal: {s}");
        }
    }

    #[test]
    fn versionless_cc_by_is_accepted_as_any_version() {
        let o = normalize(&RawLicense::verified("CC BY"));
        assert_eq!(o.code, "CC-BY");
        assert_eq!(
            o.url.as_deref(),
            Some("https://creativecommons.org/licenses/by/")
        );
        assert_eq!(is_redistributable(&o), Decision::Accept);
    }

    #[test]
    fn accepts_cc_by_sa_all_versions() {
        for s in [
            "CC BY-SA 4.0",
            "cc-by-sa-3.0",
            "https://creativecommons.org/licenses/by-sa/4.0/",
        ] {
            assert_eq!(fam(s), LicenseFamily::CcBySa, "signal: {s}");
            assert_eq!(decide(s), Decision::Accept, "signal: {s}");
        }
        let o = normalize(&RawLicense::verified(
            "https://creativecommons.org/licenses/by-sa/4.0/",
        ));
        assert_eq!(o.code, "CC-BY-SA-4.0");
    }

    #[test]
    fn rejects_nc_and_nd() {
        for s in [
            "CC BY-NC 4.0",
            "CC BY-ND 4.0",
            "CC BY-NC-SA 4.0",
            "CC BY-NC-ND 4.0",
        ] {
            assert!(!fam(s).is_whitelisted(), "signal: {s}");
            assert!(matches!(decide(s), Decision::Reject { .. }), "signal: {s}");
        }
    }

    #[test]
    fn rejects_all_rights_reserved() {
        assert_eq!(fam("All Rights Reserved"), LicenseFamily::AllRightsReserved);
        assert_eq!(
            fam("Copyright 2020 Someone"),
            LicenseFamily::AllRightsReserved
        );
        assert!(matches!(
            decide("All Rights Reserved"),
            Decision::Reject { .. }
        ));
    }

    #[test]
    fn rejects_unknown_and_empty() {
        assert_eq!(fam(""), LicenseFamily::Unknown);
        assert_eq!(fam("some random text"), LicenseFamily::Unknown);
        assert!(matches!(decide(""), Decision::Reject { .. }));
        assert!(matches!(
            decide("license: see website"),
            Decision::Reject { .. }
        ));
    }

    #[test]
    fn rejects_ambiguous_conflicting_signals() {
        // A page carrying two contradictory licences must not be optimistically
        // accepted.
        let o = normalize(&RawLicense::verified("Public Domain — All Rights Reserved"));
        assert_eq!(o.family, LicenseFamily::Unknown);
        assert_eq!(o.code, "Ambiguous");
        assert!(matches!(is_redistributable(&o), Decision::Reject { .. }));
    }

    #[test]
    fn self_declared_pd_is_low_confidence_not_rejected() {
        let raw = RawLicense::declared("Public Domain");
        let o = normalize(&raw);
        assert_eq!(o.family, LicenseFamily::PublicDomain);
        assert_eq!(o.confidence, Confidence::Unverified);
        assert_eq!(is_redistributable(&o), Decision::LowConfidence);
    }

    #[test]
    fn self_declared_does_not_rescue_a_rejected_licence() {
        let raw = RawLicense::declared("CC BY-NC 4.0");
        let o = normalize(&raw);
        assert!(matches!(is_redistributable(&o), Decision::Reject { .. }));
    }

    #[test]
    fn accepts_spelled_out_cc_licenses() {
        // The long form libraries like Mutopia write in the file header.
        for s in [
            "Creative Commons Attribution 4.0",
            "Creative Commons Attribution-ShareAlike 4.0",
            "Creative Commons Attribution-ShareAlike 3.0 Unported",
        ] {
            assert!(fam(s).is_whitelisted(), "signal: {s}");
            assert_eq!(decide(s), Decision::Accept, "signal: {s}");
        }
        assert_eq!(fam("Creative Commons Attribution 4.0"), LicenseFamily::CcBy);
        assert_eq!(
            fam("Creative Commons Attribution-ShareAlike 4.0"),
            LicenseFamily::CcBySa
        );
        assert_eq!(fam("Creative Commons Zero"), LicenseFamily::Cc0);
    }

    #[test]
    fn rejects_spelled_out_nc_and_nd() {
        for s in [
            "Creative Commons Attribution-NonCommercial 4.0",
            "Creative Commons Attribution-NoDerivatives 4.0",
            "Creative Commons Attribution-NonCommercial-ShareAlike 4.0",
        ] {
            assert!(!fam(s).is_whitelisted(), "signal: {s}");
            assert!(matches!(decide(s), Decision::Reject { .. }), "signal: {s}");
        }
    }

    #[test]
    fn evaluate_pairs_outcome_and_decision() {
        let (o, d) = evaluate(&RawLicense::verified("CC-BY-SA-4.0"));
        assert_eq!(o.code, "CC-BY-SA-4.0");
        assert_eq!(d, Decision::Accept);
    }
}
