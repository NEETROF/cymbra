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

//! The periodic reports (change: add-discord-notifications, D7 —
//! `scripts/discord/reports.md` is the field-by-field contract).
//!
//! Pure: which period is due, which figures survive the minimum of
//! contributors, and the rendered embed. Three rules hold everywhere:
//!
//! - **no count of people** — a report states what was done (sessions, words
//!   read, ratings…), never how many players or accounts did it; head counts
//!   are read only to apply the minimum and to order rankings;
//! - a figure whose contributing accounts are fewer than `k`
//!   (`discord.reports.min_contributors`, default **1**) is dropped;
//! - **nothing to say ⇒ nothing posted**: `None` rather than zeroes.

use chrono::{Datelike, Duration, NaiveDate, Weekday};

use crate::render::{Embed, Locale, Message, ScoreCard, escape};

/// How often a product reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cadence {
    Daily,
    Weekly,
}

impl Cadence {
    /// `daily` / `weekly` (case-insensitive); anything else is `default`.
    pub fn parse(raw: &str, default: Cadence) -> Cadence {
        match raw.trim().to_ascii_lowercase().as_str() {
            "daily" => Cadence::Daily,
            "weekly" => Cadence::Weekly,
            _ => default,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Cadence::Daily => "daily",
            Cadence::Weekly => "weekly",
        }
    }
}

/// A closed reporting period of UTC days, `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub cadence: Cadence,
}

impl Period {
    /// The last day inside the period.
    pub fn last_day(&self) -> NaiveDate {
        self.end - Duration::days(1)
    }
}

/// The period a run on `today` must report, if any. `lag_days` delays a
/// product whose figures arrive by device sync (Lingua: 1), so the days synced
/// meanwhile are counted. A daily product reports the day before
/// `today - lag`; a weekly one reports the ISO week (Monday–Sunday) that ended
/// just before `today - lag`, and only when that day is a Monday — so each
/// period is due on exactly one day.
pub fn due_period(cadence: Cadence, today: NaiveDate, lag_days: i64) -> Option<Period> {
    let anchor = today - Duration::days(lag_days);
    match cadence {
        Cadence::Daily => Some(Period {
            start: anchor - Duration::days(1),
            end: anchor,
            cadence,
        }),
        Cadence::Weekly => (anchor.weekday() == Weekday::Mon).then(|| Period {
            start: anchor - Duration::days(7),
            end: anchor,
            cadence,
        }),
    }
}

/// A summed figure and the number of distinct accounts behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Figure {
    pub value: i64,
    pub contributors: i64,
}

impl Figure {
    pub fn new(value: i64, contributors: i64) -> Self {
        Self {
            value,
            contributors,
        }
    }

    /// The value, when it is worth publishing: non-zero and backed by at least
    /// `k` accounts.
    pub fn shown(self, k: i64) -> Option<i64> {
        (self.value > 0 && self.contributors >= k.max(1)).then_some(self.value)
    }
}

/// A catalog piece with its plays over the period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PieceStat {
    pub card: ScoreCard,
    pub plays: i64,
    /// Distinct players — gates the line, never printed.
    pub players: i64,
}

/// The Cymbra Music figures of a period.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MusicFigures {
    /// Sessions played; contributors = distinct players.
    pub sessions: Figure,
    /// Average accuracy (0–100) over those sessions.
    pub accuracy_pct: Option<f64>,
    /// Ratings submitted or updated; contributors = distinct raters.
    pub ratings: Figure,
    /// Scores whose rating reached consensus.
    pub consensus: i64,
    /// Names of the items accepted into the catalog (already public).
    pub accepted: Vec<String>,
    /// Most played accepted catalog pieces, most played first.
    pub top: Vec<PieceStat>,
}

/// The Cymbra ID figures of a period, over the accounts created in it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IdFigures {
    /// Accounts created — gates the report, never printed.
    pub new_accounts: i64,
    /// First sign-in method of each new account: (`local`/`google`/`apple`, count).
    pub methods: Vec<(String, i64)>,
    /// Recorded locale of each new account: (locale, count).
    pub locales: Vec<(String, i64)>,
}

