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

//! French's lemmatisation cascade (add-lingua-french-analysis D2).
//!
//! For each token, lowercased (`’` read as `'`) and in NFC:
//! 1. **the pack's forms** — the one lemma the forms tables chose for the form
//!    (add-lingua-french-forms-tables, M8: one form, one lemma);
//! 2. **an unlisted plural** — only for a form written in lowercase that the pack
//!    does not hold at all, and only when its singular is not in the pack either:
//!    `-eaux` → `-eau`, any other `-aux` → `-al`, a final `-s` after a letter
//!    other than `s` dropped. Left alone: forms of four letters or fewer,
//!    singulars in `-us`, `-is`, `-ès`, `-os`, a hyphenated or elided word, and
//!    the passé simple's `-âmes`, `-îmes`, `-ûmes`, `-âtes`, `-îtes`, `-ûtes`;
//! 3. **the form itself**.
//!
//! The order is the determinism contract: changing it, or any rule of it, bumps
//! French's analyser version.
//!
//! **The tables decide.** Step 2 never reads a form as a word the pack lists: its
//! singular must be unknown too, so the tables' own choices stand — `étés`, the
//! noun's plural whose singular `été` the tables read as *être*, stays itself, an
//! unknown word, and so do `vivants` (*vivant* reads as *vivre*) and `sorts`. The
//! cascade never chooses between a form's readings. Only lowercase forms: a
//! capitalised one outside the pack is a name or a people the proper-noun rules
//! already set aside (`Wisigoths`), which the rule would only rename (`Niaux` →
//! `nial`).
//!
//! **Measured and rejected** (design D2), each moving tokens of a raw corpus or of
//! UD's French treebanks for the worse or for nothing:
//! - a capital read without its accent (`Ecole` → `école`): edited French sets its
//!   accents on capitals;
//! - `oe` read as `œ` (`manoeuvre`): the tables map the dictionary's ASCII
//!   spellings themselves;
//! - a plural or a feminine read through its listed singular (`vivantes` →
//!   *vivre*): it undoes the tables' choice word by word (`étés` → *être*);
//! - verb endings checked against the lexicon (`promenèrent` → *promener*):
//!   guesswork where the tables are exact (`silve` → *silver*);
//! - a capital `A` read as `à`: the tokeniser's knowledge, not the cascade's;
//! - Spanish's cascade: French has no retired accented spellings to retry, and its
//!   enclitics are hyphenated, which the tokenisation pre-pass splits.

use unicode_normalization::UnicodeNormalization;

use super::lexicon::Lexicon;

/// The passé simple's first- and second-person plural endings: `dormîmes` and
/// `cessâtes` are verbs, never the plural of a `dormîme`.
const PASSE_SIMPLE_ENDINGS: &[&str] = &["âmes", "îmes", "ûmes", "âtes", "îtes", "ûtes"];

/// The endings of a singular already ending in `-s`: `campus`, `souris`,
/// `succès`, `repos`, `bras`/`os`… and `-ss` (`stress`).
const SINGULAR_S_ENDINGS: &[&str] = &["ss", "us", "is", "ès", "os"];

/// Lemmatises one French token; the result is lowercase and in NFC.
pub(crate) fn lemmatize(form: &str, lexicon: &(impl Lexicon + ?Sized)) -> String {
    let lower: String = form.replace('\u{2019}', "'").to_lowercase().nfc().collect();

    if let Some(lemma) = lexicon.lemma_of(&lower) {
        return lemma.to_owned();
    }
    if !form.chars().any(char::is_uppercase)
        && !lexicon.contains(&lower)
        && let Some(singular) = unlisted_plural(&lower)
        && !lexicon.contains(&singular)
    {
        return singular;
    }
    lower
}

