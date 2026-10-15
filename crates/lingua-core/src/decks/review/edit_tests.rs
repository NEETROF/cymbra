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

//! The edit of the card under review (add-lingua-card-edit), scenario by scenario.

use std::collections::BTreeMap;

use super::*;
use crate::analysis::language::StudiedLanguage;
use crate::decks::card::{EncounterSource, Provenance};
use crate::decks::fsrs::{FsrsParams, Rating};
use crate::decks::review::SessionSummary;
use crate::knowledge::status::KnownSource;

const EN: StudiedLanguage = StudiedLanguage::English;
const SENTENCE: &str = "The four-poster bed is colloquially referred to as a tester.";
const NOW: i64 = 5_000;

fn card(lemma: &str, gloss: Option<&str>) -> Card {
    let mut card = Card::new(
        lemma,
        lemma,
        Provenance {
            sentence: SENTENCE.to_owned(),
            source: EncounterSource::Web {
                url: "https://example.com".to_owned(),
            },
            captured_at: 100,
        },
        gloss.map(str::to_owned),
        "fr",
    );
    card.updated_at = 100;
    card
}

/// A session over `lemmas`, in that order, at its first card.
fn queued(lemmas: &[&str]) -> ReviewSession {
    ReviewSession {
        queue: lemmas.iter().map(|l| (EN, (*l).to_owned())).collect(),
        position: 0,
        revealed: false,
        asked: BTreeMap::new(),
        summary: SessionSummary::default(),
        practice: false,
    }
}

fn deck_of(cards: Vec<Card>) -> Deck {
    let mut deck = Deck::new();
    for card in cards {
        deck.upsert(EN, card);
    }
    deck
}

/// The pack of the card's language, as the binding hands it in: a lemma's gloss, if any.
fn lookup(entries: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
    move |lemma| {
        entries
            .iter()
            .find(|(l, _)| *l == lemma)
            .map(|(_, g)| (*g).to_owned())
    }
}

fn save<'a>(form: &'a str, gloss: &'a str, shown: Option<&'a str>) -> CardEdit<'a> {
    CardEdit {
        form,
        gloss,
        shown_gloss: shown,
        native: "fr",
        now: NOW,
    }
}

/// Edits the current card of `session` over `deck` with no pack and no status.
fn edit_plainly(
    session: &mut ReviewSession,
    deck: &mut Deck,
    form: &str,
) -> Result<EditOutcome, EditError> {
    session.edit(
        deck,
        &mut KnowledgeState::new(),
        &save(form, "", None),
        &lookup(&[]),
    )
}

#[test]
fn a_form_is_trimmed_collapsed_stripped_and_bounded() {
    let long = "a".repeat(FORM_MAX_UNITS);
    let too_long = "a".repeat(FORM_MAX_UNITS + 1);
    let cases: [(&str, Result<&str, EditError>); 9] = [
        ("four-poster bed,", Ok("four-poster bed")),
        ("  referred \t to\n as  ", Ok("referred to as")),
        ("\u{ab} Referred to as \u{bb}", Ok("Referred to as")),
        ("cafe\u{301}", Ok("caf\u{e9}")),
        ("   ", Err(EditError::Empty)),
        (", \u{2014} .", Err(EditError::Empty)),
        (&long, Ok(&long)),
        (&too_long, Err(EditError::FormTooLong)),
        ("d'abord", Ok("d'abord")),
    ];
    for (typed, saved) in cases {
        assert_eq!(
            normalised_form(typed).as_deref(),
            saved.as_ref().map(|s| *s),
            "{typed:?}"
        );
    }
    // A letter of two UTF-16 units: JavaScript's `length`.
    assert_eq!(
        normalised_form(&"\u{1d400}".repeat(61)),
        Err(EditError::FormTooLong)
    );
}

#[test]
fn a_translation_is_collapsed_empty_is_none_and_bounded() {
    let long = "é".repeat(GLOSS_MAX_UNITS);
    let too_long = "é".repeat(GLOSS_MAX_UNITS + 1);
    let cases: [(&str, Result<Option<&str>, EditError>); 5] = [
        ("  se  relâcher ", Ok(Some("se relâcher"))),
        ("", Ok(None)),
        (" \n ", Ok(None)),
        (&long, Ok(Some(&long))),
        (&too_long, Err(EditError::GlossTooLong)),
    ];
    for (typed, saved) in cases {
        assert_eq!(
            normalised_gloss(typed),
            saved.map(|g| g.map(str::to_owned)),
            "{typed:?}"
        );
    }
    let names = [
        EditError::Empty,
        EditError::FormTooLong,
        EditError::GlossTooLong,
    ]
    .map(EditError::wire);
    assert_eq!(names, ["empty", "form_too_long", "gloss_too_long"]);
}

