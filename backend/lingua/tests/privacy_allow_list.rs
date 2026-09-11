// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Privacy allow-list contract (task 4.3): the `cymbra.lingua.v1` wire protocol must
//! carry no field for a read page's URL, its text, a per-site/per-page counter or any
//! browsing-history signal. The proof is a schema review pinned as a test: parse the
//! field identifiers out of every proto and assert none names a forbidden signal.
//!
//! The single legitimate origin a user may attach — a card's `source` — is the sole
//! exception, and it is exact-matched so a `source_url` or `source_history` could never
//! slip through under its cover.

/// Field names that are always allowed even though a substring rule might flag them.
/// `source` is the user's own attached card origin (allow-listed by design); it is the
/// only URL-shaped value in the protocol and lives nowhere but a card the user made.
const ALLOWED_EXACT: &[&str] = &["source", "source_sentence"];

/// Substrings that would betray page-level tracking if they appeared in a field name.
const FORBIDDEN_SUBSTRINGS: &[&str] = &[
    "url", "page", "history", "site", "visit", "referrer", "tab", "domain", "path", "title",
    "hostname", "dwell", "scroll",
];

/// Extract `<field_name>` from proto message field lines (`<type> <name> = <n>;`),
/// ignoring comments, so the assertion is about the schema, not its prose.
fn field_names(proto: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in proto.lines() {
        // Drop line comments so a doc mention of "URL" never counts as a field.
        let line = raw.split("//").next().unwrap_or("").trim();
        // A field line ends in `= <number>;` and is inside a message, not a service.
        let Some(eq) = line.find('=') else { continue };
        if !line.ends_with(';') || line.starts_with("rpc ") || line.contains('(') {
            continue;
        }
        // The identifier is the last whitespace-separated token before `=`.
        let Some(name) = line[..eq].split_whitespace().next_back() else {
            continue;
        };
        if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !name.is_empty() {
            out.push(name.to_string());
        }
    }
    out
}

#[test]
fn no_proto_field_names_a_browsing_signal() {
    let protos = [
        ("known_words", include_str!("../proto/known_words.proto")),
        ("deck", include_str!("../proto/deck.proto")),
        ("stats", include_str!("../proto/stats.proto")),
    ];
    let mut checked = 0usize;
    for (name, proto) in protos {
        for field in field_names(proto) {
            checked += 1;
            if ALLOWED_EXACT.contains(&field.as_str()) {
                continue;
            }
            for bad in FORBIDDEN_SUBSTRINGS {
                assert!(
                    !field.contains(bad),
                    "{name}.proto field `{field}` matches forbidden browsing signal `{bad}` \
                     — the lingua wire protocol must not carry page/URL/history data"
                );
            }
        }
    }
    // Guard the guard: if the parser silently matched nothing, the test proves nothing.
    assert!(
        checked >= 20,
        "expected to inspect the proto fields, saw {checked}"
    );
}

#[test]
fn the_card_source_is_the_only_url_shaped_field() {
    // The one allow-listed origin exists (a card's `source`) and nothing url-shaped
    // exists outside it: `source` appears in deck.proto and in no other proto.
    let deck = include_str!("../proto/deck.proto");
    assert!(
        field_names(deck).iter().any(|f| f == "source"),
        "a card must keep its user-attached `source`"
    );
    for other in [
        include_str!("../proto/known_words.proto"),
        include_str!("../proto/stats.proto"),
    ] {
        assert!(
            !field_names(other).iter().any(|f| f == "source"),
            "only a deck card may carry a `source`"
        );
    }
}

#[test]
fn the_admin_proto_carries_no_account_identifier() {
    // The ops console is aggregates-only (change: add-lingua-back-office, D2): no
    // `lingua_admin.proto` response message may carry a field attributable to an
    // account. A leak would require a `.proto` change — this pins it as a test.
    // (Substrings like "account" are fine: `active_accounts` is a COUNT; the concern
    // is an *identifier* field, so identifiers are matched exactly.)
    let proto = include_str!("../proto/lingua_admin.proto");
    let account_identifiers = [
        "user_id",
        "account_id",
        "owner_id",
        "uid",
        "user",
        "account",
        "handle",
        "email",
        "subject",
    ];
    let mut checked = 0usize;
    for field in field_names(proto) {
        checked += 1;
        assert!(
            !account_identifiers.contains(&field.as_str()),
            "lingua_admin.proto field `{field}` names an account identifier — the ops \
             console must serve aggregates only"
        );
    }
    assert!(
        checked >= 15,
        "expected to inspect the admin proto fields, saw {checked}"
    );
}