/// The Cymbra Lingua figures of a period.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LinguaFigures {
    pub words_read: Figure,
    pub words_learned: Figure,
    pub reviews: Figure,
    /// (language, active accounts) — orders the row, counts never printed.
    pub languages: Vec<(String, i64)>,
}

/// The current global-leaderboard season, for the weekly top pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeasonInfo {
    /// Last day of the season (inclusive).
    pub last_day: NaiveDate,
    pub days_left: i64,
}

/// How many pieces the daily report and the weekly post list.
pub const DAILY_TOP: usize = 10;
pub const WEEKLY_TOP: usize = 50;

/// Room kept for a list inside an embed description (Discord: 4096).
const DESCRIPTION_BUDGET: usize = 3800;

const MONTHS_EN: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const MONTHS_FR: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

fn date(d: NaiveDate, locale: Locale) -> String {
    let m = d.month0() as usize;
    match locale {
        Locale::En => format!("{} {} {}", d.day(), MONTHS_EN[m], d.year()),
        Locale::Fr => format!("{} {} {}", d.day(), MONTHS_FR[m], d.year()),
    }
}

/// "7 August 2026" for a day, "week of 3 August 2026" for a week.
fn period_label(p: &Period, locale: Locale) -> String {
    match (p.cadence, locale) {
        (Cadence::Daily, _) => date(p.start, locale),
        (Cadence::Weekly, Locale::En) => format!("week of {}", date(p.start, locale)),
        (Cadence::Weekly, Locale::Fr) => format!("semaine du {}", date(p.start, locale)),
    }
}