#[test]
fn the_occurrence_is_a_whole_word_letter_case_aside_as_written() {
    let cases = [
        (SENTENCE, "referred to as", Some("referred to as")),
        (SENTENCE, "Referred To As", Some("referred to as")),
        (SENTENCE, "four-poster bed", Some("four-poster bed")),
        (SENTENCE, "poster", Some("poster")),
        (SENTENCE, "test", None),
        (SENTENCE, "ferred", None),
        (SENTENCE, "", None),
        ("Une \u{c9}cole.", "école", Some("\u{c9}cole")),
        ("short", "a much longer form", None),
    ];
    for (sentence, form, found) in cases {
        assert_eq!(
            occurrence(sentence, form),
            found,
            "{form:?} in {sentence:?}"
        );
    }
}

/// (stored gloss, its language, the reader's mark).
type Gloss<'a> = (Option<&'a str>, &'a str, bool);

#[test]
fn spec_an_edit_settles_the_cards_gloss_the_readers_own_first() {
    const PACK: &[(&str, &str)] = &[
        ("referred to as", "appelé"),
        ("colloquially referred to as", "familièrement appelé"),
    ];
    let old = "colloquially referred to as";
    // (stored, shown in the field, saved form, saved translation) → settled
    let cases: [(Gloss, Option<&str>, &str, &str, Gloss); 7] = [
        // The pack knows the new form.
        (
            (Some("familièrement appelé"), "fr", false),
            Some("familièrement appelé"),
            "referred to as",
            "familièrement appelé",
            (Some("appelé"), "fr", false),
        ),
        // The pack does not: the text the reader saw stays.
        (
            (Some("familièrement appelé"), "fr", false),
            Some("familièrement appelé"),
            "referred to a tester",
            "familièrement appelé",
            (Some("familièrement appelé"), "fr", false),
        ),
        // The field showed the pack's gloss over an English text: that is what stays.
        (
            (Some("colloquially called"), "en", false),
            Some("familièrement appelé"),
            "referred to a tester",
            "familièrement appelé",
            (Some("familièrement appelé"), "fr", false),
        ),
        // The reader's translation wins over the pack's.
        (
            (None, "fr", false),
            None,
            "referred to as",
            "  dénommé ",
            (Some("dénommé"), "fr", true),
        ),
        // A reader's translation outlives a later form edit, labelled as it was.
        (
            (Some("so-called"), "en", true),
            Some("so-called"),
            "referred to as",
            "so-called",
            (Some("so-called"), "en", true),
        ),
        // Withdrawing one's translation: the pack's for the form…
        (
            (Some("dénommé"), "fr", true),
            Some("dénommé"),
            old,
            "",
            (Some("familièrement appelé"), "fr", false),
        ),
        // …or none.
        (
            (Some("dénommé"), "fr", true),
            Some("dénommé"),
            "referred to a tester",
            "",
            (None, "fr", false),
        ),
    ];
    for (i, ((gloss, language, mark), shown, form, typed, settled)) in cases.into_iter().enumerate()
    {
        let mut edited = card(old, gloss);
        edited.gloss_language = language.to_owned();
        edited.gloss_by_reader = mark;
        let mut deck = deck_of(vec![edited]);
        let mut session = queued(&[old]);
        let outcome = session
            .edit(
                &mut deck,
                &mut KnowledgeState::new(),
                &save(form, typed, shown),
                &lookup(PACK),
            )
            .expect("accepted");
        assert!(outcome.changed, "case {i}");
        let saved = session.current(&deck).expect("still current");
        assert_eq!(saved.lemma, form, "case {i}");
        assert_eq!(
            (
                saved.gloss.as_deref(),
                saved.gloss_language.as_str(),
                saved.gloss_by_reader
            ),
            settled,
            "case {i}"
        );
    }
}

