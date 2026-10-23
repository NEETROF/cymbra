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

//! The closed-class words of a studied language (change `add-lingua-phrase-gloss`,
//! design D3).
//!
//! A word-by-word gloss of a selection is only worth showing for the words that
//! carry meaning on their own: glossing `of` or `have` inside a phrase tells the
//! reader nothing. The tables below hold the English closed classes as
//! **dictionary forms** — the check runs after lemmatisation, so `is` reaches it
//! as `be` and the `n't` of a contraction as `not` — in the six classes the spec
//! names. They are a judgement, pinned by one test per class; a frequency cut
//! would not do, since the hundred commonest lemmas hold `time`, `people`,
//! `say` and `know`, exactly the words worth a row.
//!
//! Spanish's tables (add-lingua-spanish-analysis D5), French's
//! (add-lingua-french-analysis D3) and Italian's (add-lingua-italian-analysis D3)
//! follow, in the same six classes, each holding its language's dictionary forms
//! and the inflected forms a pack may keep as lemmas of their own; French's flag
//! « pas » (M21), Italian's « non ».
//!
//! Every table MUST stay sorted: lookups binary-search them directly (a test
//! enforces the ordering). Adding a studied language means adding its tables,
//! like its tokeniser pre-pass.

use super::language::StudiedLanguage;

/// Articles and the other determiners: possessives, demonstratives and the
/// quantifiers that stand before a noun (`some`, `each`, `every`…).
const DETERMINERS: &[&str] = &[
    "a", "all", "an", "another", "any", "both", "each", "either", "every", "few", "his", "its",
    "many", "much", "my", "neither", "other", "our", "several", "some", "such", "that", "the",
    "their", "these", "this", "those", "your",
];

/// Personal, reflexive, possessive, relative, interrogative and indefinite
/// pronouns. `i` is the lowercase lemma the cascade gives `I`.
const PRONOUNS: &[&str] = &[
    "anybody",
    "anyone",
    "anything",
    "everybody",
    "everyone",
    "everything",
    "he",
    "her",
    "hers",
    "herself",
    "him",
    "himself",
    "i",
    "it",
    "itself",
    "me",
    "mine",
    "myself",
    "nobody",
    "none",
    "nothing",
    "oneself",
    "ours",
    "ourselves",
    "she",
    "somebody",
    "someone",
    "something",
    "theirs",
    "them",
    "themselves",
    "they",
    "us",
    "we",
    "what",
    "whatever",
    "which",
    "whichever",
    "who",
    "whoever",
    "whom",
    "whose",
    "you",
    "yours",
    "yourself",
    "yourselves",
];

/// Prepositions, and the particles of phrasal verbs (`give up`, `set off`,
/// `run away`), which are the same short words.
const PREPOSITIONS_AND_PARTICLES: &[&str] = &[
    "about",
    "above",
    "across",
    "after",
    "against",
    "along",
    "amid",
    "among",
    "around",
    "at",
    "away",
    "before",
    "behind",
    "below",
    "beneath",
    "beside",
    "besides",
    "between",
    "beyond",
    "by",
    "despite",
    "down",
    "during",
    "except",
    "for",
    "from",
    "in",
    "inside",
    "into",
    "near",
    "of",
    "off",
    "on",
    "onto",
    "out",
    "outside",
    "over",
    "per",
    "since",
    "through",
    "throughout",
    "till",
    "to",
    "toward",
    "towards",
    "under",
    "underneath",
    "until",
    "unto",
    "up",
    "upon",
    "via",
    "with",
    "within",
    "without",
];

/// Coordinating and subordinating conjunctions.
const CONJUNCTIONS: &[&str] = &[
    "although", "and", "as", "because", "but", "if", "nor", "or", "so", "than", "though", "unless",
    "when", "whenever", "where", "whereas", "wherever", "whether", "while", "yet",
];

/// Auxiliaries and modals. `be`, `have` and `do` are the lemmas the irregulars
/// table gives `is`, `has`, `did`, `'s`, `'ve`…; `will` and `would` catch `'ll`
/// and `'d`.
const AUXILIARIES_AND_MODALS: &[&str] = &[
    "be", "can", "could", "do", "have", "may", "might", "must", "ought", "shall", "should", "will",
    "would",
];

/// Negation. `not` is what the tokeniser's pre-pass makes of every `n't`;
/// `cannot` is the one negated modal the tokeniser cannot split. `never` is
/// left out on purpose: it is an adverb with content of its own.
const NEGATION: &[&str] = &["cannot", "no", "not"];

/// The six English tables, checked in turn by [`is_function_word`].
const ENGLISH: &[&[&str]] = &[
    DETERMINERS,
    PRONOUNS,
    PREPOSITIONS_AND_PARTICLES,
    CONJUNCTIONS,
    AUXILIARIES_AND_MODALS,
    NEGATION,
];

