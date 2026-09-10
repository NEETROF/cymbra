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

//! Cascading lemmatisation (design D3).
//!
//! For each token: exception table (irregulars) → the pack's forms→lemmas FST
//! → morphy-style rule candidates validated against the lexicon →
//! out-of-lexicon regular-plural fallback → the lowercased form itself.
//!
//! The plural fallback is the preshot's hardest-won lesson: without it,
//! `endeavor` and `endeavors` (both outside a frequency lexicon) count as two
//! distinct words and the "honest count" lies in the other direction.
//! Ambiguity is NOT resolved here (one form, one lemma, deterministically);
//! the knowledge model resolves multi-candidate knowledge in the learner's
//! favour on its side.

use super::lexicon::Lexicon;

/// English irregulars whose lemma no suffix rule can reach. The slice MUST
/// stay sorted by form: lookups binary-search it directly (a test enforces
/// the ordering).
const IRREGULARS: &[(&str, &str)] = &[
    ("'d", "would"),
    ("'ll", "will"),
    ("'m", "be"),
    ("'re", "be"),
    ("'s", "be"),
    ("'ve", "have"),
    ("am", "be"),
    ("are", "be"),
    ("ate", "eat"),
    ("been", "be"),
    ("began", "begin"),
    ("begun", "begin"),
    ("being", "be"),
    ("best", "good"),
    ("better", "good"),
    ("bought", "buy"),
    ("broke", "break"),
    ("broken", "break"),
    ("brought", "bring"),
    ("built", "build"),
    ("came", "come"),
    ("caught", "catch"),
    ("children", "child"),
    ("chose", "choose"),
    ("chosen", "choose"),
    ("did", "do"),
    ("does", "do"),
    ("done", "do"),
    ("drawn", "draw"),
    ("drew", "draw"),
    ("driven", "drive"),
    ("drove", "drive"),
    ("eaten", "eat"),
    ("fallen", "fall"),
    ("feet", "foot"),
    ("fell", "fall"),
    ("felt", "feel"),
    ("flew", "fly"),
    ("flown", "fly"),
    ("fought", "fight"),
    ("found", "find"),
    ("further", "far"),
    ("furthest", "far"),
    ("gave", "give"),
    ("geese", "goose"),
    ("given", "give"),
    ("goes", "go"),
    ("gone", "go"),
    ("got", "get"),
    ("gotten", "get"),
    ("grew", "grow"),
    ("grown", "grow"),
    ("had", "have"),
    ("has", "have"),
    ("heard", "hear"),
    ("held", "hold"),
    ("is", "be"),
    ("kept", "keep"),
    ("knew", "know"),
    ("known", "know"),
    ("lain", "lie"),
    ("least", "little"),
    ("led", "lead"),
    ("left", "leave"),
    ("less", "little"),
    ("lost", "lose"),
    ("made", "make"),
    ("meant", "mean"),
    ("men", "man"),
    ("met", "meet"),
    ("mice", "mouse"),
    ("more", "many"),
    ("most", "many"),
    ("paid", "pay"),
    ("people", "person"),
    ("ran", "run"),
    ("rose", "rise"),
    ("said", "say"),
    ("sat", "sit"),
    ("saw", "see"),
    ("seen", "see"),
    ("sent", "send"),
    ("shot", "shoot"),
    ("slept", "sleep"),
    ("sold", "sell"),
    ("sought", "seek"),
    ("spent", "spend"),
    ("spoke", "speak"),
    ("spoken", "speak"),
    ("stood", "stand"),
    ("taken", "take"),
    ("taught", "teach"),
    ("teeth", "tooth"),
    ("thought", "think"),
    ("threw", "throw"),
    ("thrown", "throw"),
    ("told", "tell"),
    ("took", "take"),
    ("understood", "understand"),
    ("was", "be"),
    ("went", "go"),
    ("were", "be"),
    ("women", "woman"),
    ("won", "win"),
    ("wore", "wear"),
    ("worn", "wear"),
    ("worse", "bad"),
    ("worst", "bad"),
    ("written", "write"),
    ("wrote", "write"),
];

/// Lemmatises one token; the result is always lowercase.
pub fn lemmatize(form: &str, lexicon: &(impl Lexicon + ?Sized)) -> String {
    let lower = form.replace('\u{2019}', "'").to_lowercase();

    if let Ok(idx) = IRREGULARS.binary_search_by_key(&lower.as_str(), |(f, _)| f) {
        return IRREGULARS[idx].1.to_owned();
    }
    if let Some(lemma) = lexicon.lemma_of(&lower) {
        return lemma.to_owned();
    }
    if let Some(lemma) = best_rule_candidate(&lower, lexicon) {
        return lemma;
    }
    if let Some(lemma) = out_of_lexicon_plural(&lower) {
        return lemma;
    }
    lower
}

