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

//! The tags a pack's grammar stores, whatever native language it is glossed in: the
//! studied language's pinned tag pool (add-lingua-pack-lexical-layer D4) and a noun run's
//! gender, read from the noun's readings (D5).

use std::collections::{BTreeMap, BTreeSet};

use lingua_core::packs::grammar::Tag;

use crate::{BuildError, PackInputs};

/// The tag pool the grammar blobs index into (add-lingua-pack-lexical-layer D4).
///
/// Without a pin, every tag in one sorted pool, as before. With one — the studied
/// language's `tags.tsv` — the pin in its own order, then the readings' tags it does not
/// hold, then the tags only senses carry, each part sorted: a form's readings are then
/// stored alike whatever tags a native language's senses use. A pinned line that is not a
/// canonical tag, or that repeats one, fails the build by name.
pub(crate) fn tag_pool(
    pin: Option<&[String]>,
    reading_tags: BTreeSet<String>,
    sense_tags: BTreeSet<String>,
) -> Result<Vec<String>, BuildError> {
    let Some(pin) = pin else {
        let mut all = reading_tags;
        all.extend(sense_tags);
        return Ok(all.into_iter().collect());
    };
    let mut pinned = BTreeSet::new();
    for line in pin {
        let canonical = Tag::parse_strict(line)
            .map(|tag| tag.to_ud())
            .map_err(|e| BuildError::Grammar(format!("pinned tag {line:?}: {e}")))?;
        if canonical != *line {
            return Err(BuildError::Grammar(format!(
                "pinned tag {line:?} is not written as {canonical:?}"
            )));
        }
        if !pinned.insert(line.as_str()) {
            return Err(BuildError::Grammar(format!(
                "pinned tag {line:?} is listed twice"
            )));
        }
    }
    let readings: Vec<String> = reading_tags
        .iter()
        .filter(|tag| !pinned.contains(tag.as_str()))
        .cloned()
        .collect();
    let senses: Vec<String> = sense_tags
        .into_iter()
        .filter(|tag| !pinned.contains(tag.as_str()) && !reading_tags.contains(tag))
        .collect();
    Ok(pin.iter().cloned().chain(readings).chain(senses).collect())
}

/// A gloss's runs as the builder files them: the dictionary form, then each run's canonical
/// tag and the senses it covers.
pub(crate) type CanonicalRuns<'a> = (&'a str, Vec<(String, u8)>);

/// The sense runs, each tag canonical, a noun's carrying the gender its dictionary form is
/// read with (add-lingua-pack-lexical-layer D5), whatever the native language's glosses say.
///
/// What the readings of the noun's dictionary form as itself say decides:
/// - exactly one gender: every noun run of its gloss carries it, and a run naming the other
///   one fails the build by name;
/// - both genders (`estudiante`): every noun run is bare, whatever gender the sense table
///   wrote, as a reference pair's reducer writes it;
/// - no gender: the runs stay as the sense table writes them. A reference pair's reducer
///   writes them bare; the archived scenario *A Romance pack fits the vocabulary* reads
///   `leche`'s feminine from its run alone.
pub(crate) fn noun_runs(inputs: &PackInputs) -> Result<Vec<CanonicalRuns<'_>>, BuildError> {
    let parse = |text: &str| {
        Tag::parse_strict(text).map_err(|e| BuildError::Grammar(format!("tag {text:?}: {e}")))
    };
    let mut genders: BTreeMap<&str, BTreeSet<&'static str>> = BTreeMap::new();
    for reading in inputs.readings.iter().filter(|r| r.form == r.lemma) {
        let tag = parse(&reading.tag)?;
        if let (NOUN, Some(gender)) = (tag.pos, tag.features.get(GENDER)) {
            genders
                .entry(reading.lemma.as_str())
                .or_default()
                .insert(*gender);
        }
    }
    let mut out = Vec::with_capacity(inputs.senses.len());
    for (lemma, runs) in &inputs.senses {
        // The genders the noun is read with: none is no entry, never an empty set.
        let read = genders.get(lemma.as_str());
        let mut tagged = Vec::with_capacity(runs.len());
        for (text, count) in runs {
            let mut tag = parse(text)?;
            if tag.pos == NOUN {
                match read {
                    None => {}
                    Some(found) if found.len() == 1 => {
                        let gender = *found.first().expect("one gender");
                        if let Some(said) = tag.features.get(GENDER)
                            && *said != gender
                        {
                            return Err(BuildError::Grammar(format!(
                                "the noun run of {lemma:?} says Gender={said}, \
                                 but its readings give it only Gender={gender}"
                            )));
                        }
                        tag.features.insert(GENDER, gender);
                    }
                    Some(_) => {
                        tag.features.remove(GENDER);
                    }
                }
            }
            tagged.push((tag.to_ud(), *count));
        }
        out.push((lemma.as_str(), tagged));
    }
    Ok(out)
}

/// The noun's part of speech and its gender feature, as the tag vocabulary spells them.
const NOUN: &str = "NOUN";
const GENDER: &str = "Gender";