#[test]
fn a_save_that_changes_nothing_and_a_refused_one_write_nothing() {
    let original = deck_of(vec![card("seldom", Some("rarement"))]);
    let too_long = "x".repeat(GLOSS_MAX_UNITS + 1);
    let cases = [
        (save("seldom", "rarement", Some("rarement")), Ok(false)),
        (save(" seldom.", " rarement ", Some("rarement")), Ok(false)),
        (
            save(" ,", "rarement", Some("rarement")),
            Err(EditError::Empty),
        ),
        (
            save("seldom", &too_long, Some("rarement")),
            Err(EditError::GlossTooLong),
        ),
    ];
    for (edit, expected) in cases {
        let mut deck = original.clone();
        let mut knowledge = KnowledgeState::new();
        let mut session = queued(&["seldom"]);
        let got = session
            .edit(&mut deck, &mut knowledge, &edit, &lookup(&[]))
            .map(|o| o.changed);
        assert_eq!(got, expected, "{edit:?}");
        assert_eq!(deck, original, "{edit:?}");
        assert_eq!(knowledge, KnowledgeState::new());
        assert!(!session.is_revealed());
    }
    // Without a card under review, nothing happens.
    let mut deck = original.clone();
    let outcome = edit_plainly(&mut queued(&[]), &mut deck, "x").unwrap();
    assert_eq!(outcome, EditOutcome::default());
    assert_eq!(deck, original);
}

#[test]
fn spec_a_free_selection_trimmed_is_renamed_marked_and_removed_under_its_old_form() {
    let old = "colloquially referred to as";
    let shown = "familièrement appelé";
    let mut graded = card(old, Some(shown));
    graded.review.reps = 2;
    graded.review.last_review = Some(50);
    let mut deck = deck_of(vec![graded.clone(), card("tester", None)]);
    let mut knowledge = KnowledgeState::new();
    knowledge.set_status_at(EN, old, Status::Learning, 100_000);
    let mut session = queued(&[old, "tester"]);
    let outcome = session
        .edit(
            &mut deck,
            &mut knowledge,
            &save("referred to as", shown, Some(shown)),
            &lookup(&[]),
        )
        .unwrap();
    assert_eq!(
        outcome,
        EditOutcome {
            changed: true,
            merged: None,
            retired: false
        }
    );
    assert!(session.is_revealed(), "still revealed");
    assert_eq!(session.remaining(), 2);
    let renamed = session.current(&deck).unwrap();
    assert_eq!(renamed.lemma, "referred to as");
    assert_eq!(renamed.encountered_form, "referred to as");
    assert_eq!(
        renamed.provenance.sentence, SENTENCE,
        "the sentence never changes"
    );
    assert_eq!(
        renamed.review, graded.review,
        "not an answer: no schedule moves"
    );
    assert_eq!(renamed.updated_at, NOW);
    assert_eq!(
        session.summary(),
        SessionSummary::default(),
        "no review counted"
    );
    assert!(deck.get(EN, old).is_none());
    assert_eq!(deck.removed_at(EN, old), Some(NOW));
    // D9: the old form's `learning` cleared, the new one `learning`, both stamped.
    assert_eq!(knowledge.explicit_status(EN, old), None);
    assert_eq!(knowledge.status_updated_at(EN, old), NOW * 1000);
    assert_eq!(
        knowledge.explicit_status(EN, "referred to as"),
        Some(Status::Learning)
    );
    assert_eq!(
        knowledge.status_updated_at(EN, "referred to as"),
        NOW * 1000
    );
    // A form the sentence does not hold: the form itself is the form met.
    session
        .edit(
            &mut deck,
            &mut knowledge,
            &save("termed", shown, Some(shown)),
            &lookup(&[]),
        )
        .unwrap();
    assert_eq!(session.current(&deck).unwrap().encountered_form, "termed");
    // The next answer is the card's first: graded as usual.
    assert!(session.grade(&mut deck, &FsrsParams::default(), Rating::Good, NOW));
    assert_eq!(deck.get(EN, "termed").unwrap().review.reps, 3);
}

