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

//! Spanish's lemmatisation cascade (add-lingua-spanish-analysis D2–D4).
//!
//! For each token, lowercased and in NFC: the pack's forms → the same form
//! without its acute accents (the spellings the 2010 rules retired: `fué`,
//! `dió`, the preposition `á`, `sólo`, `éste`) → the enclitic rule → a plural
//! fallback for forms outside the lexicon → the form itself. The order is the
//! determinism contract: changing it bumps Spanish's analyser version.
//!
//! Enclitics are a rule, never a table: two pronouns after any infinitive,
//! gerund or imperative of any verb are combinatorial. The rule runs only when
//! the whole form is not listed — a listed word is never split — and only
//! accepts a base the lexicon lists, shaped to take enclitics, whose written
//! accent (if the clitics moved one onto it) sits on the vowel the base itself
//! stresses.

use unicode_normalization::UnicodeNormalization;

use super::lexicon::Lexicon;

/// The clitic pronouns, longer first so `nos` is tried before `os` and `los`
/// before `lo`.
const CLITICS: &[&str] = &[
    "nos", "los", "las", "les", "me", "te", "se", "os", "lo", "la", "le",
];

/// The affirmative imperatives of one syllable, the only monosyllables an
/// enclitic may attach to (`dime`, `hazlo`, `vete`, `idos`). Sorted for binary
/// search (a test enforces it).
const MONOSYLLABIC_IMPERATIVES: &[&str] = &[
    "da", "dad", "di", "haz", "id", "pon", "sal", "sé", "ten", "ve", "ved", "ven",
];

/// Lemmatises one Spanish token; the result is lowercase and in NFC.
pub(crate) fn lemmatize(form: &str, lexicon: &(impl Lexicon + ?Sized)) -> String {
    let lower: String = form.replace('\u{2019}', "'").to_lowercase().nfc().collect();

    if let Some(lemma) = lexicon.lemma_of(&lower) {
        return lemma.to_owned();
    }
    let plain = without_acute_accents(&lower);
    if plain != lower
        && let Some(lemma) = lexicon.lemma_of(&plain)
    {
        return lemma.to_owned();
    }
    if let Some(lemma) = enclitic_lemma(&lower, lexicon) {
        return lemma;
    }
    if !lexicon.contains(&lower)
        && let Some(singular) = out_of_lexicon_plural(&lower)
    {
        return singular;
    }
    lower
}

/// `á é í ó ú` → `a e i o u`; `ü` and `ñ` are letters of their own and stay.
fn without_acute_accents(word: &str) -> String {
    word.chars().map(unaccented).collect()
}

fn unaccented(c: char) -> char {
    match c {
        'á' => 'a',
        'é' => 'e',
        'í' => 'i',
        'ó' => 'o',
        'ú' => 'u',
        other => other,
    }
}

fn is_acute(c: char) -> bool {
    matches!(c, 'á' | 'é' | 'í' | 'ó' | 'ú')
}

fn is_vowel(c: char) -> bool {
    matches!(unaccented(c), 'a' | 'e' | 'i' | 'o' | 'u') || c == 'ü'
}

fn is_strong(c: char) -> bool {
    matches!(unaccented(c), 'a' | 'e' | 'o')
}

/// The lemma of a verb form carrying one or two enclitic pronouns, or none.
/// One clitic is tried before two; within each, longer clitics first.
fn enclitic_lemma(lower: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    for clitic in CLITICS {
        if let Some(base) = lower.strip_suffix(clitic)
            && let Some(lemma) = base_lemma(base, clitic, lexicon)
        {
            return Some(lemma);
        }
    }
    for outer in CLITICS {
        let Some(rest) = lower.strip_suffix(outer) else {
            continue;
        };
        for inner in CLITICS {
            if let Some(base) = rest.strip_suffix(inner)
                && let Some(lemma) = base_lemma(base, inner, lexicon)
            {
                return Some(lemma);
            }
        }
    }
    None
}

/// The lemma of `written`, the form left once its clitics are stripped, when it
/// is a base that takes enclitics. `inner` is the clitic next to it: after
/// `nos` a base ending in `mo` lost its `s` (`vámonos`). `os` follows a
/// vosotros imperative that lost its `d` (`sentaos`), `id` (`idos`), an
/// infinitive or a gerund (`deciros`, `diciéndoos`) — never a tú imperative:
/// `dios` is no `di` + `os`.
fn base_lemma(written: &str, inner: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    if written.chars().count() < 2 {
        return None;
    }
    let mut candidates = Vec::new();
    if inner == "os" {
        if written.chars().last().is_some_and(is_vowel) {
            candidates.push(format!("{written}d"));
        }
        if written == "id" || is_infinitive_or_gerund(written) {
            candidates.push(written.to_owned());
        }
    } else {
        candidates.push(written.to_owned());
        if inner == "nos" && written.ends_with("mo") {
            candidates.push(format!("{written}s"));
        }
    }
    for base in candidates {
        if takes_enclitics(&base)
            && let Some(lemma) = lexicon.lemma_of(&base)
        {
            return Some(lemma.to_owned());
        }
        if let Some(plain) = stress_accent_removed(&base)
            && takes_enclitics(&plain)
            && let Some(lemma) = lexicon.lemma_of(&plain)
        {
            return Some(lemma.to_owned());
        }
    }
    None
}

