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

//! Word grammar in a pack (`add-lingua-word-grammar`): the tag vocabulary and
//! the byte layouts of the three additive sections that carry it.
//!
//! A **tag** is a Universal Dependencies part of speech, optionally followed
//! by features in UD's `FEATS` notation, sorted by name as UD sorts them:
//! `VERB|Mood=Ind|Tense=Past|VerbForm=Fin`. The vocabulary is closed — the 17
//! UD parts of speech and the features of [`FEATURES`] — and chosen for the
//! Romance packs to come, not for English alone (design D1). A pack stores
//! each distinct tag once, as text, so the container never enumerates a
//! feature: naming a new one is a change to [`FEATURES`] and to the labels,
//! never to the layout.
//!
//! The builder parses strictly and fails on anything outside the vocabulary;
//! the reader parses leniently and skips a feature it does not know, so a pack
//! from a later vocabulary still loads and says what it can.

use std::collections::BTreeMap;

use serde::Serialize;

/// The Universal Dependencies parts of speech (UPOS), in UD's order.
pub const PARTS_OF_SPEECH: [&str; 17] = [
    "ADJ", "ADP", "ADV", "AUX", "CCONJ", "DET", "INTJ", "NOUN", "NUM", "PART", "PRON", "PROPN",
    "PUNCT", "SCONJ", "SYM", "VERB", "X",
];

/// The UD morphological features the vocabulary names, each with its values,
/// sorted by feature name (byte order, which is UD's order for these names).
pub const FEATURES: &[(&str, &[&str])] = &[
    ("Case", &["Acc", "Com", "Dat", "Nom"]),
    ("Definite", &["Def", "Ind"]),
    ("Degree", &["Cmp", "Pos", "Sup"]),
    ("Gender", &["Com", "Fem", "Masc", "Neut"]),
    ("Mood", &["Cnd", "Imp", "Ind", "Sub"]),
    ("Number", &["Plur", "Sing"]),
    ("Person", &["1", "2", "3"]),
    ("PronType", &["Art", "Dem", "Ind", "Int", "Prs", "Rel"]),
    ("Reflex", &["Yes"]),
    ("Tense", &["Fut", "Imp", "Past", "Pqp", "Pres"]),
    ("VerbForm", &["Fin", "Ger", "Inf", "Part"]),
];

/// A part of speech and its features, drawn from the closed vocabulary.
///
/// Serialised as `{"pos": "VERB", "features": {"Tense": "Past", …}}` from an
/// ordered map, so its JSON is deterministic; `features` is left out when
/// empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Tag {
    /// The UD part of speech.
    pub pos: &'static str,
    /// The UD features, by name.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub features: BTreeMap<&'static str, &'static str>,
}

/// Why a tag is outside the vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagError {
    /// The text is empty.
    Empty,
    /// The part of speech is not one of UD's seventeen.
    UnknownPos(String),
    /// The feature name is not in [`FEATURES`].
    UnknownFeature(String),
    /// The feature is known but this value is not.
    UnknownValue { feature: String, value: String },
    /// A feature is not written `Name=Value`, or appears twice.
    Malformed(String),
}

impl std::fmt::Display for TagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagError::Empty => write!(f, "empty tag"),
            TagError::UnknownPos(p) => write!(f, "unknown part of speech {p:?}"),
            TagError::UnknownFeature(n) => write!(f, "unknown feature {n:?}"),
            TagError::UnknownValue { feature, value } => {
                write!(f, "unknown value {value:?} for feature {feature:?}")
            }
            TagError::Malformed(s) => write!(f, "malformed feature {s:?}"),
        }
    }
}

impl std::error::Error for TagError {}

fn known_pos(text: &str) -> Option<&'static str> {
    PARTS_OF_SPEECH.iter().copied().find(|p| *p == text)
}

fn known_feature(name: &str, value: &str) -> Result<(&'static str, &'static str), TagError> {
    let (feature, values) = FEATURES
        .iter()
        .find(|(n, _)| *n == name)
        .ok_or_else(|| TagError::UnknownFeature(name.to_owned()))?;
    let value = values
        .iter()
        .copied()
        .find(|v| *v == value)
        .ok_or_else(|| TagError::UnknownValue {
            feature: name.to_owned(),
            value: value.to_owned(),
        })?;
    Ok((feature, value))
}

impl Tag {
    /// A tag with no features.
    pub fn pos_only(pos: &str) -> Result<Tag, TagError> {
        Tag::parse_strict(pos)
    }

