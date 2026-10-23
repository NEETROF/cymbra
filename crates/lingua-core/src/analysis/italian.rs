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

//! Italian's lemmatisation cascade (add-lingua-italian-analysis D2, as measured by D6).
//!
//! For each token, lowercased (`’` read as `'`) and in NFC:
//! 1. **the pack's forms** — the one lemma the forms tables chose for the form
//!    (add-lingua-italian-forms-tables, M8: one form, one lemma);
//! 2. **the enclitic rule** ([`enclitic_lemma`]) — only when the whole form is not in the
//!    pack: one clitic pronoun read off its end, a combined clitic before a single one and
//!    each the longest first (`glielo`… `gliene`; `melo`… `mene`, likewise with `te`, `se`,
//!    `ce`, `ve`; then `gli`, `mi`, `ti`, `si`, `ci`, `vi`, `lo`, `la`, `li`, `le`, `ne`), and
//!    the rest accepted only as a base the pack lists shaped to take clitics: one of the five
//!    monosyllabic imperatives `da`, `fa`, `va`, `di`, `sta` with the clitic's doubled
//!    consonant undone (`dammi`, `fallo`, `vattene`, `dimmi`, `stacci`) and read by their
//!    table, undoubled only before `gli` and `glie-` (`digli`, `fagli`, `diglielo`: `gli`
//!    never doubles; `falo`, `dami`, `stami` are none of theirs); `ecco`; an infinitive with
//!    its ending restored (`dir` → `dire`, `por` → `porre`, `-re` tried where `-e` lists
//!    nothing); a gerund or an imperative, a form the pack files under a verb, of three
//!    letters or more, in `-o` only as a gerund (`-ndo`) or a first-person plural (`-iamo`)
//!    (`parlando`, `parla`, `scrivi`, `prendete`, `andiamo`) — a clitic follows an imperative,
//!    an infinitive, a gerund or a participle, never a present indicative, so « Stone » is no
//!    `sto` + `ne` read as `stare`, nor « Sami » `sa` + `mi` read as `sapere`; else a regular
//!    past participle read as its infinitive (`sviluppata` → `sviluppare`, `formata` →
//!    `formare`), but before a lone `ne`, which would read an augmentative (`risultatone`,
//!    `vestitone`, `peccatone`, `gelatone`) as a participle. The lemma is the base's. One split
//!    only: `dirmelo` is `dir` + `melo`, never `dirme` + `lo` read again. Italian spelling
//!    moves no written accent when a clitic attaches, so Spanish's stress check has no
//!    counterpart;
//! 3. **the form itself**.
//!
//! The order is the determinism contract: changing it, or any rule of it, bumps Italian's
//! analyser version.
//!
//! **The tables decide.** Step 2 never splits a listed word (`cinema`, `sistema`, `tema`,
//! `vela` are not `cine` + `ma` or `ve` + `la`: they are listed), and no step reads a form
//! as a word the pack lists under another lemma: `vite` (vines; lives), `porte` stay what the
//! tables file them under.
//!
//! **Measured** (design D6) — a pack built from a prototype of change 9's forms tables (the
//! English Wiktionary's Italian section, MarkIT's training and development counts and
//! wordfreq's ranks, 58,113 lemmas and 296,298 forms), the cascade's lemmas against UD
//! Italian-PUD (at `71d932aa`) and MarkIT's test section (at `e210079d`), token by token with
//! and without each candidate rule. A rule was kept when it took the treebank's lemma on a
//! strict majority of the tokens it moved, and no figure fell (criteria written before the
//! figures were read). With the decided cascade the prototype resolves 98.49 % of PUD's
//! words, and 90.56 % of its content words and 93.81 % of its auxiliaries take the treebank's
//! lemma (MarkIT's test section: 98.02 %, 94.19 %, 96.98 %) — a prototype's figures, under the
//! gates change 9's tables are held to; `measure/it-ud.sh` measures the committed tables once
//! change 9 writes them.
//! - *The enclitic rule* (the change's core, reported): on the 133 verb + clitic words both
//!   treebanks write as one (`dirmelo`, which UD splits into `dir` + `me` + `lo`), it gives
//!   the verb's lemma to 91 more (45 on PUD, 46 on MarkIT), none fewer, and moves one for
//!   nothing (`congratularmi`, filed by the prototype under the reflexive `congratularsi`
//!   where PUD writes `congratulare`): 127 of the 133 take the treebank's verb. On every other
//!   measured word it moves, 17 better, none worse, 14 for nothing: a plural participle or
//!   adjective the prototype does not list read as an imperative and `ti` (`inviati` →
//!   `inviare` is right; `certificati` → `certificare`, `impiegati` → `impiegare`, `porti` →
//!   `porre`, `fatti` → `fare` where the treebank has the noun or the adjective).
//! - *Kept — a past participle base* (D2's candidate): a regular participle the tables file
//!   under an adjective (`sviluppata`, `formata`) read as its infinitive; 3 tokens better
//!   (`Formatasi`, `sviluppatasi` twice: two word types, each with `si` — a thin sample), none
//!   worse, no other word moved.
//! - *Guards added at review*, moving no word of either treebank: no verb-form base in `-o`
//!   but a gerund or a first-person plural, and none of two letters (« Stone », « Sami »); no
//!   undoubled monosyllable but before `gli`; no participle base before a lone `ne` (the
//!   augmentatives). A known cost stays: a verb is told by its infinitive's ending
//!   ([`is_verb`]), so a noun or an adjective in `-are`, `-ere`, `-ire` passes (`marlo` →
//!   `mare`); telling them apart needs a part of speech the pack does not carry.
//! - *Dropped — an unlisted plural read as its singular* (D2's step 3), every part of it: the
//!   forms tables list the regular plurals, so what is left unlisted is mostly a plural of an
//!   `-e` noun or adjective, a loan or a typo. `-i` → `-o` (with `-chi` → `-co`, `-ghi` →
//!   `-go`): 1 word better, 6 worse, 3 moved for nothing (`thailandesi` → `thailandeso`,
//!   `dettagli` → `dettaglo`), PUD's content figure falling from 90.43 % to 90.36 %;
//!   `-ci`/`-gi` read through it moved nothing, and goes with the step it is a case of — a
//!   departure from the written criterion, which keeps a base rule that moves nothing;
//!   `-e` → `-a` (with `-che` → `-ca`, `-ghe` →
//!   `-ga`): none better, 14 worse, 7 for nothing (`famigliare` → `famigliara`, `folklore` →
//!   `folklora`, `cresciute` → `cresciuta`). An unlisted plural stays itself, an unknown word.
//!
//! *Rejected — Spanish's accent retry.* Italian has no retired accented spellings; its accent
//! problem is the apostrophe written for one (`perche'`), read-lingua-italian-unmarked-spellings'.