/// `base` without its one acute accent, when that accent sits on the vowel the
/// unaccented base stresses — the accent the clitics' stress shift wrote, and
/// nothing else (`dá` → `da`, `diciéndo` → `diciendo`, never `comé` → `come`).
fn stress_accent_removed(base: &str) -> Option<String> {
    let chars: Vec<char> = base.chars().collect();
    let mut accented = chars.iter().enumerate().filter(|(_, c)| is_acute(**c));
    let (at, _) = accented.next()?;
    if accented.next().is_some() {
        return None;
    }
    let plain: Vec<char> = chars.iter().map(|c| unaccented(*c)).collect();
    (stressed_vowel(&plain) == Some(at)).then(|| plain.into_iter().collect())
}

/// An infinitive (`-ar`, `-er`, `-ir`, `-ír`) or a gerund (`-ndo`), by its shape.
fn is_infinitive_or_gerund(base: &str) -> bool {
    ["ar", "er", "ir", "ír", "ndo"]
        .iter()
        .any(|end| base.ends_with(end))
}

/// Whether a base can carry enclitics: an infinitive, a gerund, another verb
/// form of two vowel groups or more, or a monosyllabic imperative of the closed
/// list.
fn takes_enclitics(base: &str) -> bool {
    if is_infinitive_or_gerund(base) {
        return true;
    }
    let chars: Vec<char> = base.chars().collect();
    vowel_groups(&chars).len() >= 2 || MONOSYLLABIC_IMPERATIVES.binary_search(&base).is_ok()
}

/// The vowel groups of a word, as char index ranges: maximal runs of vowels,
/// split between two strong vowels (a hiatus: `le-er`) and around an accented
/// weak vowel (`re-ír`). The `u` of `que`, `qui`, `gue` and `gui` is silent and
/// belongs to no group.
fn vowel_groups(chars: &[char]) -> Vec<(usize, usize)> {
    let mut groups: Vec<(usize, usize)> = Vec::new();
    let mut open: Option<usize> = None;
    for (i, &c) in chars.iter().enumerate() {
        let silent_u = c == 'u'
            && i > 0
            && (chars[i - 1] == 'q' || chars[i - 1] == 'g')
            && chars
                .get(i + 1)
                .is_some_and(|n| matches!(n, 'e' | 'i' | 'é' | 'í'));
        if !is_vowel(c) || silent_u {
            if let Some(start) = open.take() {
                groups.push((start, i));
            }
            continue;
        }
        if let Some(start) = open {
            let prev = chars[i - 1];
            let hiatus = (is_strong(prev) && is_strong(c))
                || matches!(prev, 'í' | 'ú')
                || matches!(c, 'í' | 'ú');
            if hiatus {
                groups.push((start, i));
                open = Some(i);
            }
        } else {
            open = Some(i);
        }
    }
    if let Some(start) = open {
        groups.push((start, chars.len()));
    }
    groups
}

/// The index of the vowel an unaccented word stresses by the default rule: the
/// last group if the word ends in a consonant other than `n` or `s`, else the
/// group before it; within a group, its strong vowel, else its last.
fn stressed_vowel(chars: &[char]) -> Option<usize> {
    let groups = vowel_groups(chars);
    let last = *chars.last()?;
    let final_group = groups.len().checked_sub(1)?;
    let group = if is_vowel(last) || last == 'n' || last == 's' {
        final_group.saturating_sub(1)
    } else {
        final_group
    };
    let (start, end) = groups[group];
    (start..end)
        .find(|&i| is_strong(chars[i]))
        .or(Some(end - 1))
}