#[test]
fn spec_a_renamed_card_moves_its_words_status() {
    let known = Some(Status::Known(KnownSource::Manual));
    let (learning, ignored) = (Some(Status::Learning), Some(Status::Ignored));
    // (old form's status, new form's status) → (old's after, new's after, retired)
    type Statuses = (Option<Status>, Option<Status>);
    type After = (Option<Status>, Option<Status>, bool);
    let cases: [(Statuses, After); 5] = [
        ((learning, None), (None, learning, false)),
        ((ignored, None), (ignored, learning, false)),
        ((None, learning), (None, learning, false)),
        ((learning, known), (None, known, true)),
        ((None, ignored), (None, ignored, true)),
    ];
    let (old, new) = ("four-poster bed,", "four-poster bed");
    for ((old_status, new_status), (old_after, new_after, retired)) in cases {
        let mut deck = deck_of(vec![card(old, None), card("tester", None)]);
        let mut knowledge = KnowledgeState::new();
        for (lemma, status) in [(old, old_status), (new, new_status)] {
            if let Some(status) = status {
                knowledge.set_status_at(EN, lemma, status, 1_000);
            }
        }
        let mut session = queued(&[old, "tester"]);
        let outcome = session
            .edit(
                &mut deck,
                &mut knowledge,
                &save(old, "", None),
                &lookup(&[]),
            )
            .unwrap();
        let case = format!("{old_status:?} → {new_status:?}");
        assert_eq!(outcome.retired, retired, "{case}");
        assert_eq!(knowledge.explicit_status(EN, old), old_after, "{case}");
        assert_eq!(knowledge.explicit_status(EN, new), new_after, "{case}");
        let renamed = deck.get(EN, new).unwrap();
        assert_eq!(!renamed.review.is_due(i64::MAX - 1), retired, "{case}");
        let current = session.current_key().map(|(_, l)| l.as_str());
        assert_eq!(
            current,
            Some(if retired { "tester" } else { new }),
            "{case}"
        );
    }
}

/// A card with `reps` reviews, the last at `last`, captured at `captured`.
fn history(lemma: &str, reps: u32, last: Option<i64>, captured: i64) -> Card {
    let mut c = card(lemma, None);
    c.review.reps = reps;
    c.review.last_review = last;
    c.provenance.captured_at = captured;
    c
}

#[test]
fn spec_an_edited_form_already_in_the_deck_merges_the_two_cards() {
    // (edited: reps, last review) (other: reps, last review) → whether the edited history wins
    type Side = (u32, Option<i64>);
    let cases: [(Side, Side, bool); 5] = [
        ((2, Some(10)), (6, Some(5)), false),
        ((6, Some(5)), (2, Some(10)), true),
        ((3, Some(20)), (3, Some(10)), true),
        ((3, Some(10)), (3, Some(20)), false),
        ((3, Some(10)), (3, Some(10)), false),
    ];
    for ((e_reps, e_last), (o_reps, o_last), edited_wins) in cases {
        let edited = history("colloquially referred to as", e_reps, e_last, 300);
        let mut other = history("referred to as", o_reps, o_last, 200);
        other.provenance.sentence = "Referred to as the old one.".to_owned();
        let expected = if edited_wins {
            edited.review.clone()
        } else {
            other.review.clone()
        };
        let mut deck = deck_of(vec![edited, other]);
        let mut session = queued(&["colloquially referred to as"]);
        let outcome = edit_plainly(&mut session, &mut deck, "Referred to as").unwrap();
        let case = format!("{e_reps}/{e_last:?} vs {o_reps}/{o_last:?}");
        assert_eq!(outcome.merged.as_deref(), Some("referred to as"), "{case}");
        assert_eq!(deck.len(), 1, "{case}: one card remains");
        let survivor = deck.get(EN, "referred to as").unwrap();
        assert_eq!(survivor.review, expected, "{case}");
        assert_eq!(survivor.provenance.sentence, SENTENCE, "{case}");
        assert_eq!(survivor.encountered_form, "referred to as", "{case}");
        assert_eq!(survivor.provenance.captured_at, 200, "{case}");
        assert_eq!(survivor.updated_at, NOW, "{case}");
        assert_eq!(
            deck.removed_at(EN, "colloquially referred to as"),
            Some(NOW)
        );
        assert!(session.is_revealed(), "{case}");
    }
}

