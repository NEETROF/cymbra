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

//! A built pack measured against a treebank (add-lingua-spanish-forms-tables D5).
//!
//! Every syntactic word of a CoNLL-U file goes through lingua-core's lemmatiser with the pack's
//! lexicon, exactly as a page token does. Punctuation, numbers, symbols, foreign words, proper
//! nouns and words without a letter are left out: the reader counts none of them. Three figures
//! come out — how many tokens resolve in the lexicon, and how many content words and auxiliaries
//! take the treebank's lemma — each held to the gate the programme set.

use lingua_core::analysis::lemmatize::lemmatize;
use lingua_core::analysis::lexicon::Lexicon;
use lingua_core::packs::pack::Pack;

/// The programme's gates on UD Spanish-PUD (`docs/lingua/spanish-programme.md`, change 20).
pub const MIN_RESOLVED: f64 = 98.5;
/// Content words: nouns, verbs, adjectives, adverbs.
pub const MIN_CONTENT: f64 = 93.5;
/// Auxiliaries.
pub const MIN_AUX: f64 = 97.0;

/// What a pack does with a treebank's words.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Measurement {
    /// Words measured.
    pub tokens: usize,
    /// Words whose lemma the lexicon holds.
    pub resolved: usize,
    /// Content words, and those taking the treebank's lemma.
    pub content: usize,
    pub content_agree: usize,
    /// Auxiliaries, and those taking the treebank's lemma.
    pub aux: usize,
    pub aux_agree: usize,
}

fn percent(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        100.0
    } else {
        100.0 * part as f64 / whole as f64
    }
}

impl Measurement {
    pub fn resolved_percent(&self) -> f64 {
        percent(self.resolved, self.tokens)
    }

    pub fn content_percent(&self) -> f64 {
        percent(self.content_agree, self.content)
    }

    pub fn aux_percent(&self) -> f64 {
        percent(self.aux_agree, self.aux)
    }

    /// The figures in one line, as `lingua-pack-measure` prints them.
    pub fn summary(&self) -> String {
        format!(
            "{} words: {:.2} % resolved, {:.2} % of {} content words and {:.2} % of {} auxiliaries \
             take the treebank's lemma",
            self.tokens,
            self.resolved_percent(),
            self.content_percent(),
            self.content,
            self.aux_percent(),
            self.aux,
        )
    }

    /// The gates this measurement misses, in words; none when it passes.
    pub fn failures(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (name, got, gate) in [
            ("tokens resolved", self.resolved_percent(), MIN_RESOLVED),
            ("content-word lemmas", self.content_percent(), MIN_CONTENT),
            ("auxiliary lemmas", self.aux_percent(), MIN_AUX),
        ] {
            if got < gate {
                out.push(format!("{name}: {got:.2} % < {gate} %"));
            }
        }
        out
    }
}

/// The parts of speech the reader never counts.
const LEFT_OUT: &[&str] = &["PUNCT", "NUM", "SYM", "X", "PROPN"];

/// Measures `pack` on the syntactic words of a CoNLL-U text.
pub fn measure(pack: &Pack, conllu: &str) -> Measurement {
    let studied = pack.studied();
    let lexicon = pack.lexicon();
    let mut m = Measurement::default();
    for line in conllu.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        // Multi-word tokens (`del` = 4-5) and empty nodes (8.1) are not syntactic words.
        if cols.len() < 4 || cols[0].contains('-') || cols[0].contains('.') {
            continue;
        }
        let (form, gold, upos) = (cols[1], cols[2], cols[3]);
        if LEFT_OUT.contains(&upos) || !form.chars().any(char::is_alphabetic) {
            continue;
        }
        let lemma = lemmatize(form, studied, lexicon);
        m.tokens += 1;
        m.resolved += usize::from(lexicon.contains(&lemma));
        let agrees = lemma == gold.to_lowercase();
        match upos {
            "NOUN" | "VERB" | "ADJ" | "ADV" => {
                m.content += 1;
                m.content_agree += usize::from(agrees);
            }
            "AUX" => {
                m.aux += 1;
                m.aux_agree += usize::from(agrees);
            }
            _ => {}
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PackInputs, build_pack};
    use lingua_core::packs::meta::PackMeta;

    /// A tiny Spanish pack: `casas` → casa, `fue` → ser, `dar`, `ser`, `de`, `el`.
    fn spanish_pack() -> Pack {
        let inputs = PackInputs {
            meta: PackMeta {
                studied: "es".into(),
                native: "fr".into(),
                pack_version: "test".into(),
                analyzer_version: lingua_core::analysis::SPANISH_ANALYZER_VERSION.into(),
                levels_estimated: false,
                licences: vec!["test".into()],
            },
            sources: vec![],
            notice: "test".into(),
            form_lemma: vec![
                ("casas".into(), "casa".into()),
                ("fue".into(), "ser".into()),
                ("da".into(), "dar".into()),
            ],
            ranks: vec![
                ("casa".into(), 1),
                ("ser".into(), 2),
                ("dar".into(), 3),
                ("de".into(), 4),
                ("el".into(), 5),
            ],
            glosses: vec![],
            levels: vec![],
            expressions: vec![],
            readings: vec![],
            senses: vec![],
            lexical: None,
            tag_pool: None,
        };
        Pack::load(&build_pack(&inputs).expect("build")).expect("load")
    }

    const CONLLU: &str = "# sent_id = 1\n\
1\tLas\tel\tDET\t_\t_\t2\tdet\t_\t_\n\
2\tcasas\tcasa\tNOUN\t_\t_\t0\troot\t_\t_\n\
3-4\tdel\t_\t_\t_\t_\t_\t_\t_\t_\n\
3\tde\tde\tADP\t_\t_\t5\tcase\t_\t_\n\
4\tel\tel\tDET\t_\t_\t5\tdet\t_\t_\n\
5\tMadrid\tMadrid\tPROPN\t_\t_\t2\tnmod\t_\t_\n\
6\tfue\tser\tAUX\t_\t_\t2\tcop\t_\t_\n\
7\tdámelo\tdar\tVERB\t_\t_\t2\tdep\t_\t_\n\
8\tzzz\tzzz\tNOUN\t_\t_\t2\tdep\t_\t_\n\
9\t.\t.\tPUNCT\t_\t_\t2\tpunct\t_\t_\n";

    #[test]
    fn words_go_through_the_analyser_and_what_the_reader_skips_is_left_out() {
        let m = measure(&spanish_pack(), CONLLU);
        // Las, casas, de, el, fue, dámelo, zzz — not the range, Madrid or the full stop.
        assert_eq!(m.tokens, 7);
        // `las` resolves through nothing (no form of `el` in this pack); `zzz` neither.
        assert_eq!(m.resolved, 5);
        // casas, dámelo (the enclitic rule), zzz.
        assert_eq!((m.content, m.content_agree), (3, 3));
        assert_eq!((m.aux, m.aux_agree), (1, 1));
    }

    #[test]
    fn gates_name_what_they_miss() {
        let m = Measurement {
            tokens: 100,
            resolved: 98,
            content: 100,
            content_agree: 95,
            aux: 10,
            aux_agree: 10,
        };
        assert_eq!(m.failures(), ["tokens resolved: 98.00 % < 98.5 %"]);
        assert_eq!(
            m.summary(),
            "100 words: 98.00 % resolved, 95.00 % of 100 content words and 100.00 % of 10 \
             auxiliaries take the treebank's lemma"
        );
        let pass = Measurement {
            resolved: 99,
            ..m.clone()
        };
        assert!(pass.failures().is_empty());
    }
}
