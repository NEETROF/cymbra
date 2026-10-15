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

//! Editing the card under review (add-lingua-card-edit): the checks on an edited form and
//! translation (D3), the gloss an edit settles (D4), merging into a card the deck already holds
//! (D6), the removal a rename records (D7), the form the renamed card shows (D8), the statuses that
//! follow it (D9), and the session re-keyed onto the new card (D10).

use unicode_normalization::UnicodeNormalization;

use crate::knowledge::state::KnowledgeState;
use crate::knowledge::status::Status;

use super::super::card::{Card, without_edge_punctuation};
use super::super::fsrs::ReviewState;
use super::{Asked, Deck, ReviewSession};

/// The longest edited form, in UTF-16 code units — JavaScript's `length`, capture's own limit
/// (`MAX_SELECTION_LENGTH`).
pub const FORM_MAX_UNITS: usize = 120;
/// The longest edited translation, in UTF-16 code units.
pub const GLOSS_MAX_UNITS: usize = 200;

/// Why an edit was refused (D3): the page shows each in the interface language, never as text
/// from here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditError {
    /// The form is empty once trimmed of its spaces and edge punctuation.
    Empty,
    /// The form is longer than [`FORM_MAX_UNITS`].
    FormTooLong,
    /// The translation is longer than [`GLOSS_MAX_UNITS`].
    GlossTooLong,
}

impl EditError {
    /// The error's name on the binding's answer.
    pub fn wire(self) -> &'static str {
        match self {
            EditError::Empty => "empty",
            EditError::FormTooLong => "form_too_long",
            EditError::GlossTooLong => "gloss_too_long",
        }
    }
}

/// `text` in NFC, its runs of whitespace collapsed to one space, trimmed.
fn collapsed(text: &str) -> String {
    let nfc: String = text.nfc().collect();
    nfc.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// An edited form as it is saved (D3): NFC, whitespace collapsed, edge punctuation stripped by the
/// rule capture applies to a selection ([`without_edge_punctuation`]), letter case kept; refused
/// empty or over [`FORM_MAX_UNITS`].
pub fn normalised_form(text: &str) -> Result<String, EditError> {
    let collapsed = collapsed(text);
    let form = without_edge_punctuation(&collapsed);
    match form.encode_utf16().count() {
        0 => Err(EditError::Empty),
        n if n > FORM_MAX_UNITS => Err(EditError::FormTooLong),
        _ => Ok(form.to_owned()),
    }
}

/// An edited translation as it is saved (D3): NFC, whitespace collapsed; empty is none — the
/// reader has no translation of their own —, refused over [`GLOSS_MAX_UNITS`].
pub fn normalised_gloss(text: &str) -> Result<Option<String>, EditError> {
    let gloss = collapsed(text);
    if gloss.encode_utf16().count() > GLOSS_MAX_UNITS {
        Err(EditError::GlossTooLong)
    } else {
        Ok((!gloss.is_empty()).then_some(gloss))
    }
}

/// The first place `form` is written in `sentence` as a whole word, letter case aside, taken as
/// the sentence writes it (D8) — the place the review's front marks (`markWord`).
pub fn occurrence<'s>(sentence: &'s str, form: &str) -> Option<&'s str> {
    let wanted: Vec<char> = form.chars().flat_map(char::to_lowercase).collect();
    // Each lowercased character, with where its original character starts and whether it is the
    // first character that original lowercases to: a match starts and ends on whole characters.
    let lower: Vec<(char, usize, bool)> = sentence
        .char_indices()
        .flat_map(|(at, c)| {
            c.to_lowercase()
                .enumerate()
                .map(move |(n, l)| (l, at, n == 0))
        })
        .collect();
    let n = wanted.len();
    if n == 0 || n > lower.len() {
        return None;
    }
    (0..=lower.len() - n).find_map(|k| {
        let whole = lower[k].2 && lower.get(k + n).is_none_or(|next| next.2);
        if !whole
            || !lower[k..k + n]
                .iter()
                .map(|x| x.0)
                .eq(wanted.iter().copied())
        {
            return None;
        }
        let (from, to) = (
            lower[k].1,
            lower.get(k + n).map_or(sentence.len(), |next| next.1),
        );
        let alone = !sentence[..from]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric)
            && !sentence[to..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric);
        alone.then(|| &sentence[from..to])
    })
}

