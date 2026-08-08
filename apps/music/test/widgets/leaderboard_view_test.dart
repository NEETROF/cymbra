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
import 'package:music/services/leaderboard_service.dart';
import 'package:music/services/profile_service.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/play_activity.dart';
import 'package:music/state/play_activity_notifier.dart';
import 'package:music/state/profile_notifier.dart';
import 'package:music/state/session_notifier.dart';
import 'package:music/widgets/leaderboard_view.dart';
import 'package:music/widgets/player_profile_panel.dart';

import '../support/localized.dart';

/// Hand fake for the leaderboard read seam: returns a preset board per mode, so a
/// widget test drives the view without gRPC (a documented special case — the
/// convention's mockito default fits behaviour verification, not a fixed table).
class _FakeLeaderboardService implements LeaderboardService {
  _FakeLeaderboardService(this.boards, {this.fail = false});

  final Map<LeaderboardMode, Leaderboard> boards;
  final bool fail;

  @override
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  }) async {
    if (fail) throw Exception('boom');
    return boards[mode] ?? Leaderboard.empty;
  }

  @override
  Future<Map<String, LeaderboardStanding>> getMyStandings(
    List<String> scoreIds,
  ) async => const {};
}

LeaderboardEntry _entry(
  String userId, {
  required int rank,
  String? handle,
  double subscore = 90,
}) => LeaderboardEntry(
  rank: rank,
  userId: userId,
  handle: handle,
  displayName: null,
  subscore: subscore,
  tiebreakMetric: 10,
  achievedAtMs: 1,
);

