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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/community_screen.dart';
import 'package:music/services/global_leaderboard_service.dart';
import 'package:music/state/global_leaderboard.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/widgets/global_standing_section.dart';

import '../support/global_leaderboard_fakes.dart';
import '../support/localized.dart';

Widget _scope(FakeGlobalLeaderboardService service, Widget child) {
  final container = ProviderContainer(
    overrides: [globalLeaderboardServiceProvider.overrideWithValue(service)],
  );
  addTearDown(container.dispose);
  return UncontrolledProviderScope(container: container, child: child);
}

GlobalLeaderboard _boardWithOwn(GlobalLeaderboardEntry own, {int total = 12}) =>
    GlobalLeaderboard(
      seasonId: '2026-01-01',
      entries: const [],
      total: total,
      own: own,
    );

void main() {
  testWidgets('shows the player\'s current-season rank and score per mode', (
    tester,
  ) async {
    final service = FakeGlobalLeaderboardService(
      boards: {
        FakeGlobalLeaderboardService.key(LeaderboardMode.tempo, ''):
            _boardWithOwn(globalEntry(rank: 3, userId: 'me', score: 14.2)),
        FakeGlobalLeaderboardService.key(
          LeaderboardMode.reaction,
          '',
        ): _boardWithOwn(
          globalEntry(rank: 7, userId: 'me', score: 8.0),
          total: 20,
        ),
      },
    );
    await tester.pumpWidget(
      _scope(service, localizedApp(const GlobalStandingSection())),
    );
    await tester.pumpAndSettle();

    expect(find.text('Global standing'), findsOneWidget);
    // Both modes are shown, each with its own rank + score.
    expect(find.text('Free play'), findsOneWidget);
    expect(find.text('Rank #3 of 12'), findsOneWidget);
    expect(find.text('14.2 pts'), findsOneWidget);
    expect(find.text('Wait Mode'), findsOneWidget);
    expect(find.text('Rank #7 of 20'), findsOneWidget);
    expect(find.text('8.0 pts'), findsOneWidget);
    // Both current-season boards were read.
    expect(service.requests, [
      (LeaderboardMode.tempo, ''),
      (LeaderboardMode.reaction, ''),
    ]);
  });

  testWidgets('a private player still sees their own standing', (tester) async {
    // Nothing about the standing depends on being listed: the server returns the
    // caller their own rank among the PUBLIC entries even when they are private,
    // so the section renders identically (an empty listing, a present `own`).
    final service = FakeGlobalLeaderboardService(
      boards: {
        FakeGlobalLeaderboardService.key(
          LeaderboardMode.tempo,
          '',
        ): _boardWithOwn(
          globalEntry(rank: 2, userId: 'private-me', score: 11),
        ),
      },
    );
    await tester.pumpWidget(
      _scope(service, localizedApp(const GlobalStandingSection())),
    );
    await tester.pumpAndSettle();

    expect(find.text('Rank #2 of 12'), findsOneWidget);
    expect(find.text('11.0 pts'), findsOneWidget);
  });

  testWidgets('an unranked player is nudged to play a catalog piece', (
    tester,
  ) async {
    final service = FakeGlobalLeaderboardService(); // no own standing
    await tester.pumpWidget(
      _scope(service, localizedApp(const GlobalStandingSection())),
    );
    await tester.pumpAndSettle();

    expect(
      find.text(
        'No global ranking this season yet — play a catalog piece to enter.',
      ),
      findsNWidgets(2), // one per mode
    );
  });

  testWidgets('the standing links into the Community screen', (tester) async {
    final service = FakeGlobalLeaderboardService(
      boards: {
        FakeGlobalLeaderboardService.key(LeaderboardMode.tempo, ''):
            _boardWithOwn(globalEntry(rank: 1, userId: 'me')),
      },
    );
    await tester.pumpWidget(
      _scope(service, localizedApp(const GlobalStandingSection())),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('global-standing-open')));
    await tester.pumpAndSettle();

    expect(find.byType(CommunityScreen), findsOneWidget);
  });

  testWidgets('a failed read leaves the profile section quiet', (tester) async {
    final service = FakeGlobalLeaderboardService(fail: true);
    await tester.pumpWidget(
      _scope(service, localizedApp(const GlobalStandingSection())),
    );
    await tester.pumpAndSettle();

    // The header + entry link stay; no rank/score row and no raw error text.
    expect(find.text('Global standing'), findsOneWidget);
    expect(find.textContaining('Rank #'), findsNothing);
    expect(find.textContaining('Exception'), findsNothing);
  });
}
