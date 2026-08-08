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
import 'package:music/services/profile_service.dart';
import 'package:music/state/global_leaderboard.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/play_activity.dart';
import 'package:music/state/play_activity_notifier.dart';
import 'package:music/state/profile_notifier.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/widgets/play_heatmap.dart';
import 'package:music/widgets/player_profile_panel.dart';

import '../support/global_leaderboard_fakes.dart';
import '../support/localized.dart';

PlayActivity _activity() => PlayActivity(
  days: [DayActivity(day: DateTime(2024, 6, 13), count: 3, avgSyncPct: 84)],
  totalSessions: 5,
);

PlayerProfile _profile(String id, {String? handle}) => PlayerProfile(
  userId: id,
  handle: handle ?? id,
  displayName: null,
  visibility: 'public',
);

Widget _scope(List<Override> overrides, Widget child) {
  final container = ProviderContainer(overrides: overrides);
  addTearDown(container.dispose);
  return UncontrolledProviderScope(container: container, child: child);
}

/// A Community screen whose board lists `ada` and the signed-in `me`.
Widget _community({required String? currentUserId}) {
  final service = FakeGlobalLeaderboardService(
    boards: {
      FakeGlobalLeaderboardService.key(
        LeaderboardMode.tempo,
        '',
      ): GlobalLeaderboard(
        seasonId: '2026-07-30',
        entries: [
          globalEntry(rank: 1, userId: 'ada'),
          globalEntry(rank: 2, userId: 'me'),
        ],
        total: 2,
        own: globalEntry(rank: 2, userId: 'me'),
      ),
    },
  );
  return _scope([
    globalLeaderboardServiceProvider.overrideWithValue(service),
    currentUserIdProvider.overrideWithValue(currentUserId),
    playerProfileProvider('ada').overrideWith((ref) async => _profile('ada')),
    playActivityProvider('ada').overrideWith((ref) async => _activity()),
  ], localizedApp(const CommunityScreen()));
}

void main() {
  testWidgets('tapping another player opens the profile panel in place', (
    tester,
  ) async {
    await tester.pumpWidget(_community(currentUserId: 'me'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('@ada'));
    await tester.pumpAndSettle();

    // The panel shows that player's read-only profile…
    expect(find.byType(PlayerProfileView), findsOneWidget);
    expect(find.byType(PlayHeatmap), findsOneWidget);
    expect(find.text('5 songs played'), findsOneWidget);
    // …and the board underneath is still mounted (we did not navigate away).
    expect(find.byType(CommunityScreen), findsOneWidget);
  });

  testWidgets('the panel is read-only — no owner controls', (tester) async {
    await tester.pumpWidget(_community(currentUserId: 'me'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('@ada'));
    await tester.pumpAndSettle();

    // The visibility toggle and the analytics consent are owner-only and must
    // never appear on someone else's profile.
    expect(find.byKey(const Key('profile-visibility')), findsNothing);
    expect(find.byKey(const Key('profile-usage-consent')), findsNothing);
  });

  testWidgets('closing the panel returns to the board', (tester) async {
    await tester.pumpWidget(_community(currentUserId: 'me'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('@ada'));
    await tester.pumpAndSettle();
    expect(find.byType(PlayerProfileView), findsOneWidget);

    await tester.tap(find.byKey(const Key('player-profile-panel-close')));
    await tester.pumpAndSettle();

    expect(find.byType(PlayerProfileView), findsNothing);
    expect(find.text('@ada'), findsOneWidget); // back on the board
  });

  testWidgets('your own pseudo does not open a panel', (tester) async {
    await tester.pumpWidget(_community(currentUserId: 'me'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('@me (you)'));
    await tester.pumpAndSettle();

    expect(find.byType(PlayerProfileView), findsNothing);
  });

  testWidgets(
    'an unavailable profile shows the friendly message, not a crash',
    (tester) async {
      final service = FakeGlobalLeaderboardService(
        boards: {
          FakeGlobalLeaderboardService.key(
            LeaderboardMode.tempo,
            '',
          ): GlobalLeaderboard(
            seasonId: '2026-07-30',
            entries: [globalEntry(rank: 1, userId: 'ada')],
            total: 1,
            own: null,
          ),
        },
      );
      await tester.pumpWidget(
        _scope([
          globalLeaderboardServiceProvider.overrideWithValue(service),
          currentUserIdProvider.overrideWithValue('me'),
          // Server fail-closed: the profile read errors.
          playerProfileProvider(
            'ada',
          ).overrideWith((ref) async => throw Exception('not found')),
        ], localizedApp(const CommunityScreen())),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('@ada'));
      await tester.pumpAndSettle();

      expect(find.text("This profile isn't available."), findsOneWidget);
      expect(find.textContaining('Exception'), findsNothing);
    },
  );
}
