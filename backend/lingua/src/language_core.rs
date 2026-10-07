// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! The one rule for every language value the module receives — the studied language of
//! statuses, declared levels, daily stats and cards (add-lingua-card-language), and the
//! language of a card's gloss or of a device's native tongue
//! (add-lingua-native-language-server). Pure and host-tested; the gRPC adapters apply it
//! at the edge. The server decides nothing about which languages exist: it only keeps
//! `es`, `ES` and `es-ES` from splitting one language into three keys, and reads an
//! absent value as the language every client sent before the field existed — `en` for a
//! studied language, `fr` for a gloss or a native language.

/// The studied language a client that predates the field is understood to mean.
pub const DEFAULT_LANGUAGE: &str = "en";

/// The language of every gloss ever shipped, and the native language a client that
/// predates the field is understood to mean (add-lingua-native-language-server).
pub const DEFAULT_NATIVE_LANGUAGE: &str = "fr";

/// Longest primary subtag kept (ISO 639 codes are 2-3 letters; BCP 47 allows 8).
const MAX_LEN: usize = 8;

/// Normalise one language value: trimmed, lowercased, reduced to its primary subtag
/// (`es-ES`, `es_419`, `spa-ESP` → `es`, `es`, `spa`), capped at [`MAX_LEN`] bytes,
/// and [`DEFAULT_LANGUAGE`] when nothing is left. Never fails: an unknown code is a
/// language the server has not met yet, not an error.
pub fn normalise(raw: &str) -> String {
    normalise_or(raw, DEFAULT_LANGUAGE)
}

/// [`normalise`], given the language an empty value is read as: [`DEFAULT_LANGUAGE`]
/// for a studied language, [`DEFAULT_NATIVE_LANGUAGE`] for the language of a gloss or a
/// device's native language (add-lingua-native-language-server). The same rule
/// otherwise, and it still refuses nothing.
pub fn normalise_or(raw: &str, default: &str) -> String {
    let primary = raw
        .trim()
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    if primary.is_empty() {
        return default.to_owned();
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
    fn a_gloss_or_native_language_reads_as_french_when_empty() {
        assert_eq!(normalise_or("", DEFAULT_NATIVE_LANGUAGE), "fr");
        assert_eq!(normalise_or("   ", DEFAULT_NATIVE_LANGUAGE), "fr");
        assert_eq!(normalise_or("-MX", DEFAULT_NATIVE_LANGUAGE), "fr");
        // The studied language keeps its own default.
        assert_eq!(normalise(""), "en");
        assert_eq!(normalise_or("", DEFAULT_LANGUAGE), "en");
    }

    #[test]
    fn a_regional_native_language_folds_onto_its_primary_subtag() {
        assert_eq!(normalise_or("es-MX", DEFAULT_NATIVE_LANGUAGE), "es");
        assert_eq!(normalise_or("EN", DEFAULT_NATIVE_LANGUAGE), "en");
        assert_eq!(normalise_or(" Pt_BR ", DEFAULT_NATIVE_LANGUAGE), "pt");
        // The same rule as the studied language's, whatever the default.
        assert_eq!(
            normalise_or("es-ES", DEFAULT_NATIVE_LANGUAGE),
            normalise("es-ES")
        );
    }

    #[test]
    fn a_native_language_never_seen_is_kept_and_an_over_long_one_capped() {
        assert_eq!(normalise_or("tlh", DEFAULT_NATIVE_LANGUAGE), "tlh");
        assert_eq!(
            normalise_or("abcdefghijklmnop", DEFAULT_NATIVE_LANGUAGE),
            "abcdefgh"
        );
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
