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

//! The Italian invariance baseline's scenario (`italian_baseline.rs`): its corpus, probes and
//! reader (add-lingua-italian-baseline D5). Italian has its tokenisation pre-pass since `0.2.0`
//! (add-lingua-italian-tokenisation): elided pieces, the articulated prepositions split but for
//! the `di` family, truncations read as their word; its probes gained an elided piece and a
//! contraction a word card is handed alone, and three phrases (D7). Italian has its own
//! analysis at `1.0.0` (add-lingua-italian-analysis): its pre-pass reads every word in NFC, its
//! cascade reads a verb's clitic pronouns off an unlisted form, its closed classes flag a phrase
//! gloss's function words and an Italian document's names are set aside; its probes gain two
//! phrases with a clitic and a grammar pair (D7). Italian's detection guard, at `1.1.0`
//! (add-lingua-italian-detection-guard), takes the `misto` page's Catalan and Galician blocks out
//! of its analysis. At `1.2.0` (read-lingua-italian-unmarked-spellings) the apostrophe that stands
//! for an accent reads as the accented word: the `informale` page moves, and nothing else. At
//! `1.3.0` (keep-lingua-italian-acronyms-and-names) a word standing alone in capitals reads as the
//! acronym the pack lists: the fixture lists `NATO` beside `nato` (*nascere*), a phrase probe and a
//! grammar probe show it, and no page moves — the `notizie` page writes `ONU`, `RAI`, `PIL`, `UE`
//! and `ISTAT`, none a row. At `1.4.0` (refine-lingua-italian-reading, an Italian-specialist review
//! the owner commissioned and approved on 2026-10-24) four sentences are rewritten as Italian writes
//! them and four optional ones taken (its D1, below); the pre-pass reads eighteen more elided forms,
//! `coll'` and `degl'`, the books' truncations `ca'`, `de'`, `que'`, the truncated articulated
//! prepositions (`ne' campi` → `in` + `i`), and `mo'` and `dai` by their context; fifteen closed-class
//! words join the tables. The fixture gains the words the pre-pass now writes (`trenta`, `quale`,
//! `quei`, `mò` …) and `pò` → `poco`; the probes gain a phrase per rule — « Dai, andiamo! », « a
//! mo' di esempio », « ne' campi » … —, « davanti alla porta » and « ossia il primo » for the closed
//! words, whose flags a phrase gloss shows (a page's tokens carry none), and three grammar pairs.
//! With add-lingua-italian-expression-keys (D9), Italian's expressions are keyed and matched as
//! Italian is read, at no new analyser version: the fixture gains ten expressions — `al dente`,
//! `alla romana`, `fare l'amore`, `un po'` and `un poco`, `d'accordo`, `anch'io`, `valere la
//! pena` and `vale la pena`, `vicino a` — and the words they need, its `pack_version` moving, and
//! the probes a phrase per rule: the article of « al » inside an expression and after one, the
//! `di` retry, an elided piece against a word in full, a truncation, the headword written as the
//! page and a word the pre-pass splits.
//!
//! The corpus, `baseline/pages-it.txt`, is authored for these tests, page by page, around what a
//! later change of the Italian stage must show, with two exceptions, quoted from texts in the
//! public domain:
//! - the `manzoni` page: the opening of Alessandro Manzoni, *I promessi sposi*, in its 1840
//!   edition (Guglielmini e Redaelli, Milan, 1840–1842), « Quel ramo del lago di Como, che volge
//!   a mezzogiorno, … » to « … dagli altri monti di nome più oscuro e di forma più comune. »: the
//!   first two sentences of its first paragraph, one block each — the paragraph runs on for some
//!   seven hundred words —, in the edition's spellings (`talchè`, `purchè`) and with its
//!   typographic apostrophe. Manzoni died in 1873, so the work has been in the public domain in
//!   Italy since 1944 (the 70-year term), and was published before 1929, so it is in the public
//!   domain in the United States;
//! - the first block of the `nomi` page: the first lines of Carlo Collodi, *Le avventure di
//!   Pinocchio* (first published by Felice Paggi, Florence, 1883), « — C’era una volta.... — Un
//!   re! — diranno subito i miei piccoli lettori. — No, ragazzi, avete sbagliato. C’era una volta
//!   un pezzo di legno. », its three lines of dialogue joined in one block, its four dots as the
//!   transcription prints them. Collodi died in 1890, so the work has been in the public domain in
//!   Italy since 1961, and was published before 1929, so it is in the public domain in the United
//!   States.
//!
//! Checked on 2026-10-23: the Manzoni against it.wikisource's proofread transcription of the 1840
//! edition, the Collodi against its transcription of the Bemporad edition. Read again on 2026-10-24
//! by the Italian-specialist review (refine-lingua-italian-reading D1): the sources and the licences
//! are right.
//!
//! The authored pages as the review rewrote them (D1): `grammatica`'s letters are the ones « mi hai
//! scritto », and it gains the courtesy form, « La ringraziamo, Le risponderemo al più presto. »;
//! `ricetta`'s *frittata* is eggs beaten with parmesan; `tecnico`'s NFD block speaks of the service's
//! `affidabilità`; `elisioni` asks « Com'è possibile che non sia più nell'armadio? » and answers « E
//! sarà tutt'altro che noiosa. »; `notizie` sets its percentages with no space (« 0,3% »); and
//! `informale` gains « aspetta un pò, qual'è il problema? ».
//!
//! One phrase probe is quoted too: « Nel mezzo del cammin », the opening words of Dante
//! Alighieri's *Commedia* (Inferno I, 1; Dante died in 1321), in the public domain everywhere.
//!
//! The `narrativa` and `manzoni` pages are set as Italian is printed: the typographic apostrophe
//! `’`, « » without spaces, the dialogue dash, `È` with its accent. The other pages are typed as
//! the web often has them, with the straight apostrophe, and the `informale` page with the
//! apostrophe where an accent belongs (`perche'`, `e'`, `piu'`, `citta'`, `E'`), which the edge
//! trim of every language took off the word until change 10 read it as an accent (`perche'` is
//! `perché`, `E'` `È`, read-lingua-italian-unmarked-spellings D1, at `1.2.0`), and the accent left
//! out (`perche`, `cosi`), read through the fixture's two spelling rows, while the `elisioni` page's
//! truncations keep their apostrophe and read as their words (`po'` is `poco`,
//! add-lingua-italian-tokenisation D5). One
//! block of the `tecnico` page is committed in NFD on purpose (accents as combining marks, as a
//! terminal paste or an OCR writes them); `italian_baseline.rs` asserts it still is. The fixture
//! lists one of its words, `città` (« ville »), which the corpus has nowhere else: read as it came,
//! the decomposed word is not the pack's, so change 7's NFC shows in the golden as a gloss that
//! appears, not only as bytes that move. The `misto` page's Catalan and Galician blocks are
//! authored so that whichlang reads them as Italian, which a learner of Italian counts until
//! change 8's guard; its Latin block whichlang reads as French.
//!
//! The pack is built from the committed `tables/it/` and `tables/it-fr/` since change 16
//! (add-lingua-pack-it-fr D9), the hand-written fixture `scripts/lingua-data/testdata/it-fr/` before;
//! the engine starts on the real en-fr pack, as a French-native reader's does.

