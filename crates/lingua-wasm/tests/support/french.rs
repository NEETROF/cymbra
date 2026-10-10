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

//! The French invariance baseline's scenario (`french_baseline.rs`): its corpus, probes and
//! reader (add-lingua-french-baseline D5).
//!
//! The corpus, `baseline/pages-fr.txt`, is authored for these tests, page by page, around what a
//! later change of the French stage must show, with two exceptions, quoted from texts in the
//! public domain:
//! - the `proust` page: the opening of Marcel Proust, *Du côté de chez Swann* (Grasset, 1913),
//!   « Longtemps, je me suis couché de bonne heure. » to « … la rivalité de François Ier et de
//!   Charles Quint. », its one paragraph split into two blocks at « Et, une demi-heure après ».
//!   Proust died in 1922, so the work has been in the public domain in France since 1987 (the
//!   50-year term plus the wartime extensions, which end around 30 September 1987), was not
//!   re-protected by the 1997 move to 70 years, and was published before 1929, so it is in the
//!   public domain in the United States;
//! - the first block of the `noms` page: the first sentence of Victor Hugo, *Les Misérables*
//!   (1862), « En 1815, M. Charles-François-Bienvenu Myriel était évêque de Digne. ». Hugo died in
//!   1885.
//!
//! The `fiction` and `proust` pages are set as French is printed: the typographic apostrophe `’`,
//! the narrow no-break space (U+202F) before `?`, `!`, `;` and `»` and after `«`, the no-break
//! space (U+00A0) before `:` and after the dialogue dash. The other pages are typed as the web
//! often has them, with the straight apostrophe. One block of the `technique` page is committed
//! in NFD on purpose (accents as combining marks); `french_baseline.rs` asserts it still is.
//! The tables gloss one of its words, `mémoire` (« memory; memo; dissertation, paper; … »), which
//! the corpus has nowhere else: read as it came, the decomposed word is not the pack's, so change
//! 41's NFC shows in the golden as a gloss that appears, not only as bytes that move.
//!
//! The pack is built from the committed French tables, `tables/fr/` and `tables/fr-en/`
//! (add-lingua-pack-fr-en: French's forms, ranks, readings and levels, and fr-en's English glosses,
//! sense runs and expressions), as the real es-en pack the engine starts on is, as an
//! English-native reader's does. Until that change it was the hand-written fixture,
//! `scripts/lingua-data/testdata/fr-en/`, which stays for the tests that build it
//! (`french_baseline.rs`, `lingua-pack`'s `pipeline_testdata.rs`). A change to the committed French
//! tables can move the golden, and then re-blesses it.
//!
//! At analyser `1.0.0` French has its own analysis (add-lingua-french-analysis). Its tokenisation
//! pre-pass (add-lingua-french-tokenisation) reads the narrow no-break space as a space, an elided
//! word as a piece of its own read as the word it stands for (`l'homme` → `le` + `homme`),
//! `au`/`aux` as `à` + `le`/`les`, a hyphenated inversion as words (`dit-il` → `dit` + `il`), and
//! every word in NFC, so the NFD block's `mémoire` is the pack's. Its cascade reads the pack's
//! forms, then an unlisted lowercase plural as its singular (`syndicats` → `syndicat`), then the
//! form; its closed classes flag a phrase gloss's function words, « pas » among them (M21); its
//! names rule sets the `noms` page's `Paris`, `Lot`, `Aube`, `Jean-Pierre` and `Saint-Étienne`
//! aside, and keeps `Orange` and `Vienne` (dictionary words) and `Mme` (at a block's head only).
//! The tables hold every word the pre-pass writes (`lingua-pack`'s `committed_tables.rs` holds
//! it; `french_baseline.rs` holds the fixture to it too), and expressions holding `au` and an
//! elision, `au revoir` and `coup d'œil`, a word the pre-pass splits, `d'abord` (offered by
//! fr-en's reducer), an expression of six tokens, `au fur et à mesure`, and no `à la`, which
//! fr-en's reducer leaves out: its one sense, « in the style of », needs the word after it. The
//! pack keys expressions as French is read — the builder hands each headword to the core's French
//! reading, its determiners written as the pre-pass gives them, up to seven tokens — and names
//! each by its headword where its key differs (add-lingua-french-expression-keys): the probes
//! « Au revoir », « D’abord », « au fur et à mesure » and « à cause des » find their expression,
//! « il y a » and « il y avait » answer `il y a`, the Proust sentence `de bonne heure`,
//! « jusqu'au soir » `jusqu'à`, « à la maison » the section's own `à la maison`, and « au marché »
//! nothing. « un coup d’œil » meets `un coup` before `coup d'œil`, and « du pain et des œufs »
//! `et des`: senses their keys do not hold the context of, listed for
//! `refine-lingua-fr-en-glosses` (add-lingua-pack-fr-en D7).
//!
//! At analyser `1.1.0` French's detection guard (add-lingua-french-detection-guard) refuses the
//! `mixte` page's Occitan block, which whichlang reads as French; its Catalan block whichlang reads
//! as Italian, guard or not.

use super::{Card, PackSource, Scenario};