Future<void> _pump(WidgetTester tester, _FakeLeaderboardService fake) async {
  // A ROOT ProviderContainer (not a nested ProviderScope) so overriding the
  // keepAlive service provider doesn't trip `scoped_providers_should_specify_
  // dependencies` — the repo's widget-test override convention.
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: ProviderContainer(
        overrides: [leaderboardServiceProvider.overrideWithValue(fake)],
      ),
      child: localizedApp(
        const Scaffold(
          body: LeaderboardView(
            scoreId: 'piece-1',
            title: 'Etude',
            initialMode: LeaderboardMode.tempo,
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('shows ranked public entries with rank + score', (tester) async {
    final fake = _FakeLeaderboardService({
      LeaderboardMode.tempo: Leaderboard(
        entries: [
          _entry('a1', rank: 1, handle: '@ana', subscore: 95),
          _entry('a2', rank: 2, handle: '@bo', subscore: 80),
        ],
        total: 2,
        own: null,
      ),
    });
    await _pump(tester, fake);

    expect(find.text('#1'), findsOneWidget);
    expect(find.text('#2'), findsOneWidget);
    expect(find.text('@ana'), findsOneWidget);
    expect(find.text('95%'), findsOneWidget);
    expect(find.text('80%'), findsOneWidget);
  });

  testWidgets('tempo/reaction toggle switches the shown board', (tester) async {
    final fake = _FakeLeaderboardService({
      LeaderboardMode.tempo: Leaderboard(
        entries: [_entry('a1', rank: 1, handle: '@tempo', subscore: 95)],
        total: 1,
        own: null,
      ),
      LeaderboardMode.reaction: Leaderboard(
        entries: [_entry('r1', rank: 1, handle: '@react', subscore: 77)],
        total: 1,
        own: null,
      ),
    });
    await _pump(tester, fake);
    expect(find.text('@tempo'), findsOneWidget);
    expect(find.text('@react'), findsNothing);

    // Toggle to the reaction board.
    await tester.tap(find.text('Wait Mode'));
    await tester.pumpAndSettle();
    expect(find.text('@react'), findsOneWidget);
    expect(find.text('@tempo'), findsNothing);
  });

  testWidgets('private viewer sees own rank + PB but is not listed', (
    tester,
  ) async {
    final fake = _FakeLeaderboardService({
      LeaderboardMode.tempo: Leaderboard(
        entries: [
          _entry('a1', rank: 1, handle: '@ana', subscore: 95),
          _entry('a2', rank: 2, handle: '@bo', subscore: 70),
        ],
        total: 2,
        // The viewer (private) is not among the listed entries.
        own: _entry('me', rank: 2, subscore: 88),
      ),
    });
    await _pump(tester, fake);

    // Own standing is shown (rank + best).
    expect(find.textContaining('#2'), findsWidgets);
    expect(find.textContaining('88%'), findsOneWidget);
    // The private viewer is not in the public listing.
    expect(find.text('#3'), findsNothing);
  });

  testWidgets('own entry among the list is highlighted with (you)', (
    tester,
  ) async {
    final fake = _FakeLeaderboardService({
      LeaderboardMode.tempo: Leaderboard(
        entries: [
          _entry('a1', rank: 1, handle: '@ana', subscore: 95),
          _entry('me', rank: 2, handle: '@me', subscore: 80),
        ],
        total: 2,
        own: _entry('me', rank: 2, handle: '@me', subscore: 80),
      ),
    });
    await _pump(tester, fake);
    expect(find.textContaining('(you)'), findsOneWidget);
  });

  testWidgets('empty board shows the empty state', (tester) async {
    final fake = _FakeLeaderboardService({
      LeaderboardMode.tempo: Leaderboard.empty,
    });
    await _pump(tester, fake);
    expect(find.textContaining('No entries yet'), findsOneWidget);
  });

  testWidgets('error state offers a retry', (tester) async {
    final fake = _FakeLeaderboardService({}, fail: true);
    await _pump(tester, fake);
    expect(find.text('Retry'), findsOneWidget);
  });

  testWidgets('a long entries list scrolls to reveal the last player', (
    tester,
  ) async {
    // A full page of entries — taller than the view, so the list must scroll.
    final entries = [
      for (var i = 1; i <= 50; i++)
        _entry('u$i', rank: i, handle: '@u$i', subscore: (100 - i).toDouble()),
    ];
    final fake = _FakeLeaderboardService({
      LeaderboardMode.tempo: Leaderboard(
        entries: entries,
        total: 50,
        own: null,
      ),
    });
    await _pump(tester, fake);

    // The top is visible; the last row isn't built yet (lazy, off-screen).
    expect(find.text('#1'), findsOneWidget);
    expect(find.text('#50'), findsNothing);

    // Scrolling the list reaches the last player — proves it is scrollable.
    await tester.scrollUntilVisible(
      find.text('#50'),
      300,
      scrollable: find.byType(Scrollable).last,
    );
    expect(find.text('#50'), findsOneWidget);
  });

  group('tapping a pseudo opens the profile panel', () {
    /// Pumps the board the way it really ships from a score card: inside a
    /// `Dialog`, which has NO Scaffold — the case a Scaffold `endDrawer` could
    /// not serve, and the reason the panel is a right-anchored route.
    Future<void> pumpInDialog(
      WidgetTester tester,
      _FakeLeaderboardService fake, {
      String? currentUserId = 'me',
    }) async {
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: ProviderContainer(
            overrides: [
              leaderboardServiceProvider.overrideWithValue(fake),
              currentUserIdProvider.overrideWithValue(currentUserId),
              playerProfileProvider('a1').overrideWith(
                (ref) async => const PlayerProfile(
                  userId: 'a1',
                  handle: 'ana',
                  displayName: null,
                  visibility: 'public',
                ),
              ),
              playActivityProvider('a1').overrideWith(
                (ref) async => PlayActivity(days: const [], totalSessions: 7),
              ),
            ],
          ),
          child: localizedApp(
            const Dialog(
              child: SizedBox(
                width: 400,
                height: 600,
                child: LeaderboardView(scoreId: 'piece-1', title: ''),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
    }

    testWidgets('from inside the leaderboard dialog, without navigating', (
      tester,
    ) async {
      final fake = _FakeLeaderboardService({
        LeaderboardMode.tempo: Leaderboard(
          entries: [_entry('a1', rank: 1, handle: '@ana')],
          total: 1,
          own: null,
        ),
      });
      await pumpInDialog(tester, fake);

      await tester.tap(find.text('@ana'));
      await tester.pumpAndSettle();

      expect(find.byType(PlayerProfileView), findsOneWidget);
      expect(find.text('7 songs played'), findsOneWidget);
      // The board is still mounted underneath — no navigation happened.
      expect(find.byType(LeaderboardView), findsOneWidget);
    });

    testWidgets('but your own row is not tappable', (tester) async {
      final fake = _FakeLeaderboardService({
        LeaderboardMode.tempo: Leaderboard(
          entries: [_entry('me', rank: 1, handle: '@me')],
          total: 1,
          own: _entry('me', rank: 1, handle: '@me'),
        ),
      });
      await pumpInDialog(tester, fake);

      await tester.tap(find.text('@me (you)'));
      await tester.pumpAndSettle();
      expect(find.byType(PlayerProfileView), findsNothing);

      // Nor is the "your standing" card above the list.
      await tester.tap(find.text('Your standing'));
      await tester.pumpAndSettle();
      expect(find.byType(PlayerProfileView), findsNothing);
    });
  });
}