use unicode_normalization::UnicodeNormalization;

use super::lexicon::Lexicon;

/// The combined clitics — `me`, `te`, `se`, `ce`, `ve` or `glie` before `lo`, `la`, `li`, `le`
/// or `ne` — longest first, so `glielo` is tried before `melo` and every combined one before
/// a single clitic.
const COMBINED_CLITICS: &[&str] = &[
    "glielo", "gliela", "glieli", "gliele", "gliene", "melo", "mela", "meli", "mele", "mene",
    "telo", "tela", "teli", "tele", "tene", "selo", "sela", "seli", "sele", "sene", "celo", "cela",
    "celi", "cele", "cene", "velo", "vela", "veli", "vele", "vene",
];

/// The single clitic pronouns, longest first: `gli` before `li`.
const CLITICS: &[&str] = &[
    "gli", "mi", "ti", "si", "ci", "vi", "lo", "la", "li", "le", "ne",
];

/// The five monosyllabic imperatives that double a clitic's first consonant (`da'` → `dammi`)
/// and the verb each is — by this table, not by the pack, which lists `da` and `di` as
/// prepositions. Sorted for binary search (a test enforces it).
const MONOSYLLABIC_IMPERATIVES: &[(&str, &str)] = &[
    ("da", "dare"),
    ("di", "dire"),
    ("fa", "fare"),
    ("sta", "stare"),
    ("va", "andare"),
];

/// The endings of an infinitive, a lemma the pack files a verb's forms under: `-are`, `-ere`,
/// `-ire`, `-rre` (`porre`, `condurre`), and a reflexive-only verb's `-rsi` (`pentirsi`, under
/// which change 9's tables file `pentite` and `pentendo`).
const VERB_ENDINGS: &[&str] = &["are", "ere", "ire", "rre", "rsi"];

/// A regular past participle's endings, by conjugation, and the infinitive's: `-ato` … `-ate`
/// of an `-are` verb, `-uto` … `-ute` of an `-ere` verb, `-ito` … `-ite` of an `-ire` verb.
const PAST_PARTICIPLES: &[(&[&str], &str)] = &[
    (&["ato", "ata", "ati", "ate"], "are"),
    (&["uto", "uta", "uti", "ute"], "ere"),
    (&["ito", "ita", "iti", "ite"], "ire"),
];

