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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../l10n/gen/app_localizations.dart';

part 'score_catalog.g.dart';

/// Practice difficulty of a bundled score.
enum PracticeLevel { beginner, intermediate, advanced }

/// Human-readable label for a [PracticeLevel].
extension PracticeLevelLabel on PracticeLevel {
  String get label => switch (this) {
    PracticeLevel.beginner => 'Beginner',
    PracticeLevel.intermediate => 'Intermediate',
    PracticeLevel.advanced => 'Advanced',
  };

  /// Localized label (use in the UI; [label] is only a debug/fallback string).
  String localizedLabel(AppLocalizations l10n) => switch (this) {
    PracticeLevel.beginner => l10n.levelBeginner,
    PracticeLevel.intermediate => l10n.levelIntermediate,
    PracticeLevel.advanced => l10n.levelAdvanced,
  };
}

/// The identity a piece is keyed by outside the app — the BACKEND id the server
/// ranks and aggregates by (change: add-play-leaderboards), and the key its
/// device-local per-score data is stored under: the bare catalog UUID
/// ([CatalogEntry.catalogId]) for a public catalog score — what the leaderboard's
/// accepted-catalog check matches — else the upload id, else the app id, else the
/// piece [title]. [CatalogEntry.id] is namespaced (`catalog-…`/`contrib-…`), so it
/// must never be used on its own. Pure, so every caller resolves the same id.
String pieceIdentityOf(CatalogEntry? entry, String? title) =>
    entry?.catalogId ?? entry?.contributedId ?? entry?.id ?? title ?? 'demo';

/// One entry in the bundled score catalog: a public-domain MusicXML asset
/// tagged with display metadata and a practice level.
class CatalogEntry {
  /// Stable identifier (used as a list key and selection identity).
  final String id;
  final String title;
  final String composer;

  /// Bundle path of the uncompressed `.musicxml`/`.xml` asset. Empty for a
  /// user-contributed score, whose bytes come from the backend instead.
  final String assetPath;
  final PracticeLevel level;

  /// Backend id when this entry is a user-contributed score (loaded via the
  /// backend byte source); `null` for a bundled-catalog entry.
  final String? contributedId;

  /// Backend catalog id when this entry is a public-catalog score saved from the
  /// Score Hub (loaded via the catalog byte source); `null` otherwise.
  final String? catalogId;

  // Attribution + musical facets used to generate the cover art and the
  // attribution line (all optional; `null`/absent for bundled scores or until a
  // catalog row is backfilled). See widgets/score_card.dart.
  final String? source;
  final String? arranger;
  final int? minNoteValue;
  final int? tempoBpm;
  final int? noteCount;
  final int? lowestMidi;
  final int? highestMidi;
  final String? timeSig;
  final int? keyFifths;

  /// For a contributed (upload) entry: whether it is in the user's favorites.
  /// Meaningless (and `true`) for bundled/catalog entries.
  final bool favorite;

  /// For an upload: the handle of the account that uploaded it (attribution
  /// "{handle} · Cymbra"); `null` for bundled/catalog entries.
  final String? uploaderHandle;

  /// Moderation status (`pending` | `accepted`) for a rating-deck card, so a
  /// `pending` candidate can be labelled "potential new score" (change:
  /// rate-pending-scores). `null` for every other surface (bundled/upload/saved).
  final String? moderationStatus;

  /// Whether this deck card is a `pending` (not-yet-validated) candidate the
  /// community is helping evaluate.
  bool get isPending => moderationStatus == 'pending';

  /// For an upload: its public-catalog proposal state — `null` (not proposed),
  /// `pending` / `accepted` / `rejected` (change: add-score-catalog-proposal). Drives
  /// the propose action + status tag on the "mes partitions" card.
  final String? proposalStatus;

  /// The moderator's rejection reason, shown when [proposalStatus] is `rejected`.
  final String? proposalRejectionReason;