    /// Parses a tag, failing on anything outside the vocabulary. What the
    /// builder uses: a typo in a reduced table never reaches a pack.
    pub fn parse_strict(text: &str) -> Result<Tag, TagError> {
        Tag::parse(text, true)
    }

    /// Parses a tag, skipping the features the vocabulary does not know. What
    /// the reader uses. `None` only when the part of speech itself is unknown
    /// or the text is not a tag at all.
    pub fn parse_lenient(text: &str) -> Option<Tag> {
        Tag::parse(text, false).ok()
    }

    fn parse(text: &str, strict: bool) -> Result<Tag, TagError> {
        let mut fields = text.split('|');
        let pos_text = fields.next().unwrap_or_default().trim();
        if pos_text.is_empty() {
            return Err(TagError::Empty);
        }
        let pos = known_pos(pos_text).ok_or_else(|| TagError::UnknownPos(pos_text.to_owned()))?;
        let mut features = BTreeMap::new();
        for field in fields {
            let Some((name, value)) = field.split_once('=') else {
                if strict {
                    return Err(TagError::Malformed(field.to_owned()));
                }
                continue;
            };
            match known_feature(name, value) {
                Ok((name, value)) => {
                    if features.insert(name, value).is_some() && strict {
                        return Err(TagError::Malformed(field.to_owned()));
                    }
                }
                Err(e) if strict => return Err(e),
                Err(_) => {}
            }
        }
        Ok(Tag { pos, features })
    }

    /// The tag in UD notation, features sorted by name: the one spelling a
    /// pack stores.
    pub fn to_ud(&self) -> String {
        let mut out = String::from(self.pos);
        for (name, value) in &self.features {
            out.push('|');
            out.push_str(name);
            out.push('=');
            out.push_str(value);
        }
        out
    }

    /// Whether the tag carries `feature=value`.
    pub fn has(&self, feature: &str, value: &str) -> bool {
        self.features.get(feature) == Some(&value)
    }
}

/// A written form, stored as an edit of its dictionary form: the characters
/// (Unicode scalar values) to strip from the end, and the suffix to append.
/// `go` → `went` strips 2 and appends `went`; `hablar` → `hablábamos` strips 2
/// and appends `ábamos`, so a conjugation class shares its suffixes and zstd
/// compresses them away.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FormEdit {
    /// Characters removed from the end of the dictionary form.
    pub strip: u8,
    /// Text appended after stripping.
    pub suffix: String,
}

impl FormEdit {
    /// The edit turning `base` into `form`, or `None` when it does not fit the
    /// layout (more than 255 characters to strip or 255 bytes to append).
    pub fn between(base: &str, form: &str) -> Option<FormEdit> {
        let common = base
            .chars()
            .zip(form.chars())
            .take_while(|(a, b)| a == b)
            .count();
        let strip = u8::try_from(base.chars().count() - common).ok()?;
        let suffix: String = form.chars().skip(common).collect();
        if suffix.len() > u8::MAX as usize {
            return None;
        }
        Some(FormEdit { strip, suffix })
    }

    /// The form this edit makes of `base`.
    pub fn apply(&self, base: &str) -> String {
        let keep = base.chars().count().saturating_sub(self.strip as usize);
        let mut out: String = base.chars().take(keep).collect();
        out.push_str(&self.suffix);
        out
    }
}

/// One entry of a dictionary form's paradigm.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParadigmEntry {
    /// A form of this dictionary form, and what it is.
    Reading {
        /// The form, as an edit of the dictionary form.
        form: FormEdit,
        /// The tag's index in the pack's tag pool.
        tag: u16,
    },
    /// A form the analysis reads as this dictionary form which is also a
    /// reading of another one (`leaves`, filed under `leave`, of `leaf`).
    Also {
        /// The form, as an edit of this dictionary form.
        form: FormEdit,
        /// The other dictionary form's lemma id.
        other: u32,
    },
}

const READING: u8 = 0;
const ALSO: u8 = 1;

