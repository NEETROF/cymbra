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

//! A pack's metadata: which pair it serves, its versions, its licences.

use serde::{Deserialize, Serialize};

/// The metadata blob carried in a pack's container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackMeta {
    /// ISO-639-1 tag of the studied language (e.g. `"en"`).
    pub studied: String,
    /// ISO-639-1 tag of the native / gloss language (e.g. `"fr"`).
    pub native: String,
    /// The pack's own content version (data revision).
    pub pack_version: String,
    /// The analyser generation this pack was built for; the reader refuses a
    /// pack whose value does not match the running core.
    pub analyzer_version: String,
    /// One entry per bundled source's licence/attribution (the human-readable
    /// stack also lives verbatim in the NOTICE section).
    pub licences: Vec<String>,
    /// Whether the level table is estimated from word frequency rather than
    /// taken from a CEFR list (add-lingua-spanish-levels), so the extension
    /// labels those levels as estimated. A pack that says nothing reads as
    /// false, and false is never written, so such a pack keeps its bytes.
    #[serde(default, skip_serializing_if = "is_false")]
    pub levels_estimated: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl PackMeta {
    /// The pair key, e.g. `"en->fr"`.
    pub fn pair_key(&self) -> String {
        format!("{}->{}", self.studied, self.native)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WITHOUT: &str = r#"{"studied":"en","native":"fr","pack_version":"1","analyzer_version":"1.1.0","licences":["x"]}"#;

    #[test]
    fn a_pack_that_says_nothing_of_its_levels_reads_not_estimated_and_keeps_its_bytes() {
        let meta: PackMeta = serde_json::from_str(WITHOUT).expect("parse");
        assert!(!meta.levels_estimated);
        assert_eq!(serde_json::to_string(&meta).expect("serialise"), WITHOUT);
    }

    #[test]
    fn estimated_levels_are_said_and_read_back() {
        let meta = PackMeta {
            levels_estimated: true,
            ..serde_json::from_str(WITHOUT).expect("parse")
        };
        let json = serde_json::to_string(&meta).expect("serialise");
        assert!(json.ends_with(r#""levels_estimated":true}"#), "{json}");
        assert_eq!(
            serde_json::from_str::<PackMeta>(&json).expect("parse"),
            meta
        );
    }
}