  /// For an `accepted` user-proposed catalog score: the proposer's public handle,
  /// shown as a "proposé par @pseudo" credit (change: add-score-catalog-proposal).
  /// `null` for a crawler-ingested score or when the proposer kept their profile
  /// private — the server gates this fail-closed, so the app just shows what it gets.
  final String? contributorCredit;

  /// For a catalog score: a server-rendered audio teaser exists (change:
  /// add-score-daily-access-rewards) — enables the "listen" control of the
  /// unlock flow without a probe. `false` for bundled/upload entries.
  final bool hasPreview;

  const CatalogEntry({
    required this.id,
    required this.title,
    required this.composer,
    required this.level,
    this.assetPath = '',
    this.contributedId,
    this.catalogId,
    this.source,
    this.arranger,
    this.minNoteValue,
    this.tempoBpm,
    this.noteCount,
    this.lowestMidi,
    this.highestMidi,
    this.timeSig,
    this.keyFifths,
    this.favorite = true,
    this.uploaderHandle,
    this.moderationStatus,
    this.proposalStatus,
    this.proposalRejectionReason,
    this.contributorCredit,
    this.hasPreview = false,
  });

  /// Whether this is a user upload (byte-sourced) rather than a bundled score.
  bool get isContributed => contributedId != null;

  /// Whether this is a saved public-catalog score (byte-sourced from the
  /// catalog) rather than a bundled or contributed score.
  bool get isCatalog => catalogId != null;

  @override
  int get hashCode => id.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is CatalogEntry &&
          runtimeType == other.runtimeType &&
          id == other.id;
}

/// The curated catalog of bundled scores. Const for the POC; exposed through a
/// provider so tests can override it with in-memory entries (no asset bundle).
@riverpod
List<CatalogEntry> scoreCatalog(Ref ref) => const [
  CatalogEntry(
    id: 'ode-to-joy',
    title: 'Ode to Joy (theme)',
    composer: 'Ludwig van Beethoven',
    assetPath: 'assets/scores/beginner/ode_to_joy.musicxml',
    level: PracticeLevel.beginner,
  ),
  CatalogEntry(
    id: 'twinkle',
    title: 'Twinkle, Twinkle, Little Star',
    composer: 'Traditional',
    assetPath: 'assets/scores/beginner/twinkle.musicxml',
    level: PracticeLevel.beginner,
  ),
  CatalogEntry(
    id: 'minuet-in-g',
    title: 'Minuet in G (BWV Anh. 114)',
    composer: 'Christian Petzold',
    assetPath: 'assets/scores/intermediate/minuet_in_g.musicxml',
    level: PracticeLevel.intermediate,
  ),
  CatalogEntry(
    id: 'arabesque-l-66-no-1-in-e-major',
    title: 'Arabesque, L. 66 No. 1 in E Major',
    composer: 'Claude Debussy',
    assetPath:
        'assets/scores/intermediate/arabesque-l-66-no-1-in-e-major.musicxml',
    level: PracticeLevel.intermediate,
  ),
  CatalogEntry(
    id: 'prelude-e-minor',
    title: 'Prelude in E minor, Op. 28 No. 4',
    composer: 'Frédéric Chopin',
    assetPath: 'assets/scores/advanced/prelude_e_minor.musicxml',
    level: PracticeLevel.advanced,
  ),
];

/// The score the user picked in the library, or null before any selection.
/// Watched by the notation notifier to know which asset to load.
///
/// `keepAlive` so the selection survives the gap between the library setting it
/// and the partition screen mounting to watch it (an auto-dispose provider would
/// drop the state in that window).
@Riverpod(keepAlive: true)
class SelectedScore extends _$SelectedScore {
  @override
  CatalogEntry? build() => null;

  /// Records [entry] as the active score (drives the partition screen).
  void select(CatalogEntry entry) => state = entry;

  /// Clears the selection (change: add-client-transport-deadlines): cancelling
  /// the score-open wait clears it so the notation load's stale-entry guards
  /// discard the late result — no separate "cancelled" flag needed.
  void clear() => state = null;
}