/// The singular of a plural the lexicon does not know at all, so the two count
/// as one word (add-lingua-spanish-analysis D4). Forms of four letters or fewer
/// and singulars in `-is`/`-us` are left alone.
fn out_of_lexicon_plural(w: &str) -> Option<String> {
    if w.chars().count() <= 4 || w.ends_with("is") || w.ends_with("us") {
        return None;
    }
    // `luces` → `luz`; after a consonant (`dulces`) it is the `-s` rule.
    if let Some(stem) = w.strip_suffix("ces")
        && stem.chars().last().is_some_and(is_vowel)
    {
        return Some(format!("{stem}z"));
    }
    if let Some(stem) = w.strip_suffix("iones") {
        return Some(format!("{stem}ión"));
    }
    if let Some(stem) = w.strip_suffix("es") {
        let mut back = stem.chars().rev();
        if let (Some(last), Some(prev)) = (back.next(), back.next())
            && matches!(last, 'l' | 'r' | 'n' | 'd' | 'j' | 'y')
            && is_vowel(prev)
        {
            // An accent the plural wrote on a stem ending in vowel + `n` goes:
            // `exámenes` → `examen`.
            return Some(if last == 'n' {
                without_acute_accents(stem)
            } else {
                stem.to_owned()
            });
        }
    }
    if let Some(stem) = w.strip_suffix('s')
        && stem.chars().last().is_some_and(is_vowel)
    {
        return Some(stem.to_owned());
    }
    None
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

    fn verbs() -> FstLexicon<Vec<u8>> {
        lexicon(
            &[
                ("da", "dar"),
                ("di", "decir"),
                ("diga", "decir"),
                ("diciendo", "decir"),
                ("come", "comer"),
                ("va", "ir"),
                ("vamos", "ir"),
                ("id", "ir"),
                ("sentad", "sentar"),
                ("sentemos", "sentar"),
                ("levanta", "levantar"),
                ("sale", "salir"),
                ("fue", "ser"),
                ("dio", "dar"),
                ("solo", "solo"),
            ],
            &[
                "dar", "decir", "comer", "ir", "sentar", "levantar", "salir", "ser", "hacer",
                "reír", "escribir", "solo",
            ],
        )
    }

    #[test]
    fn the_closed_list_is_sorted_for_binary_search() {
        for pair in MONOSYLLABIC_IMPERATIVES.windows(2) {
            assert!(
                pair[0] < pair[1],
                "{:?} must sort before {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn a_listed_form_wins_over_every_rule() {
        let lex = verbs();
        assert_eq!(lemmatize("sale", &lex), "salir"); // never sa + le
        assert_eq!(lemmatize("Come", &lex), "comer");
    }

    #[test]
    fn old_spellings_are_read_without_their_accents_before_any_rule() {
        let lex = verbs();
        assert_eq!(lemmatize("fué", &lex), "ser");
        assert_eq!(lemmatize("dió", &lex), "dar");
        assert_eq!(lemmatize("Sólo", &lex), "solo");
    }

    #[test]
    fn enclitics_reach_the_verb() {
        let lex = verbs();
        for (form, lemma) in [
            ("dámelo", "dar"),
            ("dime", "decir"),
            ("dígame", "decir"),
            ("diciéndole", "decir"),
            ("cómelo", "comer"),
            ("hacerlo", "hacer"),
            ("reírse", "reír"),
            ("escribírsela", "escribir"),
            ("levántate", "levantar"),
            ("vámonos", "ir"),
            ("sentémonos", "sentar"),
            ("sentaos", "sentar"),
            ("idos", "ir"),
        ] {
            assert_eq!(lemmatize(form, &lex), lemma, "{form}");
        }
    }

    #[test]
    fn an_accent_the_base_does_not_stress_is_no_stress_shift() {
        // `come` stresses its first vowel: `comé` is not `come` with a moved accent.
        assert_eq!(lemmatize("comélo", &verbs()), "comélo");
    }

    #[test]
    fn a_monosyllable_outside_the_closed_list_takes_no_enclitic() {
        // `va` is listed, but it is not an imperative of the closed list.
        assert_eq!(lemmatize("vale", &verbs()), "vale");
    }

    #[test]
    fn a_base_the_lexicon_does_not_list_is_not_split() {
        assert_eq!(lemmatize("carmela", &verbs()), "carmela");
    }

    #[test]
    fn stress_follows_the_default_rule_and_the_strong_vowel() {
        let at = |w: &str| stressed_vowel(&w.chars().collect::<Vec<_>>());
        assert_eq!(at("da"), Some(1));
        assert_eq!(at("come"), Some(1)); // co-me
        assert_eq!(at("diciendo"), Some(4)); // di-CIEN-do: the strong vowel of `ie`
        assert_eq!(at("escribir"), Some(6)); // ends in r: the last group
        assert_eq!(at("vamos"), Some(1)); // ends in s: the group before
        assert_eq!(at("leer"), Some(2)); // le-er: a hiatus
        assert_eq!(at("quiere"), Some(3)); // the u of `qui` is silent
    }

    #[test]
    fn unlisted_plurals_reach_their_singular() {
        let empty = lexicon(&[], &["casa"]);
        for (plural, singular) in [
            ("luces", "luz"),
            ("actrices", "actriz"),
            ("dulces", "dulce"),
            ("canciones", "canción"),
            ("árboles", "árbol"),
            ("ciudades", "ciudad"),
            ("reyes", "rey"),
            ("relojes", "reloj"),
            ("exámenes", "examen"),
            ("madres", "madre"),
            ("posibles", "posible"),
        ] {
            assert_eq!(lemmatize(plural, &empty), singular, "{plural}");
        }
    }

    #[test]
    fn singulars_in_is_or_us_and_short_forms_are_left_alone() {
        let empty = lexicon(&[], &["casa"]);
        for form in ["crisis", "virus", "análisis", "dios", "tres"] {
            assert_eq!(lemmatize(form, &empty), form);
        }
    }

    #[test]
    fn a_decomposed_accent_is_read_composed() {
        let lex = lexicon(&[("está", "estar")], &["estar"]);
        assert_eq!(lemmatize("esta\u{0301}", &lex), "estar");
    }
}