#[test]
fn spec_the_readers_translation_survives_a_merge_from_either_side() {
    // (edited: gloss, mark) (other: gloss, mark) (typed) → merged (gloss, mark)
    type Side<'a> = (Option<&'a str>, bool);
    let cases: [(Side, Side, &str, Side); 5] = [
        (
            (Some("appelé"), false),
            (Some("dit"), true),
            "appelé",
            (Some("dit"), true),
        ),
        (
            (Some("appelé"), false),
            (Some("dit"), false),
            "nommé",
            (Some("nommé"), true),
        ),
        (
            (Some("nommé"), true),
            (Some("dit"), true),
            "nommé",
            (Some("nommé"), true),
        ),
        (
            (Some("appelé"), false),
            (Some("dit"), false),
            "appelé",
            (Some("appelé"), false),
        ),
        (
            (None, false),
            (Some("dit"), false),
            "",
            (Some("dit"), false),
        ),
    ];
    for ((e_gloss, e_mark), (o_gloss, o_mark), typed, merged) in cases {
        let mut edited = card("colloquially referred to as", e_gloss);
        edited.gloss_by_reader = e_mark;
        let mut other = card("referred to as", o_gloss);
        other.gloss_by_reader = o_mark;
        other.media = Some(crate::decks::card::Media {
            blob_ref: "b".to_owned(),
            source: crate::decks::card::MediaSource::Capture,
            sync_policy: crate::decks::card::SyncPolicy::LocalOnly,
        });
        let mut deck = deck_of(vec![edited, other]);
        let mut session = queued(&["colloquially referred to as"]);
        session
            .edit(
                &mut deck,
                &mut KnowledgeState::new(),
                &save("referred to as", typed, e_gloss),
                &lookup(&[]),
            )
            .unwrap();
        let survivor = deck.get(EN, "referred to as").unwrap();
        assert_eq!(
            (survivor.gloss.as_deref(), survivor.gloss_by_reader),
            merged,
            "{e_gloss:?}/{e_mark} + {o_gloss:?}/{o_mark}, typed {typed:?}"
        );
        assert!(survivor.media.is_some(), "the other card's media");
    }
}

fn rest(session: &ReviewSession) -> Vec<&str> {
    session.queue[session.position..]
        .iter()
        .map(|(_, l)| l.as_str())
        .collect()
}

#[test]
fn spec_a_merge_drops_the_other_card_from_the_session_and_keeps_its_answer() {
    let params = FsrsParams::default();
    let two = || {
        deck_of(vec![
            card("referred to as", None),
            card("colloquially", None),
        ])
    };

    // The card merged into is due later in the session: it does not come up again.
    let mut deck = deck_of(vec![
        card("a", None),
        card("b", None),
        card("colloquially", None),
        card("referred to as", None),
    ]);
    let mut session = queued(&["a", "colloquially", "b", "referred to as"]);
    session.grade(&mut deck, &params, Rating::Good, NOW);
    edit_plainly(&mut session, &mut deck, "referred to as").unwrap();
    assert_eq!(rest(&session), ["referred to as", "b"]);

    // The card merged into was answered earlier: the merged card's answer moves nothing.
    let mut deck = two();
    let mut session = queued(&["referred to as", "colloquially"]);
    assert!(session.grade(&mut deck, &params, Rating::Good, NOW));
    let reps = deck.get(EN, "referred to as").unwrap().review.reps;
    edit_plainly(&mut session, &mut deck, "referred to as").unwrap();
    assert_eq!(rest(&session), ["referred to as"]);
    assert!(!session.grade(&mut deck, &params, Rating::Good, NOW + 60));
    assert_eq!(deck.get(EN, "referred to as").unwrap().review.reps, reps);

    // A missed card renamed comes back under its new form, its first answer already given.
    let mut deck = two();
    let mut session = queued(&["colloquially", "referred to as"]);
    assert!(session.grade(&mut deck, &params, Rating::Again, NOW));
    assert!(session.grade(&mut deck, &params, Rating::Good, NOW));
    edit_plainly(&mut session, &mut deck, "termed").unwrap();
    assert_eq!(session.current_key(), Some(&(EN, "termed".to_owned())));
    assert!(!session.grade(&mut deck, &params, Rating::Good, NOW));
    assert_eq!(session.summary().recovered, 1);
}

#[test]
fn spec_only_the_letter_case_renames_the_card() {
    let mut deck = deck_of(vec![history("Referred to as", 4, Some(10), 100)]);
    let mut session = queued(&["Referred to as"]);
    let outcome = edit_plainly(&mut session, &mut deck, "referred to as").unwrap();
    assert_eq!(outcome.merged, None);
    assert_eq!(deck.len(), 1);
    assert_eq!(
        deck.get(EN, "referred to as").unwrap().review.reps,
        4,
        "with its history"
    );
    assert_eq!(deck.removed_at(EN, "Referred to as"), Some(NOW));
}

