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

//! The L1/L2 profile (design D3).
//!
//! `native_language` — the language of comfort (glosses, future
//! translations) — is kept distinct from the studied languages. Everything
//! language-dependent downstream (glosses, packs, knowledge state) keys by
//! **pair** (studied → native); the pair key is formed at runtime, so adding
//! a pair is data, never code. The MVP ships (English → French) only.

use serde::{Deserialize, Serialize};

use crate::analysis::language::StudiedLanguage;

/// A language the user is comfortable reading in — the target of glosses and
/// future translations. Distinct from [`StudiedLanguage`]: the UI locale is
/// not necessarily the gloss language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NativeLanguage {
    /// French.
    French,
    /// English.
    English,
    /// Spanish.
    Spanish,
    /// Italian.
    Italian,
    /// Portuguese.
    Portuguese,
}

impl NativeLanguage {
    /// ISO-639-1 tag used in pair keys and pack file names.
    pub fn tag(self) -> &'static str {
        match self {
            NativeLanguage::French => "fr",
            NativeLanguage::English => "en",
            NativeLanguage::Spanish => "es",
            NativeLanguage::Italian => "it",
            NativeLanguage::Portuguese => "pt",
        }
    }
}

/// ISO-639-1 tag of a studied language, for pair keys and pack file names.
pub fn studied_tag(studied: StudiedLanguage) -> &'static str {
    match studied {
        StudiedLanguage::English => "en",
    }
}

/// A studied→native pair: the key under which a pack, a gloss set and the
/// knowledge state live. Built from any (studied, native) combination — no
/// per-pair code exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LanguagePair {
    /// The language being learned.
    pub studied: StudiedLanguage,
    /// The language its glosses are written in.
    pub native: NativeLanguage,
}

impl LanguagePair {
    /// The stable pair key, e.g. `"en->fr"` — names the pack to load and the
    /// gloss set to read.
    pub fn key(self) -> String {
        format!("{}->{}", studied_tag(self.studied), self.native.tag())
    }
}

/// The user's language profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// The language of comfort — glosses and future translations target it.
    pub native_language: NativeLanguage,
    /// The languages being learned, in the user's own order.
    pub studied_languages: Vec<StudiedLanguage>,
}

impl Profile {
    /// The MVP profile: learning English, comfortable in French.
    pub fn english_for_french() -> Self {
        Self {
            native_language: NativeLanguage::French,
            studied_languages: vec![StudiedLanguage::English],
        }
    }

    /// The pair for one studied language, or `None` if it is not in the
    /// profile.
    pub fn pair_for(&self, studied: StudiedLanguage) -> Option<LanguagePair> {
        self.studied_languages
            .contains(&studied)
            .then_some(LanguagePair {
                studied,
                native: self.native_language,
            })
    }

    /// Every pair the profile activates, in the studied-language order.
    pub fn pairs(&self) -> Vec<LanguagePair> {
        self.studied_languages
            .iter()
            .map(|&studied| LanguagePair {
                studied,
                native: self.native_language,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mvp_profile_yields_the_en_fr_pair() {
        let profile = Profile::english_for_french();
        let pair = profile
            .pair_for(StudiedLanguage::English)
            .expect("english is studied");
        assert_eq!(pair.key(), "en->fr");
        assert_eq!(profile.pairs(), vec![pair]);
    }

    #[test]
    fn a_pair_is_data_not_code() {
        // Forming a hypothetical (en -> es) pair needs no new code path.
        let pair = LanguagePair {
            studied: StudiedLanguage::English,
            native: NativeLanguage::Spanish,
        };
        assert_eq!(pair.key(), "en->es");
    }

    #[test]
    fn pair_for_unstudied_language_is_none() {
        let profile = Profile {
            native_language: NativeLanguage::French,
            studied_languages: vec![],
        };
        assert_eq!(profile.pair_for(StudiedLanguage::English), None);
    }
}
