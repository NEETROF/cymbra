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

//! The deck card schema (design D1).
//!
//! A card carries the lemma, the form actually encountered, the originating
//! context sentence, where it was met, an optional gloss, an optional media
//! slot (reserved but never populated in this change), and its FSRS review
//! state. Provenance is captured at creation because the originating sentence
//! can only be taken at the moment of the encounter — it can never be
//! reconstructed afterwards. Every field is serialisable so the versioned
//! schema is a stable contract for backup/restore and the future sync.

use serde::{Deserialize, Serialize};

use super::fsrs::ReviewState;

/// Where a card's encounter came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterSource {
    /// A web page, with its URL.
    Web { url: String },
    /// An AI-agent session (e.g. Claude Code), with an opaque session id.
    AgentSession { session: String },
    /// Imported in bulk (reserved; no importer in the MVP).
    Import,
}

/// How an encounter is stamped in time and place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// The originating context sentence, captured at encounter time.
    pub sentence: String,
    /// Where the encounter happened.
    pub source: EncounterSource,
    /// Unix-epoch seconds of the encounter (supplied by the caller).
    pub captured_at: i64,
}

/// Where an attached media item came from. Reserved for image capture,
/// deferred to a later change; declared now so the schema never migrates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaSource {
    /// A user screenshot/photo.
    Capture,
    /// A public image-bank pick.
    Stock,
    /// A generated image.
    Generated,
}

/// Whether a media item may leave the device. Reserved alongside
/// [`MediaSource`]; local-only by default when populated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncPolicy {
    /// Never leaves the device.
    LocalOnly,
    /// May sync (opt-in), when sync exists.
    Synced,
}

/// An attached media item. The slot exists in the schema from day 1; nothing
/// populates it in this change (image capture is a later change).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Media {
    /// A content-addressed handle to the blob (opaque to the core).
    pub blob_ref: String,
    /// Where the image came from.
    pub source: MediaSource,
    /// Whether it may leave the device.
    pub sync_policy: SyncPolicy,
}

/// A deck card. Keyed elsewhere by `(studied language, lemma)` — one card per
/// lemma; a multi-word expression is a lemma with spaces and thus its own
/// card. Not `Eq`: the FSRS state carries `f64`s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    /// The dictionary form this card teaches.
    pub lemma: String,
    /// The surface form actually encountered (may differ from the lemma).
    pub encountered_form: String,
    /// Where and when it was met, with the context sentence.
    pub provenance: Provenance,
    /// The gloss, written in `gloss_language`, if available at creation.
    pub gloss: Option<String>,
    /// The language the gloss is written in, as an ISO 639-1 tag: the native language of the
    /// engine the card was created on (add-lingua-card-gloss-language D1). `fr` for every card
    /// created before the label existed, and left out of the backup when `fr`, so a French
    /// reader's backup keeps its bytes and a build released before the label reads one that
    /// carries it (`Card` does not deny unknown fields). Read through
    /// [`gloss_language_or_french`]: a blank label in a backup is `fr`.
    #[serde(
        default = "french",
        deserialize_with = "read_gloss_language",
        skip_serializing_if = "is_french"
    )]
    pub gloss_language: String,
    /// Optional attached media (reserved; unpopulated in this change).
    pub media: Option<Media>,
    /// The FSRS review state (never reviewed until first graded).
    pub review: ReviewState,
    /// Last-change time (epoch **seconds** — the deck's native unit, as used by
    /// `grade`/`retire`) for cross-device last-write-wins sync
    /// (`add-lingua-connected-clients`). Stamped at creation (= `captured_at`) and
    /// bumped on every grade / mark-known. `#[serde(default)]` so an older backup
    /// restores with 0 (which loses to any real timestamp). The sync wire uses
    /// millis, so the boundary multiplies by 1000.
    #[serde(default)]
    pub updated_at: i64,
}

/// The gloss language of every card created before the label existed.
pub const FRENCH: &str = "fr";