use super::{Card, PackSource, Scenario};

/// Dictionary forms whose gloss the card shows: the verbs every page uses, M8's homographs,
/// elided, articulated and enclitic words as the baseline read them at `0.1.0`, words from the
/// corpus. Some have none in the pack, on purpose.
const LEMMAS: &[&str] = &[
    "essere",
    "avere",
    "andare",
    "fare",
    "potere",
    "dovere",
    "volere",
    "dire",
    "venire",
    "prendere",
    "ancora",
    "pesca",
    "legge",
    "leggere",
    "porta",
    "portare",
    "sale",
    "salire",
    "vite",
    "vita",
    "canto",
    "cantare",
    "era",
    "uomo",
    "l'uomo",
    "acqua",
    "dell'acqua",
    "c'è",
    "del",
    "nel",
    "al",
    "col",
    "po",
    "dirmelo",
    "eccolo",
    "il",
    "di",
    "a",
    "e",
    "è",
    "o",
    "faro",
    "marinaio",
    "tempesta",
    "mare",
    "orizzonte",
    "città",
    "casa",
    "pane",
    "cinema",
    "frittata",
    "cmq",
    "xké",
];

/// Selections a reader glosses: the fixture's expressions, the Manzoni sentence, elided,
/// articulated and enclitic selections, the web's apostrophe for an accent, an acronym standing
/// alone in capitals (keep-lingua-italian-acronyms-and-names D7), the expressions as Italian
/// reads them (add-lingua-italian-expression-keys D9), and the blocks that are not Italian
/// (English, Spanish, French, Catalan, Galician, Latin).
const PHRASES: &[&str] = &[
    "a poco a poco",
    "ad alta voce",
    "a causa dello sciopero",
    "Non so se riuscirò a farcela da solo",
    "fare colazione",
    "per favore",
    "Quel ramo del lago di Como, che volge a mezzogiorno",
    "L'uomo che aspettavo",
    "la temperatura dell'acqua",
    "all'alba",
    "un po' di pane",
    "C'è un po' di pane",
    "c'è stato un errore",
    "del pane e delle uova",
    "nel cassetto",
    "Nel mezzo del cammin",
    "col cane",
    "dirmelo subito",
    "vattene",
    "eccolo di nuovo",
    "Non lo so, dimmelo tu",
    "Eccolo, è arrivato dall'Italia",
    "E' troppo tardi perche' la citta' e' piu' lontana",
    "I paesi della NATO",
    // refine-lingua-italian-reading: a phrase per rule, and the closed words D6 adds.
    "Dai, andiamo!",
    "mi dai una mano?",
    "a mo' di esempio",
    "Lorenzo de' Medici",
    "ne' campi",
    "trent'anni fa",
    "gliel'ho detto",
    "davanti alla porta",
    "ossia il primo",
    "non di il che",
    // add-lingua-italian-expression-keys D9: a phrase per rule.
    "cotto al dente",
    "vicino al mare",
    "a causa del ritardo",
    "anch'io",
    "anche io",
    "un poco di pane",
    "vale la pena",
    "d'accordo",
    "The lighthouse stood",
    "El faro se alzaba",
    "Le phare se dressait",
    "El far s'alçava",
    "Non era unha noite tranquila",
    "Pharus in margine rupis",
];