/// The singular of a lowercase plural, by its ending alone, or `None` when the
/// form is not read as one (step 2 of the cascade): `-eaux` → `-eau` before any
/// other `-aux` → `-al` (`perdreaux` is no `perdreal`); `-s` after a letter other
/// than `s`. `-eux` and `-oux` are not stripped: an unlisted `-eux` is far likelier
/// an adjective (`sablonneux`) than the plural of an `-eu` noun, and the tables list
/// those.
fn unlisted_plural(lower: &str) -> Option<String> {
    if lower.chars().count() <= 4 || lower.contains('-') || lower.contains('\'') {
        return None;
    }
    if let Some(stem) = lower.strip_suffix("eaux") {
        return Some(format!("{stem}eau"));
    }
    if let Some(stem) = lower.strip_suffix("aux") {
        return Some(format!("{stem}al"));
    }
    let ending_ok = |endings: &[&str]| !endings.iter().any(|end| lower.ends_with(end));
    match lower.strip_suffix('s') {
        Some(stem) if ending_ok(SINGULAR_S_ENDINGS) && ending_ok(PASSE_SIMPLE_ENDINGS) => {
            Some(stem.to_owned())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::lexicon::{FstLexicon, build_lexicon_blobs};

    /// A small pack: forms → lemmas, and the lemmas listed as their own forms.
    fn lexicon(forms: &[(&str, &str)], lemmas: &[&str]) -> FstLexicon<Vec<u8>> {
        let (bytes, pool) = build_lexicon_blobs(forms, lemmas).expect("build");
        FstLexicon::from_slices(bytes, &pool).expect("load")
    }

    fn empty() -> FstLexicon<Vec<u8>> {
        lexicon(&[], &[])
    }

    #[test]
    fn a_listed_form_wins_over_every_rule() {
        let lex = lexicon(
            &[("porte", "porter"), ("portes", "porter"), ("été", "être")],
            &["porter", "être"],
        );
        assert_eq!(lemmatize("porte", &lex), "porter");
        assert_eq!(lemmatize("portes", &lex), "porter");
        assert_eq!(lemmatize("Porte", &lex), "porter");
        assert_eq!(lemmatize("ÉTÉ", &lex), "être");
    }

    #[test]
    fn an_unlisted_plural_reaches_its_unlisted_singular() {
        let lex = empty();
        assert_eq!(lemmatize("mégalithes", &lex), "mégalithe");
        assert_eq!(lemmatize("vicissitudes", &lex), "vicissitude");
        assert_eq!(lemmatize("chevaux", &lex), "cheval");
        assert_eq!(lemmatize("bateaux", &lex), "bateau");
        // `-eaux` is read before `-aux`.
        assert_eq!(lemmatize("perdreaux", &lex), "perdreau");
    }

    #[test]
    fn a_plural_whose_singular_the_pack_lists_stays_itself() {
        // The tables decide (M8): `été` reads as *être*, `vivant` as *vivre*; the rule never
        // reads `étés` or `vivants` through them, nor as a singular the pack holds.
        let lex = lexicon(
            &[("été", "être"), ("vivant", "vivre"), ("sort", "sortir")],
            &["être", "vivre", "sortir", "cheval"],
        );
        assert_eq!(lemmatize("étés", &lex), "étés");
        assert_eq!(lemmatize("vivants", &lex), "vivants");
        assert_eq!(lemmatize("sorts", &lex), "sorts");
        // A listed singular holds its plural back, whatever lemma it reads as.
        assert_eq!(lemmatize("chevaux", &lex), "chevaux");
    }

    #[test]
    fn singulars_in_s_and_x_short_words_and_runs_are_left_alone() {
        let lex = empty();
        for form in [
            "heureux",
            "bijoux",
            "campus",
            "souris",
            "succès",
            "repos",
            "stress",
            "gens",
            "fils",
            "arc-en-ciels",
            "prud'hommes",
        ] {
            assert_eq!(lemmatize(form, &lex), form, "{form}");
        }
    }

    #[test]
    fn passe_simple_endings_are_left_alone() {
        let lex = empty();
        for form in [
            "dormîmes",
            "cessâmes",
            "reçûmes",
            "cessâtes",
            "dormîtes",
            "reçûtes",
        ] {
            assert_eq!(lemmatize(form, &lex), form, "{form}");
        }
    }

    #[test]
    fn a_capitalised_form_outside_the_pack_is_left_alone() {
        let lex = empty();
        assert_eq!(lemmatize("Belfons", &lex), "belfons");
        assert_eq!(lemmatize("Wisigoths", &lex), "wisigoths");
        // A capital anywhere: `eBooks` is no lowercase form.
        assert_eq!(lemmatize("eBooks", &lex), "ebooks");
    }

    #[test]
    fn the_lemma_is_lowercase_and_composed() {
        let lex = lexicon(&[("mémoire", "mémoire")], &[]);
        // `mémoire` written with `e` + U+0301 reaches the pack's composed form.
        assert_eq!(lemmatize("me\u{301}moire", &lex), "mémoire");
        // `MÉGALITHES` decomposed: composed and lowercased, never cut, being capitalised.
        assert_eq!(lemmatize("ME\u{301}GALITHES", &lex), "mégalithes");
        // Lowercase and decomposed, the plural rule reads the composed form.
        assert_eq!(lemmatize("me\u{301}galithes", &lex), "mégalithe");
        // The typographic apostrophe reads as the straight one.
        assert_eq!(lemmatize("L\u{2019}homme", &lex), "l'homme");
    }
}
