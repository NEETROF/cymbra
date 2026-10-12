// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The pure core of « Traduction en ligne » (change: add-lingua-online-translation, D3–D6):
//! what a request may hold, the prompt built from it, how a provider's answer is read, where
//! its marks land, and what a request costs. No I/O — the module and the adapters call this.

/// The longest sentence sent, in characters — the extension refuses longer ones too.
pub const MAX_SENTENCE_CHARS: usize = 1_000;

/// The languages a pair is made of (the language matrix): any two different ones.
pub const LANGUAGES: [&str; 3] = ["en", "fr", "es"];

/// The instruction every provider gets. Fixed, so the prompt's prefix never changes; the
/// selection is marked in the source, and the provider names the words that render it, as a
/// list, because a rendering is often split ("remettait … à plus tard").
pub const SYSTEM_PROMPT: &str = r#"You translate one sentence from a book for a language learner.
The learner selected a passage, marked with <b>…</b> in the source.
Return:
- "translation": a natural, grammatical translation of the whole sentence, without any tags.
- "highlighted": the substrings of your translation that render the selected passage, in the order they appear. Copy each one character for character from "translation". Usually there is one; give several only when the rendering is split by other words. Include every word that carries the passage's meaning (the whole idiom or phrasal verb), and nothing that belongs to the rest of the sentence.

Examples:
Source: She <b>gave up</b> smoking last year. → French
{"translation": "Elle a arrêté de fumer l'année dernière.", "highlighted": ["a arrêté"]}
Source: Me <b>tomó el pelo</b> durante toda la cena. → English
{"translation": "He pulled my leg throughout the whole dinner.", "highlighted": ["pulled my leg"]}
Source: He <b>put</b> the meeting <b>off</b> until Monday. → French
{"translation": "Il a repoussé la réunion à lundi.", "highlighted": ["a repoussé"]}
Source: Il <b>a remis</b> la lettre <b>à plus tard</b>. → English
{"translation": "He put the letter off until later.", "highlighted": ["put", "off until later"]}"#;

/// A request the service will relay, once checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    pub before: String,
    pub selection: String,
    pub after: String,
    pub source: &'static str,
    pub target: &'static str,
}

impl Checked {
    /// `<source>-<target>`, the pair the usage figures are kept under.
    pub fn pair(&self) -> String {
        format!("{}-{}", self.source, self.target)
    }
}

/// Why a request is refused before any provider is asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    Empty,
    TooLong,
    BadSelection,
    BadPair,
}

fn language(code: &str) -> Option<&'static str> {
    LANGUAGES.into_iter().find(|l| *l == code)
}

fn language_name(code: &str) -> &'static str {
    match code {
        "fr" => "French",
        "es" => "Spanish",
        _ => "English",
    }
}

/// The byte index of the UTF-16 offset `at` in `s`, or `None` when it falls inside a
/// character or past the end.
fn byte_at_utf16(s: &str, at: u32) -> Option<usize> {
    let mut units = 0u32;
    for (i, c) in s.char_indices() {
        if units == at {
            return Some(i);
        }
        if units > at {
            return None;
        }
        units += c.len_utf16() as u32;
    }
    (units == at).then_some(s.len())
}

/// Check a request: a non-empty sentence within the bound, a non-empty selection inside it,
/// and two different languages of the matrix.
pub fn check(
    sentence: &str,
    start: u32,
    end: u32,
    source: &str,
    target: &str,
) -> Result<Checked, Refusal> {
    if sentence.trim().is_empty() {
        return Err(Refusal::Empty);
    }
    if sentence.chars().count() > MAX_SENTENCE_CHARS {
        return Err(Refusal::TooLong);
    }
    let (Some(source), Some(target)) = (language(source), language(target)) else {
        return Err(Refusal::BadPair);
    };
    if source == target {
        return Err(Refusal::BadPair);
    }
    let (Some(a), Some(b)) = (byte_at_utf16(sentence, start), byte_at_utf16(sentence, end)) else {
        return Err(Refusal::BadSelection);
    };
    if a >= b || sentence[a..b].trim().is_empty() {
        return Err(Refusal::BadSelection);
    }
    Ok(Checked {
        before: sentence[..a].to_string(),
        selection: sentence[a..b].to_string(),
        after: sentence[b..].to_string(),
        source,
        target,
    })
}

