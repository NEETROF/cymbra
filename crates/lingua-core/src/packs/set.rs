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
//! language, which a call that names none is answered in.

use std::collections::BTreeMap;

use crate::analysis::language::StudiedLanguage;

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
        }
    }
}

impl std::error::Error for PackSetError {}

/// The packs an engine holds, at most one per studied language.
pub struct PackSet {
    default: StudiedLanguage,
    packs: BTreeMap<StudiedLanguage, Pack>,
}

impl PackSet {
    /// A set holding `pack`, whose language becomes the default.
    pub fn new(pack: Pack) -> Self {
        let default = pack.studied();
        let mut packs = BTreeMap::new();
        packs.insert(default, pack);
        PackSet { default, packs }
    }

    /// Adds another language's pack, returning its language. A second pack for a
    /// language already held is refused, and the set is left as it was.
    pub fn add(&mut self, pack: Pack) -> Result<StudiedLanguage, PackSetError> {
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
    use crate::packs::pack::tests::sample_pack_bytes_for;

    const EN: StudiedLanguage = StudiedLanguage::English;
    const ES: StudiedLanguage = StudiedLanguage::Spanish;

    fn english() -> Pack {
        Pack::load(&sample_pack_bytes_for("en", ANALYZER_VERSION)).expect("en pack")
    }

    fn spanish() -> Pack {
        Pack::load(&sample_pack_bytes_for("es", SPANISH_ANALYZER_VERSION)).expect("es pack")
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
