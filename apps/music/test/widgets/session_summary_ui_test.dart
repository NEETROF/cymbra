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
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/widgets/mistake_replay.dart';
import 'package:music/widgets/session_summary_modal.dart';

import '../support/localized.dart';

/// The summary now surfaces a post-session leaderboard standing (change: add-
/// play-leaderboards), so it reads the leaderboard seam through Riverpod. An
/// empty board keeps these UI tests focused on the summary itself (no standing
/// rows) while satisfying the required ProviderScope.
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

/// Wraps [home] in the app localizations + a ROOT ProviderContainer with the
/// empty leaderboard seam, so the summary modal's standing section can build.
/// A root container (not a nested ProviderScope) keeps the keepAlive override
/// off `scoped_providers_should_specify_dependencies` — the repo convention.
Widget _scoped(Widget home) => UncontrolledProviderScope(
  container: ProviderContainer(
    overrides: [
      leaderboardServiceProvider.overrideWithValue(_EmptyLeaderboardService()),
    ],
  ),
  child: localizedApp(home),
);

NoteJudgment _onset(
  int i, {
  required bool wait,
  required TimingVerdict v,
  double sustain = 1,
}) => NoteJudgment(
  noteIndex: i,
  pitch: 60 + i,
  startMs: i * 500,
  waitMode: wait,
  verdict: v,
  timingOffsetMs: wait ? null : 0,
  reactionMs: wait ? 50 : null,
  sustainRatio: v == TimingVerdict.missed ? 0 : sustain,
);

SessionResult _mixed() => SessionResult.fromJudgments(
  pieceId: 'p',
  title: 'Sonata',
  hands: 'both',
  judgments: [
    _onset(0, wait: false, v: TimingVerdict.perfect),
    _onset(1, wait: true, v: TimingVerdict.good),
  ],
  bestCombo: 2,
  playedAtMs: 0,
  speed: 1,
);

SessionResult _pureFree() => SessionResult.fromJudgments(
  pieceId: 'p',
  title: 'Etude',
  hands: 'right',
  judgments: [_onset(0, wait: false, v: TimingVerdict.perfect)],
  bestCombo: 1,
  playedAtMs: 0,
  speed: 1,
);

Future<void> _open(WidgetTester tester, SessionResult r) async {
  await tester.pumpWidget(
    _scoped(
      Scaffold(
        body: Builder(
          builder: (context) => ElevatedButton(
            onPressed: () => showSessionSummary(context, r),
            child: const Text('go'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('go'));
  await tester.pumpAndSettle();
}

void main() {
  group('summary modal', () {
    testWidgets('a mixed run shows both tempo and reaction sub-scores', (
      tester,
    ) async {
      await _open(tester, _mixed());
      expect(find.text('Tempo'), findsOneWidget);
      expect(find.text('Reaction'), findsOneWidget);
      expect(find.text('Replay mistakes'), findsOneWidget);
      // Explicit choices: replay, practice a section, retry, or the close cross
      // (quit). No silent dismiss.
      expect(find.text('Practice a section'), findsOneWidget);
      expect(find.text('Retry'), findsOneWidget);
      expect(find.byIcon(Icons.close), findsOneWidget);
      // Each sub-score shows an average-timing tendency line.
      expect(find.textContaining('avg'), findsWidgets);
    });

    testWidgets('a pure free run shows only the tempo sub-score', (
      tester,
    ) async {
      await _open(tester, _pureFree());
      expect(find.text('Tempo'), findsOneWidget);
      expect(find.text('Reaction'), findsNothing);
    });

    testWidgets(
      'on a short viewport the actions stay reachable and the X quits',
      (tester) async {
        tester.view.physicalSize = const Size(1200, 480); // phone landscape
        tester.view.devicePixelRatio = 1.0;
        addTearDown(tester.view.reset);

        SummaryAction? action;
        await tester.pumpWidget(
          _scoped(
            Scaffold(
              body: Builder(
                builder: (context) => ElevatedButton(
                  onPressed: () async =>
                      action = await showSessionSummary(context, _mixed()),
                  child: const Text('go'),
                ),
              ),
            ),
          ),
        );
        await tester.tap(find.text('go'));
        await tester.pumpAndSettle();

        // The stats scroll; the buttons and the close cross stay pinned/visible.
        expect(find.text('Replay mistakes'), findsOneWidget);
        expect(find.text('Practice a section'), findsOneWidget);
        expect(find.text('Retry'), findsOneWidget);
        expect(find.byIcon(Icons.close), findsOneWidget);

        await tester.tap(find.byIcon(Icons.close));
        await tester.pumpAndSettle();
        expect(action, SummaryAction.close);
      },
    );

    testWidgets('practice a section returns the practice action', (
      tester,
    ) async {
      SummaryAction? action;
      await tester.pumpWidget(
        _scoped(
          Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () async =>
                    action = await showSessionSummary(context, _pureFree()),
                child: const Text('go'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('go'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('summary-practice')));
      await tester.pumpAndSettle();
      expect(action, SummaryAction.practice);
    });

    testWidgets('the close cross returns close and dismisses the modal', (
      tester,
    ) async {
      SummaryAction? action;
      await tester.pumpWidget(
        _scoped(
          Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () async =>
                    action = await showSessionSummary(context, _pureFree()),
                child: const Text('go'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('go'));
      await tester.pumpAndSettle();
      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();
      expect(action, SummaryAction.close);
      expect(find.byIcon(Icons.close), findsNothing);
    });
  });

  group('replay mistake classification', () {
    test('correct notes are not flagged; mistakes are', () {
      NoteJudgment j(TimingVerdict v, {bool wrong = false, double s = 1}) =>
          NoteJudgment(
            noteIndex: 0,
            pitch: 60,
            startMs: 0,
            waitMode: false,
            verdict: v,
            sustainRatio: s,
            wrong: wrong,
          );
      expect(markFor(j(TimingVerdict.perfect)), ReplayMark.correct);
      expect(markFor(j(TimingVerdict.good)), ReplayMark.correct);
      expect(markFor(j(TimingVerdict.missed)), ReplayMark.missed);
      expect(markFor(j(TimingVerdict.late)), ReplayMark.mistimed);
      expect(
        markFor(j(TimingVerdict.perfect, s: 0.2)),
        ReplayMark.shortSustain,
      );
      expect(markFor(j(TimingVerdict.missed, wrong: true)), ReplayMark.wrong);
    });
  });
}