/// What the reader saved in the form, and what the binding knows around it.
#[derive(Debug, Clone, Copy)]
pub struct CardEdit<'a> {
    /// The word or expression field, as typed.
    pub form: &'a str,
    /// The translation field, as typed.
    pub gloss: &'a str,
    /// What the translation field was filled with: the gloss the review showed, if any (D2).
    pub shown_gloss: Option<&'a str>,
    /// The engine's native language, labelling a gloss the edit writes (D4).
    pub native: &'a str,
    /// The edit's time, epoch seconds.
    pub now: i64,
}

/// What an accepted edit did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditOutcome {
    /// Whether anything was written: a save that changes nothing writes nothing (D2).
    pub changed: bool,
    /// The lemma of the card the edited one was merged into, as the deck spells it (D6).
    pub merged: Option<String>,
    /// The new form is one the reader made known or ignored: the card was retired and the
    /// session moved on (D9).
    pub retired: bool,
}

/// Writes a gloss the edit settled.
fn set_gloss(card: &mut Card, gloss: Option<String>, language: &str, by_reader: bool) {
    card.gloss = gloss;
    card.gloss_language = language.to_owned();
    card.gloss_by_reader = by_reader;
}

/// Settles the edited card's gloss for `form` (D4): a translation the reader changed to some text
/// is theirs; one they emptied gives the pack's gloss, else none; one left as it was filled keeps a
/// gloss the reader wrote, else takes the pack's for the new form, else keeps the text the reader
/// saw in the field.
fn settle_gloss(
    card: &mut Card,
    form: &str,
    typed: Option<String>,
    shown: Option<String>,
    edit: &CardEdit,
    pack_gloss: &dyn Fn(&str) -> Option<String>,
) {
    if typed != shown {
        match typed {
            Some(text) => set_gloss(card, Some(text), edit.native, true),
            None => set_gloss(card, pack_gloss(form), edit.native, false),
        }
    } else if card.gloss_by_reader {
        // The reader's own words outlive a later form edit.
    } else if let Some(gloss) = pack_gloss(form) {
        set_gloss(card, Some(gloss), edit.native, false);
    } else if shown != card.gloss {
        // The field showed the pack's gloss of the old form: that is the text the reader kept.
        set_gloss(card, shown, edit.native, false);
    }
}

/// Whether the edited card's review state is the better evidence (D6): more reviews, then the more
/// recent last review; the other card's on a full tie.
fn edited_history_wins(edited: &ReviewState, other: &ReviewState) -> bool {
    (edited.reps, edited.last_review) > (other.reps, other.last_review)
}

/// The card `edited` and `other` become (D6), under `other`'s lemma.
fn merged(edited: Card, other: &Card) -> Card {
    let mut out = edited;
    out.lemma = other.lemma.clone();
    if !edited_history_wins(&out.review, &other.review) {
        out.review = other.review.clone();
    }
    if !out.gloss_by_reader && (other.gloss_by_reader || out.gloss.is_none()) {
        set_gloss(
            &mut out,
            other.gloss.clone(),
            &other.gloss_language,
            other.gloss_by_reader,
        );
    }
    out.provenance.captured_at = out.provenance.captured_at.min(other.provenance.captured_at);
    if other.media.is_some() {
        out.media = other.media.clone();
    }
    out
}

