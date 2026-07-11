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

import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/theme/cymbra_theme.dart';
import 'package:music/theme/scoring_style.dart';
import 'package:music/widgets/mistake_replay.dart';

import '../support/localized.dart';

NoteJudgment _j(int i, TimingVerdict v, {bool wrong = false, double s = 1}) =>
    NoteJudgment(
      noteIndex: wrong ? -1 : i,
      pitch: 60 + i,
      startMs: i * 400,
      waitMode: false,
      verdict: v,
      sustainRatio: s,
      wrong: wrong,
    );

SessionResult _result(List<NoteJudgment> js) => SessionResult.fromJudgments(
  pieceId: 'p',
  title: 'Replay Piece',
  hands: 'both',
  judgments: js,
  bestCombo: 2,
  playedAtMs: 0,
  speed: 1,
);

void paintOnce(CustomPainter painter, Size size) {
  final recorder = PictureRecorder();
  painter.paint(Canvas(recorder), size);
  recorder.endRecording().dispose();
}

void main() {
  group('MistakeReplayPainter', () {
    test('paints every mark kind without throwing', () {
      const painter = MistakeReplayPainter(judgments: [], progress: 0);
      paintOnce(painter, const Size(320, 200));

      final full = MistakeReplayPainter(
        judgments: [
          _j(0, TimingVerdict.perfect),
          _j(1, TimingVerdict.good),
          _j(2, TimingVerdict.late),
          _j(3, TimingVerdict.perfect, s: 0.1),
          _j(4, TimingVerdict.missed),
          _j(5, TimingVerdict.missed, wrong: true),
        ],
        progress: 0.5,
      );
      paintOnce(full, const Size(320, 200));
    });

    test('shouldRepaint tracks progress and judgments', () {
      const a = MistakeReplayPainter(judgments: [], progress: 0);
      const b = MistakeReplayPainter(judgments: [], progress: 0.2);
      expect(a.shouldRepaint(b), isTrue);
      expect(a.shouldRepaint(a), isFalse);
    });
  });

  group('ScoringTierStyle / verdictColor', () {
    test('each tier maps to a distinct accent colour', () {
      final colors = {for (var t = 0; t <= 4; t++) t.tierColor};
      expect(colors.length, 5);
      // Out-of-range clamps into the valid band.
      expect(9.tierColor, 4.tierColor);
    });

    test('verdict colours: miss uses error, hits do not', () {
      expect(
        verdictColor(TimingVerdict.missed, fallback: CymbraColors.secondary),
        CymbraColors.error,
      );
      expect(
        verdictColor(TimingVerdict.perfect, fallback: CymbraColors.secondary),
        isNot(CymbraColors.error),
      );
    });
  });

  testWidgets('replay dialog shows the legend and closes', (tester) async {
    await tester.pumpWidget(
      localizedApp(
        Scaffold(
          body: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () => showMistakeReplay(
                context,
                _result([
                  _j(0, TimingVerdict.perfect),
                  _j(1, TimingVerdict.missed),
                ]),
              ),
              child: const Text('go'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('go'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));

    expect(find.text('Missed'), findsOneWidget);
    expect(find.text('Mistimed'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.close));
    await tester.pumpAndSettle();
    expect(find.text('Missed'), findsNothing);
  });
}