/// Page text never carries our tag: a literal `<b>`/`</b>` in it is neutralised so the only
/// mark the provider sees is the selection's.
fn neutral(text: &str) -> String {
    text.replace("<b>", "‹b›").replace("</b>", "‹/b›")
}

/// The user message for one request.
pub fn user_prompt(c: &Checked) -> String {
    format!(
        "Source: {}<b>{}</b>{} → {}",
        neutral(&c.before),
        neutral(&c.selection),
        neutral(&c.after),
        language_name(c.target)
    )
}

/// A provider's answer, once read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub translation: String,
    pub highlighted: Vec<String>,
}

/// Read a provider's structured answer. An answer that is not the expected JSON, has an empty
/// translation, or one far longer than the sentence could be, is unreadable (`None`) — the
/// next provider is asked.
pub fn read_answer(json: &str, c: &Checked) -> Option<Answer> {
    #[derive(serde::Deserialize)]
    struct Wire {
        translation: String,
        #[serde(default)]
        highlighted: Vec<String>,
    }
    let w: Wire = serde_json::from_str(json.trim()).ok()?;
    let translation = w.translation.trim().to_string();
    let sentence_chars =
        c.before.chars().count() + c.selection.chars().count() + c.after.chars().count();
    if translation.is_empty() || translation.chars().count() > sentence_chars * 4 + 200 {
        return None;
    }
    Some(Answer {
        translation,
        highlighted: w.highlighted,
    })
}

/// The marks of `highlighted` in `translation`, as UTF-16 offsets, each found after the
/// previous one. Any segment missing, empty or out of order drops every mark: no mark is better
/// than a misplaced one.
pub fn place_marks(translation: &str, highlighted: &[String]) -> Vec<(u32, u32)> {
    let utf16 = |byte: usize| translation[..byte].encode_utf16().count() as u32;
    let mut out = Vec::with_capacity(highlighted.len());
    let mut from = 0usize;
    for segment in highlighted {
        let segment = segment.trim();
        if segment.is_empty() {
            return Vec::new();
        }
        let Some(rel) = translation[from..].find(segment) else {
            return Vec::new();
        };
        let start = from + rel;
        let end = start + segment.len();
        out.push((utf16(start), utf16(end)));
        from = end;
    }
    out
}

