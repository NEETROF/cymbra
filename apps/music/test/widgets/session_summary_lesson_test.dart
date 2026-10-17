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
import 'package:music/state/leaderboard.dart';
import 'package:music/state/lesson_context.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/widgets/session_summary_modal.dart';

import '../support/localized.dart';

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
  // Enough perfect/missed onsets to land near [pct] overall.
  const n = 20;
  final good = (n * pct / 100).round();
  return SessionResult.fromJudgments(
    pieceId: 'p',
    title: 'Au clair de la lune',
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

LessonContext _lesson(int passPct) => LessonContext(
  lessonTitle: 'Fa et sol',
  hands: Hand.right,
  speed: 0.6,
  waitMode: true,
  metronome: false,
  readingAid: NoteReadingAid.name,
  passPct: passPct,
  onResult: (_) {},
);

Future<SummaryAction?> _open(
  WidgetTester tester,
  SessionResult r, {
  LessonContext? lesson,
}) async {
  SummaryAction? chosen;
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: ProviderContainer(
        overrides: [
          leaderboardServiceProvider.overrideWithValue(
            _EmptyLeaderboardService(),
          ),
        ],
      ),
      child: localizedApp(
        Scaffold(
          body: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () async =>
                  chosen = await showSessionSummary(context, r, lesson: lesson),
              child: const Text('go'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('go'));
  await tester.pumpAndSettle();
  return chosen;
}

void main() {
  // change: add-music-piece-lessons
  testWidgets('in a lesson run the goal is named as met and the primary '
      'action leads back to the lesson', (tester) async {
    await _open(tester, _run(90), lesson: _lesson(80));
    expect(find.byKey(const Key('summary-goal')), findsOneWidget);
    expect(find.text('Goal 80 % met'), findsOneWidget);
    final back = find.byKey(const Key('summary-back-to-lesson'));
    expect(back, findsOneWidget);
    // Mistakes and retry are still there, as secondary actions.
    expect(find.text('Retry'), findsOneWidget);
  });

  testWidgets('a missed goal is stated plainly, never celebrated', (
    tester,
  ) async {
    await _open(tester, _run(60), lesson: _lesson(80));
    expect(find.text('Goal 80 % · not yet'), findsOneWidget);
  });

  testWidgets('choosing back-to-lesson returns that action', (tester) async {
    SummaryAction? chosen;
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: ProviderContainer(
          overrides: [
            leaderboardServiceProvider.overrideWithValue(
              _EmptyLeaderboardService(),
            ),
          ],
        ),
        child: localizedApp(
          Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () async => chosen = await showSessionSummary(
                  context,
                  _run(90),
                  lesson: _lesson(80),
                ),
                child: const Text('go'),
              ),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('go'));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('summary-back-to-lesson')));
    await tester.pumpAndSettle();
    expect(chosen, SummaryAction.backToLesson);
  });

  testWidgets('outside a lesson nothing changes: no goal, no back action', (
    tester,
  ) async {
    await _open(tester, _run(90));
    expect(find.byKey(const Key('summary-goal')), findsNothing);
    expect(find.byKey(const Key('summary-back-to-lesson')), findsNothing);
  });
}
