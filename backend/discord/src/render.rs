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
//! sender also disables mention parsing and link embeds on top of it.

use crate::event::RecordMode;

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

/// A rendered Discord message — plain `content`, under Discord's 2000-char cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub content: String,
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
            lines.push("Available now in Cymbra Music.".into());
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
            lines.push("Disponible dès maintenant dans Cymbra Music.".into());
        }
    }
    Some(Message {
        content: lines.join("\n"),
    })
}

/// A score accepted into the catalog. `None` when it has no title to show.
pub fn score_accepted(card: &ScoreCard, locale: Locale) -> Option<Message> {
    let piece = piece(card)?;
    let content = match locale {
        Locale::En => {
            format!("🎼 **New score in the catalog:** {piece}\nPlay it now in Cymbra Music.")
        }
        Locale::Fr => format!(
            "🎼 **Nouvelle partition au catalogue :** {piece}\nÀ jouer dès maintenant dans Cymbra Music."
        ),
    };
    Some(Message { content })
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
                "🏆 **New season record** on {piece}\n{mode}: **{figure:.1} %**\nThink you can beat it? Play it in Cymbra Music."
            )
        }
        Locale::Fr => {
            let mode = match mode {
                RecordMode::Tempo => "Jeu libre",
                RecordMode::Reaction => "Mode Attente",
            };
            let figure = format!("{figure:.1}").replace('.', ",");
            format!(
                "🏆 **Nouveau record de la saison** sur {piece}\n{mode} : **{figure} %**\nTu peux faire mieux ? Joue-la dans Cymbra Music."
            )
        }
    };
    Some(Message { content })
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "🎹 **New sound in the catalog: Upright Piano KW**\nKeyboard\nLicence: CC0 1.0\nAvailable now in Cymbra Music."
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
            "🎼 **New score in the catalog:** **Gymnopédie No.1** — Erik Satie\nPlay it now in Cymbra Music."
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
