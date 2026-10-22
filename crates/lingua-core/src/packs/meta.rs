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

//! A pack's metadata: which pair it serves, its versions, its licences; and the metadata of
//! the two parts a pack is also built as, a base and a layer (split-lingua-pack-format D1).

use serde::{Deserialize, Serialize};

use super::pack::{PackError, read_part};

/// The metadata blob carried in a pack's container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackMeta {
    /// ISO-639-1 tag of the studied language (e.g. `"en"`).
    pub studied: String,
    /// ISO-639-1 tag of the native / gloss language (e.g. `"fr"`). A string, so
    /// no pack byte moves; the reader checks it at load
    /// (generalise-lingua-native-language D1).
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
    /// The pair key, `<studied>-<native>` (e.g. `"en-fr"`), as
    /// [`crate::knowledge::LanguagePair::key`] forms it.
    pub fn pair_key(&self) -> String {
        format!("{}-{}", self.studied, self.native)
    }
}

/// Which of a pack's two parts a container holds (split-lingua-pack-format D1): a *base*, a
/// studied language's sections, or a *layer*, a pair's. Written as the metadata's `part` field,
/// `"base"` or `"layer"`; a whole pack's metadata has none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Part {
    /// A studied language's sections: forms, lemmas, ranks, levels, dictionary words, the
    /// pool's studied blocks and the paradigms.
    Base,
    /// A pair's sections: glosses, their runs and the tags only those carry, expressions, and
    /// the pair's NOTICE.
    Layer,
}

impl Part {
    /// The part's name, as the metadata writes it.
    pub fn name(self) -> &'static str {
        match self {
            Part::Base => "base",
            Part::Layer => "layer",
        }
    }

    /// The part this one is assembled with.
    pub fn other(self) -> Part {
        match self {
            Part::Base => Part::Layer,
            Part::Layer => Part::Base,
        }
    }
}

impl std::fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// The `part` a metadata blob names, read before any other field: `None` for a whole pack's.
/// A part lacks fields a whole pack's metadata requires, so a reader of whole packs probes it
/// first, and refuses a part by name rather than as a missing field (D1).
pub(crate) fn probe_part(meta_json: &[u8]) -> Result<Option<Part>, serde_json::Error> {
    #[derive(Deserialize)]
    struct Probe {
        #[serde(default)]
        part: Option<Part>,
    }
    serde_json::from_slice::<Probe>(meta_json).map(|probe| probe.part)
}

/// A base's metadata (split-lingua-pack-format D1, D2, D4): the studied language whose sections
/// it holds, the analyser generation they were built for, the digest a layer names it by, and
/// whether its levels are estimated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseMeta {
    /// Always [`Part::Base`].
    pub part: Part,
    /// ISO-639-1 tag of the studied language.
    pub studied: String,
    /// The analyser generation the base was built for, checked at assembly as `load` checks a
    /// whole pack's.
    pub analyzer_version: String,
    /// The sha256, in lowercase hex, of the base container's bytes from its section count to
    /// its end (D4): no metadata, so any reader of the file can recompute it.
    pub base_version: String,
    /// Whether the base's levels are estimated from word frequency: a property of the studied
    /// language's levels (D2). Written only when true, as a whole pack's.
    #[serde(default, skip_serializing_if = "is_false")]
    pub levels_estimated: bool,
}

impl BaseMeta {
    /// A base's metadata, read from its container alone; refused when the container holds a
    /// layer or a whole pack.
    pub fn read(bytes: &[u8]) -> Result<BaseMeta, PackError> {
        read_part(bytes, Part::Base).map(|(meta, _)| meta)
    }
}

/// A layer's metadata (split-lingua-pack-format D1, D2, D4): the pair whose sections it holds,
/// its `pack_version` and licences — the whole pack's — and the `base_version` of the base it
/// was built against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerMeta {
    /// Always [`Part::Layer`].
    pub part: Part,
    /// ISO-639-1 tag of the studied language: its base's.
    pub studied: String,
    /// ISO-639-1 tag of the native language its glosses are written in.
    pub native: String,
    /// The pair's content version: the whole pack's.
    pub pack_version: String,
    /// The [`BaseMeta::base_version`] of the base it was built with; assembly refuses any other.
    pub base_version: String,
    /// The pair's full licence stack, both sides: the whole pack's.
    pub licences: Vec<String>,
}

impl LayerMeta {
    /// A layer's metadata, read from its container alone; refused when the container holds a
    /// base or a whole pack.
    pub fn read(bytes: &[u8]) -> Result<LayerMeta, PackError> {
        read_part(bytes, Part::Layer).map(|(meta, _)| meta)
    }

