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
//! translations) — is kept distinct from the studied languages, and is never
//! one of them. Glosses and packs key by **pair** (`<studied>-<native>`,
//! formed at runtime, so adding a pair is data, never code), whereas knowledge
//! state keys by studied language alone, so it does not depend on the native
//! language. The pairs shipped are en-fr and es-fr
//! (generalise-lingua-native-language).

use serde::{Deserialize, Serialize};

use crate::analysis::language::StudiedLanguage;

/// A language the user is comfortable reading in — the target of glosses and
/// future translations. Distinct from [`StudiedLanguage`]: the UI locale is
/// not necessarily the gloss language.
///
/// The variants keep their names and their order: a backup writes the names
/// (generalise-lingua-native-language D2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NativeLanguage {
    /// French.
    French,
    /// English.
    English,
    /// Spanish.
    Spanish,
}

impl NativeLanguage {
    /// Every native language the core knows, in declaration order.
    pub const ALL: [NativeLanguage; 3] = [
        NativeLanguage::French,
        NativeLanguage::English,
        NativeLanguage::Spanish,
    ];

    /// ISO-639-1 tag used in pair keys and pack file names.
    pub fn tag(self) -> &'static str {
        match self {
            NativeLanguage::French => "fr",
            NativeLanguage::English => "en",
            NativeLanguage::Spanish => "es",
        }
    }

    /// The native language a tag names, or `None` for one the core does not
    /// know. Exact, like [`StudiedLanguage::from_tag`].
    pub fn from_tag(tag: &str) -> Option<NativeLanguage> {
        NativeLanguage::ALL.into_iter().find(|l| l.tag() == tag)
    }

    /// The studied language with the same tag, which a reader of this native
    /// language never studies. `None` for a language the core cannot analyse;
    /// every native language it knows is studied too since French became one
    /// (add-lingua-french-baseline), so a French-native reader never studies
    /// French.
    pub fn studied(self) -> Option<StudiedLanguage> {
        StudiedLanguage::from_tag(self.tag())
    }
}

/// A studied→native pair: the key under which a pack and its glosses live.
/// Built from any (studied, native) combination — no per-pair code exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LanguagePair {
    /// The language being learned.
    pub studied: StudiedLanguage,
    /// The language its glosses are written in.
    pub native: NativeLanguage,
}

impl LanguagePair {
    /// The stable pair key, `<studied>-<native>` (e.g. `"en-fr"`), as the
    /// packs' file names read.
    pub fn key(self) -> String {
        format!("{}-{}", self.studied.tag(), self.native.tag())
    }
}

/// Why a choice of studied languages was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileError {
    /// No language: a reader always studies at least one.
    Empty,
    /// A language named twice.
    Duplicate(StudiedLanguage),
    /// The reader's native language, which they never study.
    NativeStudied(StudiedLanguage),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileError::Empty => write!(f, "a reader studies at least one language"),
            ProfileError::Duplicate(language) => {
                write!(f, "\"{}\" is named twice", language.tag())
            }
            ProfileError::NativeStudied(language) => write!(
                f,
                "\"{}\" is the native language, which a reader never studies",
                language.tag()
            ),
        }
    }
}

impl std::error::Error for ProfileError {}

/// The user's language profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// The language of comfort — glosses and future translations target it.
    pub native_language: NativeLanguage,
    /// The languages being learned, in the user's own order.
    pub studied_languages: Vec<StudiedLanguage>,
}

/// Refuses an empty list, a language named twice, or the native language's
/// studied counterpart.
fn check(native: NativeLanguage, languages: &[StudiedLanguage]) -> Result<(), ProfileError> {
    if languages.is_empty() {
        return Err(ProfileError::Empty);
    }
    for (i, language) in languages.iter().enumerate() {
        if languages[..i].contains(language) {
            return Err(ProfileError::Duplicate(*language));
        }
    }
    match native.studied() {
        Some(own) if languages.contains(&own) => Err(ProfileError::NativeStudied(own)),
        _ => Ok(()),
    }
}