/// Spanish articles and other determiners: possessives, demonstratives, quantifiers.
/// Inflected forms too, which a pack may keep as their own lemmas (design D5).
const ES_DETERMINERS: &[&str] = &[
    "alguna",
    "algunas",
    "alguno",
    "algunos",
    "algún",
    "ambas",
    "ambos",
    "aquel",
    "aquella",
    "aquellas",
    "aquellos",
    "cada",
    "cualquier",
    "cualquiera",
    "demasiada",
    "demasiadas",
    "demasiado",
    "demasiados",
    "el",
    "esa",
    "esas",
    "ese",
    "esos",
    "esta",
    "estas",
    "este",
    "estos",
    "la",
    "las",
    "lo",
    "los",
    "mi",
    "mis",
    "misma",
    "mismas",
    "mismo",
    "mismos",
    "mucha",
    "muchas",
    "mucho",
    "muchos",
    "ninguna",
    "ningunas",
    "ninguno",
    "ningunos",
    "ningún",
    "nuestra",
    "nuestras",
    "nuestro",
    "nuestros",
    "otra",
    "otras",
    "otro",
    "otros",
    "poca",
    "pocas",
    "poco",
    "pocos",
    "su",
    "sus",
    "tanta",
    "tantas",
    "tanto",
    "tantos",
    "toda",
    "todas",
    "todo",
    "todos",
    "tu",
    "tus",
    "un",
    "una",
    "unas",
    "unos",
    "varias",
    "varios",
    "vuestra",
    "vuestras",
    "vuestro",
    "vuestros",
];

/// Spanish personal, clitic, prepositional, possessive, relative, interrogative and
/// indefinite pronouns.
const ES_PRONOUNS: &[&str] = &[
    "algo", "alguien", "aquello", "conmigo", "consigo", "contigo", "cual", "cuales", "cuya",
    "cuyas", "cuyo", "cuyos", "cuál", "cuáles", "cuánta", "cuántas", "cuánto", "cuántos", "ella",
    "ellas", "ello", "ellos", "eso", "esto", "la", "las", "le", "les", "lo", "los", "me", "mí",
    "mía", "mías", "mío", "míos", "nada", "nadie", "nos", "nosotras", "nosotros", "os", "que",
    "quien", "quienes", "quién", "quiénes", "qué", "se", "suya", "suyas", "suyo", "suyos", "sí",
    "te", "ti", "tuya", "tuyas", "tuyo", "tuyos", "tú", "usted", "ustedes", "vos", "vosotras",
    "vosotros", "yo", "él",
];

/// Spanish prepositions.
const ES_PREPOSITIONS: &[&str] = &[
    "a", "ante", "bajo", "con", "contra", "de", "desde", "durante", "en", "entre", "excepto",
    "hacia", "hasta", "mediante", "para", "por", "salvo", "según", "sin", "sobre", "tras",
];

/// Spanish coordinating and subordinating conjunctions.
const ES_CONJUNCTIONS: &[&str] = &[
    "aunque", "como", "cuando", "donde", "e", "mientras", "ni", "o", "pero", "porque", "pues",
    "que", "si", "sino", "u", "y",
];

/// Spanish auxiliaries and modals, as the forms tables lemmatise their forms.
const ES_AUXILIARIES_AND_MODALS: &[&str] = &["deber", "estar", "haber", "poder", "ser", "soler"];

/// Spanish negation. `nunca` is left out, as English's `never` is: an adverb with
/// content of its own.
const ES_NEGATION: &[&str] = &["no"];

/// The six Spanish tables (add-lingua-spanish-analysis D5).
const SPANISH: &[&[&str]] = &[
    ES_DETERMINERS,
    ES_PRONOUNS,
    ES_PREPOSITIONS,
    ES_CONJUNCTIONS,
    ES_AUXILIARIES_AND_MODALS,
    ES_NEGATION,
];

/// French articles and other determiners: `le`, `un`, `du`, `des`, the
/// demonstratives, the possessives, `quel`, the indefinites `aucun`, `nul`,
/// `chaque`, `plusieurs`, `quelque`, `tout`, `même`, `autre`, `tel`, and the
/// quantifiers `beaucoup`, `peu`, `trop`, `tant` (English keeps `many`, `much`,
/// `few`; Spanish `mucho`, `poco`, `tanto`). Inflected forms too, which a pack may
/// keep as lemmas of their own (`cet`, `ma`, `mes`, `ton`, `ta`, `tes`, `la`,
/// `une`, `toutes`…), so that a reduction moving a form to its own lemma gives a
/// gloss no row for it. `certain` is left out: one GSD use in four is the
/// adjective « sure ». `ton` stays, though it hides the noun « tone » (10 of GSD's
/// 18 uses), as `son` hides « sound » (21 of 3,287).
const FR_DETERMINERS: &[&str] = &[
    "aucun",
    "aucune",
    "autre",
    "autres",
    "beaucoup",
    "ce",
    "ces",
    "cet",
    "cette",
    "chaque",
    "des",
    "du",
    "la",
    "le",
    "les",
    "leur",
    "leurs",
    "ma",
    "mes",
    "mon",
    "même",
    "mêmes",
    "nos",
    "notre",
    "nul",
    "nulle",
    "peu",
    "plusieurs",
    "quel",
    "quelle",
    "quelles",
    "quelque",
    "quelques",
    "quels",
    "sa",
    "ses",
    "son",
    "ta",
    "tant",
    "tel",
    "telle",
    "telles",
    "tels",
    "tes",
    "ton",
    "tous",
    "tout",
    "toute",
    "toutes",
    "trop",
    "un",
    "une",
    "vos",
    "votre",
];