/// Dictionary forms whose gloss the card shows: the verbs every page uses, M8's homographs,
/// elided and contracted words, compounds, words from the corpus. Some have none in the pack, on
/// purpose.
const LEMMAS: &[&str] = &[
    "être",
    "avoir",
    "aller",
    "faire",
    "pouvoir",
    "dire",
    "venir",
    "prendre",
    "voir",
    "vivre",
    "porte",
    "porter",
    "fils",
    "couvent",
    "couver",
    "vis",
    "as",
    "car",
    "pas",
    "son",
    "été",
    "est",
    "homme",
    "l'homme",
    "aujourd'hui",
    "presqu'île",
    "au",
    "aux",
    "du",
    "des",
    "le",
    "de",
    "à",
    "y",
    "maison",
    "phare",
    "marin",
    "tempête",
    "peut-être",
    "arc-en-ciel",
    "porte-monnaie",
    "bougie",
    "évêque",
    "mdr",
    "omelette",
];

/// Selections a reader glosses: expressions, the Proust sentence, elided, contracted and inverted
/// selections — one set with the narrow no-break space —, the expressions holding `au` and an
/// elision, a word the pre-pass splits, six tokens, another tense and an article as written
/// (add-lingua-french-expression-keys), the `homographes` page's « pas » and `son`, both flagged
/// (M21 and M8's cost), and the
/// `fiction` page's `Personne`, not, and the blocks that are not French (English, Spanish,
/// Catalan, Occitan, Italian).
const PHRASES: &[&str] = &[
    "pommes de terre",
    "il y a",
    "tout de suite",
    "à cause des",
    "mettre à jour",
    "tout le monde",
    "Longtemps, je me suis couché de bonne heure.",
    "l'homme qu'il attendait",
    "jusqu'au soir",
    "Aujourd'hui",
    "au marché",
    "du pain et des œufs",
    "dit-il",
    "Y a-t-il encore du café",
    "S\u{2019}il pleut, viendras-tu\u{202f}?",
    "Au revoir",
    "un coup d\u{2019}œil",
    "D\u{2019}abord",
    "au fur et à mesure",
    "il y avait",
    "à la maison",
    "Il ne fait pas un pas sans son chien, et le son de sa voix le rassure.",
    "Personne au village ne se souvenait",
    "The lighthouse stood",
    "El faro se alzaba",
    "El far s'alçava",
    "Lo far se quilhava",
    "Il faro si ergeva",
];

/// (word as written, dictionary form) pairs a word card asks the grammar of: each answers its
/// readings (add-lingua-french-grammar-tables) and its gloss, and the pieces a split word is made
/// of — an elided word handed alone (`l’`) is one piece, `le`; `au` is `à` and `le`.
const GRAMMAR: &[(&str, &str)] = &[
    ("est", "être"),
    ("sont", "être"),
    ("était", "être"),
    ("fut", "être"),
    ("soyez", "être"),
    ("été", "être"),
    ("été", "été"),
    ("a", "avoir"),
    ("ai", "avoir"),
    ("as", "avoir"),
    ("eût", "avoir"),
    ("va", "aller"),
    ("allez", "aller"),
    ("fait", "faire"),
    ("faites", "faire"),
    ("pût", "pouvoir"),
    ("dit", "dire"),
    ("vînmes", "venir"),
    ("prenez", "prendre"),
    ("porte", "porter"),
    ("porte", "porte"),
    ("vis", "vivre"),
    ("vis", "voir"),
    ("couvent", "couver"),
    ("couvent", "couvent"),
    ("fils", "fils"),
    ("au", "au"),
    ("du", "du"),
    ("l'homme", "homme"),
    ("l\u{2019}", "le"),
    ("au", "à"),
];

/// The pages analysed again for the reader with a history.
const READER_PAGES: &[&str] = &["actualites", "homographes", "elisions", "mixte"];

const PAGE_NAMES: &[&str] = &[
    "actualites",
    "fiction",
    "proust",
    "homographes",
    "elisions",
    "contractions",
    "inversions",
    "noms",
    "informel",
    "recette",
    "technique",
    "mixte",
    "grammaire",
];

pub const FRENCH: Scenario = Scenario {
    pair: "fr-en",
    pack: PackSource::Tables,
    beside: &["es-en"],
    test: "french_baseline",
    pages: "pages-fr.txt",
    page_names: PAGE_NAMES,
    lemmas: LEMMAS,
    phrases: PHRASES,
    grammar: GRAMMAR,
    reader_pages: READER_PAGES,
    statuses: &[
        ("phare", "known", 1.0),
        ("marin", "learning", 2.0),
        ("mdr", "ignored", 3.0),
        ("fenêtre", "known", 4.0),
        ("village", "known", 5.0),
        // Withdrawn: « Remettre à apprendre ».
        ("village", "", 6.0),
    ],
    exposures: ["mer", "tempête", "horizon", "lumière", "vague"],
    cards: [
        Card {
            lemma: "horizon",
            form: "horizon",
            sentence: "Il resta à regarder l’horizon.",
            url: "https://example.com/fiction",
            gloss: Some("horizon"),
        },
        Card {
            lemma: "récolte",
            form: "récoltes",
            sentence: "Le prix du pain a augmenté à cause des mauvaises récoltes.",
            url: "https://example.com/contractions",
            gloss: None,
        },
        Card {
            lemma: "rassurer",
            form: "rassure",
            sentence: "Le son de sa voix le rassure.",
            url: "",
            gloss: None,
        },
    ],
    reader_phrase: "Le vieux marin ferma les fenêtres du phare",
};
