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

use super::language::StudiedLanguage;
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

/// Lemmatises one token of the studied language; the result is always
/// lowercase.
///
/// English runs its cascade: irregulars, the pack's forms, morphy-style
/// rules, the regular-plural fallback. Spanish runs its own
/// (add-lingua-spanish-analysis, [`super::spanish`]): the pack's forms, old
/// spellings, enclitics, its plural fallback — never an English table or
/// rule, which would read `has` (of *haber*) as `have`. French, until its own
/// cascade lands, is the baseline ([`lemmatize_baseline`],
/// add-lingua-french-baseline D3).
pub fn lemmatize(
    form: &str,
    studied: StudiedLanguage,
    lexicon: &(impl Lexicon + ?Sized),
) -> String {
    match studied {
        StudiedLanguage::English => lemmatize_english(form, lexicon),
        StudiedLanguage::Spanish => super::spanish::lemmatize(form, lexicon),
        StudiedLanguage::French => lemmatize_baseline(form, lexicon),
    }
}

/// The baseline lemmatisation of a language whose rules are not written: the
/// pack's lemma for the lowercased form, else the lowercased form itself.
///
/// What it lacks, on purpose (spec `lingua-analysis`, *Analysis by studied
/// language*): no exception table (`as`, `are`, `ate` stay themselves where
/// English's irregulars would read `be` or `eat`), no morphological rule (no
/// plural fallback, so `mes` is never `me`), no accent retry (a form with an
/// acute accent is never looked up without it, as Spanish's old spellings are),
/// no enclitic split, no Unicode normalisation. Each is a rule of a language's
/// own, which its analyser version bumps when it lands.
fn lemmatize_baseline(form: &str, lexicon: &(impl Lexicon + ?Sized)) -> String {
    let lower = form.replace('\u{2019}', "'").to_lowercase();
    match lexicon.lemma_of(&lower) {
        Some(lemma) => lemma.to_owned(),
        None => lower,
    }
}

/// English's cascade, unchanged since analyser 1.1.0.
fn lemmatize_english(form: &str, lexicon: &(impl Lexicon + ?Sized)) -> String {
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
        assert_eq!(lemmatize("went", StudiedLanguage::English, &lex), "go");
        // FST wins over rules: nothing maps "stopped" in the FST, the rules
        // reach "stop" through consonant undoubling.
        assert_eq!(lemmatize("stopped", StudiedLanguage::English, &lex), "stop");
    }

    #[test]
    fn identity_is_the_last_resort() {
        let (bytes, pool) = build_lexicon_blobs(&[], &[]).expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        assert_eq!(
            lemmatize("Seldom", StudiedLanguage::English, &lex),
            "seldom"
        );
    }

    #[test]
    fn spec_scenario_a_spanish_word_that_looks_english_keeps_its_own_lemma() {
        // English's cascade would read `has` as `have` and `ate` as `eat` (its
        // irregulars), and `mes` as `me` (its plural rule). Spanish's cascade
        // never runs an English table or rule.
        let (bytes, pool) =
            build_lexicon_blobs(&[], &["have", "be", "eat", "much", "me"]).expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        for form in ["has", "are", "ate", "more", "mes"] {
            assert_eq!(lemmatize(form, StudiedLanguage::Spanish, &lex), form);
        }
        assert_eq!(lemmatize("has", StudiedLanguage::English, &lex), "have");
        // With a pack that lists it, `has` is a form of *haber*.
        let (bytes, pool) = build_lexicon_blobs(&[("has", "haber")], &["haber"]).expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        assert_eq!(lemmatize("has", StudiedLanguage::Spanish, &lex), "haber");
    }

    #[test]
    fn spec_scenario_no_english_or_spanish_rule_runs_on_french() {
        // add-lingua-french-baseline D3: English's cascade would read `as`/`are` as `be`, `ate`
        // as `eat`, `has` as `have` and `mes` as `me`; Spanish's, `dámelo` as `dar`. The baseline
        // gives each back as itself when the pack does not list it.
        const FR: StudiedLanguage = StudiedLanguage::French;
        let (bytes, pool) =
            build_lexicon_blobs(&[], &["be", "have", "eat", "me", "dar", "a", "esta"])
                .expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        for form in ["as", "are", "ate", "a", "mes", "has", "dámelo"] {
            assert_eq!(lemmatize(form, FR, &lex), form, "{form:?}");
        }
        assert_eq!(lemmatize("has", StudiedLanguage::English, &lex), "have");
        // No accent retry: `ésta` is not looked up as `esta`, which the pack lists.
        assert_eq!(lemmatize("ésta", FR, &lex), "ésta");
        assert_eq!(lemmatize("ésta", StudiedLanguage::Spanish, &lex), "esta");
    }

    #[test]
    fn french_takes_the_lemma_its_pack_lists_else_the_lowercased_form() {
        const FR: StudiedLanguage = StudiedLanguage::French;
        let (bytes, pool) = build_lexicon_blobs(
            &[("as", "avoir"), ("mes", "mon"), ("été", "être")],
            &["avoir", "mon", "être"],
        )
        .expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        assert_eq!(lemmatize("As", FR, &lex), "avoir");
        assert_eq!(lemmatize("mes", FR, &lex), "mon");
        assert_eq!(lemmatize("ÉTÉ", FR, &lex), "être");
        // Unlisted, the form itself, lowercased; the typographic apostrophe reads as the straight
        // one, as the tokeniser gives it.
        assert_eq!(lemmatize("Maisons", FR, &lex), "maisons");
        assert_eq!(lemmatize("L\u{2019}homme", FR, &lex), "l'homme");
    }

    #[test]
    fn spanish_takes_the_lemma_its_pack_lists_before_any_rule() {
        let (bytes, pool) =
            build_lexicon_blobs(&[("has", "haber"), ("comió", "comer")], &["haber", "comer"])
                .expect("build");
        let lex = FstLexicon::from_slices(bytes, &pool).expect("load");
        assert_eq!(lemmatize("Has", StudiedLanguage::Spanish, &lex), "haber");
        assert_eq!(lemmatize("comió", StudiedLanguage::Spanish, &lex), "comer");
        // An unlisted plural reaches its singular (add-lingua-spanish-analysis D4).
        assert_eq!(lemmatize("Casas", StudiedLanguage::Spanish, &lex), "casa");
    }
}