fn french() -> String {
    FRENCH.to_owned()
}

fn is_french(language: &str) -> bool {
    language == FRENCH
}

/// The gloss language a label names: trimmed and lowercased, so `" EN "` is `en`; blank is
/// [`FRENCH`], the default a label left out means. Any other tag is kept as it is, known to
/// this build or not: the label says what the gloss is, it does not choose a pack.
pub fn gloss_language_or_french(label: &str) -> String {
    let label = label.trim();
    if label.is_empty() {
        french()
    } else {
        label.to_lowercase()
    }
}

fn read_gloss_language<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let label = String::deserialize(deserializer)?;
    Ok(gloss_language_or_french(&label))
}

impl Card {
    /// Creates a fresh, never-reviewed card, its gloss written in `gloss_language` (an ISO
    /// 639-1 tag): the native language of the engine creating it.
    pub fn new(
        lemma: &str,
        encountered_form: &str,
        provenance: Provenance,
        gloss: Option<String>,
        gloss_language: &str,
    ) -> Self {
        let updated_at = provenance.captured_at;
        Self {
            lemma: lemma.to_owned(),
            encountered_form: encountered_form.to_owned(),
            provenance,
            gloss,
            gloss_language: gloss_language.to_owned(),
            media: None,
            review: ReviewState::new(),
            updated_at,
        }
    }

    /// A card seeded for level-targeted feeding (`add-lingua-cefr-levels`),
    /// not from a real reading encounter. There is no originating sentence, so
    /// it is empty, and the source is [`EncounterSource::Import`] rather than a
    /// fabricated URL or agent session. The encountered form is the lemma
    /// itself. The gloss is the pack's, written in `gloss_language`.
    pub fn seeded(lemma: &str, gloss: Option<String>, gloss_language: &str, at: i64) -> Self {
        Card::new(
            lemma,
            lemma,
            Provenance {
                sentence: String::new(),
                source: EncounterSource::Import,
                captured_at: at,
            },
            gloss,
            gloss_language,
        )
    }

    /// Whether the lemma is a multi-word expression.
    pub fn is_expression(&self) -> bool {
        self.lemma.contains(' ')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn web_provenance(sentence: &str, url: &str) -> Provenance {
        Provenance {
            sentence: sentence.to_owned(),
            source: EncounterSource::Web {
                url: url.to_owned(),
            },
            captured_at: 1_700_000_000,
        }
    }

    #[test]
    fn spec_card_created_from_the_popup() {
        let card = Card::new(
            "conundrum",
            "conundrum",
            web_provenance("The real conundrum is trust.", "https://example.com/a"),
            Some("casse-tête".to_owned()),
            "fr",
        );
        assert_eq!(card.lemma, "conundrum");
        assert_eq!(card.gloss_language, "fr");
        assert_eq!(card.provenance.sentence, "The real conundrum is trust.");
        assert!(
            matches!(&card.provenance.source, EncounterSource::Web { url } if url == "https://example.com/a")
        );
        assert!(card.media.is_none());
        assert!(!card.is_expression());
    }

    #[test]
    fn spec_expression_card() {
        let card = Card::new(
            "compelling starting point",
            "compelling starting point",
            web_provenance(
                "The suggestions remain a compelling starting point.",
                "https://example.com/b",
            ),
            None,
            "fr",
        );
        assert!(card.is_expression());
        assert_eq!(
            card.provenance.sentence,
            "The suggestions remain a compelling starting point."
        );
    }

    #[test]
    fn seeded_card_uses_import_and_has_no_sentence() {
        let card = Card::seeded("nuance", Some("nuance".to_owned()), "fr", 1_700_000_000);
        assert_eq!(card.lemma, "nuance");
        assert_eq!(card.encountered_form, "nuance");
        assert_eq!(card.gloss_language, "fr");
        assert_eq!(card.provenance.source, EncounterSource::Import);
        assert!(card.provenance.sentence.is_empty());
        assert_eq!(card.provenance.captured_at, 1_700_000_000);
        assert_eq!(card.updated_at, 1_700_000_000);
    }

    #[test]
    fn card_roundtrips_through_serde() {
        let card = Card::new(
            "run",
            "running",
            web_provenance("They keep running.", "https://x"),
            None,
            "fr",
        );
        let json = serde_json::to_string(&card).expect("serialise");
        let back: Card = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(card, back);
    }

    #[test]
    fn a_french_card_serialises_without_its_gloss_language() {
        // Every card created before the label existed is French-glossed: its bytes do not move.
        let card = Card::new(
            "harbour",
            "harbours",
            web_provenance("Ships rest in the harbours.", "https://x"),
            Some("port".to_owned()),
            "fr",
        );
        let json = serde_json::to_string(&card).expect("serialise");
        assert!(!json.contains("gloss_language"), "{json}");
        let back: Card = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back.gloss_language, "fr");
        assert_eq!(card, back);
    }