impl Profile {
    /// The MVP profile: learning English, comfortable in French.
    pub fn english_for_french() -> Self {
        Self::studying(NativeLanguage::French, StudiedLanguage::English)
    }

    /// A reader of `native` studying `studied` alone: the profile an engine
    /// starts with, and returns to on a full reset, from its first pack
    /// (generalise-lingua-native-language D5). The pack's pair guarantees the
    /// two differ.
    pub fn studying(native: NativeLanguage, studied: StudiedLanguage) -> Self {
        Self {
            native_language: native,
            studied_languages: vec![studied],
        }
    }

    /// Whether this is the MVP profile, the one every reader starts with. A
    /// backup leaves it out (add-lingua-studied-language-profile).
    pub fn is_default(&self) -> bool {
        *self == Self::english_for_french()
    }

    /// Sets the studied languages, the primary first. Refuses an empty list, a
    /// language named twice, or the native language, leaving the current ones
    /// in place.
    pub fn set_studied_languages(
        &mut self,
        languages: Vec<StudiedLanguage>,
    ) -> Result<(), ProfileError> {
        check(self.native_language, &languages)?;
        self.studied_languages = languages;
        Ok(())
    }

    /// Sets the native language and the studied languages together, refused as
    /// [`Profile::set_studied_languages`] refuses a choice — including one that
    /// would study the new native language. Both change, or neither: there is
    /// no lone native-language setter, since switching a reader of English to
    /// English native would be refused half-way in either order.
    pub fn set(
        &mut self,
        native: NativeLanguage,
        languages: Vec<StudiedLanguage>,
    ) -> Result<(), ProfileError> {
        check(native, &languages)?;
        self.native_language = native;
        self.studied_languages = languages;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: StudiedLanguage = StudiedLanguage::English;
    const ES: StudiedLanguage = StudiedLanguage::Spanish;
    const FR: StudiedLanguage = StudiedLanguage::French;

    #[test]
    fn a_native_language_round_trips_through_its_tag() {
        for native in NativeLanguage::ALL {
            assert_eq!(NativeLanguage::from_tag(native.tag()), Some(native));
        }
        assert_eq!(
            NativeLanguage::ALL.map(NativeLanguage::tag),
            ["fr", "en", "es"]
        );
        for unknown in ["de", "it", "pt", "FR", "", "en-fr"] {
            assert_eq!(NativeLanguage::from_tag(unknown), None, "{unknown:?}");
        }
    }

    #[test]
    fn a_native_language_maps_to_the_studied_language_of_its_tag() {
        assert_eq!(NativeLanguage::English.studied(), Some(EN));
        assert_eq!(NativeLanguage::Spanish.studied(), Some(ES));
        assert_eq!(NativeLanguage::French.studied(), Some(FR));
    }

    #[test]
    fn spec_scenario_a_french_native_reader_cannot_study_french() {
        // add-lingua-french-baseline D8: through either setter, and the profile stays.
        let mut profile = Profile::english_for_french();
        assert_eq!(
            profile.set_studied_languages(vec![EN, FR]),
            Err(ProfileError::NativeStudied(FR))
        );
        assert_eq!(
            profile.set(NativeLanguage::French, vec![FR]),
            Err(ProfileError::NativeStudied(FR))
        );
        assert!(profile.is_default(), "a refused choice leaves the profile");
        assert_eq!(
            ProfileError::NativeStudied(FR).to_string(),
            "\"fr\" is the native language, which a reader never studies"
        );
        // A reader of English or Spanish may study French.
        profile
            .set(NativeLanguage::English, vec![FR, ES])
            .expect("French for an English reader");
        assert_eq!(profile.studied_languages, vec![FR, ES]);
        let mut spanish = Profile::studying(NativeLanguage::Spanish, EN);
        spanish
            .set_studied_languages(vec![FR])
            .expect("French for a Spanish reader");
        assert_eq!(
            LanguagePair {
                studied: FR,
                native: NativeLanguage::English
            }
            .key(),
            "fr-en"
        );
    }

    #[test]
    fn a_pair_key_reads_studied_then_native() {
        let pair = LanguagePair {
            studied: EN,
            native: NativeLanguage::French,
        };
        assert_eq!(pair.key(), "en-fr");
        // A pair is data, not code: es-en needs no new code path.
        let pair = LanguagePair {
            studied: ES,
            native: NativeLanguage::English,
        };
        assert_eq!(pair.key(), "es-en");
    }

    #[test]
    fn the_default_profile_is_english_for_french() {
        let profile = Profile::english_for_french();
        assert_eq!(profile.native_language, NativeLanguage::French);
        assert_eq!(profile.studied_languages, vec![EN]);
        assert!(profile.is_default());
        assert_eq!(Profile::studying(NativeLanguage::French, EN), profile);
        assert!(!Profile::studying(NativeLanguage::English, ES).is_default());
        assert!(!Profile::studying(NativeLanguage::French, ES).is_default());
    }

    #[test]
    fn studied_languages_keep_their_order_and_refuse_a_bad_choice() {
        let mut profile = Profile::english_for_french();
        assert!(profile.is_default());
        profile
            .set_studied_languages(vec![ES, EN])
            .expect("two languages");
        assert_eq!(profile.studied_languages, vec![ES, EN]);
        assert!(!profile.is_default());

        assert_eq!(
            profile.set_studied_languages(vec![]),
            Err(ProfileError::Empty)
        );
        assert_eq!(
            profile.set_studied_languages(vec![EN, ES, EN]),
            Err(ProfileError::Duplicate(EN))
        );
        assert_eq!(
            profile.studied_languages,
            vec![ES, EN],
            "a refused choice leaves the current one"
        );
        assert_eq!(
            ProfileError::Duplicate(ES).to_string(),
            "\"es\" is named twice"
        );
        assert_eq!(
            ProfileError::Empty.to_string(),
            "a reader studies at least one language"
        );
    }

    #[test]
    fn the_native_language_is_never_studied() {
        let mut profile = Profile::studying(NativeLanguage::English, ES);
        assert_eq!(
            profile.set_studied_languages(vec![ES, EN]),
            Err(ProfileError::NativeStudied(EN))
        );
        assert_eq!(
            profile,
            Profile::studying(NativeLanguage::English, ES),
            "a refused choice leaves the profile as it was"
        );
        assert_eq!(
            ProfileError::NativeStudied(EN).to_string(),
            "\"en\" is the native language, which a reader never studies"
        );
        // A French reader may study English and Spanish.
        let mut french = Profile::english_for_french();
        assert_eq!(french.set_studied_languages(vec![ES, EN]), Ok(()));
    }

    #[test]
    fn the_native_and_studied_languages_change_together_or_not_at_all() {
        let mut profile = Profile::english_for_french();
        profile
            .set_studied_languages(vec![EN, ES])
            .expect("two languages");

        // Switching a reader of English to English native is refused whole.
        assert_eq!(
            profile.set(NativeLanguage::English, vec![EN, ES]),
            Err(ProfileError::NativeStudied(EN))
        );
        assert_eq!(profile.native_language, NativeLanguage::French);
        assert_eq!(profile.studied_languages, vec![EN, ES]);
        assert_eq!(
            profile.set(NativeLanguage::Spanish, vec![]),
            Err(ProfileError::Empty)
        );
        assert_eq!(
            profile.set(NativeLanguage::Spanish, vec![EN, EN]),
            Err(ProfileError::Duplicate(EN))
        );
        assert_eq!(profile.native_language, NativeLanguage::French);

        // A valid choice replaces both.
        profile
            .set(NativeLanguage::English, vec![ES])
            .expect("Spanish for an English reader");
        assert_eq!(profile, Profile::studying(NativeLanguage::English, ES));
        profile
            .set(NativeLanguage::French, vec![EN])
            .expect("back to the default");
        assert!(profile.is_default());
    }
}
