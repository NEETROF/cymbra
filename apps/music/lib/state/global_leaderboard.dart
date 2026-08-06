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

// GLOBAL leaderboard view models (change: add-global-leaderboard). Plain
// immutable classes with no Flutter/gRPC imports, so the Community screen and its
// notifiers are testable without the native library or a live backend (the service
// seam maps the wire types into these).
//
// The tempo/reaction split is the SAME axis as the per-piece boards, so these
// reuse [LeaderboardMode] rather than introducing a parallel enum.

/// One ranked player on a global board. Carries only NON-SENSITIVE public fields
/// (the handle/display name the server chose to expose) — never an email or any
/// moderation/curator field.
class GlobalLeaderboardEntry {
  const GlobalLeaderboardEntry({
    required this.rank,
    required this.userId,
    required this.handle,
    required this.displayName,
    required this.score,
    required this.contributingPieces,
    required this.reachedAtMs,
  });

  /// 1-based rank among the public entries.
  final int rank;
  final String userId;
  final String? handle;
  final String? displayName;

  /// The difficulty-weighted best-N global season score (the ranked value).
  final double score;

  /// How many pieces fed the score (a display detail + the first tie-break).
  final int contributingPieces;
  final int reachedAtMs;

  /// The best label to show for this player: the handle, else the display name.
  String? get label => handle ?? displayName;
}

/// A global board as shown: the season it belongs to, the ranked page of public
/// entries, the total public count, and the viewer's own standing (present
/// whenever they scored this season — even when they are private and therefore
/// NOT in [entries]).
class GlobalLeaderboard {
  const GlobalLeaderboard({
    required this.seasonId,
    required this.entries,
    required this.total,
    required this.own,
  });

  final String seasonId;
  final List<GlobalLeaderboardEntry> entries;
  final int total;

  /// The viewer's own score + rank among the public entries, or null when they
  /// have no score in this season.
  final GlobalLeaderboardEntry? own;

  static const GlobalLeaderboard empty = GlobalLeaderboard(
    seasonId: '',
    entries: [],
    total: 0,
    own: null,
  );

  /// Whether [entry] is the viewer's own row (for highlighting it in the list).
  bool isViewer(GlobalLeaderboardEntry entry) =>
      own != null && entry.userId == own!.userId;
}

/// The seasons a selector may offer: the live one plus the snapshotted past ones
/// (most recent first).
class GlobalSeasons {
  const GlobalSeasons({
    required this.currentSeasonId,
    required this.pastSeasonIds,
  });

  final String currentSeasonId;
  final List<String> pastSeasonIds;

  static const GlobalSeasons empty = GlobalSeasons(
    currentSeasonId: '',
    pastSeasonIds: [],
  );

  /// Every selectable season, current first.
  List<String> get all => [currentSeasonId, ...pastSeasonIds];
}
