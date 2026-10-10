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

//! French invariance baseline (`docs/lingua/language-matrix-programme.md`, change 39,
//! add-lingua-french-baseline).
//!
//! French is a studied language with its own analysis (add-lingua-french-analysis, analyser
//! `1.0.0`), at analyser `1.1.0` since its detection guard keeps Catalan, Occitan and Romanian
//! blocks out (add-lingua-french-detection-guard): the `mixte` page's Occitan block is not
//! analysed, its Catalan block whichlang reads as Italian. At `1.2.0` since a word is read
//! without its soft hyphens (ignore-lingua-soft-hyphens), which no page of the corpus holds. Its tokenisation pre-pass
//! (add-lingua-french-tokenisation) reads the narrow no-break space as a space, splits an elided
//! word from the word it is joined to and reads it as the word it stands for, each piece with its
//! own span (`l'homme` → `le` + `homme`; `aujourd'hui` whole), `au`/`aux` as `à` + `le`/`les`
//! sharing a span, `du`/`des` whole, a hyphenated inversion as words (`dit-il` → `dit` + `il`), and
//! every word in NFC, its span the source's. Its cascade reads the pack's forms, then an unlisted
//! lowercase plural as its unlisted singular, then the form; its closed classes flag a phrase
//! gloss's function words, « pas » among them (M21); a document's names are set aside — Spanish's
//! rule, a capital after an elided piece, a hyphenated run as one form.
//! This freezes what the engine makes of raw French text today, over a thirteen-page corpus
//! (`baseline/pages-fr.txt`, its sources in `support/french.rs`), so that each later change of
//! the French stage shows its effect as the diff of a re-bless. It is not French support: no pair
//! studying French is listed.
//!
//! The pack is built from the committed French tables, `tables/fr/` and `tables/fr-en/`
//! (add-lingua-pack-fr-en, which handed the baseline over from the hand-written fixture,
//! `scripts/lingua-data/testdata/fr-en/`, kept for the tests that build it); the engine starts on
//! the real es-en pack, built from the committed tables, as an English-native reader's engine does.
//!
//! A pull request that changes `baseline/fr-en.golden` re-blesses it with
//! `LINGUA_BLESS=1 cargo test -p lingua-wasm --test french_baseline` and says why: a French rule
//! that bumps French's analyser version (and the fixture's manifest with it), a change to the
//! committed French tables (`tables/fr/` or `tables/fr-en/`: a dictionary update, a reduction rule,
//! the readings or the levels), the fixture replaced by the committed tables (done, once), an
//! es-en update, which moves the `beside es-en` line alone, or probes a change of the French stage
//! adds, the others unmoved (add-lingua-french-expression-keys, add-lingua-french-word-card: the
//! card's 21 grammar probes, 40 lemmas and « l’homme »; match-lingua-french-elided-pieces: four
//! phrase probes, « de l’eau », « Il a décidé de le faire », « parce qu’il pleut » and « d’un
//! hiver »). `lingua-pack-update` re-blesses on its own
//! branch. The extension's `test/word-card-fr-en.spec.ts` renders this golden's grammar and phrase
//! probes, and is re-blessed with it (`yarn vitest run test/word-card-fr-en.spec.ts -u`).
//!
//! Host only: the pack builder is native (C zstd), and the wasm surface is the same methods.

#![cfg(not(target_arch = "wasm32"))]

mod support;

use std::sync::OnceLock;

use lingua_core::analysis::FRENCH_ANALYZER_VERSION;
use lingua_core::analysis::language::StudiedLanguage;
use lingua_core::analysis::tokenize::tokenize;
use support::french::{CARD_PROBES, FRENCH, MORE_LEMMAS, REFERENCE_GRAMMAR};
use support::{PackSource, first_difference, testdata_pack};
use unicode_normalization::UnicodeNormalization;

/// The `technique` page's block committed in NFD.
const NFD_BLOCK: usize = 3;