/// Encodes a paradigm: per entry `kind u8 | strip u8 | len u8 | suffix |`
/// then `tag u16` (a reading) or `lemma id u32` (another dictionary form),
/// little-endian.
pub fn encode_paradigm(entries: &[ParadigmEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    for entry in entries {
        let (kind, form) = match entry {
            ParadigmEntry::Reading { form, .. } => (READING, form),
            ParadigmEntry::Also { form, .. } => (ALSO, form),
        };
        out.push(kind);
        out.push(form.strip);
        out.push(form.suffix.len() as u8);
        out.extend_from_slice(form.suffix.as_bytes());
        match entry {
            ParadigmEntry::Reading { tag, .. } => out.extend_from_slice(&tag.to_le_bytes()),
            ParadigmEntry::Also { other, .. } => out.extend_from_slice(&other.to_le_bytes()),
        }
    }
    out
}

/// Decodes a paradigm written by [`encode_paradigm`]; `None` when the bytes
/// are not one.
pub fn decode_paradigm(bytes: &[u8]) -> Option<Vec<ParadigmEntry>> {
    let mut out = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        let kind = *bytes.get(at)?;
        let strip = *bytes.get(at + 1)?;
        let len = *bytes.get(at + 2)? as usize;
        let suffix_end = at + 3 + len;
        let suffix = std::str::from_utf8(bytes.get(at + 3..suffix_end)?)
            .ok()?
            .to_owned();
        let form = FormEdit { strip, suffix };
        match kind {
            READING => {
                let raw: [u8; 2] = bytes.get(suffix_end..suffix_end + 2)?.try_into().ok()?;
                out.push(ParadigmEntry::Reading {
                    form,
                    tag: u16::from_le_bytes(raw),
                });
                at = suffix_end + 2;
            }
            ALSO => {
                let raw: [u8; 4] = bytes.get(suffix_end..suffix_end + 4)?.try_into().ok()?;
                out.push(ParadigmEntry::Also {
                    form,
                    other: u32::from_le_bytes(raw),
                });
                at = suffix_end + 4;
            }
            _ => return None,
        }
    }
    Some(out)
}

/// One run of a gloss: `count` consecutive senses sharing a tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SenseRun {
    /// The tag's index in the pack's tag pool.
    pub tag: u16,
    /// How many senses of the gloss the run covers.
    pub count: u8,
}

/// Encodes a gloss's runs: `tag u16 LE | count u8` each, in gloss order.
pub fn encode_runs(runs: &[SenseRun]) -> Vec<u8> {
    let mut out = Vec::with_capacity(runs.len() * 3);
    for run in runs {
        out.extend_from_slice(&run.tag.to_le_bytes());
        out.push(run.count);
    }
    out
}

/// Decodes runs written by [`encode_runs`]; `None` when the bytes are not
/// whole runs.
pub fn decode_runs(bytes: &[u8]) -> Option<Vec<SenseRun>> {
    let (runs, rest) = bytes.as_chunks::<3>();
    if !rest.is_empty() {
        return None;
    }
    Some(
        runs.iter()
            .map(|[lo, hi, count]| SenseRun {
                tag: u16::from_le_bytes([*lo, *hi]),
                count: *count,
            })
            .collect(),
    )
}

/// Encodes an id-indexed blob, the layout `gloss.zst` uses once decompressed:
/// `count u32 | count*(id u32, off u32, len u32) | payload`, little-endian,
/// offsets relative to the payload. `entries` must be sorted by id.
pub fn encode_indexed(entries: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut payload = Vec::new();
    let mut out = Vec::new();
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (id, bytes) in entries {
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        payload.extend_from_slice(bytes);
    }
    out.extend_from_slice(&payload);
    out
}

/// A decompressed id-indexed blob, kept as bytes: entries are decoded when
/// looked up, never all at load (the analyser runs in every tab's content
/// script on Chromium).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedBlob {
    raw: Vec<u8>,
    payload: usize,
    /// `(id, offset, length)`, strictly increasing ids.
    index: Vec<(u32, u32, u32)>,
}

impl IndexedBlob {
    /// Reads the index of a blob written by [`encode_indexed`], checking every
    /// entry against the payload; `None` when the bytes are not one.
    pub fn parse(raw: Vec<u8>) -> Option<IndexedBlob> {
        let word = |at: usize| -> Option<u32> {
            Some(u32::from_le_bytes(raw.get(at..at + 4)?.try_into().ok()?))
        };
        let count = word(0)? as usize;
        let payload = count.checked_mul(12)?.checked_add(4)?;
        if payload > raw.len() {
            return None;
        }
        let mut index = Vec::with_capacity(count);
        for i in 0..count {
            let at = 4 + i * 12;
            let (id, off, len) = (word(at)?, word(at + 4)?, word(at + 8)?);
            let end = payload
                .checked_add(off as usize)?
                .checked_add(len as usize)?;
            if end > raw.len() || index.last().is_some_and(|(prev, _, _)| *prev >= id) {
                return None;
            }
            index.push((id, off, len));
        }
        Some(IndexedBlob {
            raw,
            payload,
            index,
        })
    }