/// French personal (`je` … `elles`, `me`, `te`, `se`, `moi`, `toi`, `soi`,
/// `lui`, `eux`, `y`, `en`), demonstrative (`ce`, `ceci`, `cela`, `ça`, `celui`
/// and its forms), relative and interrogative (`qui`, `que`, `quoi`, `dont`,
/// `lequel` and its forms) and indefinite pronouns (`quelqu'un`, `chacun`,
/// `rien`, `autrui`, `quiconque`). `personne` is left out: 153 of GSD's 171 uses
/// are the noun « person ».
const FR_PRONOUNS: &[&str] = &[
    "auquel",
    "autrui",
    "auxquelles",
    "auxquels",
    "ce",
    "ceci",
    "cela",
    "celle",
    "celle-ci",
    "celle-là",
    "celles",
    "celles-ci",
    "celles-là",
    "celui",
    "celui-ci",
    "celui-là",
    "ceux",
    "ceux-ci",
    "ceux-là",
    "chacun",
    "chacune",
    "desquelles",
    "desquels",
    "dont",
    "duquel",
    "elle",
    "elles",
    "en",
    "eux",
    "il",
    "ils",
    "je",
    "la",
    "laquelle",
    "le",
    "lequel",
    "les",
    "lesquelles",
    "lesquels",
    "leur",
    "lui",
    "me",
    "moi",
    "nous",
    "on",
    "que",
    "quelqu'un",
    "qui",
    "quiconque",
    "quoi",
    "rien",
    "se",
    "soi",
    "te",
    "toi",
    "tu",
    "vous",
    "y",
    "ça",
];

/// French prepositions, `jusque` among them (`jusqu'ici`), and `afin`, `près`
/// of the locutions `afin de`, `près de`. `voici` and `voilà`, presentatives, are
/// left out.
const FR_PREPOSITIONS: &[&str] = &[
    "afin",
    "après",
    "avant",
    "avec",
    "chez",
    "contre",
    "dans",
    "de",
    "depuis",
    "derrière",
    "devant",
    "durant",
    "dès",
    "en",
    "entre",
    "envers",
    "hormis",
    "hors",
    "jusque",
    "malgré",
    "outre",
    "par",
    "parmi",
    "pendant",
    "pour",
    "près",
    "sans",
    "sauf",
    "selon",
    "sous",
    "sur",
    "vers",
    "via",
    "à",
];

/// French coordinating and subordinating conjunctions, with `parce` and `tandis`
/// of `parce que`, `tandis que`. `or` is left out: 51 of GSD's 69 uses are the
/// noun « gold ».
const FR_CONJUNCTIONS: &[&str] = &[
    "car", "comme", "donc", "et", "lorsque", "mais", "ni", "ou", "où", "parce", "puisque", "quand",
    "que", "quoique", "si", "tandis",
];

/// French auxiliaries and modals (Spanish's `ser`, `haber`, `poder`, `deber`).
/// `falloir`, `vouloir`, `aller` and `faire` are left out: content verbs, as
/// Spanish's `querer`, `ir` and `hacer`.
const FR_AUXILIARIES_AND_MODALS: &[&str] = &["avoir", "devoir", "pouvoir", "être"];

/// French negation. `pas` is a function word (M21) although its form is also
/// the noun « step » (10 of GSD's 1,047 uses); `point` (« ne … point »,
/// literary: 173 of GSD's 185 uses are the noun), `plus`, `jamais` and `guère`
/// are left out — adverbs with content of their own, as English's `never` and
/// Spanish's `nunca`.
const FR_NEGATION: &[&str] = &["ne", "non", "pas"];

/// The six French tables (add-lingua-french-analysis D3). Like Spanish's, and
/// unlike English's, a word may stand in two classes: `ce`, `le`, `la`, `les` and
/// `leur` are determiners and pronouns, `en` a pronoun and a preposition, `que` a
/// pronoun and a conjunction.
const FRENCH: &[&[&str]] = &[
    FR_DETERMINERS,
    FR_PRONOUNS,
    FR_PREPOSITIONS,
    FR_CONJUNCTIONS,
    FR_AUXILIARIES_AND_MODALS,
    FR_NEGATION,
];

/// Italian articles and other determiners (add-lingua-italian-analysis D3): the articles
/// `il`, `lo`, `la`, `i`, `gli`, `le`, `un`, `uno`, `una`; the demonstratives `questo`,
/// `quello` (`quel`, `quei`, `quegli`), `codesto`; the possessives; `ogni`, `qualche`,
/// `alcuno`, `nessuno`, `ciascuno`, `tale`, `altro`, `stesso`; the `di` family, `del`,
/// `dello`, `della`, `dei`, `degli`, `delle`, which the pre-pass keeps whole (M21); and the
/// quantifiers `molto`, `tanto`, `poco`, `tutto`, `troppo`, as Spanish's hold `mucho`,
/// `poco`, `tanto`, `todo` and French's `beaucoup`, `peu`, `tant`, `tout`. Inflected forms
/// too, which a pack may keep as lemmas of their own (`questa`, `mia`, `nessun`, `tutti`).
/// `qualsiasi` and `qualunque` by measurement (design, *Measured at implementation*): a
/// determiner in each of MarkIT's test section's occurrences (2 of 2, 1 of 1; with UD
/// Italian-PUD, 4 of 4 and 2 of 3, « una persona qualunque » the other). `certo` and `vario`
/// are left out: adjectives with a meaning of their own, as French's `certain`.
const IT_DETERMINERS: &[&str] = &[
    "alcun",
    "alcuna",
    "alcune",
    "alcuni",
    "alcuno",
    "altra",
    "altre",
    "altri",
    "altro",
    "ciascun",
    "ciascuna",
    "ciascuno",
    "codesta",
    "codeste",
    "codesti",
    "codesto",
    "degli",
    "dei",
    "del",
    "della",
    "delle",
    "dello",
    "gli",
    "i",
    "il",
    "la",
    "le",
    "lo",
    "loro",
    "mia",
    "mie",
    "miei",
    "mio",
    "molta",
    "molte",
    "molti",
    "molto",
    "nessun",
    "nessuna",
    "nessuno",
    "nostra",
    "nostre",
    "nostri",
    "nostro",
    "ogni",
    "poca",
    "poche",
    "pochi",
    "poco",
    "qualche",
    "qualsiasi",
    "qualunque",
    "quegli",
    "quei",
    "quel",
    "quella",
    "quelle",
    "quelli",
    "quello",
    "questa",
    "queste",
    "questi",
    "questo",
    "stessa",
    "stesse",
    "stessi",
    "stesso",
    "sua",
    "sue",
    "suo",
    "suoi",
    "tale",
    "tali",
    "tanta",
    "tante",
    "tanti",
    "tanto",
    "troppa",
    "troppe",
    "troppi",
    "troppo",
    "tua",
    "tue",
    "tuo",
    "tuoi",
    "tutta",
    "tutte",
    "tutti",
    "tutto",
    "un",
    "una",
    "uno",
    "vostra",
    "vostre",
    "vostri",
    "vostro",
];

