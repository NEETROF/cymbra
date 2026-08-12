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

//! The cross-domain badge registry + its evaluation (change: add-achievement-badges).
//!
//! Pure, host-testable, no I/O. [`REGISTRY`] is the ONE place a badge is defined
//! (design D7): every badge declares a stable `key`, a [`BadgeFamily`], the
//! [`BadgeMetric`] it is measured against, the `threshold` that earns it, and —
//! for a graduated series — a `track` + `tier`. Nothing else enumerates badges;
//! the awarding logic, the wire projection and the app grid all derive from here,
//! so adding a badge needs no app release (design D6: label + description ship as
//! inline-localized maps).
//!
//! Two rules shape [`evaluate`]:
//!
//! * **The union** (design D3) — a badge is earned when it has EVER been granted
//!   **or** when its counter currently clears the threshold. The grant row records
//!   *when*, not *whether*: that makes a badge survive its underlying activity
//!   being purged (a leaderboard best cascade-deleted with its piece, a proposal
//!   returning to review, play sessions ageing out of retention) AND makes a badge
//!   defined today retroactive with no backfill.
//! * **Clamping** — a badge shown as earned reports its threshold as the current
//!   value, so an earned badge never renders as `3/20`.

use std::collections::HashMap;

use chrono::NaiveDate;

/// The activity area a badge belongs to. Families group the profile grid; a family
/// that declares no badge simply does not render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BadgeFamily {
    Play,
    Consistency,
    Ranking,
    Contribution,
    Curation,
    Learning,
}

impl BadgeFamily {
    /// The wire string (also the client's grouping key).
    pub fn as_str(self) -> &'static str {
        match self {
            BadgeFamily::Play => "play",
            BadgeFamily::Consistency => "consistency",
            BadgeFamily::Ranking => "ranking",
            BadgeFamily::Contribution => "contribution",
            BadgeFamily::Curation => "curation",
            BadgeFamily::Learning => "learning",
        }
    }
}

/// Which counter a badge is earned against. Every metric is derived from a record
/// the app already keeps — no new per-user tracking table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeMetric {
    // --- curation (pre-existing, from `score_ratings` + the rewards ledger) ---
    /// Total ratings the user has recorded.
    RatingCount,
    /// Ratings that settled ALIGNED with the ground truth (the honesty proxy).
    AlignedCount,
    /// Ratings where the user was the FIRST rater of the score.
    FirstRaterCount,
    // --- play (`play_sessions`) ---
    /// Play sessions recorded.
    SessionCount,
    /// Distinct pieces played.
    DistinctPieces,
    /// Sessions finished above [`HIGH_ACCURACY_PCT`].
    HighAccuracySessions,
    // --- consistency (`play_sessions`, bucketed by the player's LOCAL day) ---
    /// Distinct local days with at least one session.
    DaysPlayed,
    /// Longest run of consecutive local days played.
    LongestStreak,
    // --- ranking (`leaderboard_bests` + `global_season_snapshots`) ---
    /// Per-piece boards the user holds a personal best on.
    RankedBoards,
    /// Boards where that best currently sits in the top [`TOP_PLACEMENT`].
    TopThreeFinishes,
    /// Closed seasons the user finished in the global top [`TOP_PLACEMENT`].
    SeasonPodiums,
    // --- contribution (`catalog_scores` + `soundfonts`) ---
    /// Catalog score proposals of the user's that were accepted.
    AcceptedProposals,
    /// SoundFont contributions of the user's that were accepted.
    AcceptedSoundFonts,
    // --- learning (`course_progress`) ---
    /// Courses the user has completed.
    CoursesCompleted,
}

impl BadgeMetric {
    /// The wire string. The three curation values are UNCHANGED from
    /// `curation_rewards_core::BadgeMetric`, so an app build that predates this
    /// change keeps reading its grid.
    pub fn as_str(self) -> &'static str {
        match self {
            BadgeMetric::RatingCount => "rating_count",
            BadgeMetric::AlignedCount => "aligned_count",
            BadgeMetric::FirstRaterCount => "first_rater_count",
            BadgeMetric::SessionCount => "session_count",
            BadgeMetric::DistinctPieces => "distinct_pieces",
            BadgeMetric::HighAccuracySessions => "high_accuracy_sessions",
            BadgeMetric::DaysPlayed => "days_played",
            BadgeMetric::LongestStreak => "longest_streak",
            BadgeMetric::RankedBoards => "ranked_boards",
            BadgeMetric::TopThreeFinishes => "top_three_finishes",
            BadgeMetric::SeasonPodiums => "season_podiums",
            BadgeMetric::AcceptedProposals => "accepted_proposals",
            BadgeMetric::AcceptedSoundFonts => "accepted_soundfonts",
            BadgeMetric::CoursesCompleted => "courses_completed",
        }
    }
}

/// The overall synchronization % a session must clear to count as "high accuracy"
/// (the `virtuoso` track). Read by the repo's counter query.
pub const HIGH_ACCURACY_PCT: f64 = 90.0;

/// The placement that counts as a podium, on a per-piece board and in a closed
/// season alike. Read by the repo's ranking queries.
pub const TOP_PLACEMENT: i64 = 3;

/// Inline-localized text, the pattern `music.courses.title_json` already
/// established on this service (design D6): the wire carries every language and
/// the client picks its ACTIVE DISPLAY language — which is not necessarily the
/// account language — falling back to `en`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalizedText {
    pub en: &'static str,
    pub fr: &'static str,
    pub es: &'static str,
    pub it: &'static str,
}

