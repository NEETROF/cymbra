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

/// One local day of play activity — a heatmap cell (change: add-play-activity-
/// profile). [avgSyncPct] (0..100) drives the cell's color; [count] its intensity
/// + tooltip.
class DayActivity {
  const DayActivity({
    required this.day,
    required this.count,
    required this.avgSyncPct,
    this.practiceCount = 0,
  });

  /// The player's local calendar day (date-only; time is meaningless here).
  final DateTime day;

  /// Scored plays that day — the only sessions [avgSyncPct] is computed from.
  final int count;
  final double avgSyncPct;

  /// Practice (selective/unscored) sessions that day (change: add-measure-range-
  /// practice). Practice carries no synchronization percentage, so it never
  /// contributes to the day's success **color** — only to its intensity/tooltip.
  final int practiceCount;

  /// Whether the day was active at all (a scored play or a practice).
  bool get hasActivity => count > 0 || practiceCount > 0;

  /// Whether the day holds **only** practice — it must render as a neutral
  /// active cell, never as a 0 %/failure colour.
  bool get isPracticeOnly => count == 0 && practiceCount > 0;
}

/// A player's per-day activity plus their songs-played total, as read for the
/// profile heatmap.
class PlayActivity {
  const PlayActivity({
    required this.days,
    required this.totalSessions,
    this.totalPractices = 0,
  });

  /// Per-day activity, ordered by day. Days with no play are simply absent (the
  /// heatmap renders those cells blank).
  final List<DayActivity> days;

  /// Songs-played total across the returned window.
  final int totalSessions;

  /// Practice-sessions total across the returned window.
  final int totalPractices;

  static const PlayActivity empty = PlayActivity(days: [], totalSessions: 0);

  /// Index activity by day (date-only key) for O(1) cell lookup by the heatmap.
  Map<DateTime, DayActivity> byDay() => {
    for (final d in days) DateTime.utc(d.day.year, d.day.month, d.day.day): d,
  };
}
