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

//! One pack per studied language, held together (generalise-lingua-wasm-engine).
//!
//! An engine serves every language a reader studies from one state: one engine per
//! language would carry, and back up, its own whole state, and Firefox and Safari
//! serve every tab from a single engine. The first pack loaded gives the default
//! language, which a call that names none is answered in, and the native language
//! every pack of the set is glossed in (generalise-lingua-native-language D3).

use std::collections::BTreeMap;

use crate::analysis::language::StudiedLanguage;
use crate::knowledge::profile::NativeLanguage;

use super::pack::Pack;

/// Why the set refused a pack or a call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackSetError {
    /// A pack for this language is already held.
    AlreadyHeld(StudiedLanguage),
    /// The tag names no language this core can analyse.
    UnknownLanguage(String),
    /// The language is one the core analyses, but no pack for it is held.
    NotHeld(StudiedLanguage),
    /// The pack is glossed in another native language than the packs held.
    OtherNative {
        /// The native language of the packs held.
        held: NativeLanguage,
        /// The native language of the pack refused.
        pack: NativeLanguage,
    },
}

impl std::fmt::Display for PackSetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackSetError::AlreadyHeld(l) => {
                write!(f, "a pack for {:?} is already loaded", l.tag())
            }
            PackSetError::UnknownLanguage(tag) => {
                write!(f, "{tag:?} names no language this core can analyse")
            }
            PackSetError::NotHeld(l) => write!(f, "no pack for {:?} is loaded", l.tag()),
            PackSetError::OtherNative { held, pack } => write!(
                f,
                "the packs loaded are glossed in {:?}, so a pack glossed in {:?} is refused",
                held.tag(),
                pack.tag()
            ),
        }
    }
}

impl std::error::Error for PackSetError {}

/// The packs an engine holds, at most one per studied language, all glossed in one
/// native language.
pub struct PackSet {
    default: StudiedLanguage,
    native: NativeLanguage,
    packs: BTreeMap<StudiedLanguage, Pack>,
}

impl PackSet {
    /// A set holding `pack`, whose language becomes the default and whose native
    /// language the set's.
    pub fn new(pack: Pack) -> Self {
        let default = pack.studied();
        let native = pack.native();
        let mut packs = BTreeMap::new();
        packs.insert(default, pack);
        PackSet {
            default,
            native,
            packs,
        }
    }

    /// Adds another language's pack, returning its language. A pack glossed in
    /// another native language, or a second pack for a language already held, is
    /// refused, and the set is left as it was.
    pub fn add(&mut self, pack: Pack) -> Result<StudiedLanguage, PackSetError> {
        if pack.native() != self.native {
            return Err(PackSetError::OtherNative {
                held: self.native,
                pack: pack.native(),
            });
        }
        let language = pack.studied();
        if self.packs.contains_key(&language) {
            return Err(PackSetError::AlreadyHeld(language));
        }
        self.packs.insert(language, pack);
        Ok(language)
    }

    /// The language of the first pack loaded.
    pub fn default_language(&self) -> StudiedLanguage {
        self.default
    }

    /// The native language every pack held is glossed in: the first pack's.
    pub fn native(&self) -> NativeLanguage {
        self.native
    }

    /// The languages held, the default first, then the others in order.
    pub fn languages(&self) -> Vec<StudiedLanguage> {
        std::iter::once(self.default)
            .chain(self.packs.keys().copied().filter(|l| *l != self.default))
            .collect()
    }

    /// The pack for `language`, if held.
    pub fn get(&self, language: StudiedLanguage) -> Option<&Pack> {
        self.packs.get(&language)
    }

    /// Every language held with its pack, in language order.
    pub fn iter(&self) -> impl Iterator<Item = (StudiedLanguage, &Pack)> {
        self.packs.iter().map(|(l, p)| (*l, p))
    }