    /// The bytes filed under `id`.
    pub fn get(&self, id: u32) -> Option<&[u8]> {
        let at = self.index.binary_search_by_key(&id, |(i, _, _)| *i).ok()?;
        let (_, off, len) = self.index[at];
        let start = self.payload + off as usize;
        self.raw.get(start..start + len as usize)
    }

    /// Whether the blob files nothing.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}

/// Encodes the tag pool: one tag per line, id = line index.
pub fn encode_tag_pool(tags: &[String]) -> Vec<u8> {
    tags.join("\n").into_bytes()
}

/// Decodes the tag pool leniently: a line the vocabulary cannot read at all
/// is `None`, so the entries pointing at it are skipped rather than the pack
/// refused.
pub fn decode_tag_pool(bytes: &[u8]) -> Option<Vec<Option<Tag>>> {
    let text = std::str::from_utf8(bytes).ok()?;
    if text.is_empty() {
        return Some(Vec::new());
    }
    Some(text.split('\n').map(Tag::parse_lenient).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_part_of_speech_and_feature_value_round_trips() {
        for pos in PARTS_OF_SPEECH {
            assert_eq!(Tag::parse_strict(pos).unwrap().to_ud(), pos);
        }
        for (name, values) in FEATURES {
            for value in *values {
                let text = format!("NOUN|{name}={value}");
                let tag = Tag::parse_strict(&text).unwrap();
                assert_eq!(tag.to_ud(), text);
                assert!(tag.has(name, value));
            }
        }
    }

    #[test]
    fn the_vocabulary_is_sorted_as_ud_sorts_it() {
        let names: Vec<&str> = FEATURES.iter().map(|(n, _)| *n).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        let mut pos = PARTS_OF_SPEECH.to_vec();
        pos.sort_unstable();
        assert_eq!(pos, PARTS_OF_SPEECH);
    }

    #[test]
    fn features_print_sorted_by_name_whatever_order_they_came_in() {
        let tag = Tag::parse_strict("VERB|VerbForm=Fin|Tense=Past|Mood=Ind").unwrap();
        assert_eq!(tag.to_ud(), "VERB|Mood=Ind|Tense=Past|VerbForm=Fin");
    }

    #[test]
    fn strict_parsing_names_what_is_outside_the_vocabulary() {
        assert_eq!(
            Tag::parse_strict("VRB"),
            Err(TagError::UnknownPos("VRB".into()))
        );
        assert_eq!(
            Tag::parse_strict("VERB|Polite=Form"),
            Err(TagError::UnknownFeature("Polite".into()))
        );
        assert_eq!(
            Tag::parse_strict("VERB|Tense=Aor"),
            Err(TagError::UnknownValue {
                feature: "Tense".into(),
                value: "Aor".into()
            })
        );
        assert_eq!(
            Tag::parse_strict("VERB|Tense"),
            Err(TagError::Malformed("Tense".into()))
        );
        assert_eq!(
            Tag::parse_strict("VERB|Tense=Past|Tense=Pres"),
            Err(TagError::Malformed("Tense=Pres".into()))
        );
        assert_eq!(Tag::parse_strict(""), Err(TagError::Empty));
        assert!(
            TagError::UnknownFeature("Polite".into())
                .to_string()
                .contains("Polite")
        );
    }

    #[test]
    fn lenient_parsing_skips_an_unknown_feature_and_keeps_the_rest() {
        let tag = Tag::parse_lenient("PRON|Person=2|Polite=Form|Tense").unwrap();
        assert_eq!(tag.to_ud(), "PRON|Person=2");
        assert_eq!(Tag::parse_lenient("WORD"), None);
    }

    #[test]
    fn the_romance_categories_fit_the_vocabulary() {
        for text in [
            "VERB|Mood=Ind|Tense=Past|VerbForm=Fin",
            "VERB|Mood=Ind|Tense=Imp|VerbForm=Fin",
            "VERB|Mood=Sub|Tense=Pres|VerbForm=Fin",
            "VERB|Mood=Sub|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin",
            "VERB|Mood=Sub|Tense=Fut|VerbForm=Fin",
            "VERB|Mood=Ind|Tense=Pqp|VerbForm=Fin",
            "VERB|Mood=Cnd|VerbForm=Fin",
            "VERB|Mood=Imp|VerbForm=Fin",
            "VERB|Number=Plur|Person=1|VerbForm=Inf",
            "VERB|VerbForm=Ger",
            "VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part",
            "NOUN|Gender=Fem",
            "PRON|Case=Dat|Number=Sing|Person=1|PronType=Prs",
            "PRON|Case=Acc|Person=3|PronType=Prs|Reflex=Yes",
            "PRON|Case=Com|Number=Sing|Person=1|PronType=Prs",
            "DET|Definite=Def|PronType=Art",
        ] {
            assert_eq!(Tag::parse_strict(text).unwrap().to_ud(), text);
        }
    }

    #[test]
    fn a_form_edit_rebuilds_the_form_from_its_dictionary_form() {
        for (base, form) in [
            ("go", "went"),
            ("walk", "walked"),
            ("put", "put"),
            ("hablar", "hablábamos"),
            ("mouse", "mice"),
            ("decir", "dijéramos"),
        ] {
            let edit = FormEdit::between(base, form).unwrap();
            assert_eq!(edit.apply(base), form, "{base} → {form}");
        }
        assert_eq!(
            FormEdit::between("hablar", "hablábamos").unwrap(),
            FormEdit {
                strip: 2,
                suffix: "ábamos".into()
            }
        );
        assert_eq!(FormEdit::between("go", &"x".repeat(300)), None);
    }

    #[test]
    fn a_paradigm_round_trips_and_malformed_bytes_are_refused() {
        let entries = vec![
            ParadigmEntry::Reading {
                form: FormEdit::between("leave", "left").unwrap(),
                tag: 3,
            },
            ParadigmEntry::Also {
                form: FormEdit::between("leave", "leaves").unwrap(),
                other: 70_000,
            },
        ];
        let bytes = encode_paradigm(&entries);
        assert_eq!(decode_paradigm(&bytes), Some(entries));
        assert_eq!(decode_paradigm(&bytes[..bytes.len() - 1]), None);
        assert_eq!(decode_paradigm(&[9, 0, 0]), None);
        assert_eq!(decode_paradigm(&[0, 0, 2, 0xff, 0xfe, 1, 0]), None);
    }

    #[test]
    fn runs_round_trip_and_a_partial_run_is_refused() {
        let runs = vec![
            SenseRun { tag: 1, count: 1 },
            SenseRun { tag: 513, count: 2 },
        ];
        assert_eq!(decode_runs(&encode_runs(&runs)), Some(runs));
        assert_eq!(decode_runs(&[1, 0]), None);
    }

    #[test]
    fn an_indexed_blob_finds_its_entries_and_checks_its_bounds() {
        let raw = encode_indexed(&[(2, b"ab".to_vec()), (7, b"cde".to_vec())]);
        let blob = IndexedBlob::parse(raw.clone()).unwrap();
        assert_eq!(blob.get(2), Some(&b"ab"[..]));
        assert_eq!(blob.get(7), Some(&b"cde"[..]));
        assert_eq!(blob.get(3), None);
        assert!(!blob.is_empty());
        assert_eq!(IndexedBlob::parse(raw[..raw.len() - 1].to_vec()), None);
        assert_eq!(IndexedBlob::parse(vec![1, 0]), None);
        // Ids out of order are not an index.
        let unsorted = encode_indexed(&[(7, b"a".to_vec()), (2, b"b".to_vec())]);
        assert_eq!(IndexedBlob::parse(unsorted), None);
        assert!(IndexedBlob::parse(encode_indexed(&[])).unwrap().is_empty());
    }

    #[test]
    fn the_tag_pool_reads_leniently() {
        let pool = encode_tag_pool(&["NOUN|Number=Plur".into(), "WORD".into(), "VERB".into()]);
        let tags = decode_tag_pool(&pool).unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].as_ref().unwrap().to_ud(), "NOUN|Number=Plur");
        assert_eq!(tags[1], None);
        assert_eq!(decode_tag_pool(b""), Some(Vec::new()));
        assert_eq!(decode_tag_pool(&[0xff]), None);
    }

    #[test]
    fn a_tag_serialises_its_features_only_when_it_has_some() {
        let tag = Tag::parse_strict("VERB|Tense=Past|VerbForm=Part").unwrap();
        assert_eq!(
            serde_json::to_string(&tag).unwrap(),
            r#"{"pos":"VERB","features":{"Tense":"Past","VerbForm":"Part"}}"#
        );
        assert_eq!(
            serde_json::to_string(&Tag::pos_only("NOUN").unwrap()).unwrap(),
            r#"{"pos":"NOUN"}"#
        );
    }
}