fn fr() -> Option<String> {
    Some("fr".to_owned())
}

/// The golden as this build renders it, once for every test of the target.
fn rendered() -> &'static str {
    static RENDERED: OnceLock<String> = OnceLock::new();
    RENDERED.get_or_init(|| FRENCH.render(Some("fr")))
}

#[test]
fn french_output_has_not_moved() {
    let actual = rendered();
    let Some(expected) = FRENCH.bless_or_read(actual) else {
        return;
    };
    assert!(
        actual == expected,
        "French output moved (add-lingua-french-baseline: the French golden moves only with a \
         reason).\n\
         First difference — {}\n\
         If this pull request means to change French output (a French rule that bumps French's \
         analyser version, a change to the committed French tables, tables/fr/ or tables/fr-en/, \
         or an es-en update), re-bless with\n  \
         LINGUA_BLESS=1 cargo test -p lingua-wasm --test french_baseline\n\
         and say why in the pull request. Otherwise the change is wrong.",
        first_difference(&expected, actual)
    );
}

/// The probes are the reference's, then the card's 21, then the 40 lemmas as themselves, each
/// probe once (add-lingua-french-word-card D9); the phrases end on « l’homme ».
#[test]
fn the_probes_are_the_references_then_the_cards_then_forty_lemmas() {
    let (reference, rest) = FRENCH.grammar.split_at(REFERENCE_GRAMMAR.len());
    let (card, more) = rest.split_at(CARD_PROBES.len());
    assert_eq!(reference, REFERENCE_GRAMMAR);
    assert_eq!(reference.len(), 31);
    assert_eq!(card, CARD_PROBES);
    assert_eq!(card.len(), 21);
    assert_eq!(more.len(), 40);
    for ((written, lemma), expected) in more.iter().zip(MORE_LEMMAS) {
        assert_eq!((written, lemma), (expected, expected));
    }
    let mut probes = FRENCH.grammar.to_vec();
    probes.sort_unstable();
    probes.dedup();
    assert_eq!(probes.len(), FRENCH.grammar.len(), "each probe once");
    assert_eq!(FRENCH.phrases.last(), Some(&"l\u{2019}homme"));
}

#[test]
fn the_corpus_reads_as_expected() {
    FRENCH.check_corpus();
    // One block of the technical page is committed in NFD on purpose: an editor or a formatter
    // that composes the file must fail here, not move the golden unnoticed.
    let pages = FRENCH.pages();
    let (_, technique) = pages
        .iter()
        .find(|(name, _)| name == "technique")
        .expect("the technique page");
    let block = &technique[NFD_BLOCK];
    assert!(
        block.contains('\u{0301}'),
        "the technique page's NFD block was composed"
    );
    assert_ne!(
        block.nfc().collect::<String>(),
        *block,
        "NFC composes the NFD block"
    );
    assert_eq!(
        block.nfd().collect::<String>(),
        *block,
        "the NFD block is wholly decomposed"
    );
    // The fiction and Proust pages are set in French punctuation.
    for name in ["fiction", "proust"] {
        let (_, blocks) = pages.iter().find(|(n, _)| n == name).expect(name);
        let text = blocks.concat();
        assert!(
            text.contains('\u{2019}'),
            "{name}: the typographic apostrophe"
        );
        assert!(
            text.contains("\u{202f}»"),
            "{name}: a narrow no-break space before »"
        );
    }
    let fiction = pages
        .iter()
        .find(|(n, _)| n == "fiction")
        .map(|(_, blocks)| blocks.concat())
        .expect("fiction");
    assert!(fiction.contains("\u{202f}?") && fiction.contains("\u{202f}!"));
}

