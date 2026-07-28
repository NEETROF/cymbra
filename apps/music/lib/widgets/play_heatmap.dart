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

import 'package:flutter/material.dart';

import '../state/play_activity.dart';

/// Date-only key (ignores time/timezone) used to match a day to its activity.
DateTime heatmapDayKey(DateTime d) => DateTime.utc(d.year, d.month, d.day);

/// The heatmap grid as week-columns of 7 days (Mon→Sun), the last column
/// containing [end]. Pure so the layout is unit-testable. Days later than [end]
/// in the final week are still returned (the widget renders them blank).
List<List<DateTime>> heatmapColumns(DateTime end, int weeks) {
  final endDay = heatmapDayKey(end);
  // Monday on/before `end`, then step back to the first visible week.
  final endMonday = endDay.subtract(
    Duration(days: (endDay.weekday - DateTime.monday) % 7),
  );
  final startMonday = endMonday.subtract(Duration(days: (weeks - 1) * 7));
  return [
    for (var w = 0; w < weeks; w++)
      [for (var d = 0; d < 7; d++) startMonday.add(Duration(days: w * 7 + d))],
  ];
}

/// Cell color for a played day: hue from the day's average overall
/// synchronization percentage ([syncPct], the requested weighting), opacity from
/// the [count] of songs played (more plays → fuller cell). Pure (colors passed
/// in) so it is unit-testable without a `BuildContext`.
Color heatColor({
  required double syncPct,
  required int count,
  required Color low,
  required Color high,
}) {
  final t = (syncPct.clamp(0, 100)) / 100.0;
  final base = Color.lerp(low, high, t)!;
  // 1 play → 45% opacity, saturating toward full by ~6 plays.
  final intensity = 0.45 + 0.55 * (count.clamp(1, 6) / 6);
  return base.withValues(alpha: intensity.clamp(0.0, 1.0));
}

/// A GitHub-style contribution grid of a player's activity (change: add-play-
/// activity-profile): one cell per day, colored by the day's average overall
/// synchronization percentage, with the songs-played count conveyed by the cell's
/// intensity and a tooltip. Days with no play render blank. Pure presentation —
/// it takes the [activity] directly, so it renders in tests from a fake aggregate
/// with no native library or backend.
class PlayHeatmap extends StatelessWidget {
  const PlayHeatmap({
    super.key,
    required this.activity,
    this.endDate,
    this.weeks = 26,
    this.cellSize = 12,
    this.cellGap = 3,
  });

  final PlayActivity activity;

  /// The last (most recent) day shown; defaults to today.
  final DateTime? endDate;
  final int weeks;
  final double cellSize;
  final double cellGap;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final end = heatmapDayKey(endDate ?? DateTime.now());
    final columns = heatmapColumns(end, weeks);
    final byDay = activity.byDay();
    // Empty-day fill: a fully-opaque muted tone (surfaceContainerHighest nudged
    // toward the foreground) so the calendar grid stays legible on any theme —
    // even before any activity — instead of dissolving into a dark background.
    final blank = Color.alphaBlend(
      scheme.onSurface.withValues(alpha: 0.10),
      scheme.surfaceContainerHighest,
    );
    // Hairline that outlines every cell, so the grid reads as a grid.
    final cellBorder = scheme.onSurface.withValues(alpha: 0.06);

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          for (final column in columns)
            Padding(
              padding: EdgeInsets.only(right: cellGap),
              child: Column(
                children: [
                  for (final day in column)
                    Padding(
                      padding: EdgeInsets.only(bottom: cellGap),
                      child: _cell(
                        day,
                        byDay[heatmapDayKey(day)],
                        end,
                        scheme,
                        blank,
                        cellBorder,
                      ),
                    ),
                ],
              ),
            ),
        ],
      ),
    );
  }

  Widget _cell(
    DateTime day,
    DayActivity? activity,
    DateTime end,
    ColorScheme scheme,
    Color blank,
    Color border,
  ) {
    final isFuture = day.isAfter(end);
    // Empty (or future) days render blank — no tooltip.
    if (activity == null || isFuture) {
      return _box(blank, border);
    }
    return Tooltip(
      // Compact, language-neutral: "84% · ×3" (avg sync % · songs played).
      message: '${activity.avgSyncPct.round()}% · ×${activity.count}',
      child: _box(
        heatColor(
          syncPct: activity.avgSyncPct,
          count: activity.count,
          low: scheme.tertiary,
          high: scheme.primary,
        ),
        border,
      ),
    );
  }

  Widget _box(Color color, Color border) => Container(
    width: cellSize,
    height: cellSize,
    decoration: BoxDecoration(
      color: color,
      borderRadius: BorderRadius.circular(2),
      border: Border.all(color: border, width: 0.5),
    ),
  );
}