/// Lemmatises one Italian token; the result is lowercase and in NFC.
pub(crate) fn lemmatize(form: &str, lexicon: &(impl Lexicon + ?Sized)) -> String {
    let lower: String = form.replace('\u{2019}', "'").to_lowercase().nfc().collect();

    if let Some(lemma) = lexicon.lemma_of(&lower) {
        return lemma.to_owned();
    }
    if let Some(lemma) = enclitic_lemma(&lower, lexicon) {
        return lemma;
    }
    lower
}

/// The lemma of a verb form carrying a clitic pronoun or a combined clitic, or `None` (step
/// 2). `lower` is the whole form, lowercase, in NFC and not in the pack. A word holding an
/// apostrophe or a hyphen carries no clitic: the pre-pass already read its pieces.
fn enclitic_lemma(lower: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    if lower.contains(['\'', '-']) {
        return None;
    }
    COMBINED_CLITICS
        .iter()
        .chain(CLITICS)
        .find_map(|clitic| base_lemma(lower.strip_suffix(clitic)?, clitic, lexicon))
}

/// The lemma of `rest`, what is left once `clitic` is read off, when it is a base that takes
/// clitics: one of the five monosyllabic imperatives with the clitic's first consonant
/// doubled, or before `gli` undoubled; `ecco`; an infinitive with its ending restored; a
/// gerund or an imperative ([`verb_form_lemma`]); a regular past participle, but before a lone
/// `ne` (an augmentative: `risultatone` is no `risultato` + `ne`).
fn base_lemma(rest: &str, clitic: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    if let Some(verb) = monosyllabic_imperative(rest, clitic) {
        return Some(verb.to_owned());
    }
    // One of the five without its doubling, before another clitic than `gli`, is no imperative
    // of theirs (`falo`, `dami`, `stami`): they double every other clitic's consonant.
    if monosyllable(rest).is_some() {
        return None;
    }
    if rest == "ecco" {
        return lexicon.lemma_of(rest).map(str::to_owned);
    }
    if rest.chars().count() < 2 {
        return None;
    }
    if rest.ends_with('r') {
        // `dir` → `dire`; `por` → `porre` only when the pack lists no `pore`.
        let restored = ["e", "re"]
            .iter()
            .find_map(|ending| lexicon.lemma_of(&format!("{rest}{ending}")))?;
        return is_verb(restored).then(|| restored.to_owned());
    }
    if let Some(lemma) = verb_form_lemma(rest, lexicon) {
        return Some(lemma);
    }
    if clitic == "ne" {
        return None;
    }
    past_participle_lemma(rest, lexicon)
}

/// The verb of a gerund (`parlando`) or an imperative (`parla`, `prendete`, `andiamo`): a form
/// the pack files under a verb, of three letters or more, ending in `-o` only as a gerund
/// (`-ndo`) or a first-person plural (`-iamo`). A clitic follows only an imperative, an
/// infinitive, a gerund or a participle: a present indicative never takes one (« Stone » is no
/// `sto` + `ne`, `stare`), and a monosyllable is the table's (« Sami » is no `sa` + `mi`,
/// `sapere`).
fn verb_form_lemma(rest: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    if rest.chars().count() <= 2
        || (rest.ends_with('o') && !rest.ends_with("ndo") && !rest.ends_with("iamo"))
    {
        return None;
    }
    lexicon
        .lemma_of(rest)
        .filter(|lemma| is_verb(lemma))
        .map(str::to_owned)
}

/// The verb of one of the five monosyllabic imperatives `rest` writes before `clitic`: the
/// clitic's first consonant doubled (`dam` + `mi`, `vat` + `tene`), or `gli` and the combined
/// `glie-` without doubling (`di` + `gli`).
fn monosyllabic_imperative(rest: &str, clitic: &str) -> Option<&'static str> {
    let base = if clitic.starts_with("gli") {
        rest
    } else {
        let doubled = clitic.chars().next()?;
        rest.strip_suffix(doubled)?
    };
    monosyllable(base)
}

/// The verb of one of the five monosyllabic imperatives, by its table.
fn monosyllable(base: &str) -> Option<&'static str> {
    MONOSYLLABIC_IMPERATIVES
        .binary_search_by_key(&base, |&(form, _)| form)
        .ok()
        .map(|index| MONOSYLLABIC_IMPERATIVES[index].1)
}

