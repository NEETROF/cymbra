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

//! A pack's dictionary words, when they are not the lemmas it glosses
//! (add-lingua-pack-lexical-layer D1, D3).

use std::collections::BTreeSet;

use lingua_core::analysis::lexicon::FstLexicon;

use crate::{BuildError, PackInputs};

/// The lexical table (add-lingua-pack-lexical-layer D1, D3): one bit per lemma id, least
/// significant bit first, set for each of the pack's dictionary words.
///
/// `None` when the inputs name no dictionary words, or name exactly the lemmas the pack
/// glosses — `glossed`, by lemma id — so that a pack whose glosses are its studied
/// language's reference keeps the bytes it had without the table. A dictionary word is
/// placed at the id its gloss would be filed under, which is the id the reader looks it up
/// by. When the table is written, the lexicon must be the studied language's alone: a
/// dictionary word or a glossed lemma that is neither the lemma of a form nor a ranked lemma
/// is refused by name, so no native language's glosses can add a lemma.
pub(crate) fn lexical_table(
    inputs: &PackInputs,
    lex: &FstLexicon<&[u8]>,
    glossed: &[(u32, &str)],
) -> Result<Option<Vec<u8>>, BuildError> {
    let Some(words) = &inputs.lexical else {
        return Ok(None);
    };
    let set = |bits: &mut [u8], id: u64| bits[(id / 8) as usize] |= 1 << (id % 8);
    let mut glossed_bits = vec![0u8; lex.lemma_count().div_ceil(8)];
    for (id, _) in glossed {
        set(&mut glossed_bits, u64::from(*id));
    }
    let mut bits = vec![0u8; glossed_bits.len()];
    let mut placed = true;
    for word in words {
        match lex.id_of(word) {
            Some(id) => set(&mut bits, id),
            None => placed = false,
        }
    }
    if placed && bits == glossed_bits {
        return Ok(None);
    }
    let studied: BTreeSet<&str> = inputs
        .form_lemma
        .iter()
        .map(|(_, lemma)| lemma.as_str())
        .chain(inputs.ranks.iter().map(|(lemma, _)| lemma.as_str()))
        .collect();
    if let Some(word) = words.iter().find(|word| !studied.contains(word.as_str())) {
        return Err(BuildError::Lexical(format!(
            "dictionary word {word:?} is neither the lemma of a form nor a ranked lemma"
        )));
    }
    if let Some((lemma, _)) = inputs
        .glosses
        .iter()
        .find(|(lemma, _)| !studied.contains(lemma.as_str()))
    {
        return Err(BuildError::Lexical(format!(
            "{lemma:?} is glossed but is neither the lemma of a form nor a ranked lemma"
        )));
    }
    Ok(Some(bits))
}