    /// The language a call names and its pack. `None` names the default
    /// language. An unknown tag, or a language with no pack held, is refused.
    pub fn resolve(&self, tag: Option<&str>) -> Result<(StudiedLanguage, &Pack), PackSetError> {
        let language = match tag {
            None => self.default,
            Some(tag) => StudiedLanguage::from_tag(tag)
                .ok_or_else(|| PackSetError::UnknownLanguage(tag.to_owned()))?,
        };
        self.packs
            .get(&language)
            .map(|pack| (language, pack))
            .ok_or(PackSetError::NotHeld(language))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{ANALYZER_VERSION, SPANISH_ANALYZER_VERSION};
    use crate::packs::pack::tests::{sample_pack_bytes_for, sample_pack_bytes_with};

    const EN: StudiedLanguage = StudiedLanguage::English;
    const ES: StudiedLanguage = StudiedLanguage::Spanish;
    const FR: NativeLanguage = NativeLanguage::French;

    fn english() -> Pack {
        Pack::load(&sample_pack_bytes_for("en", ANALYZER_VERSION)).expect("en pack")
    }

    fn spanish() -> Pack {
        Pack::load(&sample_pack_bytes_for("es", SPANISH_ANALYZER_VERSION)).expect("es pack")
    }

    /// A pack studying Spanish glossed in English.
    fn spanish_for_english() -> Pack {
        Pack::load(&sample_pack_bytes_with(
            "es",
            "en",
            SPANISH_ANALYZER_VERSION,
        ))
        .expect("es-en")
    }

    #[test]
    fn the_first_pack_gives_the_native_language() {
        assert_eq!(PackSet::new(english()).native(), FR);
        assert_eq!(
            PackSet::new(spanish_for_english()).native(),
            NativeLanguage::English
        );
    }

    #[test]
    fn a_pack_of_another_native_language_is_refused_before_anything_else() {
        let mut set = PackSet::new(english());
        let refused = set.add(spanish_for_english());
        assert_eq!(
            refused,
            Err(PackSetError::OtherNative {
                held: FR,
                pack: NativeLanguage::English
            })
        );
        assert_eq!(set.languages(), [EN], "the set is left as it was");
        assert_eq!(set.native(), FR);
        assert_eq!(
            refused.unwrap_err().to_string(),
            "the packs loaded are glossed in \"fr\", so a pack glossed in \"en\" is refused"
        );

        // Refused as another native language, though its studied language is held too.
        let mut spanish_held = PackSet::new(spanish());
        assert_eq!(
            spanish_held.add(spanish_for_english()),
            Err(PackSetError::OtherNative {
                held: FR,
                pack: NativeLanguage::English
            })
        );
        assert_eq!(spanish_held.languages(), [ES]);

        // A second language of the same native language is still added.
        assert_eq!(set.add(spanish()), Ok(ES));
        assert_eq!(set.languages(), [EN, ES]);
    }

    #[test]
    fn the_first_pack_gives_the_default_language() {
        let set = PackSet::new(english());
        assert_eq!(set.default_language(), EN);
        assert_eq!(set.languages(), [EN]);
        let (language, pack) = set.resolve(None).expect("default");
        assert_eq!(language, EN);
        assert_eq!(pack.studied(), EN);
    }

    #[test]
    fn another_language_is_added_and_named_by_its_tag() {
        let mut set = PackSet::new(english());
        assert_eq!(set.add(spanish()), Ok(ES));
        assert_eq!(set.languages(), [EN, ES]);
        assert_eq!(set.resolve(Some("es")).map(|(l, _)| l), Ok(ES));
        assert_eq!(
            set.resolve(None).map(|(l, _)| l),
            Ok(EN),
            "the default stays"
        );
        assert!(set.get(ES).is_some());
        let held: Vec<StudiedLanguage> = set.iter().map(|(l, _)| l).collect();
        assert_eq!(held, [EN, ES]);
    }

    #[test]
    fn the_default_comes_first_whatever_the_language_order() {
        let mut set = PackSet::new(spanish());
        assert_eq!(set.add(english()), Ok(EN));
        assert_eq!(set.default_language(), ES);
        assert_eq!(set.languages(), [ES, EN]);
    }

    #[test]
    fn a_second_pack_for_a_held_language_is_refused() {
        let mut set = PackSet::new(english());
        assert_eq!(set.add(english()), Err(PackSetError::AlreadyHeld(EN)));
        assert_eq!(set.languages(), [EN], "the set is left as it was");
        assert_eq!(
            PackSetError::AlreadyHeld(EN).to_string(),
            "a pack for \"en\" is already loaded"
        );
    }

    #[test]
    fn a_language_without_a_pack_and_an_unknown_tag_are_refused() {
        let set = PackSet::new(english());
        let not_held = set.resolve(Some("es")).map(|(l, _)| l);
        assert_eq!(not_held, Err(PackSetError::NotHeld(ES)));
        assert_eq!(
            PackSetError::NotHeld(ES).to_string(),
            "no pack for \"es\" is loaded"
        );
        let unknown = set.resolve(Some("pt")).map(|(l, _)| l);
        assert_eq!(unknown, Err(PackSetError::UnknownLanguage("pt".into())));
        assert!(
            PackSetError::UnknownLanguage("pt".into())
                .to_string()
                .contains("\"pt\"")
        );
        assert!(set.get(ES).is_none());
    }
}
