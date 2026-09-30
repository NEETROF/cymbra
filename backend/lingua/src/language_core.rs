// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The one rule for every studied-language value the module receives — statuses,
//! declared levels, daily stats and cards (add-lingua-card-language). Pure and
//! host-tested; the gRPC adapters apply it at the edge. The server decides nothing about
//! which languages exist: it only keeps `es`, `ES` and `es-ES` from splitting one
//! language into three keys, and reads an absent value as the language every client
//! sent before the field existed.

/// The language a client that predates the field is understood to mean.
pub const DEFAULT_LANGUAGE: &str = "en";

/// Longest primary subtag kept (ISO 639 codes are 2-3 letters; BCP 47 allows 8).
const MAX_LEN: usize = 8;

/// Normalise one language value: trimmed, lowercased, reduced to its primary subtag
/// (`es-ES`, `es_419`, `spa-ESP` → `es`, `es`, `spa`), capped at [`MAX_LEN`] bytes,
/// and [`DEFAULT_LANGUAGE`] when nothing is left. Never fails: an unknown code is a
/// language the server has not met yet, not an error.
pub fn normalise(raw: &str) -> String {
    let primary = raw
        .trim()
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    if primary.is_empty() {
        return DEFAULT_LANGUAGE.to_owned();
    }
    primary.chars().take(MAX_LEN).collect()
}

/// The languages a pull accepts: each normalised, deduplicated in first-seen order, and
/// English only when the client named none — the client that predates card languages.
pub fn accepted_languages(list: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(list.len().max(1));
    for raw in list {
        if raw.trim().is_empty() {
            continue;
        }
        let lang = normalise(raw);
        if !out.contains(&lang) {
            out.push(lang);
        }
    }
    if out.is_empty() {
        out.push(DEFAULT_LANGUAGE.to_owned());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regional_and_upper_case_codes_fold_onto_the_primary_subtag() {
        assert_eq!(normalise("es-ES"), "es");
        assert_eq!(normalise("ES"), "es");
        assert_eq!(normalise("es_419"), "es");
        assert_eq!(normalise("spa-ESP"), "spa");
        assert_eq!(normalise("  En "), "en");
    }

    #[test]
    fn nothing_left_reads_as_english() {
        assert_eq!(normalise(""), "en");
        assert_eq!(normalise("   "), "en");
        assert_eq!(normalise("-ES"), "en");
    }

    #[test]
    fn an_over_long_value_is_capped_not_refused() {
        assert_eq!(normalise("abcdefghijklmnop"), "abcdefgh");
    }

    #[test]
    fn a_language_never_seen_is_kept_as_is() {
        assert_eq!(normalise("pt"), "pt");
    }

    #[test]
    fn an_empty_accepted_set_means_english_only() {
        assert_eq!(accepted_languages(&[]), vec!["en"]);
        assert_eq!(accepted_languages(&[String::new(), " ".into()]), vec!["en"]);
    }

    #[test]
    fn accepted_languages_are_normalised_and_deduplicated_in_order() {
        let list = [
            "es-ES".to_string(),
            "en".into(),
            "ES".into(),
            "pt_BR".into(),
        ];
        assert_eq!(accepted_languages(&list), vec!["es", "en", "pt"]);
    }
}
