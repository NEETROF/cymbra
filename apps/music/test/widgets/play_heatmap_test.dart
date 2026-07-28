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

  testWidgets('an empty activity renders no tooltips (all cells blank)', (
    tester,
  ) async {
    await tester.pumpWidget(_host(PlayActivity.empty, DateTime(2024, 6, 15)));
    expect(find.byType(Tooltip), findsNothing);
  });
}
