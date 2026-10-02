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

//! Message rendering (change: add-discord-notifications, design D9).
//!
//! Pure: the worker loads the subject's public fields into a card and this module
//! turns it into a message in the server's configured locale. Every field that
//! came from a user (a proposed score's title, an uploaded font's name) is
//! **escaped**, so a title cannot inject a link, a heading or a mention; the
//! sender also disables mention parsing and link embeds on top of it. The only
//! link in a message is ours: the Cymbra Music page of the site, in the server's
//! locale.

use crate::event::{AnnouncementEvent, RecordMode};

/// The server locale (design D9): one per server, English by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    En,
    Fr,
}

impl Locale {
    /// `fr` → French, anything else → English.
    pub fn parse(s: &str) -> Self {
        if s.trim().eq_ignore_ascii_case("fr") {
            Locale::Fr
        } else {
            Locale::En
        }
    }
}

/// A rendered Discord message: plain `content` (under Discord's 2000-character
/// cap) and/or one embed (the periodic reports).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub content: String,
    pub embed: Option<Embed>,
}

/// A Discord embed: a title, a description (up to 4096 characters — room for a
/// ranking) and an optional footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Embed {
    pub title: String,
    pub description: String,
    pub footer: Option<String>,
}

impl Message {
    /// A plain-text message.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            embed: None,
        }
    }

    /// A message made of one embed and no text.
    pub fn embed(embed: Embed) -> Self {
        Self {
            content: String::new(),
            embed: Some(embed),
        }
    }
}

/// The public fields of an accepted SoundFont.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundFontCard {
    pub label: String,
    /// `keyboard` or `percussion`.
    pub instrument: String,
    pub license: String,
    pub attribution: Option<String>,
}

/// The public fields of an accepted catalog score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreCard {
    pub title: Option<String>,
    pub composer: Option<String>,
}

/// Longest user-supplied field kept in a message.
const MAX_FIELD_CHARS: usize = 120;

/// Neutralize a user-supplied field: one line, bounded, Markdown and mentions
/// escaped. An empty result means "nothing to show".
pub fn escape(raw: &str) -> String {
    let one_line = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::with_capacity(one_line.len());
    for (i, c) in one_line.chars().enumerate() {
        if i == MAX_FIELD_CHARS {
            out.push('…');
            break;
        }
        match c {
            '\\' | '*' | '_' | '~' | '`' | '|' | '>' | '#' | '[' | ']' | '(' | ')' | '<' | ':'
            | '-' => {
                out.push('\\');
                out.push(c);
            }
            // A zero-width space after `@` defeats @everyone / @here / <@id>.
            '@' => out.push_str("@\u{200B}"),
            _ => out.push(c),
        }
    }
    out
}

fn field(raw: Option<&str>) -> Option<String> {
    raw.map(escape).filter(|s| !s.is_empty())
}

/// "Title — Composer", or just the title; `None` when the piece has no title.
fn piece(card: &ScoreCard) -> Option<String> {
    let title = field(card.title.as_deref())?;
    Some(match field(card.composer.as_deref()) {
        Some(composer) => format!("**{title}** — {composer}"),
        None => format!("**{title}**"),
    })
}

/// A SoundFont accepted into the catalog. `None` when it has no name to show.
pub fn soundfont_accepted(card: &SoundFontCard, locale: Locale) -> Option<Message> {
    let label = field(Some(&card.label))?;
    let license = field(Some(&card.license));
    let credit = field(card.attribution.as_deref());
    let percussion = card.instrument == "percussion";
    let mut lines = Vec::new();
    match locale {
        Locale::En => {
            lines.push(format!("🎹 **New sound in the catalog: {label}**"));
            lines.push(if percussion { "Drum kit" } else { "Keyboard" }.to_string());
            if let Some(l) = license {
                lines.push(format!("Licence: {l}"));
            }
            if let Some(c) = credit {
                lines.push(format!("Credit: {c}"));
            }
            lines.push("Available now in [Cymbra Music](https://cymbra.app/en/music/).".into());
        }
        Locale::Fr => {
            lines.push(format!("🎹 **Nouveau son au catalogue : {label}**"));
            lines.push(if percussion { "Batterie" } else { "Clavier" }.to_string());
            if let Some(l) = license {
                lines.push(format!("Licence : {l}"));
            }
            if let Some(c) = credit {
                lines.push(format!("Crédit : {c}"));
            }
            lines.push(
                "Disponible dès maintenant dans [Cymbra Music](https://cymbra.app/music/).".into(),
            );
        }
    }
    Some(Message::text(lines.join("\n")))
}

