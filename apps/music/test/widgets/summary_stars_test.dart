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

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/leaderboard_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/leaderboard.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/piece_bests.dart';
import 'package:music/state/piece_stars_core.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/widgets/session_summary_modal.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';

class _EmptyLeaderboardService implements LeaderboardService {
  @override
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  }) async => Leaderboard.empty;

  @override
  Future<Map<String, LeaderboardStanding>> getMyStandings(
    List<String> scoreIds,
  ) async => const {};
}

SessionResult _run(double pct) {
  const n = 20;
  final good = (n * pct / 100).round();
  return SessionResult.fromJudgments(
    pieceId: 'p',
    title: 'Twinkle',
    hands: 'right',
    judgments: [
      for (var i = 0; i < n; i++)
        NoteJudgment(
          noteIndex: i,
          pitch: 60 + (i % 8),
          startMs: i * 500,
          waitMode: false,
          verdict: i < good ? TimingVerdict.perfect : TimingVerdict.missed,
          timingOffsetMs: i < good ? 0 : null,
          reactionMs: null,
          sustainRatio: i < good ? 1 : 0,
        ),
    ],
    bestCombo: good,
    playedAtMs: 0,
    speed: 0.6,
  );
}

Future<ProviderContainer> _open(
  WidgetTester tester,
  SessionResult r, {
  Map<String, String> prefs = const {},
}) async {
  final c = ProviderContainer(
    overrides: [
      leaderboardServiceProvider.overrideWithValue(_EmptyLeaderboardService()),
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
    ],
  );
  addTearDown(c.dispose);
  // Let the local bests restore before the summary reads them.
  c.read(localPieceBestsProvider);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        Scaffold(
          body: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () => showSessionSummary(context, r),
              child: const Text('go'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  await tester.tap(find.text('go'));
  await tester.pumpAndSettle();
  return c;
}

int _filled(WidgetTester tester) => tester
    .widgetList<Icon>(
      find.descendant(
        of: find.byKey(const Key('summary-stars')),
        matching: find.byType(Icon),
      ),
    )
    .where((i) => i.icon == Icons.star_rounded)
    .length;

void main() {
  group('summary stars (change: add-music-piece-stars)', () {
    testWidgets('a first run shows its stars and celebrates them', (
      tester,
    ) async {
      final run = _run(82);
      final c = await _open(tester, run);
      final stars = starsFor(run.overallSyncPct);
      expect(stars, 2);
      expect(_filled(tester), 2);
      expect(find.byKey(const Key('summary-new-star')), findsOneWidget);
      expect(find.text('NEW STAR!'), findsOneWidget);
      expect(find.text('2 stars on your first run'), findsOneWidget);
      // The run became the device's best for the piece.
      expect(
        c.read(localPieceBestsProvider.notifier).bestOf('p'),
        run.overallSyncPct,
      );
    });

    testWidgets('a run above the best adds a star and names the jump', (
      tester,
    ) async {
      await _open(
        tester,
        _run(96),
        prefs: {
          'piece.bests.v1': jsonEncode({'p': 88}),
        },
      );
      expect(_filled(tester), 3);
      expect(find.text('2 → 3 stars · previous best 88 %'), findsOneWidget);
    });

    testWidgets('matching the stars already held celebrates nothing', (
      tester,
    ) async {
      await _open(
        tester,
        _run(83),
        prefs: {
          'piece.bests.v1': jsonEncode({'p': 88}),
        },
      );
      expect(_filled(tester), 2);
      expect(find.byKey(const Key('summary-new-star')), findsNothing);
    });

    testWidgets('a lower run shows its own stars and leaves the best alone', (
      tester,
    ) async {
      final c = await _open(
        tester,
        _run(65),
        prefs: {
          'piece.bests.v1': jsonEncode({'p': 88}),
        },
      );
      expect(_filled(tester), 1, reason: 'this run reached one star');
      expect(find.byKey(const Key('summary-new-star')), findsNothing);
      expect(c.read(localPieceBestsProvider.notifier).bestOf('p'), 88);
    });
  });
}