/// Thousands grouped: `318,540` (en) / `318 540` (fr, narrow no-break space).
pub fn number(n: i64, locale: Locale) -> String {
    let digits = n.unsigned_abs().to_string();
    let sep = match locale {
        Locale::En => ",",
        Locale::Fr => "\u{202F}",
    };
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push_str(sep);
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

fn percent(p: f64, locale: Locale) -> String {
    let p = p.clamp(0.0, 100.0).round() as i64;
    match locale {
        Locale::En => format!("{p}%"),
        Locale::Fr => format!("{p}\u{202F}%"),
    }
}

/// A language or locale code as a name: `en`, `en-GB` → English / anglais.
pub fn language_name(code: &str, locale: Locale) -> String {
    let primary = code
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let (en, fr) = match primary.as_str() {
        "en" => ("English", "anglais"),
        "fr" => ("French", "français"),
        "es" => ("Spanish", "espagnol"),
        "it" => ("Italian", "italien"),
        "de" => ("German", "allemand"),
        "pt" => ("Portuguese", "portugais"),
        "nl" => ("Dutch", "néerlandais"),
        _ => return escape(&code.to_ascii_uppercase()),
    };
    match locale {
        Locale::En => en.to_string(),
        Locale::Fr => fr.to_string(),
    }
}

fn method_name(provider: &str, locale: Locale) -> String {
    match (provider, locale) {
        ("local", Locale::En) => "Email".into(),
        ("local", Locale::Fr) => "E-mail".into(),
        ("google", _) => "Google".into(),
        ("apple", _) => "Apple".into(),
        (other, _) => escape(other),
    }
}

fn piece_line(card: &ScoreCard) -> Option<String> {
    let title = card
        .title
        .as_deref()
        .map(escape)
        .filter(|t| !t.is_empty())?;
    Some(
        match card
            .composer
            .as_deref()
            .map(escape)
            .filter(|c| !c.is_empty())
        {
            Some(c) => format!("**{title}** — {c}"),
            None => format!("**{title}**"),
        },
    )
}

/// A numbered list of the pieces backed by at least `k` players, within
/// `limit` lines and the description budget left over by `used`.
fn ranking(top: &[PieceStat], k: i64, limit: usize, used: usize, locale: Locale) -> Vec<String> {
    let mut lines = Vec::new();
    let mut size = used;
    for piece in top.iter().filter(|p| p.players >= k.max(1) && p.plays > 0) {
        if lines.len() == limit {
            break;
        }
        let Some(name) = piece_line(&piece.card) else {
            continue;
        };
        let plays = match (locale, piece.plays) {
            (Locale::En, 1) => "1 play".to_string(),
            (Locale::En, n) => format!("{} plays", number(n, locale)),
            (Locale::Fr, 1) => "1 partie".to_string(),
            (Locale::Fr, n) => format!("{} parties", number(n, locale)),
        };
        let line = format!("{}. {name} · {plays}", lines.len() + 1);
        if size + line.chars().count() + 1 > DESCRIPTION_BUDGET {
            break;
        }
        size += line.chars().count() + 1;
        lines.push(line);
    }
    lines
}

/// The Cymbra Music activity report. `None` when nothing survives.
pub fn music_report(f: &MusicFigures, period: &Period, k: i64, locale: Locale) -> Option<Message> {
    let mut blocks: Vec<String> = Vec::new();
    let mut stats: Vec<String> = Vec::new();
    if let Some(n) = f.sessions.shown(k) {
        stats.push(match locale {
            Locale::En => format!("**Sessions** {}", number(n, locale)),
            Locale::Fr => format!("**Parties** {}", number(n, locale)),
        });
        if let Some(acc) = f.accuracy_pct {
            stats.push(match locale {
                Locale::En => format!("**Average accuracy** {}", percent(acc, locale)),
                Locale::Fr => format!("**Précision moyenne** {}", percent(acc, locale)),
            });
        }
    }
    if let Some(n) = f.ratings.shown(k) {
        let consensus = (f.consensus > 0).then_some(f.consensus);
        stats.push(match (locale, consensus) {
            (Locale::En, Some(c)) => format!(
                "**Scores rated** {} ({} reached consensus)",
                number(n, locale),
                number(c, locale)
            ),
            (Locale::En, None) => format!("**Scores rated** {}", number(n, locale)),
            (Locale::Fr, Some(c)) => format!(
                "**Partitions notées** {} (dont {} au consensus)",
                number(n, locale),
                number(c, locale)
            ),
            (Locale::Fr, None) => format!("**Partitions notées** {}", number(n, locale)),
        });
    }
    if !stats.is_empty() {
        blocks.push(stats.join(" · "));
    }
    let accepted: Vec<String> = f
        .accepted
        .iter()
        .map(|n| escape(n))
        .filter(|n| !n.is_empty())
        .take(DAILY_TOP)
        .map(|n| format!("• {n}"))
        .collect();
    if !accepted.is_empty() {
        let heading = match locale {
            Locale::En => "**New in the catalog**",
            Locale::Fr => "**Nouveautés du catalogue**",
        };
        blocks.push(format!("{heading}\n{}", accepted.join("\n")));
    }
    let used: usize = blocks.iter().map(|b| b.chars().count() + 2).sum();
    let top = ranking(&f.top, k, DAILY_TOP, used + 40, locale);
    if !top.is_empty() {
        let heading = match locale {
            Locale::En => "**Most played**",
            Locale::Fr => "**Les plus jouées**",
        };
        blocks.push(format!("{heading}\n{}", top.join("\n")));
    }
    if blocks.is_empty() {
        return None;
    }
    Some(Message::embed(Embed {
        title: format!("Cymbra Music — {}", period_label(period, locale)),
        description: blocks.join("\n\n"),
        footer: None,
    }))
}

/// Shares of `rows`, most frequent first (ties by name), as `name 60%`.
fn shares(rows: &[(String, i64)], name: impl Fn(&str) -> String, locale: Locale) -> Vec<String> {
    let total: i64 = rows.iter().map(|(_, n)| *n).sum();
    if total <= 0 {
        return Vec::new();
    }
    let mut rows: Vec<(String, i64)> = rows.iter().filter(|(_, n)| *n > 0).cloned().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.iter()
        .map(|(k, n)| {
            format!(
                "{} {}",
                name(k),
                percent(*n as f64 * 100.0 / total as f64, locale)
            )
        })
        .collect()
}

/// Names ranked by their count, most frequent first, at most `limit`.
fn ranked_names(
    rows: &[(String, i64)],
    min: i64,
    limit: usize,
    name: impl Fn(&str) -> String,
) -> Vec<String> {
    let mut rows: Vec<&(String, i64)> = rows.iter().filter(|(_, n)| *n >= min.max(1)).collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut names: Vec<String> = Vec::new();
    for (code, _) in rows {
        let n = name(code);
        if !names.contains(&n) {
            names.push(n);
        }
        if names.len() == limit {
            break;
        }
    }
    names
}

/// The Cymbra ID report: how the period's new accounts signed up and their
/// languages — never how many there were. `None` below `k` new accounts.
pub fn id_report(f: &IdFigures, period: &Period, k: i64, locale: Locale) -> Option<Message> {
    if f.new_accounts <= 0 || f.new_accounts < k.max(1) {
        return None;
    }
    let mut blocks = Vec::new();
    let methods = shares(&f.methods, |p| method_name(p, locale), locale);
    if !methods.is_empty() {
        let label = match locale {
            Locale::En => "**Sign-in methods**",
            Locale::Fr => "**Méthodes de connexion**",
        };
        blocks.push(format!("{label} {}", methods.join(" · ")));
    }
    // An unrecorded locale is English (the apps' fallback).
    let locales: Vec<(String, i64)> = f
        .locales
        .iter()
        .map(|(l, n)| {
            (
                if l.trim().is_empty() {
                    "en".into()
                } else {
                    l.clone()
                },
                *n,
            )
        })
        .collect();
    let languages = ranked_names(&locales, 1, 3, |c| language_name(c, locale));
    if !languages.is_empty() {
        let label = match locale {
            Locale::En => "**Top languages**",
            Locale::Fr => "**Langues principales**",
        };
        blocks.push(format!("{label} {}", languages.join(" · ")));
    }
    if blocks.is_empty() {
        return None;
    }
    Some(Message::embed(Embed {
        title: format!("Cymbra ID — {}", period_label(period, locale)),
        description: blocks.join("\n"),
        footer: None,
    }))
}

/// The Cymbra Lingua report. Its figures are lower bounds: only signed-in
/// accounts whose activity has reached the server are counted — the footer
/// says so. `None` when nothing survives.
pub fn lingua_report(
    f: &LinguaFigures,
    period: &Period,
    k: i64,
    locale: Locale,
) -> Option<Message> {
    let rows: [(Figure, &str, &str); 3] = [
        (f.words_read, "Words read", "Mots lus"),
        (f.words_learned, "Words learned", "Mots appris"),
        (f.reviews, "Reviews done", "Révisions"),
    ];
    let mut lines: Vec<String> = rows
        .iter()
        .filter_map(|(fig, en, fr)| {
            let n = fig.shown(k)?;
            let label = match locale {
                Locale::En => en,
                Locale::Fr => fr,
            };
            Some(format!("**{label}** {}", number(n, locale)))
        })
        .collect();
    if lines.is_empty() {
        return None;
    }
    // Only when two or more languages each reach `k`: a lone language says
    // nothing (today every sync is `en`).
    let languages = ranked_names(&f.languages, k, 3, |c| language_name(c, locale));
    if languages.len() >= 2 {
        let label = match locale {
            Locale::En => "**Languages studied**",
            Locale::Fr => "**Langues étudiées**",
        };
        lines.push(format!("{label} {}", languages.join(" · ")));
    }
    let footer = match locale {
        Locale::En => "Counts signed-in accounts whose activity this period has reached the server",
        Locale::Fr => {
            "Compte les comptes connectés dont l'activité de la période est parvenue au serveur"
        }
    };
    Some(Message::embed(Embed {
        title: format!("Cymbra Lingua — {}", period_label(period, locale)),
        description: lines.join("\n"),
        footer: Some(footer.into()),
    }))
}

/// The weekly most played pieces, with the season's remaining days. `None`
/// when no piece survives (a season alone is not worth a post).
pub fn top_pieces_report(
    top: &[PieceStat],
    season: Option<SeasonInfo>,
    period: &Period,
    k: i64,
    locale: Locale,
) -> Option<Message> {
    let lines = ranking(top, k, WEEKLY_TOP, 200, locale);
    if lines.is_empty() {
        return None;
    }
    let mut description = lines.join("\n");
    if let Some(s) = season {
        let days = match (locale, s.days_left) {
            (Locale::En, 1) => "1 day left".to_string(),
            (Locale::En, n) => format!("{n} days left"),
            (Locale::Fr, 1) => "1 jour restant".to_string(),
            (Locale::Fr, n) => format!("{n} jours restants"),
        };
        let season_line = match locale {
            Locale::En => format!("**Season** ends {} · {days}", date(s.last_day, locale)),
            Locale::Fr => format!("**Saison** jusqu'au {} · {days}", date(s.last_day, locale)),
        };
        description = format!("{description}\n\n{season_line}");
    }
    let title = match locale {
        Locale::En => format!("Most played pieces — {}", period_label(period, locale)),
        Locale::Fr => format!("Pièces les plus jouées — {}", period_label(period, locale)),
    };
    Some(Message::embed(Embed {
        title,
        description,
        footer: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn day(start: NaiveDate) -> Period {
        Period {
            start,
            end: start + Duration::days(1),
            cadence: Cadence::Daily,
        }
    }

    fn week(start: NaiveDate) -> Period {
        Period {
            start,
            end: start + Duration::days(7),
            cadence: Cadence::Weekly,
        }
    }

    fn piece(title: &str, plays: i64, players: i64) -> PieceStat {
        PieceStat {
            card: ScoreCard {
                title: Some(title.into()),
                composer: Some("Bach".into()),
            },
            plays,
            players,
        }
    }

    fn content(m: Option<Message>) -> (String, String, Option<String>) {
        let e = m.expect("a message").embed.expect("an embed");
        (e.title, e.description, e.footer)
    }

    #[test]
    fn cadence_parses_and_falls_back() {
        assert_eq!(Cadence::parse(" Weekly ", Cadence::Daily), Cadence::Weekly);
        assert_eq!(Cadence::parse("daily", Cadence::Weekly), Cadence::Daily);
        assert_eq!(Cadence::parse("hourly", Cadence::Weekly), Cadence::Weekly);
        assert_eq!(Cadence::Daily.as_str(), "daily");
        assert_eq!(Cadence::Weekly.as_str(), "weekly");
    }

    #[test]
    fn a_daily_product_reports_yesterday() {
        let p = due_period(Cadence::Daily, d(2026, 9, 29), 0).unwrap();
        assert_eq!((p.start, p.end), (d(2026, 9, 28), d(2026, 9, 29)));
        assert_eq!(p.last_day(), d(2026, 9, 28));
        // With a one-day lag, the day before that.
        let p = due_period(Cadence::Daily, d(2026, 9, 29), 1).unwrap();
        assert_eq!(p.start, d(2026, 9, 27));
    }

    #[test]
    fn a_weekly_product_reports_once_on_monday() {
        // 2026-09-28 is a Monday: the week of 21–27 September is due.
        let p = due_period(Cadence::Weekly, d(2026, 9, 28), 0).unwrap();
        assert_eq!((p.start, p.end), (d(2026, 9, 21), d(2026, 9, 28)));
        assert_eq!(p.last_day(), d(2026, 9, 27));
        for other in 29..=30 {
            assert_eq!(due_period(Cadence::Weekly, d(2026, 9, other), 0), None);
        }
    }

    #[test]
    fn a_device_synced_week_is_due_on_tuesday() {
        assert_eq!(due_period(Cadence::Weekly, d(2026, 9, 28), 1), None);
        let p = due_period(Cadence::Weekly, d(2026, 9, 29), 1).unwrap();
        assert_eq!((p.start, p.end), (d(2026, 9, 21), d(2026, 9, 28)));
    }

    #[test]
    fn a_figure_shows_when_non_zero_and_backed_by_k() {
        assert_eq!(Figure::new(12, 1).shown(1), Some(12));
        assert_eq!(Figure::new(12, 1).shown(0), Some(12));
        assert_eq!(Figure::new(12, 2).shown(3), None);
        assert_eq!(Figure::new(0, 4).shown(1), None);
    }

    #[test]
    fn numbers_and_names_follow_the_locale() {
        assert_eq!(number(318_540, Locale::En), "318,540");
        assert_eq!(number(318_540, Locale::Fr), "318\u{202F}540");
        assert_eq!(number(999, Locale::En), "999");
        assert_eq!(number(-1_000, Locale::En), "-1,000");
        assert_eq!(language_name("en-GB", Locale::En), "English");
        assert_eq!(language_name("fr_FR", Locale::Fr), "français");
        for (code, en) in [
            ("es", "Spanish"),
            ("it", "Italian"),
            ("de", "German"),
            ("pt", "Portuguese"),
            ("nl", "Dutch"),
        ] {
            assert_eq!(language_name(code, Locale::En), en);
        }
        assert_eq!(language_name("ja", Locale::En), "JA");
        assert_eq!(method_name("local", Locale::Fr), "E-mail");
        assert_eq!(method_name("local", Locale::En), "Email");
        assert_eq!(method_name("apple", Locale::En), "Apple");
        assert_eq!(method_name("github", Locale::En), "github");
    }

    #[test]
    fn the_music_report_states_activity_never_people() {
        let f = MusicFigures {
            sessions: Figure::new(47, 12),
            accuracy_pct: Some(78.4),
            ratings: Figure::new(9, 4),
            consensus: 3,
            accepted: vec!["Gymnopédie No. 1 — Satie".into()],
            top: vec![piece("Air", 11, 6), piece("Aria", 1, 1)],
        };
        let (title, body, footer) = content(music_report(&f, &day(d(2026, 8, 7)), 1, Locale::En));
        assert_eq!(title, "Cymbra Music — 7 August 2026");
        assert!(body.contains(
            "**Sessions** 47 · **Average accuracy** 78% · **Scores rated** 9 (3 reached consensus)"
        ));
        assert!(body.contains("**New in the catalog**\n• Gymnop"));
        assert!(body.contains(
            "**Most played**\n1. **Air** — Bach · 11 plays\n2. **Aria** — Bach · 1 play"
        ));
        assert_eq!(footer, None);
        // The 12 players and 4 raters appear nowhere: every line is accounted
        // for above.
        assert_eq!(body.lines().count(), 8, "{body}");
        assert!(!body.to_lowercase().contains("player"));

        let (_, fr, _) = content(music_report(&f, &week(d(2026, 9, 21)), 1, Locale::Fr));
        assert!(fr.contains("**Parties** 47 · **Précision moyenne** 78\u{202F}% · **Partitions notées** 9 (dont 3 au consensus)"));
        assert!(fr.contains("**Nouveautés du catalogue**") && fr.contains("**Les plus jouées**"));
        assert!(fr.contains("1 partie") && fr.contains("11 parties"));
    }

    #[test]
    fn a_raised_minimum_drops_small_figures_and_pieces() {
        let f = MusicFigures {
            sessions: Figure::new(3, 1),
            accuracy_pct: Some(90.0),
            ratings: Figure::new(2, 1),
            consensus: 0,
            accepted: vec![],
            top: vec![piece("Air", 3, 1)],
        };
        assert!(music_report(&f, &day(d(2026, 8, 7)), 5, Locale::En).is_none());
        // At the default of 1 the same figures are published.
        let (_, body, _) = content(music_report(&f, &day(d(2026, 8, 7)), 1, Locale::En));
        assert!(body.contains("**Sessions** 3") && body.contains("**Scores rated** 2\n"));
        let fr = content(music_report(&f, &day(d(2026, 8, 7)), 1, Locale::Fr)).1;
        assert!(fr.contains("**Partitions notées** 2"));
    }

    #[test]
    fn an_accepted_item_alone_is_enough_and_nothing_is_nothing() {
        let only = MusicFigures {
            accepted: vec!["Salamander Grand".into(), "  ".into()],
            ..Default::default()
        };
        let (_, body, _) = content(music_report(&only, &day(d(2026, 8, 7)), 5, Locale::En));
        assert_eq!(body, "**New in the catalog**\n• Salamander Grand");
        assert!(
            music_report(&MusicFigures::default(), &day(d(2026, 8, 7)), 1, Locale::En).is_none()
        );
    }

    #[test]
    fn untitled_and_empty_pieces_are_skipped_and_the_list_is_bounded() {
        let mut top = vec![
            PieceStat {
                card: ScoreCard {
                    title: None,
                    composer: None,
                },
                plays: 9,
                players: 3,
            },
            PieceStat {
                card: ScoreCard {
                    title: Some("Solo".into()),
                    composer: None,
                },
                plays: 8,
                players: 3,
            },
            piece("Zero", 0, 0),
        ];
        top.extend((0..60).map(|i| piece(&format!("P{i}"), 5, 2)));
        let lines = ranking(&top, 1, WEEKLY_TOP, 0, Locale::En);
        assert_eq!(lines.len(), WEEKLY_TOP);
        assert_eq!(lines[0], "1. **Solo** · 8 plays");
        // Very long names stop at the description budget instead.
        let long: Vec<PieceStat> = (0..50)
            .map(|i| piece(&format!("{i}{}", "-".repeat(400)), 5, 2))
            .collect();
        let lines = ranking(&long, 1, WEEKLY_TOP, 0, Locale::En);
        assert!(lines.len() < WEEKLY_TOP);
        assert!(lines.iter().map(|l| l.chars().count() + 1).sum::<usize>() <= DESCRIPTION_BUDGET);
    }

    #[test]
    fn the_id_report_gives_shares_and_languages_not_a_count() {
        let f = IdFigures {
            new_accounts: 7,
            methods: vec![
                ("google".into(), 4),
                ("local".into(), 2),
                ("apple".into(), 1),
            ],
            locales: vec![("fr".into(), 4), ("".into(), 2), ("fr-FR".into(), 1)],
        };
        let (title, body, _) = content(id_report(&f, &week(d(2026, 9, 21)), 1, Locale::En));
        assert_eq!(title, "Cymbra ID — week of 21 September 2026");
        assert_eq!(
            body,
            "**Sign-in methods** Google 57% · Email 29% · Apple 14%\n**Top languages** French · English"
        );
        // The 7 new accounts are not stated: the body is exactly the two rows.
        let (fr_title, fr, _) = content(id_report(&f, &week(d(2026, 9, 21)), 1, Locale::Fr));
        assert_eq!(fr_title, "Cymbra ID — semaine du 21 septembre 2026");
        assert!(fr.starts_with("**Méthodes de connexion** Google 57\u{202F}%"));
        assert!(fr.contains("**Langues principales** français · anglais"));
    }

    #[test]
    fn the_id_report_is_silent_without_enough_new_accounts() {
        let none = IdFigures::default();
        assert!(id_report(&none, &week(d(2026, 9, 21)), 1, Locale::En).is_none());
        let two = IdFigures {
            new_accounts: 2,
            methods: vec![("google".into(), 2)],
            locales: vec![],
        };
        assert!(id_report(&two, &week(d(2026, 9, 21)), 5, Locale::En).is_none());
        let (_, body, _) = content(id_report(&two, &week(d(2026, 9, 21)), 1, Locale::En));
        assert_eq!(body, "**Sign-in methods** Google 100%");
        let bare = IdFigures {
            new_accounts: 1,
            methods: vec![],
            locales: vec![],
        };
        assert!(id_report(&bare, &week(d(2026, 9, 21)), 1, Locale::En).is_none());
    }

    #[test]
    fn the_lingua_report_publishes_activity_with_its_caveat() {
        let f = LinguaFigures {
            words_read: Figure::new(318_540, 1),
            words_learned: Figure::new(1_207, 1),
            reviews: Figure::new(0, 0),
            languages: vec![("en".into(), 1)],
        };
        let (title, body, footer) =
            content(lingua_report(&f, &week(d(2026, 9, 21)), 1, Locale::En));
        assert_eq!(title, "Cymbra Lingua — week of 21 September 2026");
        assert_eq!(body, "**Words read** 318,540\n**Words learned** 1,207");
        assert!(footer.unwrap().starts_with("Counts signed-in accounts"));
        let (_, fr, fr_footer) = content(lingua_report(&f, &week(d(2026, 9, 21)), 1, Locale::Fr));
        assert!(fr.starts_with("**Mots lus** 318\u{202F}540\n**Mots appris**"));
        assert!(fr_footer.unwrap().starts_with("Compte les comptes"));
    }

    #[test]
    fn the_languages_row_needs_two_languages_reaching_k() {
        let mut f = LinguaFigures {
            reviews: Figure::new(30, 3),
            languages: vec![("es".into(), 2), ("en".into(), 5), ("it".into(), 1)],
            ..Default::default()
        };
        let body = content(lingua_report(&f, &week(d(2026, 9, 21)), 1, Locale::En)).1;
        assert!(body.ends_with("**Languages studied** English · Spanish · Italian"));
        let fr = content(lingua_report(&f, &week(d(2026, 9, 21)), 1, Locale::Fr)).1;
        assert!(fr.contains("**Révisions** 30") && fr.contains("**Langues étudiées** anglais"));
        // With k = 2 only English and Spanish reach it — still two.
        let body = content(lingua_report(&f, &week(d(2026, 9, 21)), 2, Locale::En)).1;
        assert!(body.ends_with("English · Spanish"));
        // With k = 3 only English: the row is omitted.
        let body = content(lingua_report(&f, &week(d(2026, 9, 21)), 3, Locale::En)).1;
        assert!(!body.contains("Languages"));
        // Nothing to say at all.
        f.reviews = Figure::default();
        assert!(lingua_report(&f, &week(d(2026, 9, 21)), 1, Locale::En).is_none());
    }

    #[test]
    fn the_weekly_top_lists_pieces_and_the_season() {
        let top = vec![piece("Air", 11, 6), piece("Aria", 2, 1)];
        let season = SeasonInfo {
            last_day: d(2026, 10, 27),
            days_left: 29,
        };
        let (title, body, _) = content(top_pieces_report(
            &top,
            Some(season),
            &week(d(2026, 9, 21)),
            1,
            Locale::En,
        ));
        assert_eq!(title, "Most played pieces — week of 21 September 2026");
        assert!(body.starts_with("1. **Air** — Bach · 11 plays\n2. **Aria**"));
        assert!(body.ends_with("**Season** ends 27 October 2026 · 29 days left"));
        let one_day = SeasonInfo {
            last_day: d(2026, 9, 29),
            days_left: 1,
        };
        let (fr_title, fr, _) = content(top_pieces_report(
            &top,
            Some(one_day),
            &week(d(2026, 9, 21)),
            1,
            Locale::Fr,
        ));
        assert!(fr_title.starts_with("Pièces les plus jouées — semaine du"));
        assert!(fr.ends_with("**Saison** jusqu'au 29 septembre 2026 · 1 jour restant"));
        let en_one = content(top_pieces_report(
            &top,
            Some(one_day),
            &week(d(2026, 9, 21)),
            1,
            Locale::En,
        ))
        .1;
        assert!(en_one.ends_with("1 day left"));
        let fr_many = content(top_pieces_report(
            &top,
            Some(season),
            &week(d(2026, 9, 21)),
            1,
            Locale::Fr,
        ))
        .1;
        assert!(fr_many.ends_with("29 jours restants"));
        // No season known: the list alone.
        let bare = content(top_pieces_report(
            &top,
            None,
            &week(d(2026, 9, 21)),
            1,
            Locale::En,
        ))
        .1;
        assert!(!bare.contains("Season"));
        // A raised minimum can empty the list: nothing posted.
        assert!(
            top_pieces_report(&top, Some(season), &week(d(2026, 9, 21)), 7, Locale::En).is_none()
        );
    }
}