impl LocalizedText {
    /// The `{"en":…,"fr":…,"es":…,"it":…}` JSON object the wire carries. Hand-built
    /// rather than serde-derived because the inputs are `const` string literals
    /// authored in review — a quote in one would be visible there, and escaping is
    /// applied anyway so a future entry cannot break the payload.
    pub fn to_json(self) -> String {
        format!(
            "{{\"en\":\"{}\",\"fr\":\"{}\",\"es\":\"{}\",\"it\":\"{}\"}}",
            escape(self.en),
            escape(self.fr),
            escape(self.es),
            escape(self.it),
        )
    }
}

/// Minimal JSON string escaping for the registry's literals.
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// One badge definition. `track` groups a graduated series (`curator_1/2/3` is one
/// `curator` track at tiers 1/2/3) so the grid can collapse it to a single tile
/// (design D5); a standalone badge has `track: None` and `tier: 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BadgeDef {
    pub key: &'static str,
    pub family: BadgeFamily,
    pub metric: BadgeMetric,
    pub threshold: i64,
    pub track: Option<&'static str>,
    pub tier: i32,
    pub label: LocalizedText,
    pub description: LocalizedText,
}

/// Shorthand for a standalone (untracked) badge.
const fn badge(
    key: &'static str,
    family: BadgeFamily,
    metric: BadgeMetric,
    threshold: i64,
    label: LocalizedText,
    description: LocalizedText,
) -> BadgeDef {
    BadgeDef {
        key,
        family,
        metric,
        threshold,
        track: None,
        tier: 0,
        label,
        description,
    }
}

/// Shorthand for one tier of a graduated track.
#[allow(clippy::too_many_arguments)]
const fn tiered(
    key: &'static str,
    family: BadgeFamily,
    metric: BadgeMetric,
    threshold: i64,
    track: &'static str,
    tier: i32,
    label: LocalizedText,
    description: LocalizedText,
) -> BadgeDef {
    BadgeDef {
        key,
        family,
        metric,
        threshold,
        track: Some(track),
        tier,
        label,
        description,
    }
}

/// Shorthand for a [`LocalizedText`] literal.
const fn t(
    en: &'static str,
    fr: &'static str,
    es: &'static str,
    it: &'static str,
) -> LocalizedText {
    LocalizedText { en, fr, es, it }
}