/// Italian personal pronouns, stressed (`io` … `loro`, `me`, `te`, `sé`, `egli`, `ella`,
/// `esso`) and clitic (`mi`, `ti`, `si`, `ci`, `vi`, `lo`, `la`, `li`, `le`, `ne`, `gli`, and
/// the variants `me`, `te`, `ce`, `ve` of « me lo », « ce l'ho », written apart); the combined
/// clitics written as one word before a verb, `glielo`, `gliela`, `glieli`, `gliele`,
/// `gliene`; relative and interrogative (`che`, `cui`, `chi`, `quale`) and indefinite
/// (`ciò`, `qualcosa`, `qualcuno`, `ognuno`, `chiunque`, `niente`, `nulla`); the
/// demonstratives `colui`, `colei`, `coloro`, `costui`, `costei`, `costoro` (MarkIT: `colui`
/// and `coloro` pronouns, 1 of 1 each; pooled with PUD, 1 of 1 and 2 of 3; the four others
/// written in neither, and pronouns alone in the dictionary). The other
/// combined clitics are written apart before a verb (« me lo dice ») and joined only after
/// one (`dirmelo`, which the enclitic rule reads as `dire`): as one word standing alone they
/// are nouns — `mela` (apple), `vela` (sail), `tela` (cloth), `velo` (veil), `cene`
/// (dinners) — which a row here would hide in every gloss. `quanto` and `cosa` are left out:
/// « quanto » is also « how much », « cosa » the noun « thing ».
const IT_PRONOUNS: &[&str] = &[
    "ce", "che", "chi", "chiunque", "ci", "ciò", "colei", "coloro", "colui", "costei", "costoro",
    "costui", "cui", "egli", "ella", "essa", "esse", "essi", "esso", "gli", "gliela", "gliele",
    "glieli", "glielo", "gliene", "io", "la", "le", "lei", "li", "lo", "loro", "lui", "me", "mi",
    "ne", "niente", "noi", "nulla", "ognuna", "ognuno", "qualcosa", "qualcuna", "qualcuno",
    "quale", "quali", "si", "sé", "te", "ti", "tu", "ve", "vi", "voi",
];

/// Italian prepositions: the simple ones (`di`, `a` and its `ad` before a vowel, `da`, `in`,
/// `con`, `su`, `per`, `tra`, `fra`) and `senza`, `verso`, `contro`, `sopra`, `sotto`,
/// `dentro`, `fuori`, `dopo`, `durante`, `tranne`, `presso`, `oltre`, `attraverso`,
/// `mediante`, `circa`, `entro`; and, by measurement, `fino` (« fino a »), `nonostante`,
/// `tramite`, `malgrado` and `eccetto` — each introducing a phrase in every one of its
/// occurrences the treebanks write (MarkIT's test section: `fino` 5 of 5, `nonostante` 3 of 3;
/// pooled with UD Italian-PUD: 27 of 27, 9 of 9, `tramite` 6 of 6, `malgrado` 1 of 1;
/// `eccetto` written in neither, and a preposition alone in the dictionary). The articulated
/// ones never reach this: the pre-pass splits them (`nel` → `in` + `il`) or keeps the `di`
/// family whole, a determiner. `secondo` and `lungo` are left out: everyday adjectives (« il
/// secondo giorno », « un lungo viaggio ») a preposition's row would hide in every gloss, as
/// French measured `certain` out. Left out by measurement (design D6): `prima` — MarkIT's
/// test section tags none of its 14 as a preposition, and has it introduce a phrase (« prima
/// di ») in 5 of them (pooled: 19 of 49); 5 are the adjective « la prima volta » —, and `salvo`
/// — its one MarkIT occurrence is the adjective « safe ».
const IT_PREPOSITIONS: &[&str] = &[
    "a",
    "ad",
    "attraverso",
    "circa",
    "con",
    "contro",
    "da",
    "dentro",
    "di",
    "dopo",
    "durante",
    "eccetto",
    "entro",
    "fino",
    "fra",
    "fuori",
    "in",
    "malgrado",
    "mediante",
    "nonostante",
    "oltre",
    "per",
    "presso",
    "senza",
    "sopra",
    "sotto",
    "su",
    "tra",
    "tramite",
    "tranne",
    "verso",
];

/// Italian coordinating and subordinating conjunctions, `ed` and `od` before a vowel among
/// them, and `dove` (« where »), as French lists `où` and Spanish `donde`: relative or
/// interrogative in each of its occurrences (MarkIT's test section 6 of 6, pooled with UD
/// Italian-PUD 24 of 24).
const IT_CONJUNCTIONS: &[&str] = &[
    "affinché",
    "anzi",
    "benché",
    "che",
    "cioè",
    "come",
    "dove",
    "dunque",
    "e",
    "ed",
    "eppure",
    "finché",
    "infatti",
    "ma",
    "mentre",
    "né",
    "o",
    "od",
    "oppure",
    "ovvero",
    "perché",
    "perciò",
    "però",
    "poiché",
    "purché",
    "quando",
    "quindi",
    "se",
    "sebbene",
    "siccome",
    "tuttavia",
];