impl ReviewSession {
    /// Edits the card under review (add-lingua-card-edit D10): checks the form and translation
    /// (D3), settles the gloss (D4) — `pack_gloss` reads the pack of the card's language at a lemma
    /// —, renames the card or merges it into another (D6, D7, D8), moves the statuses (D9), and
    /// re-keys the session onto the card, which stays current and revealed — unless its new form is
    /// known or ignored, which retires it and moves on. Not an answer: no review is counted and no
    /// schedule moves. A refused edit changes nothing; without a current card, nothing happens.
    pub fn edit(
        &mut self,
        deck: &mut Deck,
        knowledge: &mut KnowledgeState,
        edit: &CardEdit,
        pack_gloss: &dyn Fn(&str) -> Option<String>,
    ) -> Result<EditOutcome, EditError> {
        let form = normalised_form(edit.form)?;
        let typed = normalised_gloss(edit.gloss)?;
        let Some((lang, old)) = self.current_key().cloned() else {
            return Ok(EditOutcome::default());
        };
        let Some(mut card) = deck.get(lang, &old).cloned() else {
            return Ok(EditOutcome::default());
        };
        let shown = edit.shown_gloss.map(collapsed).filter(|g| !g.is_empty());
        if form == old && typed == shown {
            return Ok(EditOutcome::default());
        }
        let other = (form != old)
            .then(|| deck.other_card_named(lang, &form, &old))
            .flatten();
        let key = other.clone().unwrap_or(form);
        settle_gloss(&mut card, &key, typed, shown, edit, pack_gloss);
        card.updated_at = edit.now;
        if key == old {
            deck.upsert(lang, card);
            self.revealed = true;
            return Ok(EditOutcome {
                changed: true,
                ..EditOutcome::default()
            });
        }

        card.encountered_form = occurrence(&card.provenance.sentence, &key)
            .unwrap_or(&key)
            .to_owned();
        let card = match other.as_deref().and_then(|lemma| deck.get(lang, lemma)) {
            Some(target) => merged(card, target),
            None => Card {
                lemma: key.clone(),
                ..card
            },
        };
        deck.remove(lang, &old, edit.now);
        deck.upsert(lang, card);

        let at_ms = edit.now * 1000;
        if knowledge.explicit_status(lang, &old) == Some(Status::Learning) {
            knowledge.clear_status_at(lang, &old, at_ms);
        }
        let retired = match knowledge.explicit_status(lang, &key) {
            None => {
                knowledge.set_status_at(lang, &key, Status::Learning, at_ms);
                false
            }
            Some(Status::Learning) => false,
            Some(_) => deck.retire(lang, &key, edit.now),
        };

        self.rekey((lang, old), (lang, key), other.is_some());
        if retired {
            self.advance();
        } else {
            self.revealed = true;
        }
        Ok(EditOutcome {
            changed: true,
            merged: other,
            retired,
        })
    }

    /// Moves the session from `old` to `new` (D6, D10): every place of `old` becomes `new`'s; on a
    /// merge, a later place of the card merged into is dropped, so the word does not come twice,
    /// and its asks this session are carried over, so a card already answered is not graded again.
    fn rekey(&mut self, old: Key, new: Key, merge: bool) {
        let position = self.position;
        self.queue = std::mem::take(&mut self.queue)
            .into_iter()
            .enumerate()
            .filter_map(|(at, key)| {
                if key == old {
                    Some(new.clone())
                } else if merge && key == new && at > position {
                    None
                } else {
                    Some(key)
                }
            })
            .collect();
        let edited = self.asked.remove(&old);
        let other = if merge { self.asked.remove(&new) } else { None };
        let asked = match (edited, other) {
            (Some(e), Some(o)) => Some(Asked {
                times: e.times.max(o.times),
                missed: e.missed,
            }),
            (e, o) => e.or(o),
        };
        if let Some(asked) = asked {
            self.asked.insert(new, asked);
        }
    }
}

type Key = (crate::analysis::language::StudiedLanguage, String);

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