/// The badge catalogue — the single source of truth (design D7: static Rust, so a
/// new badge is a backend release but never an app one).
///
/// The seven CURATION entries keep the keys and thresholds
/// `curation_rewards_core::BADGES` shipped with, so every `curation_grants` row
/// already written stays valid and no badge already earned is lost (no data
/// migration). Their French labels are the ones the app used to hard-code.
///
/// The other families' thresholds are a deliberate FIRST PASS (design Open
/// Questions): low enough that an active player sees early wins instead of a wall
/// of padlocks, and safe to raise later — a raised threshold can never un-earn a
/// badge, because a granted badge stays earned (design D3).
pub const REGISTRY: &[BadgeDef] = &[
    // --- play ------------------------------------------------------------
    badge(
        "first_performance",
        BadgeFamily::Play,
        BadgeMetric::SessionCount,
        1,
        t(
            "First Performance",
            "Première interprétation",
            "Primera interpretación",
            "Prima esecuzione",
        ),
        t(
            "Record your first play session.",
            "Enregistrez votre première session de jeu.",
            "Registra tu primera sesión de interpretación.",
            "Registra la tua prima sessione di esecuzione.",
        ),
    ),
    tiered(
        "performer_1",
        BadgeFamily::Play,
        BadgeMetric::SessionCount,
        25,
        "performer",
        1,
        t(
            "Performer I",
            "Interprète I",
            "Intérprete I",
            "Interprete I",
        ),
        t(
            "Play 25 sessions.",
            "Jouez 25 sessions.",
            "Toca 25 sesiones.",
            "Suona 25 sessioni.",
        ),
    ),
    tiered(
        "performer_2",
        BadgeFamily::Play,
        BadgeMetric::SessionCount,
        150,
        "performer",
        2,
        t(
            "Performer II",
            "Interprète II",
            "Intérprete II",
            "Interprete II",
        ),
        t(
            "Play 150 sessions.",
            "Jouez 150 sessions.",
            "Toca 150 sesiones.",
            "Suona 150 sessioni.",
        ),
    ),
    tiered(
        "performer_3",
        BadgeFamily::Play,
        BadgeMetric::SessionCount,
        750,
        "performer",
        3,
        t(
            "Performer III",
            "Interprète III",
            "Intérprete III",
            "Interprete III",
        ),
        t(
            "Play 750 sessions.",
            "Jouez 750 sessions.",
            "Toca 750 sesiones.",
            "Suona 750 sessioni.",
        ),
    ),
    tiered(
        "repertoire_1",
        BadgeFamily::Play,
        BadgeMetric::DistinctPieces,
        10,
        "repertoire",
        1,
        t(
            "Repertoire I",
            "Répertoire I",
            "Repertorio I",
            "Repertorio I",
        ),
        t(
            "Play 10 different pieces.",
            "Jouez 10 morceaux différents.",
            "Toca 10 piezas diferentes.",
            "Suona 10 brani diversi.",
        ),
    ),
    tiered(
        "repertoire_2",
        BadgeFamily::Play,
        BadgeMetric::DistinctPieces,
        50,
        "repertoire",
        2,
        t(
            "Repertoire II",
            "Répertoire II",
            "Repertorio II",
            "Repertorio II",
        ),
        t(
            "Play 50 different pieces.",
            "Jouez 50 morceaux différents.",
            "Toca 50 piezas diferentes.",
            "Suona 50 brani diversi.",
        ),
    ),
    tiered(
        "virtuoso_1",
        BadgeFamily::Play,
        BadgeMetric::HighAccuracySessions,
        10,
        "virtuoso",
        1,
        t("Virtuoso I", "Virtuose I", "Virtuoso I", "Virtuoso I"),
        t(
            "Finish 10 sessions above 90% accuracy.",
            "Terminez 10 sessions au-dessus de 90 % de précision.",
            "Termina 10 sesiones por encima del 90 % de precisión.",
            "Completa 10 sessioni sopra il 90 % di precisione.",
        ),
    ),
    tiered(
        "virtuoso_2",
        BadgeFamily::Play,
        BadgeMetric::HighAccuracySessions,
        100,
        "virtuoso",
        2,
        t("Virtuoso II", "Virtuose II", "Virtuoso II", "Virtuoso II"),
        t(
            "Finish 100 sessions above 90% accuracy.",
            "Terminez 100 sessions au-dessus de 90 % de précision.",
            "Termina 100 sesiones por encima del 90 % de precisión.",
            "Completa 100 sessioni sopra il 90 % di precisione.",
        ),
    ),
    // --- consistency -----------------------------------------------------
    tiered(
        "regular_1",
        BadgeFamily::Consistency,
        BadgeMetric::DaysPlayed,
        10,
        "regular",
        1,
        t("Regular I", "Régulier I", "Constante I", "Costante I"),
        t(
            "Play on 10 different days.",
            "Jouez 10 jours différents.",
            "Toca en 10 días diferentes.",
            "Suona in 10 giorni diversi.",
        ),
    ),
    tiered(
        "regular_2",
        BadgeFamily::Consistency,
        BadgeMetric::DaysPlayed,
        50,
        "regular",
        2,
        t("Regular II", "Régulier II", "Constante II", "Costante II"),
        t(
            "Play on 50 different days.",
            "Jouez 50 jours différents.",
            "Toca en 50 días diferentes.",
            "Suona in 50 giorni diversi.",
        ),
    ),
    tiered(
        "regular_3",
        BadgeFamily::Consistency,
        BadgeMetric::DaysPlayed,
        200,
        "regular",
        3,
        t(
            "Regular III",
            "Régulier III",
            "Constante III",
            "Costante III",
        ),
        t(
            "Play on 200 different days.",
            "Jouez 200 jours différents.",
            "Toca en 200 días diferentes.",
            "Suona in 200 giorni diversi.",
        ),
    ),
    tiered(
        "streak_1",
        BadgeFamily::Consistency,
        BadgeMetric::LongestStreak,
        3,
        "streak",
        1,
        t("Streak I", "Série I", "Racha I", "Serie I"),
        t(
            "Play 3 days in a row.",
            "Jouez 3 jours d'affilée.",
            "Toca 3 días seguidos.",
            "Suona 3 giorni di fila.",
        ),
    ),
    tiered(
        "streak_2",
        BadgeFamily::Consistency,
        BadgeMetric::LongestStreak,
        7,
        "streak",
        2,
        t("Streak II", "Série II", "Racha II", "Serie II"),
        t(
            "Play 7 days in a row.",
            "Jouez 7 jours d'affilée.",
            "Toca 7 días seguidos.",
            "Suona 7 giorni di fila.",
        ),
    ),
    tiered(
        "streak_3",
        BadgeFamily::Consistency,
        BadgeMetric::LongestStreak,
        30,
        "streak",
        3,
        t("Streak III", "Série III", "Racha III", "Serie III"),
        t(
            "Play 30 days in a row.",
            "Jouez 30 jours d'affilée.",
            "Toca 30 días seguidos.",
            "Suona 30 giorni di fila.",
        ),
    ),
    // --- ranking ---------------------------------------------------------
    badge(
        "contender",
        BadgeFamily::Ranking,
        BadgeMetric::RankedBoards,
        5,
        t("Contender", "Prétendant", "Aspirante", "Sfidante"),
        t(
            "Set a personal best on 5 leaderboards.",
            "Inscrivez un record personnel sur 5 classements.",
            "Registra un récord personal en 5 clasificaciones.",
            "Registra un primato personale in 5 classifiche.",
        ),
    ),
    tiered(
        "podium_1",
        BadgeFamily::Ranking,
        BadgeMetric::TopThreeFinishes,
        1,
        "podium",
        1,
        t("Podium I", "Podium I", "Podio I", "Podio I"),
        t(
            "Reach the top 3 of a leaderboard.",
            "Atteignez le top 3 d'un classement.",
            "Alcanza el top 3 de una clasificación.",
            "Raggiungi la top 3 di una classifica.",
        ),
    ),
    tiered(
        "podium_2",
        BadgeFamily::Ranking,
        BadgeMetric::TopThreeFinishes,
        10,
        "podium",
        2,
        t("Podium II", "Podium II", "Podio II", "Podio II"),
        t(
            "Reach the top 3 of 10 leaderboards.",
            "Atteignez le top 3 de 10 classements.",
            "Alcanza el top 3 de 10 clasificaciones.",
            "Raggiungi la top 3 di 10 classifiche.",
        ),
    ),
    badge(
        "season_laureate",
        BadgeFamily::Ranking,
        BadgeMetric::SeasonPodiums,
        1,
        t(
            "Season Laureate",
            "Lauréat de saison",
            "Laureado de temporada",
            "Laureato di stagione",
        ),
        t(
            "Finish a closed season in the global top 3.",
            "Terminez une saison close dans le top 3 mondial.",
            "Termina una temporada cerrada en el top 3 mundial.",
            "Chiudi una stagione nella top 3 mondiale.",
        ),
    ),
    // --- contribution ----------------------------------------------------
    tiered(
        "publisher_1",
        BadgeFamily::Contribution,
        BadgeMetric::AcceptedProposals,
        1,
        "publisher",
        1,
        t("Publisher I", "Éditeur I", "Editor I", "Editore I"),
        t(
            "Get a score proposal accepted into the catalog.",
            "Faites accepter une proposition de partition au catalogue.",
            "Consigue que una propuesta de partitura entre en el catálogo.",
            "Fai accettare una proposta di spartito nel catalogo.",
        ),
    ),
    tiered(
        "publisher_2",
        BadgeFamily::Contribution,
        BadgeMetric::AcceptedProposals,
        10,
        "publisher",
        2,
        t("Publisher II", "Éditeur II", "Editor II", "Editore II"),
        t(
            "Get 10 score proposals accepted into the catalog.",
            "Faites accepter 10 propositions de partitions au catalogue.",
            "Consigue que 10 propuestas de partituras entren en el catálogo.",
            "Fai accettare 10 proposte di spartiti nel catalogo.",
        ),
    ),
    badge(
        "sound_donor",
        BadgeFamily::Contribution,
        BadgeMetric::AcceptedSoundFonts,
        1,
        t(
            "Sound Donor",
            "Donateur de son",
            "Donante de sonido",
            "Donatore di suono",
        ),
        t(
            "Get a SoundFont contribution accepted.",
            "Faites accepter une contribution de SoundFont.",
            "Consigue que se acepte una contribución de SoundFont.",
            "Fai accettare un contributo SoundFont.",
        ),
    ),
    // --- curation (keys + thresholds UNCHANGED — already-granted rows) -----
    badge(
        "first_note",
        BadgeFamily::Curation,
        BadgeMetric::RatingCount,
        1,
        t(
            "First Note",
            "Première note",
            "Primera nota",
            "Prima valutazione",
        ),
        t(
            "Rate your first score.",
            "Notez votre première partition.",
            "Valora tu primera partitura.",
            "Valuta il tuo primo spartito.",
        ),
    ),
    tiered(
        "curator_1",
        BadgeFamily::Curation,
        BadgeMetric::RatingCount,
        10,
        "curator",
        1,
        t("Curator I", "Curateur I", "Curador I", "Curatore I"),
        t(
            "Rate 10 scores.",
            "Notez 10 partitions.",
            "Valora 10 partituras.",
            "Valuta 10 spartiti.",
        ),
    ),
    tiered(
        "curator_2",
        BadgeFamily::Curation,
        BadgeMetric::RatingCount,
        100,
        "curator",
        2,
        t("Curator II", "Curateur II", "Curador II", "Curatore II"),
        t(
            "Rate 100 scores.",
            "Notez 100 partitions.",
            "Valora 100 partituras.",
            "Valuta 100 spartiti.",
        ),
    ),
    tiered(
        "curator_3",
        BadgeFamily::Curation,
        BadgeMetric::RatingCount,
        500,
        "curator",
        3,
        t("Curator III", "Curateur III", "Curador III", "Curatore III"),
        t(
            "Rate 500 scores.",
            "Notez 500 partitions.",
            "Valora 500 partituras.",
            "Valuta 500 spartiti.",
        ),
    ),
    tiered(
        "sharp_ear_1",
        BadgeFamily::Curation,
        BadgeMetric::AlignedCount,
        25,
        "sharp_ear",
        1,
        t(
            "Sharp Ear I",
            "Oreille fine I",
            "Oído fino I",
            "Orecchio fino I",
        ),
        t(
            "Have 25 of your ratings settle in line with the verdict.",
            "Faites concorder 25 de vos notes avec le verdict.",
            "Consigue que 25 de tus valoraciones coincidan con el veredicto.",
            "Fai coincidere 25 delle tue valutazioni con il verdetto.",
        ),
    ),
    tiered(
        "sharp_ear_2",
        BadgeFamily::Curation,
        BadgeMetric::AlignedCount,
        100,
        "sharp_ear",
        2,
        t(
            "Sharp Ear II",
            "Oreille fine II",
            "Oído fino II",
            "Orecchio fino II",
        ),
        t(
            "Have 100 of your ratings settle in line with the verdict.",
            "Faites concorder 100 de vos notes avec le verdict.",
            "Consigue que 100 de tus valoraciones coincidan con el veredicto.",
            "Fai coincidere 100 delle tue valutazioni con il verdetto.",
        ),
    ),
    badge(
        "trailblazer",
        BadgeFamily::Curation,
        BadgeMetric::FirstRaterCount,
        20,
        t("Trailblazer", "Éclaireur", "Pionero", "Pioniere"),
        t(
            "Be the first to rate 20 scores.",
            "Soyez le premier à noter 20 partitions.",
            "Sé el primero en valorar 20 partituras.",
            "Sii il primo a valutare 20 spartiti.",
        ),
    ),
    // --- learning --------------------------------------------------------
    tiered(
        "student_1",
        BadgeFamily::Learning,
        BadgeMetric::CoursesCompleted,
        1,
        "student",
        1,
        t("Student I", "Élève I", "Estudiante I", "Studente I"),
        t(
            "Complete your first course.",
            "Terminez votre premier cours.",
            "Completa tu primer curso.",
            "Completa il tuo primo corso.",
        ),
    ),
    tiered(
        "student_2",
        BadgeFamily::Learning,
        BadgeMetric::CoursesCompleted,
        10,
        "student",
        2,
        t("Student II", "Élève II", "Estudiante II", "Studente II"),
        t(
            "Complete 10 courses.",
            "Terminez 10 cours.",
            "Completa 10 cursos.",
            "Completa 10 corsi.",
        ),
    ),
    tiered(
        "student_3",
        BadgeFamily::Learning,
        BadgeMetric::CoursesCompleted,
        42,
        "student",
        3,
        t("Student III", "Élève III", "Estudiante III", "Studente III"),
        t(
            "Complete 42 courses.",
            "Terminez 42 cours.",
            "Completa 42 cursos.",
            "Completa 42 corsi.",
        ),
    ),
];