/// Morphy-style suffix rules: generate candidates from most to least
/// specific, keep the first that is a lemma of the lexicon. Order is the
/// determinism contract — never reorder without bumping the analyzer
/// version.
fn best_rule_candidate(lower: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    rule_candidates(lower)
        .into_iter()
        .find(|candidate| lexicon.contains_lemma(candidate))
}

fn rule_candidates(w: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |c: String| {
        if c.len() > 1 && !out.contains(&c) {
            out.push(c);
        }
    };
    let n = w.len();

    // Nominal / verbal -s family.
    if let Some(stem) = w.strip_suffix("ies").filter(|_| n > 4) {
        push(format!("{stem}y"));
    }
    if let Some(stem) = w.strip_suffix("ves").filter(|_| n > 4) {
        push(format!("{stem}f"));
        push(format!("{stem}fe"));
    }
    if let Some(stem) = w.strip_suffix("es").filter(|_| n > 3) {
        push(stem.to_owned());
        push(format!("{stem}e"));
    }
    if n > 3 && w.ends_with('s') && !w.ends_with("ss") && !w.ends_with("us") && !w.ends_with("is") {
        push(w[..n - 1].to_owned());
    }

    // Past tense. `{stem}e` comes before the bare stem: with both `hope`
    // and `hop` in a lexicon, `hoped` must reach `hope` first.
    if let Some(stem) = w.strip_suffix("ied").filter(|_| n > 4) {
        push(format!("{stem}y"));
    }
    if let Some(stem) = w.strip_suffix("ed").filter(|_| n > 3) {
        push(format!("{stem}e"));
        push(stem.to_owned());
        push_undoubled(&mut push, stem);
    }

    // Progressive.
    if let Some(stem) = w.strip_suffix("ing").filter(|_| n > 4) {
        push(format!("{stem}e"));
        push(stem.to_owned());
        push_undoubled(&mut push, stem);
    }

    // Comparative / superlative / adverbial.
    if let Some(stem) = w.strip_suffix("ier").filter(|_| n > 4) {
        push(format!("{stem}y"));
    }
    if let Some(stem) = w.strip_suffix("iest").filter(|_| n > 5) {
        push(format!("{stem}y"));
    }
    if let Some(stem) = w.strip_suffix("er").filter(|_| n > 4) {
        push(format!("{stem}e"));
        push(stem.to_owned());
        push_undoubled(&mut push, stem);
    }
    if let Some(stem) = w.strip_suffix("est").filter(|_| n > 5) {
        push(format!("{stem}e"));
        push(stem.to_owned());
        push_undoubled(&mut push, stem);
    }
    if let Some(stem) = w.strip_suffix("ly").filter(|_| n > 4) {
        push(stem.to_owned());
    }

    out
}

/// `stopp` → `stop`, `bigg` → `big`: undo consonant doubling on a stem.
fn push_undoubled(push: &mut impl FnMut(String), stem: &str) {
    let bytes = stem.as_bytes();
    if bytes.len() > 2 {
        let last = bytes[bytes.len() - 1];
        let prev = bytes[bytes.len() - 2];
        if last == prev && !b"aeiou".contains(&last) && last.is_ascii_alphabetic() {
            push(stem[..stem.len() - 1].to_owned());
        }
    }
}

/// Regular-plural fallback for forms entirely outside the lexicon, so that
/// `endeavors` and `endeavor` count as one distinct word.
fn out_of_lexicon_plural(w: &str) -> Option<String> {
    let n = w.len();
    if let Some(stem) = w.strip_suffix("ies").filter(|_| n > 4) {
        return Some(format!("{stem}y"));
    }
    if n > 3 && w.ends_with('s') && !w.ends_with("ss") && !w.ends_with("us") && !w.ends_with("is") {
        return Some(w[..n - 1].to_owned());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::{FstLexicon, build_lexicon_blobs};

    #[test]
    fn irregulars_table_is_sorted_for_binary_search() {
        for pair in IRREGULARS.windows(2) {
            assert!(
                pair[0].0 < pair[1].0,
                "{:?} must sort before {:?}",
                pair[0].0,
                pair[1].0
            );
        }
    }

    #[test]
    fn cascade_order_is_irregulars_then_fst_then_rules() {
        // The FST deliberately maps "went" to a wrong lemma: the irregulars
        // table must win before the FST is consulted.
        let (bytes, pool) =
            build_lexicon_blobs(&[("went", "wend")], &["go", "stop"]).expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        assert_eq!(lemmatize("went", &lex), "go");
        // FST wins over rules: nothing maps "stopped" in the FST, the rules
        // reach "stop" through consonant undoubling.
        assert_eq!(lemmatize("stopped", &lex), "stop");
    }

    #[test]
    fn identity_is_the_last_resort() {
        let (bytes, pool) = build_lexicon_blobs(&[], &[]).expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        assert_eq!(lemmatize("Seldom", &lex), "seldom");
    }
}