/// The estimated cost of a request in micro-dollars, from prices in dollars per million
/// tokens — a price per million tokens is a price in micro-dollars per token.
pub fn cost_micro_usd(
    input_tokens: i64,
    output_tokens: i64,
    input_price: f64,
    output_price: f64,
) -> i64 {
    let cost =
        input_tokens.max(0) as f64 * input_price + output_tokens.max(0) as f64 * output_price;
    cost.round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16_of(s: &str, part: &str) -> (u32, u32) {
        let b = s.find(part).unwrap();
        let a = s[..b].encode_utf16().count() as u32;
        (a, a + part.encode_utf16().count() as u32)
    }

    #[test]
    fn a_request_is_checked_before_any_provider_is_asked() {
        let s = "He had been putting off the letter.";
        let (a, b) = utf16_of(s, "putting off");
        let c = check(s, a, b, "en", "fr").unwrap();
        assert_eq!(c.selection, "putting off");
        assert_eq!(c.pair(), "en-fr");
        assert_eq!(check("  ", 0, 1, "en", "fr"), Err(Refusal::Empty));
        assert_eq!(
            check(&"a".repeat(1_001), 0, 1, "en", "fr"),
            Err(Refusal::TooLong)
        );
        assert!(check(&"a".repeat(1_000), 0, 1, "en", "fr").is_ok());
        assert_eq!(check(s, a, b, "en", "en"), Err(Refusal::BadPair));
        assert_eq!(check(s, a, b, "de", "fr"), Err(Refusal::BadPair));
        assert_eq!(check(s, b, a, "en", "fr"), Err(Refusal::BadSelection));
        assert_eq!(check(s, 0, 999, "en", "fr"), Err(Refusal::BadSelection));
        assert_eq!(check(s, 2, 3, "en", "fr"), Err(Refusal::BadSelection)); // a space
    }

    #[test]
    fn offsets_are_utf16_as_the_extension_counts_them() {
        // « é » is one UTF-16 unit, an emoji two.
        let s = "Él 😀 se echó a reír.";
        let (a, b) = utf16_of(s, "echó a reír");
        let c = check(s, a, b, "es", "fr").unwrap();
        assert_eq!(c.selection, "echó a reír");
        // An offset inside the emoji's surrogate pair is no character boundary.
        let inside = "Él ".encode_utf16().count() as u32 + 1;
        assert_eq!(check(s, inside, b, "es", "fr"), Err(Refusal::BadSelection));
    }

    #[test]
    fn the_prompt_marks_the_selection_and_nothing_else() {
        let s = "Use <b>bold</b> to stress putting off.";
        let (a, b) = utf16_of(s, "putting off");
        let c = check(s, a, b, "en", "es").unwrap();
        assert_eq!(
            user_prompt(&c),
            "Source: Use ‹b›bold‹/b› to stress <b>putting off</b>. → Spanish"
        );
    }

    fn checked() -> Checked {
        check("Se echó a reír.", 3, 14, "es", "fr").unwrap()
    }

    #[test]
    fn an_answer_is_read_or_refused() {
        let ok = read_answer(
            r#"{"translation": " Il éclata de rire. ", "highlighted": ["éclata de rire"]}"#,
            &checked(),
        );
        assert_eq!(
            ok,
            Some(Answer {
                translation: "Il éclata de rire.".into(),
                highlighted: vec!["éclata de rire".into()]
            })
        );
        assert_eq!(
            read_answer("Wait, I need to apply the tags", &checked()),
            None
        );
        assert_eq!(read_answer(r#"{"translation": ""}"#, &checked()), None);
        let long = format!(r#"{{"translation": "{}"}}"#, "x".repeat(400));
        assert_eq!(read_answer(&long, &checked()), None);
        // No highlighted list is an answer without marks, not a failure.
        assert_eq!(
            read_answer(r#"{"translation": "Il rit."}"#, &checked())
                .unwrap()
                .highlighted,
            Vec::<String>::new()
        );
    }

    #[test]
    fn marks_follow_the_segments_in_order() {
        let t = "Il remettait la lettre à plus tard depuis des semaines.";
        let marks = place_marks(t, &["remettait".into(), "à plus tard".into()]);
        assert_eq!(
            marks,
            vec![utf16_of(t, "remettait"), utf16_of(t, "à plus tard")]
        );
        // A missing segment drops every mark rather than misplacing one.
        assert!(place_marks(t, &["remettait".into(), "postponed".into()]).is_empty());
        // Order matters: a segment is looked for after the previous one.
        assert!(place_marks(t, &["à plus tard".into(), "remettait".into()]).is_empty());
        assert!(place_marks(t, &[" ".into()]).is_empty());
        // A repeated word: the second segment is the occurrence after the first.
        let r = "la la land";
        assert_eq!(
            place_marks(r, &["la".into(), "la".into()]),
            vec![(0, 2), (3, 5)]
        );
        // Non-ASCII before the mark counts in UTF-16 units.
        let e = "😀 éclata de rire";
        assert_eq!(
            place_marks(e, &["éclata".into()]),
            vec![utf16_of(e, "éclata")]
        );
    }

    #[test]
    fn the_cost_is_in_micro_dollars() {
        // 634 in × $0.25/M + 60 out × $1.50/M = 158.5 + 90 µ$.
        assert_eq!(cost_micro_usd(634, 60, 0.25, 1.50), 249);
        assert_eq!(cost_micro_usd(0, 0, 0.25, 1.50), 0);
        assert_eq!(cost_micro_usd(-5, 10, 1.0, 1.0), 10);
    }
}
