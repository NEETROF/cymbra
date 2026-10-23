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
//! reader (add-lingua-italian-baseline D5).
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
//!   Pinocchio* (Felice Paggi, Florence, 1883), « — C’era una volta.... — Un re! — diranno subito
//!   i miei piccoli lettori. — No, ragazzi, avete sbagliato. C’era una volta un pezzo di legno. »,
//!   its three lines of dialogue joined in one block, its four dots as printed. Collodi died in
//!   1890, so the work has been in the public domain in Italy since 1961, and was published before
//!   1929, so it is in the public domain in the United States.
//!
//! Both were checked against the proofread transcriptions of it.wikisource.org on 2026-10-23.
//!
//! The `narrativa` and `manzoni` pages are set as Italian is printed: the typographic apostrophe
//! `’`, « » without spaces, the dialogue dash, `È` with its accent. The other pages are typed as
//! the web often has them, with the straight apostrophe, and the `informale` page with the
//! apostrophe where an accent belongs (`perche'`, `e'`, `piu'`, `citta'`, `E'`), which the edge
//! trim of every language takes off the word today (`perche'` is `perche`, as `po'` is `po`). One
//! block of the `tecnico` page is committed in NFD on purpose (accents as combining marks, as a
//! terminal paste or an OCR writes them); `italian_baseline.rs` asserts it still is. The fixture
//! lists one of its words, `città` (« ville »), which the corpus has nowhere else: read as it came,
//! the decomposed word is not the pack's, so change 7's NFC shows in the golden as a gloss that
//! appears, not only as bytes that move. The `misto` page's Catalan and Galician blocks are
//! authored so that whichlang reads them as Italian, which a learner of Italian counts until
//! change 8's guard; its Latin block whichlang reads as French.
//!
//! The pack is the hand-written it-fr fixture, `scripts/lingua-data/testdata/it-fr/`, until
//! change 16 commits `tables/it/` and `tables/it-fr/`; the engine starts on the real en-fr pack,
//! as a French-native reader's does.

use super::{Card, PackSource, Scenario};

/// Dictionary forms whose gloss the card shows: the verbs every page uses, M8's homographs,
/// elided, articulated and enclitic words as the baseline reads them, words from the corpus. Some
/// have none in the pack, on purpose.
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
/// articulated and enclitic selections, the web's apostrophe for an accent, and the blocks that
/// are not Italian (English, Spanish, French, Catalan, Galician, Latin).
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
    "un po' di pane",
    "c'è stato un errore",
    "del pane e delle uova",
    "nel cassetto",
    "col cane",
    "dirmelo subito",
    "vattene",
    "eccolo di nuovo",
    "E' troppo tardi perche' la citta' e' piu' lontana",
    "non di il che",
    "The lighthouse stood",
    "El faro se alzaba",
    "Le phare se dressait",
    "El far s'alçava",
    "Non era unha noite tranquila",
    "Pharus in margine rupis",
];

/// (word as written, dictionary form) pairs a word card asks the grammar of. The fixture has no
/// grammar tables: each answers its gloss alone.
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
    ("l'uomo", "uomo"),
    ("dirmelo", "dire"),
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
    pack: PackSource::Testdata,
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
