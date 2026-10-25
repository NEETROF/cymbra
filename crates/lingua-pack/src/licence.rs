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

//! The licence guard (design D3): only commercially-usable sources may enter a
//! pack; GPL, AGPL and non-commercial sources are refused. CC0 (Wikidata, add-lingua-pack-it-es
//! D9) is a licence of its own, allowed: a manifest names it as `"Cc0"`.

use serde::{Deserialize, Serialize};

/// The licence of a bundled source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Licence {
    /// AGID / WordNet-style permissive (commercial use allowed).
    Permissive,
    /// Creative Commons Attribution-ShareAlike (wordfreq, kaikki).
    CcBySa,
    /// Creative Commons Attribution.
    CcBy,
    /// Creative Commons Zero — public domain dedication (Wikidata's lexemes, read by it-es:
    /// add-lingua-pack-it-es D9). Never denied.
    Cc0,
    /// GNU General Public License — copyleft, denied.
    Gpl,
    /// GNU Affero GPL — network copyleft, denied.
    Agpl,
    /// Any non-commercial licence — denied.
    NonCommercial,
}

/// A source bundled into a pack, for the licence check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// The source name, which must also appear verbatim in the NOTICE.
    pub name: String,
    /// Its licence.
    pub licence: Licence,
}

/// Whether a licence is on the denylist (must never enter a pack).
pub fn is_denied(licence: Licence) -> bool {
    matches!(
        licence,
        Licence::Gpl | Licence::Agpl | Licence::NonCommercial
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commercial_sources_are_allowed_copyleft_and_nc_are_denied() {
        assert!(!is_denied(Licence::Permissive));
        assert!(!is_denied(Licence::CcBySa));
        assert!(!is_denied(Licence::CcBy));
        assert!(!is_denied(Licence::Cc0));
        assert!(is_denied(Licence::Gpl));
        assert!(is_denied(Licence::Agpl));
        assert!(is_denied(Licence::NonCommercial));
    }

    #[test]
    fn a_manifest_names_cc0_as_cc0() {
        // it-es's manifest names Wikidata's licence `"Cc0"` (add-lingua-pack-it-es D9).
        let source: Source =
            serde_json::from_str(r#"{"name": "Wikidata", "licence": "Cc0"}"#).unwrap();
        assert_eq!(source.licence, Licence::Cc0);
        assert!(!is_denied(source.licence));
    }
}