    /// The pair key, `<studied>-<native>`, as [`PackMeta::pair_key`] forms it.
    pub fn pair_key(&self) -> String {
        format!("{}-{}", self.studied, self.native)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::format::write_container;

    const WITHOUT: &str = r#"{"studied":"en","native":"fr","pack_version":"1","analyzer_version":"1.1.0","licences":["x"]}"#;

    #[test]
    fn a_pack_that_says_nothing_of_its_levels_reads_not_estimated_and_keeps_its_bytes() {
        let meta: PackMeta = serde_json::from_str(WITHOUT).expect("parse");
        assert!(!meta.levels_estimated);
        assert_eq!(serde_json::to_string(&meta).expect("serialise"), WITHOUT);
    }

    #[test]
    fn the_pair_key_reads_studied_then_native() {
        let meta: PackMeta = serde_json::from_str(WITHOUT).expect("parse");
        assert_eq!(meta.pair_key(), "en-fr");
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

    // — a base and a layer (split-lingua-pack-format D1) —

    fn base_meta(levels_estimated: bool) -> BaseMeta {
        BaseMeta {
            part: Part::Base,
            studied: "es".into(),
            analyzer_version: "1.0.0".into(),
            base_version: "ab".repeat(32),
            levels_estimated,
        }
    }

    fn layer_meta() -> LayerMeta {
        LayerMeta {
            part: Part::Layer,
            studied: "es".into(),
            native: "en".into(),
            pack_version: "2026.10.08".into(),
            base_version: "ab".repeat(32),
            licences: vec!["ESDB".into(), "kaikki CC BY-SA".into()],
        }
    }

    fn container(meta: &impl Serialize) -> Vec<u8> {
        write_container(&serde_json::to_vec(meta).unwrap(), &[("notice", b"x")])
    }

    #[test]
    fn a_whole_pack_s_metadata_is_unchanged_byte_for_byte() {
        // No `part` field, and every field where it was: the agent's `pack.lingua`, every pin's
        // sha256, keep their bytes.
        let meta: PackMeta = serde_json::from_str(WITHOUT).unwrap();
        assert_eq!(serde_json::to_string(&meta).unwrap(), WITHOUT);
        assert_eq!(probe_part(WITHOUT.as_bytes()).unwrap(), None);
    }

    #[test]
    fn a_base_s_metadata_round_trips_and_names_its_part_first() {
        for estimated in [false, true] {
            let meta = base_meta(estimated);
            let json = serde_json::to_string(&meta).unwrap();
            assert!(
                json.starts_with(r#"{"part":"base","studied":"es""#),
                "{json}"
            );
            assert_eq!(json.contains("levels_estimated"), estimated, "{json}");
            assert_eq!(serde_json::from_str::<BaseMeta>(&json).unwrap(), meta);
            assert_eq!(probe_part(json.as_bytes()).unwrap(), Some(Part::Base));
            assert_eq!(BaseMeta::read(&container(&meta)).unwrap(), meta);
        }
    }

    #[test]
    fn a_layer_s_metadata_round_trips_and_names_its_pair() {
        let meta = layer_meta();
        let json = serde_json::to_string(&meta).unwrap();
        assert!(
            json.starts_with(r#"{"part":"layer","studied":"es""#),
            "{json}"
        );
        assert_eq!(serde_json::from_str::<LayerMeta>(&json).unwrap(), meta);
        assert_eq!(probe_part(json.as_bytes()).unwrap(), Some(Part::Layer));
        assert_eq!(LayerMeta::read(&container(&meta)).unwrap(), meta);
        assert_eq!(meta.pair_key(), "es-en");
    }

    #[test]
    fn each_part_reader_refuses_the_other_part_and_a_whole_pack() {
        let whole = container(&serde_json::from_str::<PackMeta>(WITHOUT).unwrap());
        let (base, layer) = (container(&base_meta(false)), container(&layer_meta()));
        for (bytes, found) in [(&layer, Some(Part::Layer)), (&whole, None)] {
            let err = BaseMeta::read(bytes).expect_err("not a base");
            assert!(err.to_string().contains("expected a pack's base"), "{err}");
            assert!(matches!(
                err,
                PackError::WrongPart { expected: Part::Base, found: f } if f == found
            ));
        }
        for (bytes, found) in [(&base, Some(Part::Base)), (&whole, None)] {
            assert!(matches!(
                LayerMeta::read(bytes),
                Err(PackError::WrongPart { expected: Part::Layer, found: f }) if f == found
            ));
        }
        assert!(matches!(
            BaseMeta::read(b"not a pack"),
            Err(PackError::Format(_))
        ));
        let unparsable = write_container(b"{not json", &[]);
        assert!(matches!(
            LayerMeta::read(&unparsable),
            Err(PackError::BadMeta(_))
        ));
        // A part that lacks one of its own fields is refused as bad metadata.
        let short = write_container(br#"{"part":"base","studied":"es"}"#, &[]);
        assert!(matches!(BaseMeta::read(&short), Err(PackError::BadMeta(_))));
    }

    #[test]
    fn a_part_the_core_does_not_know_is_bad_metadata() {
        assert!(probe_part(br#"{"part":"core","studied":"en"}"#).is_err());
        assert_eq!(Part::Base.other(), Part::Layer);
        assert_eq!(Part::Layer.other(), Part::Base);
        assert_eq!(format!("{} {}", Part::Base, Part::Layer), "base layer");
    }
}