/// The verb a regular past participle belongs to, when the pack lists that verb
/// ([`PAST_PARTICIPLES`]): `sviluppata` → `sviluppare`, which the tables may file under the
/// adjective `sviluppato` (M8), so the verb-form base does not reach it (`sviluppatasi`).
fn past_participle_lemma(rest: &str, lexicon: &(impl Lexicon + ?Sized)) -> Option<String> {
    PAST_PARTICIPLES.iter().find_map(|(endings, infinitive)| {
        let stem = endings
            .iter()
            .find_map(|ending| rest.strip_suffix(ending))?;
        let lemma = lexicon.lemma_of(&format!("{stem}{infinitive}"))?;
        is_verb(lemma).then(|| lemma.to_owned())
    })
}

/// Whether a lemma of the pack is a verb, by its infinitive's ending ([`VERB_ENDINGS`]). A known
/// cost: a noun or an adjective with such an ending passes too (`mare`, `popolare`,
/// `carattere`), so an unlisted `marlo` reads as `mare`; telling them apart needs a part of
/// speech the pack does not carry.
fn is_verb(lemma: &str) -> bool {
    VERB_ENDINGS.iter().any(|ending| lemma.ends_with(ending))
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

    /// The verbs the enclitic rule needs, their gerunds and imperatives filed under them as the
    /// forms tables file them, the prepositions `da` and `di`, and the words never split.
    fn verbs() -> FstLexicon<Vec<u8>> {
        lexicon(
            &[
                ("parlando", "parlare"),
                ("parla", "parlare"),
                ("scrivi", "scrivere"),
                ("prendete", "prendere"),
                ("andiamo", "andare"),
                ("va", "andare"),
                ("fa", "fare"),
                ("dici", "dire"),
                ("pentite", "pentirsi"),
                ("pentire", "pentirsi"),
                ("porte", "porta"),
            ],
            &[
                "parlare", "dire", "porre", "condurre", "dare", "andare", "vedere", "scrivere",
                "prendere", "fare", "stare", "ecco", "da", "di", "cinema", "sistema", "tema",
                "vela", "porta", "vita", "pentirsi", "fiore", "mare",
            ],
        )
    }

    fn reads(form: &str, lex: &FstLexicon<Vec<u8>>) -> String {
        lemmatize(form, lex)
    }

    #[test]
    fn the_tables_are_sorted_and_longest_first() {
        for pair in MONOSYLLABIC_IMPERATIVES.windows(2) {
            assert!(pair[0].0 < pair[1].0, "{pair:?}");
        }
        for table in [COMBINED_CLITICS, CLITICS] {
            for pair in table.windows(2) {
                assert!(pair[0].len() >= pair[1].len(), "{pair:?}");
            }
        }
        // Every combined clitic is one of the five, or `glie`, before one of the five.
        for combined in COMBINED_CLITICS {
            let (head, tail) = combined.split_at(combined.len() - 2);
            assert!(
                ["me", "te", "se", "ce", "ve", "glie"].contains(&head),
                "{combined}"
            );
            assert!(["lo", "la", "li", "le", "ne"].contains(&tail), "{combined}");
        }
        assert_eq!(COMBINED_CLITICS.len(), 30);
    }

    #[test]
    fn a_listed_form_wins_over_every_rule() {
        let lex = verbs();
        assert_eq!(reads("Parla", &lex), "parlare");
        // The tables decide: `porte` filed under `porta` is `porta`, no rule runs.
        assert_eq!(reads("porte", &lex), "porta");
        assert_eq!(reads("porta", &lex), "porta");
        // A listed word is never split: not `cine` + `ma`, `siste` + `ma`, `te` + `ma`, `ve` +
        // `la`.
        for word in ["cinema", "sistema", "tema", "vela", "Vela"] {
            assert_eq!(reads(word, &lex), word.to_lowercase(), "{word}");
        }
    }

    #[test]
    fn a_clitic_on_an_infinitive() {
        let lex = verbs();
        for (form, lemma) in [
            ("parlarmi", "parlare"),
            ("dirlo", "dire"),
            ("Dirlo", "dire"),
            ("porlo", "porre"),
            ("condurlo", "condurre"),
            ("vederci", "vedere"),
        ] {
            assert_eq!(reads(form, &lex), lemma, "{form}");
        }
        // `-re` is tried only where `-e` lists nothing; `-e` listing a word that is no verb
        // reads nothing.
        assert_eq!(reads("fiorlo", &lex), "fiorlo");
        let pore = lexicon(&[], &["pore", "porre"]);
        assert_eq!(reads("porlo", &pore), "porlo");
        // A known cost: a verb is told by its ending, and `mare` ends in `-are` (`is_verb`).
        assert_eq!(reads("marlo", &lex), "mare");
    }

    #[test]
    fn a_combined_clitic() {
        let lex = verbs();
        for (form, lemma) in [
            ("dirmelo", "dire"),
            ("dargliela", "dare"),
            ("andarsene", "andare"),
            ("vedermene", "vedere"),
            ("darcela", "dare"),
            ("dirvelo", "dire"),
            ("dirtele", "dire"),
        ] {
            assert_eq!(reads(form, &lex), lemma, "{form}");
        }
    }

    #[test]
    fn a_clitic_on_a_gerund_and_on_an_imperative() {
        let lex = verbs();
        for (form, lemma) in [
            ("parlandogli", "parlare"),
            ("parlandone", "parlare"),
            ("scrivimi", "scrivere"),
            ("prendetelo", "prendere"),
            ("andiamocene", "andare"),
            ("parlagliene", "parlare"),
            // A reflexive-only verb's imperative and infinitive, filed under it.
            ("pentitevi", "pentirsi"),
            ("pentirmi", "pentirsi"),
        ] {
            assert_eq!(reads(form, &lex), lemma, "{form}");
        }
    }

    #[test]
    fn a_monosyllabic_imperative() {
        // The table names the verb, whatever the pack lists: `da` and `di` are prepositions here.
        let lex = verbs();
        for (form, lemma) in [
            ("dammi", "dare"),
            ("fallo", "fare"),
            ("vattene", "andare"),
            ("dimmi", "dire"),
            ("stacci", "stare"),
            ("digli", "dire"),
            ("fagli", "fare"),
            ("stagli", "stare"),
            ("vagli", "andare"),
            ("dammelo", "dare"),
            ("diglielo", "dire"),
            ("dacci", "dare"),
            ("vanne", "andare"),
        ] {
            assert_eq!(reads(form, &lex), lemma, "{form}");
            assert_eq!(reads(form, &lexicon(&[], &[])), lemma, "{form}, no pack");
        }
        // Without doubling, only before `gli`: the five double every other clitic's consonant,
        // so `falo`, `dami`, `dila` and `stami` are none of theirs, whatever the pack files.
        let stare = lexicon(&[("sta", "stare"), ("fa", "fare")], &["stare", "fare"]);
        for form in ["dalo", "dami", "dila", "falo", "stami"] {
            assert_eq!(reads(form, &lex), form, "{form}");
            assert_eq!(reads(form, &stare), form, "{form}");
        }
    }

    #[test]
    fn a_present_indicative_takes_no_clitic() {
        // A clitic follows an imperative, an infinitive, a gerund or a participle, never a
        // present indicative: « Stone » is no `sto` + `ne` (`stare`), « Sami » no `sa` + `mi`
        // (`sapere`), though the pack files `sto` and `sa` under their verbs.
        let lex = lexicon(
            &[
                ("sto", "stare"),
                ("sa", "sapere"),
                ("parlo", "parlare"),
                ("andiamo", "andare"),
                ("parlando", "parlare"),
            ],
            &["stare", "sapere", "parlare", "andare", "dare"],
        );
        for form in ["Stone", "stone", "Sami", "sami", "parlone", "salo"] {
            assert_eq!(reads(form, &lex), form.to_lowercase(), "{form}");
        }
        // A form in `-o` is a base as a gerund or a first-person plural; a monosyllable by its
        // table.
        assert_eq!(reads("andiamocene", &lex), "andare");
        assert_eq!(reads("parlandogli", &lex), "parlare");
        assert_eq!(reads("dammi", &lex), "dare");
    }

    #[test]
    fn ecco() {
        let lex = verbs();
        assert_eq!(reads("eccolo", &lex), "ecco");
        assert_eq!(reads("Eccomi", &lex), "ecco");
        assert_eq!(reads("eccotelo", &lex), "ecco");
        // Not listed, `ecco` is no base.
        assert_eq!(reads("eccolo", &lexicon(&[], &[])), "eccolo");
    }

    #[test]
    fn a_base_the_pack_does_not_list_is_not_split() {
        let lex = verbs();
        assert_eq!(reads("zorbarmi", &lex), "zorbarmi");
        assert_eq!(reads("zorbarlo", &lex), "zorbarlo");
        // A base the pack lists that is no verb: `vita` + `la`.
        assert_eq!(reads("vitala", &lex), "vitala");
        // A word holding an apostrophe or a hyphen carries no clitic.
        assert_eq!(reads("l'ora", &lex), "l'ora");
        assert_eq!(reads("dir-lo", &lex), "dir-lo");
        // A rest of one letter is no base.
        assert_eq!(reads("ami", &lex), "ami");
    }

    #[test]
    fn a_past_participle_with_a_clitic() {
        // D2's candidate, kept by D6's measurement: a regular participle the pack does not file
        // under its verb is read as the verb's infinitive, when the pack lists it.
        let lex = lexicon(
            &[("andato", "andare"), ("sviluppata", "sviluppato")],
            &[
                "sviluppare",
                "formare",
                "vendere",
                "finire",
                "andare",
                "sviluppato",
                "vestire",
            ],
        );
        for (form, lemma) in [
            ("sviluppatasi", "sviluppare"),
            ("Formatasi", "formare"),
            ("vendutolo", "vendere"),
            ("finitala", "finire"),
            ("formatesi", "formare"),
            // A participle the pack files under its verb, in `-o`, is read as a participle too.
            ("andatosene", "andare"),
            ("vestitolo", "vestire"),
        ] {
            assert_eq!(reads(form, &lex), lemma, "{form}");
        }
        // The infinitive must be listed, and a verb.
        assert_eq!(reads("cantatolo", &lex), "cantatolo");
        let noun = lexicon(&[], &["mare"]);
        assert_eq!(reads("maritala", &noun), "maritala");
    }

    #[test]
    fn an_augmentative_is_no_participle_with_ne() {
        // Before a lone `ne` a participle is no base: `-one` is the augmentative's ending
        // (`risultatone`, a big result), never `risultato` + `ne` read as `risultare`.
        let lex = lexicon(
            &[],
            &["risultare", "vestire", "peccare", "gelare", "parlare"],
        );
        for form in ["risultatone", "vestitone", "peccatone", "gelatone"] {
            assert_eq!(reads(form, &lex), form, "{form}");
        }
        // A combined clitic ending in `ne` still reads one: `parlatosene`.
        assert_eq!(reads("parlatosene", &lex), "parlare");
    }

    #[test]
    fn a_participle_the_tables_file_as_a_noun_stays_whole() {
        // `fatto` and `vista` filed as nouns, as the tables may (M8): neither an imperative nor a
        // regular participle, `fattolo` and `vistala` stay as written.
        let lex = lexicon(&[], &["fatto", "vista", "fare", "vedere"]);
        assert_eq!(reads("fattolo", &lex), "fattolo");
        assert_eq!(reads("vistala", &lex), "vistala");
    }

    #[test]
    fn an_unlisted_plural_stays_itself() {
        // D2's step 3 was measured out (D6): an unlisted plural is an unknown word as written,
        // never read as a singular the rule would guess (`thailandesi` is no `thailandeso`).
        let lex = verbs();
        for form in [
            "tavoli",
            "amiche",
            "botteghe",
            "fuochi",
            "laghi",
            "thailandesi",
            "vite",
            "mari",
            "andai",
            "tre",
            "l'ora",
        ] {
            assert_eq!(reads(form, &lex), form, "{form}");
        }
        // A form the pack files under a lemma is that lemma: `porte` is `porta`.
        assert_eq!(reads("porte", &lex), "porta");
        // A capitalised form outside the pack is itself, lowercased.
        assert_eq!(reads("Rossi", &lex), "rossi");
    }

    #[test]
    fn the_lemma_is_lowercase_and_composed() {
        let lex = lexicon(&[], &["città", "perché"]);
        // `città` written with `a` + U+0300 reaches the pack's composed form.
        assert_eq!(reads("citta\u{300}", &lex), "città");
        assert_eq!(reads("CITTA\u{300}", &lex), "città");
        assert_eq!(reads("Perche\u{301}", &lex), "perché");
        // Unlisted and decomposed, the form itself, composed.
        assert_eq!(reads("universita\u{300}", &lex), "università");
        assert_eq!(reads("pieta\u{300}", &lex), "pietà");
        // The typographic apostrophe reads as the straight one.
        assert_eq!(reads("L\u{2019}uomo", &lex), "l'uomo");
    }
}
