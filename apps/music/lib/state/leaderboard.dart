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

// Leaderboard view models (change: add-play-leaderboards). Plain immutable
// classes with no Flutter/gRPC imports, so the widgets + notifier are testable
// without the native library or a live backend (the service seam maps the wire
// types into these).

/// Which board is being viewed: the **tempo** board (free-run sub-score) or the
/// **reaction** board (Wait-Mode sub-score). [wire] is the value the RPC expects.
enum LeaderboardMode {
  tempo,
  reaction;

  String get wire => switch (this) {
    LeaderboardMode.tempo => 'tempo',
    LeaderboardMode.reaction => 'reaction',
  };
}

/// One ranked player on a board. Carries only NON-SENSITIVE public fields (the
/// handle/display name the server chose to expose) — never an email or any
/// moderation/curator field.
class LeaderboardEntry {
  const LeaderboardEntry({
    required this.rank,
    required this.userId,
    required this.handle,
    required this.displayName,
    required this.subscore,
    required this.tiebreakMetric,
    required this.achievedAtMs,
  });

  /// 1-based rank among the public entries.
  final int rank;
  final String userId;
  final String? handle;
  final String? displayName;

  /// The per-mode synchronization sub-score 0..100 (the ranked value).
  final double subscore;

  /// The normalised timing tie-break (smaller is better); a display detail.
  final double tiebreakMetric;
  final int achievedAtMs;

  /// The best label to show for this player: the handle, else the display name.
  String? get label => handle ?? displayName;
}

/// A board as shown: the ranked page of public entries, the total public count,
/// and the viewer's own standing (present whenever they have scored on this
/// board — even when they are private and therefore NOT in [entries]).
class Leaderboard {
  const Leaderboard({
    required this.entries,
    required this.total,
    required this.own,
  });

  final List<LeaderboardEntry> entries;
  final int total;

  /// The viewer's own best + rank among the public entries, or null when they
  /// have never scored on this board.
  final LeaderboardEntry? own;

  static const Leaderboard empty = Leaderboard(
    entries: [],
    total: 0,
    own: null,
  );

  /// Whether [entry] is the viewer's own row (for highlighting it in the list).
  bool isViewer(LeaderboardEntry entry) =>
      own != null && entry.userId == own!.userId;
}
