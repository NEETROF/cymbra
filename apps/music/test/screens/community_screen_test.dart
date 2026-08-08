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

import '../support/global_leaderboard_fakes.dart';
import '../support/localized.dart';

/// Wrap [child] in a root [ProviderContainer] (via [UncontrolledProviderScope]),
/// the repo's widget-test convention — overriding on a root container avoids the
/// `scoped_providers_should_specify_dependencies` lint a nested
/// `ProviderScope(overrides:)` would trip.
Widget _scope(FakeGlobalLeaderboardService service, Widget child) {
  final container = ProviderContainer(
    overrides: [globalLeaderboardServiceProvider.overrideWithValue(service)],
  );
  addTearDown(container.dispose);
  return UncontrolledProviderScope(container: container, child: child);
}

GlobalLeaderboard _board({
  required List<GlobalLeaderboardEntry> entries,
  GlobalLeaderboardEntry? own,
  String seasonId = '2026-01-01',
}) => GlobalLeaderboard(
  seasonId: seasonId,
  entries: entries,
  total: entries.length,
  own: own,
);

void main() {
  testWidgets('shows the ranked global board for the current season', (
    tester,
  ) async {
    final service = FakeGlobalLeaderboardService(
      boards: {
        FakeGlobalLeaderboardService.key(LeaderboardMode.tempo, ''): _board(
          entries: [
            globalEntry(rank: 1, userId: 'ada', score: 18.4),
            globalEntry(rank: 2, userId: 'bob', score: 12.0),
          ],
        ),
      },
    );
    await tester.pumpWidget(
      _scope(service, localizedApp(const CommunityScreen())),
    );
    await tester.pumpAndSettle();

    expect(find.text('@ada'), findsOneWidget);
    expect(find.text('#1'), findsOneWidget);
    expect(find.text('18.4 pts'), findsOneWidget);
    expect(find.text('@bob'), findsOneWidget);
    // The default read is the CURRENT season (empty season id on the wire).
    expect(service.requests.first, (LeaderboardMode.tempo, ''));
  });

  testWidgets('toggling the mode shows the other global ranking', (
    tester,
  ) async {
    final service = FakeGlobalLeaderboardService(
      boards: {
        FakeGlobalLeaderboardService.key(LeaderboardMode.tempo, ''): _board(
          entries: [globalEntry(rank: 1, userId: 'tempoking')],
        ),
        FakeGlobalLeaderboardService.key(LeaderboardMode.reaction, ''): _board(
          entries: [globalEntry(rank: 1, userId: 'reactionqueen')],
        ),
      },
    );
    await tester.pumpWidget(
      _scope(service, localizedApp(const CommunityScreen())),
    );
    await tester.pumpAndSettle();
    expect(find.text('@tempoking'), findsOneWidget);

    await tester.tap(find.text('Wait Mode'));
    await tester.pumpAndSettle();

    expect(find.text('@reactionqueen'), findsOneWidget);
    expect(find.text('@tempoking'), findsNothing);
    expect(service.requests.last, (LeaderboardMode.reaction, ''));
  });

  testWidgets('selecting a past season shows its snapshotted standings', (
    tester,
  ) async {
    const past = '2026-01-01';
    final service = FakeGlobalLeaderboardService(
      seasons: const GlobalSeasons(
        currentSeasonId: '2026-01-31',
        pastSeasonIds: [past],
      ),
      boards: {
        FakeGlobalLeaderboardService.key(LeaderboardMode.tempo, ''): _board(
          entries: [globalEntry(rank: 1, userId: 'nowchamp')],
        ),
        FakeGlobalLeaderboardService.key(LeaderboardMode.tempo, past): _board(
          entries: [globalEntry(rank: 1, userId: 'lastchamp')],
          seasonId: past,
        ),
      },
    );
    await tester.pumpWidget(
      _scope(service, localizedApp(const CommunityScreen())),
    );
    await tester.pumpAndSettle();
    expect(find.text('@nowchamp'), findsOneWidget);

    // The selector only appears once there IS a past season to pick.
    await tester.tap(find.byKey(const Key('community-season')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Season of $past').last);
    await tester.pumpAndSettle();

    expect(find.text('@lastchamp'), findsOneWidget);
    expect(find.text('@nowchamp'), findsNothing);
    expect(service.requests.last, (LeaderboardMode.tempo, past));
  });

  testWidgets('the season selector is hidden while there is no past season', (
    tester,
  ) async {
    final service = FakeGlobalLeaderboardService(
      boards: {
        FakeGlobalLeaderboardService.key(LeaderboardMode.tempo, ''): _board(
          entries: [globalEntry(rank: 1, userId: 'ada')],
        ),
      },
    );
    await tester.pumpWidget(
      _scope(service, localizedApp(const CommunityScreen())),
    );
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('community-season')), findsNothing);
  });

  testWidgets('a private player sees their own standing and highlight', (
    tester,
  ) async {
    // "me" is NOT in the listing (private) but the server still returns `own`.
    final service = FakeGlobalLeaderboardService(
      boards: {
        FakeGlobalLeaderboardService.key(LeaderboardMode.tempo, ''): _board(
          entries: [
            globalEntry(rank: 1, userId: 'ada', score: 20),
            globalEntry(rank: 2, userId: 'bob', score: 5),
          ],
          own: globalEntry(rank: 2, userId: 'me', score: 9.5),
        ),
      },
    );
    await tester.pumpWidget(
      _scope(service, localizedApp(const CommunityScreen())),
    );
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('community-own-standing')), findsOneWidget);
    expect(find.text('Your standing'), findsOneWidget);
    expect(find.text('Rank #2 of 2'), findsOneWidget);
    expect(find.text('9.5 pts'), findsOneWidget);
    // Not listed to others: no row carries the private player's handle.
    expect(find.text('@me'), findsNothing);
  });

  testWidgets('a listed viewer\'s own row is highlighted in the ranking', (
    tester,
  ) async {
    final service = FakeGlobalLeaderboardService(
      boards: {
        FakeGlobalLeaderboardService.key(LeaderboardMode.tempo, ''): _board(
          entries: [
            globalEntry(rank: 1, userId: 'ada'),
            globalEntry(rank: 2, userId: 'me'),
          ],
          own: globalEntry(rank: 2, userId: 'me'),
        ),
      },
    );
    await tester.pumpWidget(
      _scope(service, localizedApp(const CommunityScreen())),
    );
    await tester.pumpAndSettle();

    expect(find.text('@me (you)'), findsOneWidget);
  });

  testWidgets('an empty board shows the empty state', (tester) async {
    final service = FakeGlobalLeaderboardService();
    await tester.pumpWidget(
      _scope(service, localizedApp(const CommunityScreen())),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('No one has scored this season yet — be the first!'),
      findsOneWidget,
    );
  });

  testWidgets('a failed read shows the error state and can retry', (
    tester,
  ) async {
    final service = FakeGlobalLeaderboardService(fail: true);
    await tester.pumpWidget(
      _scope(service, localizedApp(const CommunityScreen())),
    );
    await tester.pumpAndSettle();

    expect(find.text("Couldn't load the leaderboard."), findsOneWidget);
    final before = service.requests.length;
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();
    expect(service.requests.length, greaterThan(before));
  });
}
