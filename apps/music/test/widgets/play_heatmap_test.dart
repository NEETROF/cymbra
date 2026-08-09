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
import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/play_activity.dart';
import 'package:music/widgets/play_heatmap.dart';

Widget _host(PlayActivity activity, DateTime end) => MaterialApp(
  home: Scaffold(
    body: PlayHeatmap(activity: activity, endDate: end, weeks: 4),
  ),
);

void main() {
  group('heatmapColumns', () {
    test('returns `weeks` Mon→Sun columns, the last containing `end`', () {
      final end = DateTime(2024, 6, 15); // a Saturday
      final cols = heatmapColumns(end, 4);
      expect(cols, hasLength(4));
      expect(cols.every((c) => c.length == 7), isTrue);
      for (final c in cols) {
        expect(c.first.weekday, DateTime.monday);
        expect(c.last.weekday, DateTime.sunday);
      }
      expect(
        cols.last.any((d) => d.year == 2024 && d.month == 6 && d.day == 15),
        isTrue,
      );
    });
  });

  group('heatColor', () {
    test('hue tracks sync %; opacity tracks count', () {
      Color c(double sync, int count) => heatColor(
        syncPct: sync,
        count: count,
        low: Colors.red,
        high: Colors.green,
      );
      // Greener toward high sync.
      expect(c(10, 1).g, lessThan(c(90, 1).g));
      // Fuller (more opaque) with more plays.
      expect(c(50, 6).a, greaterThan(c(50, 1).a));
    });
  });

  testWidgets('a played day is colored with a count+avg tooltip', (
    tester,
  ) async {
    final activity = PlayActivity(
      days: [DayActivity(day: DateTime(2024, 6, 13), count: 3, avgSyncPct: 84)],
      totalSessions: 3,
    );
    await tester.pumpWidget(_host(activity, DateTime(2024, 6, 15)));

    // Exactly one played day → exactly one tooltip; empty days are blank.
    expect(find.byType(Tooltip), findsOneWidget);
    expect(tester.widget<Tooltip>(find.byType(Tooltip)).message, '84% · ×3');
  });

  testWidgets('a practice-only day is neutral, never a failure colour', (
    tester,
  ) async {
    final activity = PlayActivity(
      days: [
        DayActivity(
          day: DateTime(2024, 6, 13),
          count: 0,
          avgSyncPct: 0,
          practiceCount: 2,
        ),
      ],
      totalSessions: 0,
      totalPractices: 2,
    );
    await tester.pumpWidget(_host(activity, DateTime(2024, 6, 15)));

    expect(find.byType(Tooltip), findsOneWidget);
    // Only the practice count — the day has no synchronization percentage.
    expect(tester.widget<Tooltip>(find.byType(Tooltip)).message, '⟳×2');

    // It must NOT be painted on the success scale (0 % would read as failed).
    final scheme = ThemeData().colorScheme;
    final cell = tester.widget<Container>(
      find.descendant(
        of: find.byType(Tooltip),
        matching: find.byType(Container),
      ),
    );
    final painted = (cell.decoration! as BoxDecoration).color!;
    expect(
      painted,
      isNot(
        heatColor(
          syncPct: 0,
          count: 2,
          low: scheme.tertiary,
          high: scheme.primary,
        ),
      ),
    );
    expect(
      painted,
      practiceColor(practiceCount: 2, neutral: scheme.onSurfaceVariant),
    );
  });

  testWidgets('a mixed day keeps the scored colour and shows both counts', (
    tester,
  ) async {
    final activity = PlayActivity(
      days: [
        DayActivity(
          day: DateTime(2024, 6, 13),
          count: 2,
          avgSyncPct: 90,
          practiceCount: 1,
        ),
      ],
      totalSessions: 2,
      totalPractices: 1,
    );
    await tester.pumpWidget(_host(activity, DateTime(2024, 6, 15)));

    expect(
      tester.widget<Tooltip>(find.byType(Tooltip)).message,
      '90% · ×2 · ⟳×1',
    );
    // The hue comes from the scored average alone (practice has no sync %).
    final scheme = ThemeData().colorScheme;
    final cell = tester.widget<Container>(
      find.descendant(
        of: find.byType(Tooltip),
        matching: find.byType(Container),
      ),
    );
    final painted = (cell.decoration! as BoxDecoration).color!;
    expect(
      painted,
      heatColor(
        syncPct: 90,
        count: 3, // busy-ness counts practice; the hue does not
        low: scheme.tertiary,
        high: scheme.primary,
      ),
    );
  });

  testWidgets('an empty activity still renders the full (blank) grid', (
    tester,
  ) async {
    await tester.pumpWidget(_host(PlayActivity.empty, DateTime(2024, 6, 15)));
    // No played days → no tooltips...
    expect(find.byType(Tooltip), findsNothing);
    // ...but the calendar grid is still drawn: weeks × 7 visible cells, so the
    // heatmap reads as a grid rather than an empty gap (change: visible grid).
    expect(find.byType(Container), findsNWidgets(4 * 7));
  });
}