    #[test]
    fn an_english_glossed_card_serialises_with_its_gloss_language() {
        let card = Card::new(
            "faro",
            "faros",
            web_provenance("Los faros brillan.", "https://x"),
            Some("lighthouse".to_owned()),
            "en",
        );
        let json: serde_json::Value = serde_json::to_value(&card).expect("serialise");
        assert_eq!(json["gloss_language"], "en");
        let back: Card = serde_json::from_value(json).expect("deserialise");
        assert_eq!(back.gloss_language, "en");
        assert_eq!(card, back);
    }

    #[test]
    fn a_card_without_a_gloss_language_reads_french() {
        // What every build before the label wrote; and a build released before it reads a card
        // that carries the label as it reads any unknown field — by ignoring it.
        let mut json = serde_json::to_value(Card::new(
            "run",
            "running",
            web_provenance("They keep running.", "https://x"),
            Some("courir".to_owned()),
            "en",
        ))
        .expect("serialise");
        let written = json.as_object_mut().expect("an object");
        assert!(written.remove("gloss_language").is_some());
        let back: Card = serde_json::from_value(json.clone()).expect("deserialise");
        assert_eq!(back.gloss_language, "fr");
        assert_eq!(back.gloss.as_deref(), Some("courir"));
        json["unknown_to_this_build"] = "ignored".into();
        assert!(serde_json::from_value::<Card>(json).is_ok());
    }

    #[test]
    fn a_label_is_trimmed_and_lowercased_and_a_blank_one_is_french() {
        assert_eq!(gloss_language_or_french("en"), "en");
        assert_eq!(gloss_language_or_french(" EN "), "en");
        assert_eq!(gloss_language_or_french(" fr "), "fr");
        assert_eq!(gloss_language_or_french(""), "fr");
        assert_eq!(gloss_language_or_french("  "), "fr");
        // A tag this build does not know is kept as it is.
        assert_eq!(gloss_language_or_french("tlh"), "tlh");
    }

    #[test]
    fn a_card_with_a_blank_gloss_language_reads_french() {
        // No build writes a blank label, but a backup carrying one reads as `fr`, as one
        // without the field does — and is written again without it.
        let json = serde_json::to_value(Card::new(
            "run",
            "running",
            web_provenance("They keep running.", "https://x"),
            Some("courir".to_owned()),
            "en",
        ))
        .expect("serialise");
        for blank in ["", "  "] {
            let mut json = json.clone();
            json["gloss_language"] = blank.into();
            let back: Card = serde_json::from_value(json).expect("deserialise");
            assert_eq!(back.gloss_language, "fr", "{blank:?}");
            assert!(
                !serde_json::to_string(&back)
                    .unwrap()
                    .contains("gloss_language")
            );
        }
        let mut json = json;
        json["gloss_language"] = " EN ".into();
        let back: Card = serde_json::from_value(json).expect("deserialise");
        assert_eq!(back.gloss_language, "en");
    }
}