/// The registry entries belonging to one family (the curation subset feeds the
/// deprecated `CuratorRewards.badges` field, design D8).
pub fn family_badges(family: BadgeFamily) -> impl Iterator<Item = &'static BadgeDef> {
    REGISTRY.iter().filter(move |b| b.family == family)
}

/// The CURATION badge keys earned at these counters. The curation rewards module
/// grants exactly this subset on its own write paths (a rating recorded, a
/// settlement landing) and reports it on the deprecated `CuratorRewards.badges`
/// wire field, so a released app version keeps rendering the grid it knows
/// (design D8) — while the registry stays the only place these are defined.
pub fn earned_curation_badges(counters: &BadgeCounters) -> Vec<&'static str> {
    family_badges(BadgeFamily::Curation)
        .filter(|b| counters.value(b.metric) >= b.threshold)
        .map(|b| b.key)
        .collect()
}

/// Every counter a badge read is measured against, one value per [`BadgeMetric`].
/// Folded from the repo's single-call raw read (design D4 /
/// [`crate::badges::RawBadgeCounters::fold`]), so the timezone-sensitive
/// consistency counters are computed here and stay host-testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BadgeCounters {
    pub rating_count: i64,
    pub aligned_count: i64,
    pub first_rater_count: i64,
    pub session_count: i64,
    pub distinct_pieces: i64,
    pub high_accuracy_sessions: i64,
    pub days_played: i64,
    pub longest_streak: i64,
    pub ranked_boards: i64,
    pub top_three_finishes: i64,
    pub season_podiums: i64,
    pub accepted_proposals: i64,
    pub accepted_soundfonts: i64,
    pub courses_completed: i64,
}