/// A score accepted into the catalog. `None` when it has no title to show.
pub fn score_accepted(card: &ScoreCard, locale: Locale) -> Option<Message> {
    let piece = piece(card)?;
    let content = match locale {
        Locale::En => {
            format!(
                "🎼 **New score in the catalog:** {piece}\nPlay it now in [Cymbra Music](https://cymbra.app/en/music/)."
            )
        }
        Locale::Fr => format!(
            "🎼 **Nouvelle partition au catalogue :** {piece}\nÀ jouer dès maintenant dans [Cymbra Music](https://cymbra.app/music/)."
        ),
    };
    Some(Message::text(content))
}

/// A season record beaten on a catalog piece — anonymous: the piece, the mode and
/// the figure, never the player. `None` when the piece has no title to show.
pub fn season_record(
    card: &ScoreCard,
    mode: RecordMode,
    subscore: f32,
    locale: Locale,
) -> Option<Message> {
    let piece = piece(card)?;
    let figure = subscore.clamp(0.0, 100.0);
    let content = match locale {
        Locale::En => {
            let mode = match mode {
                RecordMode::Tempo => "Free play",
                RecordMode::Reaction => "Wait Mode",
            };
            format!(
                "🏆 **New season record** on {piece}\n{mode}: **{figure:.1} %**\nThink you can beat it? Play it in [Cymbra Music](https://cymbra.app/en/music/)."
            )
        }
        Locale::Fr => {
            let mode = match mode {
                RecordMode::Tempo => "Jeu libre",
                RecordMode::Reaction => "Mode Attente",
            };
            let figure = format!("{figure:.1}").replace('.', ",");
            format!(
                "🏆 **Nouveau record de la saison** sur {piece}\n{mode} : **{figure} %**\nTu peux faire mieux ? Joue-la dans [Cymbra Music](https://cymbra.app/music/)."
            )
        }
    };
    Some(Message::text(content))
}

/// An item accepted into the catalog, as the grouped announcement lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogItem {
    Score { id: String, card: ScoreCard },
    SoundFont { id: String, card: SoundFontCard },
}

impl CatalogItem {
    /// The same key as the item's own [`AnnouncementEvent`], so an item is
    /// announced once in its life whichever message carries it.
    pub fn dedup_key(&self) -> String {
        match self {
            CatalogItem::Score { id, .. } => AnnouncementEvent::ScoreAccepted {
                catalog_score_id: id.clone(),
            }
            .dedup_key(),
            CatalogItem::SoundFont { id, .. } => AnnouncementEvent::SoundFontAccepted {
                soundfont_id: id.clone(),
            }
            .dedup_key(),
        }
    }

    /// Whether the item has a name to show; one without is never announced.
    pub fn is_displayable(&self) -> bool {
        match self {
            CatalogItem::Score { card, .. } => piece(card).is_some(),
            CatalogItem::SoundFont { card, .. } => field(Some(&card.label)).is_some(),
        }
    }

    fn line(&self, locale: Locale) -> Option<String> {
        match self {
            CatalogItem::Score { card, .. } => piece(card).map(|p| format!("• {p}")),
            CatalogItem::SoundFont { card, .. } => {
                let label = field(Some(&card.label))?;
                Some(match locale {
                    Locale::En => format!("• 🎹 **{label}** (sound)"),
                    Locale::Fr => format!("• 🎹 **{label}** (son)"),
                })
            }
        }
    }
}

/// How many items a grouped announcement lists by name before "…and N more".
pub const MAX_LISTED: usize = 10;

/// Room kept for the heading and footer under Discord's 2000-character cap.
const LIST_BUDGET_CHARS: usize = 1700;

fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// One message for every item accepted in a grouping window: a single item keeps
/// its own detailed message; several are counted, the first [`MAX_LISTED`] named
/// (fewer if their names are long), the rest summed up. `None` when no item has
/// a name to show.
pub fn catalog_batch(items: &[CatalogItem], locale: Locale) -> Option<Message> {
    let shown: Vec<&CatalogItem> = items.iter().filter(|i| i.is_displayable()).collect();
    match shown.as_slice() {
        [] => return None,
        [CatalogItem::Score { card, .. }] => return score_accepted(card, locale),
        [CatalogItem::SoundFont { card, .. }] => return soundfont_accepted(card, locale),
        _ => {}
    }
    let scores = shown
        .iter()
        .filter(|i| matches!(i, CatalogItem::Score { .. }))
        .count();
    let sounds = shown.len() - scores;
    let heading = match locale {
        Locale::En => {
            let parts: Vec<String> = [
                (scores > 0).then(|| count(scores, "new score", "new scores")),
                (sounds > 0).then(|| count(sounds, "new sound", "new sounds")),
            ]
            .into_iter()
            .flatten()
            .collect();
            format!("🎼 **{} in the catalog**", parts.join(" and "))
        }
        Locale::Fr => {
            let parts: Vec<String> = [
                (scores > 0).then(|| count(scores, "nouvelle partition", "nouvelles partitions")),
                (sounds > 0).then(|| count(sounds, "nouveau son", "nouveaux sons")),
            ]
            .into_iter()
            .flatten()
            .collect();
            format!("🎼 **{} au catalogue**", parts.join(" et "))
        }
    };
    let mut lines = vec![heading];
    let mut used = 0;
    let mut listed = 0;
    for item in shown.iter().take(MAX_LISTED) {
        let Some(line) = item.line(locale) else {
            continue;
        };
        if used + line.chars().count() > LIST_BUDGET_CHARS {
            break;
        }
        used += line.chars().count() + 1;
        lines.push(line);
        listed += 1;
    }
    let more = shown.len() - listed;
    if more > 0 {
        lines.push(match locale {
            Locale::En => format!("…and {more} more."),
            Locale::Fr => format!("… et {more} autres."),
        });
    }
    lines.push(
        match locale {
            Locale::En => "Play them now in [Cymbra Music](https://cymbra.app/en/music/).",
            Locale::Fr => {
                "À découvrir dès maintenant dans [Cymbra Music](https://cymbra.app/music/)."
            }
        }
        .into(),
    );
    Some(Message::text(lines.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score_item(id: &str, title: &str) -> CatalogItem {
        CatalogItem::Score {
            id: id.into(),
            card: ScoreCard {
                title: Some(title.into()),
                composer: Some("Bach".into()),
            },
        }
    }

    fn font_item(id: &str) -> CatalogItem {
        CatalogItem::SoundFont {
            id: id.into(),
            card: font(),
        }
    }

    #[test]
    fn a_single_item_keeps_its_detailed_message() {
        assert_eq!(
            catalog_batch(&[score_item("c1", "Air")], Locale::En),
            score_accepted(
                &ScoreCard {
                    title: Some("Air".into()),
                    composer: Some("Bach".into())
                },
                Locale::En
            )
        );
        assert_eq!(
            catalog_batch(&[font_item("f")], Locale::Fr),
            soundfont_accepted(&font(), Locale::Fr)
        );
        assert_eq!(catalog_batch(&[], Locale::En), None);
    }

    #[test]
    fn several_items_are_counted_and_listed() {
        let items = [
            score_item("a", "Air"),
            score_item("b", "Gavotte"),
            font_item("f"),
        ];
        let en = catalog_batch(&items, Locale::En).unwrap().content;
        assert_eq!(
            en,
            "🎼 **2 new scores and 1 new sound in the catalog**\n• **Air** — Bach\n• **Gavotte** — Bach\n• 🎹 **Upright Piano KW** (sound)\nPlay them now in [Cymbra Music](https://cymbra.app/en/music/)."
        );
        let fr = catalog_batch(&items, Locale::Fr).unwrap().content;
        assert!(fr.starts_with("🎼 **2 nouvelles partitions et 1 nouveau son au catalogue**"));
        assert!(
            fr.contains("(son)")
                && fr.ends_with(
                    "À découvrir dès maintenant dans [Cymbra Music](https://cymbra.app/music/)."
                )
        );
        let sounds = catalog_batch(&[font_item("f"), font_item("g")], Locale::Fr)
            .unwrap()
            .content;
        assert!(sounds.starts_with("🎼 **2 nouveaux sons au catalogue**"));
        let one_score = catalog_batch(&[score_item("a", "Air"), font_item("f")], Locale::En)
            .unwrap()
            .content;
        assert!(one_score.starts_with("🎼 **1 new score and 1 new sound"));
    }

    #[test]
    fn a_long_batch_names_ten_and_sums_up_the_rest() {
        let items: Vec<CatalogItem> = (0..500)
            .map(|i| score_item(&i.to_string(), &format!("Piece {i}")))
            .collect();
        let en = catalog_batch(&items, Locale::En).unwrap().content;
        assert!(en.starts_with("🎼 **500 new scores in the catalog**"));
        assert_eq!(en.matches("• ").count(), MAX_LISTED);
        assert!(en.contains("…and 490 more."));
        let fr = catalog_batch(&items, Locale::Fr).unwrap().content;
        assert!(fr.contains("… et 490 autres."));
    }

    #[test]
    fn long_names_list_fewer_items_to_stay_under_the_cap() {
        let items: Vec<CatalogItem> = (0..30)
            .map(|i| score_item(&i.to_string(), &format!("{i}{}", "-".repeat(400))))
            .collect();
        let msg = catalog_batch(&items, Locale::En).unwrap().content;
        assert!(msg.chars().count() < 2000);
        assert!(msg.matches("• ").count() < MAX_LISTED);
    }

    #[test]
    fn nameless_items_are_neither_listed_nor_counted() {
        let untitled = CatalogItem::Score {
            id: "u".into(),
            card: ScoreCard {
                title: None,
                composer: None,
            },
        };
        assert!(!untitled.is_displayable());
        let mut blank = font();
        blank.label = " ".into();
        let blank = CatalogItem::SoundFont {
            id: "b".into(),
            card: blank,
        };
        assert!(!blank.is_displayable());
        assert_eq!(
            catalog_batch(&[untitled.clone(), blank.clone()], Locale::En),
            None
        );
        let msg = catalog_batch(
            &[
                untitled,
                blank,
                score_item("a", "Air"),
                score_item("b", "Aria"),
            ],
            Locale::En,
        )
        .unwrap()
        .content;
        assert!(msg.starts_with("🎼 **2 new scores in the catalog**"));
    }

    #[test]
    fn every_message_links_the_music_page_in_its_locale() {
        let en = [
            score_accepted(&score(), Locale::En),
            soundfont_accepted(&font(), Locale::En),
            season_record(&score(), RecordMode::Tempo, 90.0, Locale::En),
            catalog_batch(
                &[score_item("a", "Air"), score_item("b", "Aria")],
                Locale::En,
            ),
        ];
        for msg in en {
            let content = msg.unwrap().content;
            assert!(
                content.ends_with("[Cymbra Music](https://cymbra.app/en/music/)."),
                "{content}"
            );
        }
        let fr = [
            score_accepted(&score(), Locale::Fr),
            soundfont_accepted(&font(), Locale::Fr),
            season_record(&score(), RecordMode::Tempo, 90.0, Locale::Fr),
            catalog_batch(
                &[score_item("a", "Air"), score_item("b", "Aria")],
                Locale::Fr,
            ),
        ];
        for msg in fr {
            let content = msg.unwrap().content;
            assert!(
                content.ends_with("[Cymbra Music](https://cymbra.app/music/)."),
                "{content}"
            );
        }
    }

    #[test]
    fn an_item_shares_its_events_dedup_key() {
        assert_eq!(
            score_item("c1", "Air").dedup_key(),
            "discord:music.score_accepted:c1"
        );
        assert_eq!(
            font_item("f1").dedup_key(),
            "discord:music.soundfont_accepted:f1"
        );
    }

    fn font() -> SoundFontCard {
        SoundFontCard {
            label: "Upright Piano KW".into(),
            instrument: "keyboard".into(),
            license: "CC0 1.0".into(),
            attribution: None,
        }
    }

    fn score() -> ScoreCard {
        ScoreCard {
            title: Some("Gymnopédie No.1".into()),
            composer: Some("Erik Satie".into()),
        }
    }

    #[test]
    fn locale_parses_french_and_defaults_to_english() {
        assert_eq!(Locale::parse(" FR "), Locale::Fr);
        assert_eq!(Locale::parse("de"), Locale::En);
        assert_eq!(Locale::default(), Locale::En);
    }

    #[test]
    fn escape_neutralizes_markdown_links_and_mentions() {
        assert_eq!(
            escape("[free nitro](https://evil.example) @everyone **x**"),
            "\\[free nitro\\]\\(https\\://evil.example\\) @\u{200B}everyone \\*\\*x\\*\\*"
        );
        assert_eq!(escape("# Heading\n> quote"), "\\# Heading \\> quote");
        assert_eq!(escape("<@123>"), "\\<@\u{200B}123\\>");
        assert_eq!(escape("   "), "");
    }

    #[test]
    fn escape_bounds_the_length() {
        let long = "a".repeat(500);
        let out = escape(&long);
        assert_eq!(out.chars().count(), MAX_FIELD_CHARS + 1);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn soundfont_message_carries_the_public_fields_only() {
        let msg = soundfont_accepted(&font(), Locale::En).unwrap();
        assert_eq!(
            msg.content,
            "🎹 **New sound in the catalog: Upright Piano KW**\nKeyboard\nLicence: CC0 1.0\nAvailable now in [Cymbra Music](https://cymbra.app/en/music/)."
        );
        let mut drums = font();
        drums.instrument = "percussion".into();
        drums.attribution = Some("FreePats project".into());
        let fr = soundfont_accepted(&drums, Locale::Fr).unwrap().content;
        assert!(fr.contains("Batterie"));
        assert!(fr.contains("Crédit : FreePats project"));
        let en = soundfont_accepted(&drums, Locale::En).unwrap().content;
        assert!(en.contains("Drum kit") && en.contains("Credit: FreePats project"));
        assert!(!fr.contains("Clavier"));
    }

    #[test]
    fn a_nameless_soundfont_has_nothing_to_say() {
        let mut f = font();
        f.label = "  ".into();
        assert_eq!(soundfont_accepted(&f, Locale::En), None);
    }

    #[test]
    fn score_message_names_the_piece_and_composer() {
        let en = score_accepted(&score(), Locale::En).unwrap().content;
        assert_eq!(
            en,
            "🎼 **New score in the catalog:** **Gymnopédie No.1** — Erik Satie\nPlay it now in [Cymbra Music](https://cymbra.app/en/music/)."
        );
        let fr = score_accepted(&score(), Locale::Fr).unwrap().content;
        assert!(fr.starts_with("🎼 **Nouvelle partition au catalogue :**"));
        let anonymous = ScoreCard {
            title: Some("Air".into()),
            composer: None,
        };
        assert!(
            score_accepted(&anonymous, Locale::En)
                .unwrap()
                .content
                .contains("**Air**\n")
        );
        let untitled = ScoreCard {
            title: None,
            composer: Some("Bach".into()),
        };
        assert_eq!(score_accepted(&untitled, Locale::En), None);
    }

    #[test]
    fn a_record_message_is_anonymous_and_formats_the_figure() {
        let en = season_record(&score(), RecordMode::Tempo, 97.46, Locale::En)
            .unwrap()
            .content;
        assert!(en.contains("Free play: **97.5 %**"));
        let fr = season_record(&score(), RecordMode::Reaction, 88.0, Locale::Fr)
            .unwrap()
            .content;
        assert!(fr.contains("Mode Attente : **88,0 %**"));
        let wait = season_record(&score(), RecordMode::Reaction, 101.0, Locale::En)
            .unwrap()
            .content;
        assert!(wait.contains("Wait Mode: **100.0 %**"));
        let tempo_fr = season_record(&score(), RecordMode::Tempo, 50.0, Locale::Fr)
            .unwrap()
            .content;
        assert!(tempo_fr.contains("Jeu libre"));
        assert_eq!(
            season_record(
                &ScoreCard {
                    title: None,
                    composer: None
                },
                RecordMode::Tempo,
                1.0,
                Locale::En
            ),
            None
        );
    }

    #[test]
    fn messages_stay_under_the_discord_cap() {
        let huge = ScoreCard {
            title: Some("t".repeat(5000)),
            composer: Some("c".repeat(5000)),
        };
        for msg in [
            score_accepted(&huge, Locale::En),
            season_record(&huge, RecordMode::Tempo, 99.0, Locale::Fr),
        ] {
            assert!(msg.unwrap().content.chars().count() < 2000);
        }
    }
}