/// Italian auxiliaries and modals (Spanish's `ser`, `haber`, `poder`, `deber`; French's
/// `être`, `avoir`, `pouvoir`, `devoir`). `volere` is left out, a content verb as Spanish's
/// `querer` and French's `vouloir`; and so is `stare`, the progressive's auxiliary (« sto
/// parlando ») but also « to stay », as French leaves `aller` out — measured (design D6):
/// MarkIT's test section tags 13 of its 24 as an auxiliary, short of the clear majority (two
/// in three) that would have put it in (UD Italian-PUD: 23 of 28).
const IT_AUXILIARIES_AND_MODALS: &[&str] = &["avere", "dovere", "essere", "potere"];

/// Italian negation: `non`, `no` and `né`. `mai`, `più`, `neanche`, `nemmeno` and `neppure`
/// are left out, as English's `never`, Spanish's `nunca` and French's `jamais`: adverbs with
/// content of their own (« non più » is « no longer », « mai » « never »); so are `anche` and
/// `ancora`.
const IT_NEGATION: &[&str] = &["no", "non", "né"];

/// The six Italian tables (add-lingua-italian-analysis D3). Like Spanish's and French's, a
/// word may stand in two classes: `gli`, `la`, `le`, `lo` and `loro` are determiners and
/// pronouns, `che` a pronoun and a conjunction, `né` a conjunction and a negation.
const ITALIAN: &[&[&str]] = &[
    IT_DETERMINERS,
    IT_PRONOUNS,
    IT_PREPOSITIONS,
    IT_CONJUNCTIONS,
    IT_AUXILIARIES_AND_MODALS,
    IT_NEGATION,
];

/// Whether a dictionary form is a closed-class word of the studied language:
/// an article or other determiner, a pronoun, a preposition or particle, a
/// conjunction, an auxiliary or modal, or a negation. `lemma` is what the
/// lemmatisation cascade produced (lowercase), never a surface form.
///
/// Each language is judged by its own tables only: `de` is not an English
/// function word, nor `the` a Spanish one, nor `ne` a Spanish one, nor `il` an
/// English, Spanish or French one.
pub fn is_function_word(lemma: &str, studied: StudiedLanguage) -> bool {
    let tables: &[&[&str]] = match studied {
        StudiedLanguage::English => ENGLISH,
        StudiedLanguage::Spanish => SPANISH,
        StudiedLanguage::French => FRENCH,
        StudiedLanguage::Italian => ITALIAN,
    };
    tables
        .iter()
        .any(|table| table.binary_search(&lemma).is_ok())
}