impl BadgeCounters {
    /// The user's current value for `metric`.
    pub fn value(&self, metric: BadgeMetric) -> i64 {
        match metric {
            BadgeMetric::RatingCount => self.rating_count,
            BadgeMetric::AlignedCount => self.aligned_count,
            BadgeMetric::FirstRaterCount => self.first_rater_count,
            BadgeMetric::SessionCount => self.session_count,
            BadgeMetric::DistinctPieces => self.distinct_pieces,
            BadgeMetric::HighAccuracySessions => self.high_accuracy_sessions,
            BadgeMetric::DaysPlayed => self.days_played,
            BadgeMetric::LongestStreak => self.longest_streak,
            BadgeMetric::RankedBoards => self.ranked_boards,
            BadgeMetric::TopThreeFinishes => self.top_three_finishes,
            BadgeMetric::SeasonPodiums => self.season_podiums,
            BadgeMetric::AcceptedProposals => self.accepted_proposals,
            BadgeMetric::AcceptedSoundFonts => self.accepted_soundfonts,
            BadgeMetric::CoursesCompleted => self.courses_completed,
        }
    }
}

/// Distinct local days in `days` (the repo hands over one entry per session, so
/// the same day can appear many times and in any order).
pub fn distinct_days(days: &[NaiveDate]) -> i64 {
    let mut sorted = days.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted.len() as i64
}

/// The longest run of CONSECUTIVE local days in `days`. Duplicates and ordering
/// are irrelevant (a day played twice is still one day); a gap of two or more
/// calendar days breaks the run. Empty input is 0, a single day is 1.
pub fn longest_streak(days: &[NaiveDate]) -> i64 {
    let mut sorted = days.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let mut best = 0i64;
    let mut run = 0i64;
    let mut prev: Option<NaiveDate> = None;
    for d in sorted {
        run = match prev {
            // `succ_opt` is None only at the end of the representable calendar.
            Some(p) if p.succ_opt() == Some(d) => run + 1,
            _ => 1,
        };
        best = best.max(run);
        prev = Some(d);
    }
    best
}

/// One badge projected against a user: the definition plus where they stand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BadgeStanding {
    pub def: &'static BadgeDef,
    /// Granted OR currently over the threshold (design D3 — the union).
    pub earned: bool,
    /// The user's value for the metric, CLAMPED to the threshold when earned so an
    /// earned badge never renders as incomplete.
    pub value: i64,
    /// When the badge was granted, in unix millis. `None` for a badge earned by
    /// counter but not yet granted — which only a caller that reads WITHOUT
    /// granting can observe, since the module grants in the same call.
    pub granted_at_ms: Option<i64>,
}

/// Project the whole registry against a user's counters and their granted badges
/// (`key` → granted-at unix millis), in registry order.
///
/// `earned` is the UNION of "has ever been granted" and "the counter clears the
/// threshold" (design D3): the grant row is the durable memory of the date, and
/// the live counter is what makes a newly defined badge retroactive with no
/// backfill. A badge therefore survives its underlying activity being purged, and
/// a threshold raised later cannot un-earn it.
pub fn evaluate(counters: &BadgeCounters, granted: &HashMap<String, i64>) -> Vec<BadgeStanding> {
    REGISTRY
        .iter()
        .map(|def| {
            let raw = counters.value(def.metric);
            let granted_at_ms = granted.get(def.key).copied();
            let earned = granted_at_ms.is_some() || raw >= def.threshold;
            BadgeStanding {
                def,
                earned,
                // Earned ⇒ report the threshold: a badge granted long ago whose
                // counter has since fallen must not read "3/20".
                value: if earned { def.threshold } else { raw },
                granted_at_ms,
            }
        })
        .collect()
}