/// (word as written, dictionary form) pairs a word card asks the grammar of. The fixture has no
/// grammar tables: each answers its gloss alone, and a written word Italian's pre-pass splits
/// answers its pieces — `al`, `l'uomo` —, an elided piece handed alone none (`l’`), asked by its
/// written reading (`lo`) or by its lemma (`il`); a verb carrying its clitics, by its verb
/// (`dirmelo`, `vattene`).
const GRAMMAR: &[(&str, &str)] = &[
    ("è", "essere"),
    ("sono", "essere"),
    ("era", "essere"),
    ("fu", "essere"),
    ("sia", "essere"),
    ("fosse", "essere"),
    ("sarebbe", "essere"),
    ("ha", "avere"),
    ("ebbe", "avere"),
    ("abbia", "avere"),
    ("andò", "andare"),
    ("va", "andare"),
    ("fa", "fare"),
    ("disse", "dire"),
    ("porta", "portare"),
    ("porta", "porta"),
    ("legge", "leggere"),
    ("legge", "legge"),
    ("sale", "salire"),
    ("sale", "sale"),
    ("vite", "vita"),
    ("del", "del"),
    ("al", "a"),
    ("l'uomo", "uomo"),
    ("l’", "lo"),
    // The fixture files `lo` under `il`: asked by its lemma, the elided article shows its gloss.
    ("l’", "il"),
    ("dirmelo", "dire"),
    ("vattene", "andare"),
    // The acronym the fixture lists as written, apart from `nato` (keep-lingua-italian-acronyms-and-
    // names D7).
    ("NATO", "NATO"),
    // A word the pre-pass reads by its context, asked by each reading's dictionary form
    // (refine-lingua-italian-reading D4, D8).
    ("dai", "dare"),
    ("dai", "da"),
    ("mo'", "mò"),
];

/// The pages analysed again for the reader with a history.
const READER_PAGES: &[&str] = &["notizie", "omografi", "elisioni", "misto"];

const PAGE_NAMES: &[&str] = &[
    "notizie",
    "narrativa",
    "manzoni",
    "omografi",
    "elisioni",
    "preposizioni",
    "clitici",
    "nomi",
    "informale",
    "ricetta",
    "tecnico",
    "misto",
    "grammatica",
];

pub const ITALIAN: Scenario = Scenario {
    pair: "it-fr",
    pack: PackSource::Tables,
    beside: &["en-fr"],
    test: "italian_baseline",
    pages: "pages-it.txt",
    page_names: PAGE_NAMES,
    lemmas: LEMMAS,
    phrases: PHRASES,
    grammar: GRAMMAR,
    reader_pages: READER_PAGES,
    statuses: &[
        ("faro", "known", 1.0),
        ("marinaio", "learning", 2.0),
        ("cmq", "ignored", 3.0),
        ("finestra", "known", 4.0),
        ("paese", "known", 5.0),
        // Withdrawn: « Remettre à apprendre ».
        ("paese", "", 6.0),
    ],
    exposures: ["mare", "tempesta", "orizzonte", "luce", "onda"],
    cards: [
        Card {
            lemma: "orizzonte",
            form: "orizzonte",
            sentence: "Rimase a guardare l’orizzonte finché la luce non si spense.",
            url: "https://example.com/narrativa",
            gloss: Some("horizon"),
        },
        Card {
            lemma: "sciopero",
            form: "sciopero",
            sentence: "Il prezzo del pane è aumentato a causa dello sciopero dei camionisti.",
            url: "https://example.com/preposizioni",
            gloss: None,
        },
        Card {
            lemma: "aspettare",
            form: "aspettavo",
            sentence: "L'uomo che aspettavo non è mai arrivato.",
            url: "",
            gloss: None,
        },
    ],
    reader_phrase: "Il vecchio marinaio chiuse le finestre del faro",
};