/// Whether a dictionary form is a preposition of the studied language: English's
/// prepositions and particles, Spanish's prepositions, French's, Italian's — one
/// class of the tables [`is_function_word`] reads, each language read by its own.
/// `du`, `des`, `al` and `del` are contractions, not prepositions, and so are
/// Italian's `del`, `della`…, determiners. The phrase gloss reads it to tell an
/// expression built on a verb from one built on a preposition
/// (choose-lingua-overlapping-expressions D2).
pub fn is_preposition(lemma: &str, studied: StudiedLanguage) -> bool {
    let table: &[&str] = match studied {
        StudiedLanguage::English => PREPOSITIONS_AND_PARTICLES,
        StudiedLanguage::Spanish => ES_PREPOSITIONS,
        StudiedLanguage::French => FR_PREPOSITIONS,
        StudiedLanguage::Italian => IT_PREPOSITIONS,
    };
    table.binary_search(&lemma).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_language_reads_its_own_prepositions() {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        for lemma in ["à", "de", "jusque"] {
            assert!(
                is_preposition(lemma, FR),
                "{lemma:?} is a French preposition"
            );
        }
        for lemma in ["du", "des", "le", "être", "que"] {
            assert!(
                !is_preposition(lemma, FR),
                "{lemma:?} is no French preposition"
            );
        }
        for lemma in ["a", "de"] {
            assert!(
                is_preposition(lemma, ES),
                "{lemma:?} is a Spanish preposition"
            );
        }
        for lemma in ["al", "del", "el", "que"] {
            assert!(
                !is_preposition(lemma, ES),
                "{lemma:?} is no Spanish preposition"
            );
        }
        for lemma in ["off", "in", "up", "over"] {
            assert!(
                is_preposition(lemma, EN),
                "{lemma:?} is an English preposition"
            );
        }
        // Each language reads its own table: `à` is French, `off` English, `a` Spanish only.
        assert!(!is_preposition("à", EN) && !is_preposition("à", ES));
        assert!(!is_preposition("off", FR) && !is_preposition("off", ES));
        assert!(!is_preposition("a", FR) && !is_preposition("a", EN));
        assert!(!is_preposition("the", EN) && !is_preposition("go", EN));
    }

    const EN: StudiedLanguage = StudiedLanguage::English;

    fn all_function_words(lemmas: &[&str]) {
        for lemma in lemmas {
            assert!(is_function_word(lemma, EN), "{lemma:?} is a function word");
        }
    }

    const FR: StudiedLanguage = StudiedLanguage::French;

    const IT: StudiedLanguage = StudiedLanguage::Italian;

    /// Each lemma of a lemmatised Italian phrase, flagged or not.
    fn italian_flags(lemmas: &[&str]) -> Vec<bool> {
        lemmas
            .iter()
            .map(|lemma| is_function_word(lemma, IT))
            .collect()
    }

    #[test]
    fn spec_scenario_an_italian_phrase() {
        // « Non ho visto il cane di Marco », lemmatised with `ho` filed under `avere`.
        assert_eq!(
            italian_flags(&["non", "avere", "visto", "il", "cane", "di", "marco"]),
            [true, true, false, true, false, true, false]
        );
    }

    #[test]
    fn spec_scenario_the_italian_negation() {
        // « non lo so » and « né oggi né mai ».
        assert_eq!(italian_flags(&["non", "lo", "sapere"]), [true, true, false]);
        assert_eq!(
            italian_flags(&["né", "oggi", "né", "mai"]),
            [true, false, true, false]
        );
    }

    #[test]
    fn spec_scenario_italian_words_that_look_closed_but_carry_meaning() {
        // « più », « mai », « sto parlando », « voglio », « il secondo giorno », « molto tempo »:
        // `stare` and `volere` as the pack files `sto` and `voglio`.
        for lemma in [
            "più", "mai", "neanche", "nemmeno", "neppure", "anche", "ancora", "stare", "volere",
            "secondo", "lungo", "prima", "salvo", "certo", "quanto", "cosa", "parlare", "giorno",
            "tempo",
        ] {
            assert!(!is_function_word(lemma, IT), "{lemma:?} carries meaning");
        }
        assert_eq!(
            italian_flags(&["il", "secondo", "giorno"]),
            [true, false, false]
        );
        assert_eq!(italian_flags(&["molto", "tempo"]), [true, false]);
        // The combined clitics joined only after a verb are nouns standing alone: `mela`
        // (apple), `vela` (sail), `tela` (cloth), `velo` (veil), `cene` (dinners).
        for lemma in ["mela", "vela", "tela", "velo", "telo", "cene", "mele"] {
            assert!(!is_function_word(lemma, IT), "{lemma:?}");
        }
    }

    #[test]
    fn spec_scenario_the_pieces_of_an_italian_contraction_and_elision() {
        // « al mare » is `a` + `il` + `mare`; « l'uomo » is `lo` + `uomo`; « dell'acqua »
        // `dello` + `acqua`.
        assert_eq!(italian_flags(&["a", "il", "mare"]), [true, true, false]);
        assert_eq!(italian_flags(&["lo", "uomo"]), [true, false]);
        assert_eq!(italian_flags(&["dello", "acqua"]), [true, false]);
    }

    #[test]
    fn spec_scenario_each_language_keeps_its_own_tables() {
        // `the` is no Italian function word, nor `di` an English, Spanish or French one, nor
        // `il` an English or Spanish one (French's `il` is its own pronoun, « he »).
        assert!(!is_function_word("the", IT));
        for language in [EN, StudiedLanguage::Spanish, FR] {
            assert!(!is_function_word("di", language), "{language:?}");
            assert!(
                !is_function_word("non", language) || language == FR,
                "{language:?}"
            );
        }
        assert!(!is_function_word("il", EN) && !is_function_word("il", StudiedLanguage::Spanish));
        assert!(is_function_word("il", FR) && is_function_word("il", IT));
        for lemma in ["de", "pas", "haber", "of", "être", "jamais"] {
            assert!(!is_function_word(lemma, IT), "{lemma:?}");
        }
        // Italian's prepositions are its own.
        for lemma in [
            "di", "a", "ad", "da", "in", "con", "su", "per", "tra", "fra", "dopo",
        ] {
            assert!(
                is_preposition(lemma, IT),
                "{lemma:?} is an Italian preposition"
            );
        }
        for lemma in [
            "del", "della", "il", "che", "secondo", "lungo", "prima", "de", "of",
        ] {
            assert!(
                !is_preposition(lemma, IT),
                "{lemma:?} is no Italian preposition"
            );
        }
        assert!(!is_preposition("di", EN) && !is_preposition("di", FR));
    }

    #[test]
    fn italian_classes_hold_what_they_name() {
        for (table, sample) in [
            (
                IT_DETERMINERS,
                &[
                    "il",
                    "lo",
                    "la",
                    "i",
                    "gli",
                    "le",
                    "un",
                    "uno",
                    "una",
                    "questo",
                    "quello",
                    "codesto",
                    "mio",
                    "loro",
                    "ogni",
                    "qualche",
                    "alcuno",
                    "nessuno",
                    "ciascuno",
                    "tale",
                    "altro",
                    "stesso",
                    "del",
                    "dello",
                    "della",
                    "dei",
                    "degli",
                    "delle",
                    "molto",
                    "tanto",
                    "poco",
                    "tutto",
                    "troppo",
                    "qualsiasi",
                    "qualunque",
                ][..],
            ),
            (
                IT_PRONOUNS,
                &[
                    "io", "tu", "lui", "lei", "egli", "ella", "esso", "noi", "voi", "loro", "me",
                    "te", "sé", "mi", "ti", "si", "ci", "vi", "lo", "la", "li", "le", "ne", "gli",
                    "glielo", "gliene", "che", "cui", "chi", "quale", "ciò", "qualcosa",
                    "qualcuno", "ognuno", "chiunque", "niente", "nulla", "colui", "colei",
                    "coloro", "costui", "costei", "costoro",
                ][..],
            ),
            (
                IT_PREPOSITIONS,
                &[
                    "di",
                    "a",
                    "da",
                    "in",
                    "con",
                    "su",
                    "per",
                    "tra",
                    "fra",
                    "senza",
                    "verso",
                    "contro",
                    "sopra",
                    "sotto",
                    "dentro",
                    "fuori",
                    "dopo",
                    "durante",
                    "tranne",
                    "presso",
                    "oltre",
                    "attraverso",
                    "mediante",
                    "circa",
                    "entro",
                    "fino",
                    "nonostante",
                    "tramite",
                    "malgrado",
                    "eccetto",
                ][..],
            ),
            (
                IT_CONJUNCTIONS,
                &[
                    "e",
                    "ed",
                    "o",
                    "od",
                    "ma",
                    "però",
                    "anzi",
                    "né",
                    "che",
                    "se",
                    "perché",
                    "poiché",
                    "siccome",
                    "quando",
                    "mentre",
                    "come",
                    "dunque",
                    "quindi",
                    "perciò",
                    "infatti",
                    "cioè",
                    "sebbene",
                    "benché",
                    "affinché",
                    "finché",
                    "purché",
                    "dove",
                    "oppure",
                    "ovvero",
                    "tuttavia",
                    "eppure",
                ][..],
            ),
            (
                IT_AUXILIARIES_AND_MODALS,
                &["essere", "avere", "potere", "dovere"][..],
            ),
            (IT_NEGATION, &["non", "no", "né"][..]),
        ] {
            for word in sample {
                assert!(table.binary_search(word).is_ok(), "{word:?}");
                assert!(is_function_word(word, IT), "{word:?}");
            }
        }
    }

    #[test]
    fn an_italian_word_stands_in_two_classes_only_where_it_reads_two_ways() {
        let mut seen: Vec<&str> = Vec::new();
        let mut twice: Vec<&str> = Vec::new();
        for table in ITALIAN {
            for word in *table {
                if seen.contains(word) {
                    twice.push(word);
                }
                seen.push(word);
            }
        }
        twice.sort_unstable();
        assert_eq!(twice, ["che", "gli", "la", "le", "lo", "loro", "né"]);
    }

    #[test]
    fn each_language_is_judged_by_its_own_tables() {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        for lemma in ["de", "la", "el", "y", "que", "haber", "no"] {
            assert!(
                is_function_word(lemma, ES),
                "{lemma:?} is a Spanish function word"
            );
            assert!(!is_function_word(lemma, EN) || lemma == "no", "{lemma:?}");
        }
        for lemma in ["the", "of", "casa", "padre", "nunca", "decir"] {
            assert!(
                !is_function_word(lemma, ES),
                "{lemma:?} is not a Spanish function word"
            );
        }
        all_function_words(&["the", "of"]);
        // French has its own (add-lingua-french-analysis D3): `de` is French and Spanish, `the`
        // English only, `ne` French only.
        assert!(is_function_word("de", FR) && is_function_word("de", ES));
        assert!(!is_function_word("de", EN));
        assert!(!is_function_word("the", FR) && !is_function_word("the", ES));
        assert!(is_function_word("ne", FR));
        assert!(!is_function_word("ne", ES) && !is_function_word("ne", EN));
        // Words of the other languages' tables are not French ones.
        for lemma in ["of", "have", "haber", "el", "no", "nunca"] {
            assert!(!is_function_word(lemma, FR), "{lemma:?}");
        }
    }

    #[test]
    fn spec_scenario_a_french_phrase() {
        // « la maison de mon père », lemmatised: the closed classes are flagged, the nouns are
        // not.
        let flagged: Vec<bool> = ["la", "maison", "de", "mon", "père"]
            .iter()
            .map(|lemma| is_function_word(lemma, FR))
            .collect();
        assert_eq!(flagged, [true, false, true, true, false]);
    }

    #[test]
    fn spec_scenario_the_french_negation() {
        // « Il ne fait pas un pas »: both `pas` are flagged — the noun « step » with the
        // negation (M21) —, `fait` (*faire*) is not.
        let flagged: Vec<bool> = ["il", "ne", "faire", "pas", "un", "pas"]
            .iter()
            .map(|lemma| is_function_word(lemma, FR))
            .collect();
        assert_eq!(flagged, [true, true, false, true, true, true]);
    }

    #[test]
    fn spec_scenario_french_words_that_look_closed_but_carry_meaning() {
        for lemma in [
            "personne", "point", "or", "jamais", "plus", "guère", "certain", "faire", "falloir",
            "vouloir", "aller", "voici", "voilà",
        ] {
            assert!(!is_function_word(lemma, FR), "{lemma:?} carries meaning");
        }
    }

    /// Every word of `words` is in `table` and is a French function word.
    fn french_class_holds(table: &[&str], words: &str) {
        for word in words.split_whitespace() {
            assert!(table.binary_search(&word).is_ok(), "{word:?}");
            assert!(is_function_word(word, FR), "{word:?}");
        }
    }

    #[test]
    fn french_determiners_are_function_words() {
        // Articles, demonstratives, possessives and their inflected forms, indefinites and
        // quantifiers.
        french_class_holds(
            FR_DETERMINERS,
            "le la les un une du des ce cet cette ces mon ma mes ton ta tes son notre votre leur \
             quel aucun nul chaque plusieurs quelque tout toutes même autre tel beaucoup peu trop \
             tant",
        );
    }

    #[test]
    fn french_pronouns_are_function_words() {
        // Personal, demonstrative, relative and interrogative, indefinite.
        french_class_holds(
            FR_PRONOUNS,
            "je tu il elle on nous vous ils elles me te se moi toi soi lui eux y en ce ceci cela \
             ça celui celui-ci celui-là qui que quoi dont lequel duquel auquel quelqu'un chacun \
             rien autrui quiconque",
        );
    }

    #[test]
    fn french_prepositions_are_function_words() {
        french_class_holds(
            FR_PREPOSITIONS,
            "à de en dans par pour sur sous avec sans chez entre vers contre depuis pendant \
             durant avant après devant derrière parmi selon malgré envers hors hormis dès jusque \
             outre sauf via près afin",
        );
    }

    #[test]
    fn french_conjunctions_are_function_words() {
        french_class_holds(
            FR_CONJUNCTIONS,
            "et ou mais donc ni car que si quand comme où lorsque puisque quoique parce tandis",
        );
    }

    #[test]
    fn french_auxiliaries_and_modals_are_function_words() {
        french_class_holds(FR_AUXILIARIES_AND_MODALS, "être avoir pouvoir devoir");
        // The other common verbs carry meaning.
        for lemma in ["falloir", "vouloir", "aller", "faire"] {
            assert!(!is_function_word(lemma, FR), "{lemma:?}");
        }
    }

    #[test]
    fn french_negation_is_a_function_word_with_pas_but_not_jamais() {
        french_class_holds(FR_NEGATION, "ne pas non");
        for lemma in ["jamais", "plus", "point", "guère", "personne"] {
            assert!(!is_function_word(lemma, FR), "{lemma:?}");
        }
    }

    #[test]
    fn a_french_word_stands_in_two_classes_only_where_it_reads_two_ways() {
        // Like Spanish's, French's tables may list a word twice — and only these, so a table that
        // went stale is not hidden by a second one.
        let mut seen: Vec<&str> = Vec::new();
        let mut twice: Vec<&str> = Vec::new();
        for table in FRENCH {
            for word in *table {
                if seen.contains(word) {
                    twice.push(word);
                }
                seen.push(word);
            }
        }
        twice.sort_unstable();
        assert_eq!(twice, ["ce", "en", "la", "le", "les", "leur", "que"]);
    }

    #[test]
    fn spec_scenario_a_spanish_phrase() {
        // "la casa de mi padre", lemmatised: the closed classes are flagged, the nouns are not.
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        let flagged: Vec<bool> = ["la", "casa", "de", "mi", "padre"]
            .iter()
            .map(|lemma| is_function_word(lemma, ES))
            .collect();
        assert_eq!(flagged, [true, false, true, true, false]);
    }

    #[test]
    fn spanish_classes_hold_what_they_name() {
        const ES: StudiedLanguage = StudiedLanguage::Spanish;
        for (table, sample) in [
            (ES_DETERMINERS, &["el", "una", "mis", "esta", "todos"][..]),
            (ES_PRONOUNS, &["yo", "nos", "les", "quién", "nadie"][..]),
            (ES_PREPOSITIONS, &["a", "de", "según", "tras"][..]),
            (ES_CONJUNCTIONS, &["y", "pero", "aunque", "u"][..]),
            (
                ES_AUXILIARIES_AND_MODALS,
                &["haber", "ser", "estar", "poder"][..],
            ),
            (ES_NEGATION, &["no"][..]),
        ] {
            for word in sample {
                assert!(table.binary_search(word).is_ok(), "{word:?}");
                assert!(is_function_word(word, ES), "{word:?}");
            }
        }
    }

    #[test]
    fn every_table_is_sorted_and_free_of_duplicates() {
        for table in ENGLISH.iter().chain(SPANISH).chain(FRENCH).chain(ITALIAN) {
            for pair in table.windows(2) {
                assert!(
                    pair[0] < pair[1],
                    "{:?} must sort before {:?}",
                    pair[0],
                    pair[1]
                );
            }
        }
        // An English lemma belongs to one class: listing it twice would hide a
        // table that went stale. (Spanish, French and Italian words may stand in two.)
        let mut all: Vec<&str> = ENGLISH.iter().flat_map(|t| t.iter().copied()).collect();
        let total = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), total, "a lemma is listed in two classes");
    }

    #[test]
    fn articles_and_other_determiners_are_function_words() {
        // Articles, a possessive, a demonstrative and quantifier-determiners.
        all_function_words(&[
            "a", "an", "the", "my", "their", "this", "those", "some", "every",
        ]);
    }

    #[test]
    fn pronouns_are_function_words() {
        // Personal, reflexive, relative, interrogative, indefinite.
        all_function_words(&[
            "i",
            "it",
            "them",
            "myself",
            "who",
            "which",
            "what",
            "something",
        ]);
    }

    #[test]
    fn prepositions_and_particles_are_function_words() {
        all_function_words(&[
            "in", "of", "with", "up", "off", "out", "away", "down", "over",
        ]);
    }

    #[test]
    fn conjunctions_are_function_words() {
        all_function_words(&["and", "but", "or", "because", "although", "if", "while"]);
    }

    #[test]
    fn auxiliaries_and_modals_are_function_words() {
        // The dictionary forms `is`/`has`/`did` reach after lemmatisation.
        all_function_words(&["be", "have", "do", "will", "would", "can", "could", "must"]);
    }

    #[test]
    fn negation_is_a_function_word_but_never_is_not() {
        all_function_words(&["not", "no"]);
        assert!(!is_function_word("never", EN), "an adverb with content");
    }

    #[test]
    fn high_frequency_content_words_are_not_function_words() {
        // The commonest lemmas of English hold exactly the words worth a row.
        for lemma in [
            "time", "say", "people", "person", "know", "spite", "city", "give",
        ] {
            assert!(!is_function_word(lemma, EN), "{lemma:?} carries meaning");
        }
    }
}