/// The keys among `standings` that are earned but hold no grant row yet — exactly
/// the set the module inserts (idempotently) on a read.
pub fn newly_due(standings: &[BadgeStanding]) -> Vec<&'static str> {
    standings
        .iter()
        .filter(|s| s.earned && s.granted_at_ms.is_none())
        .map(|s| s.def.key)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn granted(pairs: &[(&str, i64)]) -> HashMap<String, i64> {
        pairs.iter().map(|(k, at)| (k.to_string(), *at)).collect()
    }

    fn standing<'a>(all: &'a [BadgeStanding], key: &str) -> &'a BadgeStanding {
        all.iter()
            .find(|s| s.def.key == key)
            .unwrap_or_else(|| panic!("no badge {key} in the registry"))
    }

    // --- registry integrity ------------------------------------------------

    #[test]
    fn registry_keys_are_unique_and_thresholds_positive() {
        let mut keys: Vec<&str> = REGISTRY.iter().map(|b| b.key).collect();
        let total = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), total, "duplicate badge key in the registry");
        assert!(REGISTRY.iter().all(|b| b.threshold >= 1));
    }

    #[test]
    fn the_seven_curation_badges_keep_their_keys_and_thresholds() {
        // Already-granted `curation_grants` rows are re-read under these keys, so
        // moving either would silently drop a badge someone earned.
        let expected: [(&str, BadgeMetric, i64); 7] = [
            ("first_note", BadgeMetric::RatingCount, 1),
            ("curator_1", BadgeMetric::RatingCount, 10),
            ("curator_2", BadgeMetric::RatingCount, 100),
            ("curator_3", BadgeMetric::RatingCount, 500),
            ("sharp_ear_1", BadgeMetric::AlignedCount, 25),
            ("sharp_ear_2", BadgeMetric::AlignedCount, 100),
            ("trailblazer", BadgeMetric::FirstRaterCount, 20),
        ];
        for (key, metric, threshold) in expected {
            let def = REGISTRY
                .iter()
                .find(|b| b.key == key)
                .unwrap_or_else(|| panic!("missing curation badge {key}"));
            assert_eq!(def.metric, metric, "{key} metric moved");
            assert_eq!(def.threshold, threshold, "{key} threshold moved");
            assert_eq!(def.family, BadgeFamily::Curation);
        }
        assert_eq!(family_badges(BadgeFamily::Curation).count(), 7);
    }

    #[test]
    fn a_track_has_ascending_tiers_on_one_metric() {
        let mut tracks: Vec<&str> = REGISTRY.iter().filter_map(|b| b.track).collect();
        tracks.sort_unstable();
        tracks.dedup();
        assert!(!tracks.is_empty());
        for track in tracks {
            let tiers: Vec<&BadgeDef> =
                REGISTRY.iter().filter(|b| b.track == Some(track)).collect();
            assert!(tiers.len() >= 2, "{track} is a track of one");
            for pair in tiers.windows(2) {
                assert_eq!(pair[0].metric, pair[1].metric, "{track} mixes metrics");
                assert_eq!(pair[0].family, pair[1].family, "{track} mixes families");
                assert!(pair[0].tier < pair[1].tier, "{track} tiers not ascending");
                assert!(
                    pair[0].threshold < pair[1].threshold,
                    "{track} thresholds not ascending"
                );
            }
        }
        // A standalone badge carries no track and tier 0.
        assert!(REGISTRY.iter().all(|b| b.track.is_some() == (b.tier > 0)));
    }

    #[test]
    fn every_family_but_none_is_represented() {
        // Every declared family currently ships at least one badge; the grid hides
        // a family that does not (which is why declaring one early is free).
        for f in [
            BadgeFamily::Play,
            BadgeFamily::Consistency,
            BadgeFamily::Ranking,
            BadgeFamily::Contribution,
            BadgeFamily::Curation,
            BadgeFamily::Learning,
        ] {
            assert!(family_badges(f).count() > 0, "{} is empty", f.as_str());
        }
    }

    #[test]
    fn family_and_metric_wire_strings_are_distinct() {
        let metrics = [
            BadgeMetric::RatingCount,
            BadgeMetric::AlignedCount,
            BadgeMetric::FirstRaterCount,
            BadgeMetric::SessionCount,
            BadgeMetric::DistinctPieces,
            BadgeMetric::HighAccuracySessions,
            BadgeMetric::DaysPlayed,
            BadgeMetric::LongestStreak,
            BadgeMetric::RankedBoards,
            BadgeMetric::TopThreeFinishes,
            BadgeMetric::SeasonPodiums,
            BadgeMetric::AcceptedProposals,
            BadgeMetric::AcceptedSoundFonts,
            BadgeMetric::CoursesCompleted,
        ];
        let mut wire: Vec<&str> = metrics.iter().map(|m| m.as_str()).collect();
        wire.sort_unstable();
        wire.dedup();
        assert_eq!(wire.len(), metrics.len());
        // The three curation strings are the ones the shipped app already parses.
        assert_eq!(BadgeMetric::RatingCount.as_str(), "rating_count");
        assert_eq!(BadgeMetric::AlignedCount.as_str(), "aligned_count");
        assert_eq!(BadgeMetric::FirstRaterCount.as_str(), "first_rater_count");
        assert_eq!(BadgeFamily::Learning.as_str(), "learning");
    }

    // --- localized text ----------------------------------------------------

    #[test]
    fn localized_text_serialises_every_language() {
        let json = t("Podium", "Podium", "Podio", "Podio").to_json();
        assert_eq!(
            json,
            r#"{"en":"Podium","fr":"Podium","es":"Podio","it":"Podio"}"#
        );
        // A quote or backslash in a future entry cannot break the payload.
        let tricky = t("a\"b", "c\\d", "e", "f").to_json();
        assert!(tricky.contains(r#"\"b"#));
        assert!(tricky.contains(r"c\\d"));
    }

    #[test]
    fn every_registry_entry_ships_all_four_languages() {
        for b in REGISTRY {
            for (what, txt) in [("label", b.label), ("description", b.description)] {
                for (lang, s) in [
                    ("en", txt.en),
                    ("fr", txt.fr),
                    ("es", txt.es),
                    ("it", txt.it),
                ] {
                    assert!(!s.trim().is_empty(), "{} {what} missing {lang}", b.key);
                }
            }
        }
    }

    // --- consistency folds (task 1.5) --------------------------------------

    #[test]
    fn distinct_days_dedupes_and_ignores_order() {
        assert_eq!(distinct_days(&[]), 0);
        let days = [
            ymd(2026, 3, 4),
            ymd(2026, 3, 2),
            ymd(2026, 3, 4), // same day, second session
            ymd(2026, 3, 2),
        ];
        assert_eq!(distinct_days(&days), 2);
    }

    #[test]
    fn local_day_bucketing_across_a_utc_boundary_counts_one_day() {
        // The repo shifts `played_at` by the client offset before handing days
        // over; two sessions either side of midnight UTC but on the SAME local day
        // arrive as the same date and must count once (spec scenario).
        use crate::play_core::local_day;
        // 2024-06-15T23:30:00Z and 2024-06-16T00:30:00Z, both at UTC+2 → the 16th.
        let late = local_day(1_718_494_200_000, 120);
        let early = local_day(1_718_494_200_000 + 3_600_000, 120);
        assert_eq!(late, early);
        assert_eq!(distinct_days(&[late, early]), 1);
        assert_eq!(longest_streak(&[late, early]), 1);
    }

    #[test]
    fn longest_streak_handles_empty_single_gap_and_middle_run() {
        assert_eq!(longest_streak(&[]), 0);
        assert_eq!(longest_streak(&[ymd(2026, 1, 5)]), 1);
        // A gap breaks the run: 1,2 | 5 → 2.
        assert_eq!(
            longest_streak(&[ymd(2026, 1, 1), ymd(2026, 1, 2), ymd(2026, 1, 5)]),
            2
        );
        // The longest run sits in the MIDDLE, not at either end: 1 | 4,5,6,7 | 20.
        let days = [
            ymd(2026, 1, 1),
            ymd(2026, 1, 4),
            ymd(2026, 1, 5),
            ymd(2026, 1, 6),
            ymd(2026, 1, 7),
            ymd(2026, 1, 20),
        ];
        assert_eq!(longest_streak(&days), 4);
        // Unordered input with duplicates gives the same answer.
        let shuffled = [
            ymd(2026, 1, 20),
            ymd(2026, 1, 5),
            ymd(2026, 1, 1),
            ymd(2026, 1, 7),
            ymd(2026, 1, 5),
            ymd(2026, 1, 4),
            ymd(2026, 1, 6),
        ];
        assert_eq!(longest_streak(&shuffled), 4);
        // A run across a month boundary is still a run.
        assert_eq!(
            longest_streak(&[ymd(2026, 1, 30), ymd(2026, 1, 31), ymd(2026, 2, 1)]),
            3
        );
    }

    // --- evaluation (task 1.6) ---------------------------------------------

    #[test]
    fn threshold_boundary_earns_exactly_at_the_threshold() {
        let below = BadgeCounters {
            session_count: 24,
            ..Default::default()
        };
        let at = BadgeCounters {
            session_count: 25,
            ..Default::default()
        };
        let none = granted(&[]);
        assert!(!standing(&evaluate(&below, &none), "performer_1").earned);
        assert!(standing(&evaluate(&at, &none), "performer_1").earned);
    }

    #[test]
    fn locked_badge_reports_the_raw_value_for_a_progress_bar() {
        // The spec's "12 of 25" case.
        let counters = BadgeCounters {
            session_count: 12,
            ..Default::default()
        };
        let s = *standing(&evaluate(&counters, &granted(&[])), "performer_1");
        assert!(!s.earned);
        assert_eq!(s.value, 12);
        assert_eq!(s.def.threshold, 25);
        assert!(s.granted_at_ms.is_none());
    }

    #[test]
    fn earned_badge_clamps_its_value_to_the_threshold() {
        let counters = BadgeCounters {
            rating_count: 640,
            ..Default::default()
        };
        let all = evaluate(&counters, &granted(&[]));
        // Well past every curator tier → each reports its own threshold, never 640.
        assert_eq!(standing(&all, "curator_1").value, 10);
        assert_eq!(standing(&all, "curator_2").value, 100);
        assert_eq!(standing(&all, "curator_3").value, 500);
        assert!(all.iter().all(|s| s.value <= s.def.threshold));
    }

    #[test]
    fn a_granted_badge_stays_earned_when_its_counter_falls_away() {
        // The union rule (design D3): the leaderboard piece was purged, the play
        // sessions aged out — the badge and its date survive.
        let all = evaluate(
            &BadgeCounters::default(),
            &granted(&[("podium_1", 1_700_000_000_000)]),
        );
        let s = *standing(&all, "podium_1");
        assert!(s.earned);
        assert_eq!(s.granted_at_ms, Some(1_700_000_000_000));
        // ...and it does NOT render as 0/1.
        assert_eq!(s.value, s.def.threshold);
        // An unearned badge is unaffected.
        assert!(!standing(&all, "podium_2").earned);
    }

    #[test]
    fn a_newly_defined_badge_is_due_retroactively_without_a_backfill() {
        // The user already qualifies but holds no grant row: it is earned on this
        // read and reported as due, so the module inserts the grant now.
        let counters = BadgeCounters {
            courses_completed: 12,
            ..Default::default()
        };
        let all = evaluate(&counters, &granted(&[]));
        assert!(standing(&all, "student_1").earned);
        assert!(standing(&all, "student_2").earned);
        assert!(!standing(&all, "student_3").earned);
        let due = newly_due(&all);
        assert!(due.contains(&"student_1"));
        assert!(due.contains(&"student_2"));
        assert!(!due.contains(&"student_3"));
    }

    #[test]
    fn an_already_granted_badge_is_not_due_again() {
        let counters = BadgeCounters {
            courses_completed: 1,
            ..Default::default()
        };
        let all = evaluate(&counters, &granted(&[("student_1", 42)]));
        assert!(!newly_due(&all).contains(&"student_1"));
    }

    #[test]
    fn evaluate_projects_the_whole_registry_in_order() {
        let all = evaluate(&BadgeCounters::default(), &granted(&[]));
        assert_eq!(all.len(), REGISTRY.len());
        for (s, def) in all.iter().zip(REGISTRY) {
            assert_eq!(s.def.key, def.key);
        }
        // A brand-new account has earned nothing and every value reads 0.
        assert!(all.iter().all(|s| !s.earned && s.value == 0));
    }

    #[test]
    fn a_player_who_never_rates_still_earns_badges() {
        // The spec's motivating scenario.
        let counters = BadgeCounters {
            session_count: 30,
            distinct_pieces: 12,
            days_played: 11,
            longest_streak: 4,
            ..Default::default()
        };
        let all = evaluate(&counters, &granted(&[]));
        let earned: Vec<&str> = all.iter().filter(|s| s.earned).map(|s| s.def.key).collect();
        assert!(earned.contains(&"first_performance"));
        assert!(earned.contains(&"performer_1"));
        assert!(earned.contains(&"repertoire_1"));
        assert!(earned.contains(&"regular_1"));
        assert!(earned.contains(&"streak_1"));
        // ...and no curation badge, since they have never rated anything.
        assert!(
            !all.iter()
                .any(|s| s.earned && s.def.family == BadgeFamily::Curation)
        );
    }

    #[test]
    fn earned_curation_badges_is_the_curation_subset_only() {
        // The behaviour `curation_rewards_core::earned_badges` used to provide,
        // now derived from the registry (the seven milestones, nothing else).
        assert!(earned_curation_badges(&BadgeCounters::default()).is_empty());

        let one = BadgeCounters {
            rating_count: 1,
            ..Default::default()
        };
        assert_eq!(earned_curation_badges(&one), vec!["first_note"]);

        let seasoned = BadgeCounters {
            rating_count: 100,
            aligned_count: 25,
            first_rater_count: 20,
            // Play activity must NOT leak into the curation subset.
            session_count: 10_000,
            courses_completed: 42,
            ..Default::default()
        };
        let got = earned_curation_badges(&seasoned);
        assert!(got.contains(&"first_note"));
        assert!(got.contains(&"curator_1")); // 10
        assert!(got.contains(&"curator_2")); // 100
        assert!(!got.contains(&"curator_3")); // 500 not reached
        assert!(got.contains(&"sharp_ear_1")); // 25 aligned
        assert!(!got.contains(&"sharp_ear_2")); // 100 aligned not reached
        assert!(got.contains(&"trailblazer")); // 20 first-rater
        assert!(got.iter().all(|k| {
            REGISTRY
                .iter()
                .any(|b| b.key == *k && b.family == BadgeFamily::Curation)
        }));
    }

    #[test]
    fn counters_address_every_metric_distinctly() {
        // Each field feeds exactly its own metric — a copy/paste slip in `value`
        // would otherwise silently mis-award a badge.
        let c = BadgeCounters {
            rating_count: 1,
            aligned_count: 2,
            first_rater_count: 3,
            session_count: 4,
            distinct_pieces: 5,
            high_accuracy_sessions: 6,
            days_played: 7,
            longest_streak: 8,
            ranked_boards: 9,
            top_three_finishes: 10,
            season_podiums: 11,
            accepted_proposals: 12,
            accepted_soundfonts: 13,
            courses_completed: 14,
        };
        assert_eq!(c.value(BadgeMetric::RatingCount), 1);
        assert_eq!(c.value(BadgeMetric::AlignedCount), 2);
        assert_eq!(c.value(BadgeMetric::FirstRaterCount), 3);
        assert_eq!(c.value(BadgeMetric::SessionCount), 4);
        assert_eq!(c.value(BadgeMetric::DistinctPieces), 5);
        assert_eq!(c.value(BadgeMetric::HighAccuracySessions), 6);
        assert_eq!(c.value(BadgeMetric::DaysPlayed), 7);
        assert_eq!(c.value(BadgeMetric::LongestStreak), 8);
        assert_eq!(c.value(BadgeMetric::RankedBoards), 9);
        assert_eq!(c.value(BadgeMetric::TopThreeFinishes), 10);
        assert_eq!(c.value(BadgeMetric::SeasonPodiums), 11);
        assert_eq!(c.value(BadgeMetric::AcceptedProposals), 12);
        assert_eq!(c.value(BadgeMetric::AcceptedSoundFonts), 13);
        assert_eq!(c.value(BadgeMetric::CoursesCompleted), 14);
    }
}