#[test]
fn a_translation_written_for_the_same_form_changes_only_the_gloss() {
    let mut deck = deck_of(vec![card("seldom", None)]);
    let mut knowledge = KnowledgeState::new();
    let mut session = queued(&["seldom"]);
    let outcome = session
        .edit(
            &mut deck,
            &mut knowledge,
            &save("seldom", "rarement", None),
            &lookup(&[]),
        )
        .unwrap();
    assert!(outcome.changed);
    let saved = deck.get(EN, "seldom").unwrap();
    assert_eq!(
        (saved.gloss.as_deref(), saved.gloss_by_reader),
        (Some("rarement"), true)
    );
    assert_eq!(saved.updated_at, NOW);
    assert!(deck.export_removals().is_empty(), "nothing removed");
    assert_eq!(knowledge, KnowledgeState::new(), "no status moves");
    assert!(session.is_revealed());
}

#[test]
fn spec_a_card_removed_by_an_edit_stays_removed_under_last_write_wins() {
    // A removal at 100, then: (the local card's time, if any; an incoming card's time) →
    // (card held after the removal, card held after the incoming copy)
    let cases: [(Option<i64>, i64, bool, bool); 6] = [
        // A stale copy arrives later: refused, the card stays removed.
        (Some(50), 80, false, false),
        // A copy of the very removal time loses the tie.
        (Some(50), 100, false, false),
        // A newer copy brings it back, the removal forgotten.
        (Some(50), 150, false, true),
        // Graded after the removal time: kept.
        (Some(150), 120, true, true),
        // No card here: the removal is recorded all the same.
        (None, 80, false, false),
        (None, 150, false, true),
    ];
    for (local, incoming, kept, back) in cases {
        let mut deck = Deck::new();
        if let Some(at) = local {
            let mut c = card("bed,", None);
            c.updated_at = at;
            deck.upsert(EN, c);
        }
        let changed = deck.apply_card_removal_lww(EN, "bed,", 100);
        let case = format!("{local:?}, then {incoming}");
        assert_eq!(changed, !kept, "{case}");
        assert_eq!(deck.get(EN, "bed,").is_some(), kept, "{case}");
        assert_eq!(deck.removed_at(EN, "bed,").is_some(), !kept, "{case}");
        let mut copy = card("bed,", None);
        copy.updated_at = incoming;
        deck.apply_card_lww(EN, copy);
        assert_eq!(deck.get(EN, "bed,").is_some(), back, "{case}");
        if back {
            assert_eq!(deck.removed_at(EN, "bed,"), None, "{case}");
        }
    }
    // A later removal moves the time, an older one does not; an echo changes nothing.
    let mut deck = Deck::new();
    assert!(deck.apply_card_removal_lww(EN, "bed,", 100));
    assert!(!deck.apply_card_removal_lww(EN, "bed,", 100));
    assert!(!deck.apply_card_removal_lww(EN, "bed,", 50));
    assert!(deck.apply_card_removal_lww(EN, "bed,", 200));
    assert_eq!(deck.export_removals(), [(EN, "bed,".to_owned(), 200)]);
    // Captured again: the card exists again, the removal forgotten.
    deck.upsert(EN, card("bed,", None));
    assert!(deck.export_removals().is_empty());
    assert!(deck.removal_languages().is_empty());
}

#[test]
fn a_deck_without_a_removal_serialises_as_before() {
    let mut deck = deck_of(vec![card("seldom", None)]);
    let plain = serde_json::to_string(&deck).unwrap();
    assert!(!plain.contains("tombstones"), "{plain}");
    deck.apply_card_removal_lww(StudiedLanguage::Spanish, "faro,", 100);
    let json = serde_json::to_value(&deck).unwrap();
    assert_eq!(json["tombstones"]["Spanish"]["faro,"], 100);
    let back: Deck = serde_json::from_value(json).unwrap();
    assert_eq!(back, deck);
    assert_eq!(
        back.removal_languages().into_iter().collect::<Vec<_>>(),
        [StudiedLanguage::Spanish]
    );
}