/// The page's blocks, analysed as French by `engine`, as JSON.
fn analysed(engine: &lingua_wasm::LinguaEngine, page: &str) -> serde_json::Value {
    let (_, blocks) = FRENCH
        .pages()
        .into_iter()
        .find(|(name, _)| name == page)
        .unwrap_or_else(|| panic!("the {page} page"));
    serde_json::from_str(&engine.analyse(blocks, fr()).expect("analysed")).expect("JSON")
}

/// The page's tokens a block has at a byte offset: (surface, lemma, end).
fn tokens_at(page: &serde_json::Value, block: usize, start: usize) -> Vec<(String, String, u64)> {
    page["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .filter(|t| t["block"] == block && t["start"] == start)
        .map(|t| {
            (
                t["surface"].as_str().expect("surface").to_owned(),
                t["lemma"].as_str().expect("lemma").to_owned(),
                t["end"].as_u64().expect("end"),
            )
        })
        .collect()
}

/// `(surface, lemma, end)`, owned.
fn token(surface: &str, lemma: &str, end: usize) -> (String, String, u64) {
    (surface.to_owned(), lemma.to_owned(), end as u64)
}

/// The byte offset of `word` in a page's block, as a whole word.
fn offset_in(page: &str, block: usize, word: &str) -> usize {
    let (_, blocks) = FRENCH
        .pages()
        .into_iter()
        .find(|(name, _)| name == page)
        .unwrap_or_else(|| panic!("the {page} page"));
    let text = &blocks[block];
    text.match_indices(word)
        .map(|(at, _)| at)
        .find(|&at| {
            let before = text[..at].chars().next_back();
            let after = text[at + word.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
        .unwrap_or_else(|| panic!("{word:?} in {page}[{block}]"))
}

/// The classes of a page's tokens written `surface`.
fn classes_of(page: &serde_json::Value, surface: &str) -> Vec<String> {
    page["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .filter(|t| t["surface"] == surface)
        .map(|t| t["class"].as_str().expect("class").to_owned())
        .collect()
}

/// A phrase gloss's (lemma, function word) pairs.
fn function_words(engine: &lingua_wasm::LinguaEngine, text: &str) -> Vec<(String, bool)> {
    let glossed: serde_json::Value =
        serde_json::from_str(&engine.phrase_gloss(text, fr()).expect("glossed")).expect("JSON");
    glossed["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .map(|t| {
            (
                t["lemma"].as_str().expect("lemma").to_owned(),
                t["function_word"].as_bool().expect("function_word"),
            )
        })
        .collect()
}

#[test]
fn french_has_its_pre_pass_and_its_analysis() {
    let mut engine = FRENCH.loaded();
    assert_eq!(engine.languages(), r#"["es","fr"]"#, "es-en beside fr-en");
    assert_eq!(engine.native_language(), "en");

    // An elision is two pieces, each with its own span, the elided one read as `le`. French
    // reports its own version, no longer one of the baseline's `0.x` (add-lingua-french-analysis
    // D5).
    let elisions = analysed(&engine, "elisions");
    let version = elisions["analyzer_version"].as_str().expect("version");
    assert_eq!(version, FRENCH_ANALYZER_VERSION);
    assert!(!version.starts_with("0."), "{version}");
    assert_eq!(tokens_at(&elisions, 0, 0), [token("Le", "le", 2)]);
    assert_eq!(tokens_at(&elisions, 0, 2), [token("homme", "homme", 7)]);

    // `au` is `à` + `le` sharing its span; `du` is whole.
    let contractions = analysed(&engine, "contractions");
    assert_eq!(contractions["analyzer_version"], FRENCH_ANALYZER_VERSION);
    let au = offset_in("contractions", 0, "au");
    assert_eq!(
        tokens_at(&contractions, 0, au),
        [token("à", "à", au + 2), token("le", "le", au + 2)]
    );
    let du = offset_in("contractions", 0, "du");
    assert_eq!(tokens_at(&contractions, 0, du), [token("du", "du", du + 2)]);

    // An inversion is read as words, each with its own span.
    let inversions = analysed(&engine, "inversions");
    let dit = offset_in("inversions", 0, "dit-il");
    assert_eq!(
        tokens_at(&inversions, 0, dit),
        [token("dit", "dire", dit + 3)]
    );
    assert_eq!(
        tokens_at(&inversions, 0, dit + 4),
        [token("il", "il", dit + 6)]
    );

    // The narrow no-break space of French punctuation is in no token and no span.
    for name in ["fiction", "proust"] {
        let (_, blocks) = FRENCH
            .pages()
            .into_iter()
            .find(|(n, _)| n == name)
            .expect(name);
        let page = analysed(&engine, name);
        for t in page["tokens"].as_array().expect("tokens") {
            let block = &blocks[t["block"].as_u64().expect("block") as usize];
            let span =
                &block[t["start"].as_u64().unwrap() as usize..t["end"].as_u64().unwrap() as usize];
            assert!(
                !span.contains('\u{202f}') && !t["surface"].as_str().unwrap().contains('\u{202f}'),
                "{name}: {t}"
            );
        }
    }

    // The closed classes (D3): the `homographes` page's negation, « pas » both times (M21), and
    // its `le`, flagged; the `fiction` page's `Personne` is not.
    let (_, homographes) = FRENCH
        .pages()
        .into_iter()
        .find(|(n, _)| n == "homographes")
        .expect("homographes");
    let negation = homographes
        .iter()
        .find(|block| block.starts_with("Il ne fait pas un pas"))
        .expect("the negation's block");
    let flags = function_words(&engine, negation);
    for lemma in ["ne", "pas", "le", "il", "un", "son"] {
        assert!(
            flags.iter().any(|(l, flagged)| l == lemma && *flagged),
            "{lemma}: {flags:?}"
        );
    }
    assert_eq!(
        flags
            .iter()
            .filter(|(l, flagged)| l == "pas" && *flagged)
            .count(),
        2,
        "{flags:?}"
    );
    assert!(
        flags.iter().all(|(l, flagged)| !*flagged || l != "faire"),
        "{flags:?}"
    );
    let (_, fiction) = FRENCH
        .pages()
        .into_iter()
        .find(|(n, _)| n == "fiction")
        .expect("fiction");
    let personne = function_words(&engine, &fiction[1]);
    assert_eq!(personne.first(), Some(&("personne".to_owned(), false)));
    assert!(personne.iter().any(|(l, flagged)| l == "ne" && *flagged));

    // The names rule (D4) on the `noms` page, over the committed tables. French's dictionary words
    // hold no names (refine-lingua-fr-en-glosses D2): a lemma fr-en glosses by a proper noun's
    // senses alone — `Paris`, `Jean-Pierre`, `Saint-Étienne` — is no dictionary word, so the rule
    // sets it aside, as it sets `Myriel`, which the lexicon does not hold; a name with a common
    // word's senses among its own — `Lot`, `Aube`, `Orange`, `Vienne` — stays a word to learn, and
    // `Mme` opens its block. The rule's French readings on the fixture are
    // `the_names_rule_reads_french_s_evidence`'s.
    let noms = analysed(&engine, "noms");
    for name in ["Paris", "Jean-Pierre", "Saint-Étienne", "Myriel"] {
        assert_eq!(
            classes_of(&noms, name),
            ["ProperNounOutOfLexicon"],
            "{name}"
        );
    }
    for word in ["Lot", "Aube", "Orange", "Vienne", "Mme"] {
        assert_eq!(classes_of(&noms, word), ["Unknown"], "{word}");
    }

    // The detection guard (add-lingua-french-detection-guard): of the `mixte` page's seven
    // blocks, the two French ones alone are analysed — not its English, Spanish and Italian
    // blocks, nor its Catalan one (whichlang reads it as Italian) or its Occitan one (read as
    // French, refused by the guard).
    let mixte = analysed(&engine, "mixte");
    let mixte_blocks: std::collections::BTreeSet<u64> = mixte["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .map(|t| t["block"].as_u64().expect("block"))
        .collect();
    assert_eq!(mixte_blocks.into_iter().collect::<Vec<_>>(), [0, 6]);

    // The reader's records are French, under the profile the engine started with (Spanish,
    // English native): the backup names French, so it is schema version 3, and the golden's
    // `backup` probe records it.
    assert_eq!(engine.studied_languages(), r#"["es"]"#);
    assert!(engine.backup().starts_with("{\n  \"schema_version\": 2,"));
    engine
        .set_status_at("phare", "known", 1_790_000_000_000.0, fr())
        .expect("a French status");
    assert!(engine.backup().starts_with("{\n  \"schema_version\": 3,"));
    let backup = rendered()
        .split("\n### backup\n")
        .nth(1)
        .expect("the backup probe");
    assert!(
        backup.starts_with("{\n  \"schema_version\": 3,"),
        "{backup}"
    );
}

#[test]
fn the_names_rule_reads_french_s_evidence() {
    // The names rule (add-lingua-french-analysis D4) over the hand-written fixture, whose 70
    // glosses leave the `noms` page's names out of French's dictionary words: Spanish's rule sets
    // `Paris` and `Lot` aside, the elided `l'` gives `Aube`'s evidence, and the runs are one form
    // each; `Orange` and `Vienne` are dictionary words (a gloss, `venir`'s), `Mme` only opens its
    // block. Over the committed tables, French's dictionary words leave out the names fr-en
    // glosses alone, and `Lot` and `Aube` are words, a common sense among theirs
    // (`french_has_its_pre_pass_and_its_analysis`).
    let engine = support::Scenario::engine(&[
        ("es-en", PackSource::Tables.pack("es-en")),
        ("fr-en", PackSource::Testdata.pack("fr-en")),
    ]);
    let noms = analysed(&engine, "noms");
    for name in ["Paris", "Lot", "Aube", "Jean-Pierre", "Saint-Étienne"] {
        assert_eq!(
            classes_of(&noms, name),
            ["ProperNounOutOfLexicon"],
            "{name}"
        );
    }
    for word in ["Orange", "Vienne", "Mme"] {
        assert_eq!(classes_of(&noms, word), ["Unknown"], "{word}");
    }
}

#[test]
fn the_fixture_lists_every_word_the_pre_pass_writes() {
    // Each elided form of the pre-pass before a vowel — `s'` before `il`, `m'` and `t'` after a
    // hyphen — and `au` and `aux`, tokenised as French, give only forms of the fixture: no word the
    // pre-pass writes is set aside as a name for want of a form (change 43 checks the real tables).
    let dir = PackSource::Testdata.manifest_dir("fr-en");
    let forms: std::collections::BTreeSet<String> = std::fs::read_to_string(dir.join("forms.tsv"))
        .expect("forms.tsv")
        .lines()
        .filter_map(|line| line.split('\t').next())
        .filter(|form| !form.is_empty())
        .map(str::to_owned)
        .collect();
    let pack = lingua_core::packs::Pack::load(&testdata_pack(&dir, "fr-en")).expect("the fixture");
    let line = "c'est ç'a d'avoir j'ai jusqu'à l'homme lorsqu'il m'a n'est puisqu'on qu'il \
                quoiqu'elle s'en s'il t'a prends-m'en va-t'en au aux";
    let tokens = tokenize(line, StudiedLanguage::French, pack.lexicon());
    let read: Vec<String> = tokens.iter().map(|t| t.text.to_lowercase()).collect();
    for written in [
        "ce", "ça", "de", "je", "jusque", "le", "lorsque", "me", "ne", "puisque", "que", "quoique",
        "se", "si", "te", "moi", "toi", "à", "les",
    ] {
        assert!(read.iter().any(|r| r == written), "{written:?} in {read:?}");
    }
    let missing: Vec<&String> = read.iter().filter(|r| !forms.contains(*r)).collect();
    assert!(missing.is_empty(), "not forms of the fixture: {missing:?}");
}

#[test]
fn the_nfd_block_s_memoire_is_glossed_once_french_composes_it() {
    // The committed tables gloss `mémoire` « memory; memo; dissertation, paper; … », a word the
    // corpus has only in its NFD block (change 39's D5): French's pre-pass reads it in NFC
    // (add-lingua-french-analysis D1), so the token is the pack's word, glossed, and its span still
    // covers the decomposed bytes. The decomposed spelling itself is no word of the pack.
    let engine = FRENCH.loaded();
    let technique = analysed(&engine, "technique");
    let decomposed = "me\u{301}moire";
    let at = offset_in("technique", NFD_BLOCK, decomposed);
    let token = technique["tokens"]
        .as_array()
        .expect("tokens")
        .iter()
        .find(|t| t["block"] == NFD_BLOCK && t["start"] == at)
        .expect("the NFD block's `mémoire`");
    assert_eq!(token["surface"], "m\u{e9}moire", "composed");
    assert_eq!(token["lemma"], "m\u{e9}moire");
    let gloss = token["gloss"].as_str().expect("a gloss");
    assert!(gloss.starts_with("memory; "), "{token}");
    assert_eq!(token["end"], at + decomposed.len(), "the decomposed bytes");
    // No token of the block is decomposed any more.
    for t in technique["tokens"].as_array().expect("tokens") {
        if t["block"] == NFD_BLOCK {
            let surface = t["surface"].as_str().expect("surface");
            assert_eq!(surface.nfc().collect::<String>(), surface, "{t}");
        }
    }
    assert_eq!(
        engine
            .gloss("m\u{e9}moire", fr())
            .expect("glossed")
            .as_deref(),
        Some(gloss),
        "the pack glosses the composed form"
    );
    assert_eq!(
        engine.gloss(decomposed, fr()).expect("looked up"),
        None,
        "the decomposed spelling has no gloss"
    );
}

#[test]
fn a_fixture_left_behind_its_analyser_names_its_manifest() {
    // A French rule bumps French's analyser version; a fixture manifest left at the old one is
    // refused by the core. The harness names the manifest to bump, not wasm-bindgen's host panic.
    let committed = PackSource::Testdata.manifest_dir("fr-en");
    let behind = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("fr-en-fixture-behind");
    let _ = std::fs::remove_dir_all(&behind);
    std::fs::create_dir_all(&behind).expect("a scratch fixture");
    for entry in std::fs::read_dir(&committed).expect("the fr-en fixture") {
        let entry = entry.expect("a fixture file");
        std::fs::copy(entry.path(), behind.join(entry.file_name())).expect("copied");
    }
    let manifest_path = behind.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).expect("manifest"))
            .expect("JSON");
    assert_eq!(manifest["meta"]["analyzer_version"], "1.2.0");
    manifest["meta"]["analyzer_version"] = "0.0.9".into();
    std::fs::write(&manifest_path, manifest.to_string()).expect("written");

    let Err(refused) = std::panic::catch_unwind(|| testdata_pack(&behind, "fr-en")) else {
        panic!("a fixture behind French's analyser is refused");
    };
    let message = refused.downcast_ref::<String>().expect("a formatted panic");
    assert!(
        message.contains("pack built for analyzer 0.0.9 but this core is 1.2.0"),
        "{message}"
    );
    assert!(
        message.contains("bump `analyzer_version` in")
            && message.contains("fr-en-fixture-behind/manifest.json"),
        "{message}"
    );
    assert!(!message.contains("wasm-bindgen"), "{message}");
    // The committed fixture loads.
    assert!(!testdata_pack(&committed, "fr-en").is_empty());
}
