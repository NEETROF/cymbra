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
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/widgets/mistake_replay.dart';
import 'package:music/widgets/session_summary_modal.dart';

import '../support/localized.dart';

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
    localizedApp(
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
    });

    testWidgets('a pure free run shows only the tempo sub-score', (
      tester,
    ) async {
      await _open(tester, _pureFree());
      expect(find.text('Tempo'), findsOneWidget);
      expect(find.text('Reaction'), findsNothing);
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
