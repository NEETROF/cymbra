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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/session_summary.dart';
import 'package:music/theme/cymbra_theme.dart';
import 'package:music/widgets/mistake_replay.dart';

import '../support/fakes.dart';
import '../support/localized.dart';

NoteJudgment _j(
  int i,
  TimingVerdict v, {
  bool wrong = false,
  double s = 1,
  int startMs = 0,
}) => NoteJudgment(
  noteIndex: wrong ? -1 : i,
  pitch: 60 + i,
  startMs: startMs,
  waitMode: false,
  verdict: v,
  sustainRatio: s,
  wrong: wrong,
);

final _score = ReplayScore(
  notes: const [
    TimedNote(pitch: 60, startMs: 0, durationMs: 500),
    TimedNote(pitch: 62, startMs: 500, durationMs: 500),
    TimedNote(pitch: 64, startMs: 1000, durationMs: 500),
  ],
  bpm: 120,
  songEndMs: 1500,
  keyFifths: 0,
  beats: 4,
  beatType: 4,
  measureStartMs: const [0, 1000],
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

Future<void> _openReplay(WidgetTester tester, SessionResult r) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
      ],
      child: localizedApp(
        Scaffold(
          body: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () => showMistakeReplay(context, _score, r),
              child: const Text('go'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('go'));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}

void main() {
  group('mistake classification', () {
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

    test('missed and wrong use the error colour', () {
      expect(colorForMark(ReplayMark.missed), CymbraColors.error);
      expect(colorForMark(ReplayMark.wrong), CymbraColors.error);
      expect(colorForMark(ReplayMark.correct), isNot(CymbraColors.error));
    });
  });

  group('ReplayScore.measureOf', () {
    test('maps an onset time to its 1-based measure', () {
      expect(_score.measureOf(0), 1);
      expect(_score.measureOf(999), 1);
      expect(_score.measureOf(1000), 2);
      expect(_score.measureOf(1400), 2);
    });
  });

  group('StaffPainter mistake overlay', () {
    test('paints mistake rings without throwing', () {
      final recorder = PictureRecorder();
      StaffPainter(
        notes: _score.notes,
        elapsedMs: 0,
        activeNotes: const {},
        bpm: _score.bpm,
        songEndMs: _score.songEndMs,
        measureStartMs: _score.measureStartMs,
        mistakeColors: const {1: CymbraColors.error},
      ).paint(Canvas(recorder), const Size(400, 300));
      recorder.endRecording().dispose();
    });
  });

  group('replay dialog', () {
    testWidgets('lists mistakes by measure and closes', (tester) async {
      await _openReplay(
        tester,
        _result([
          _j(0, TimingVerdict.perfect, startMs: 0),
          _j(1, TimingVerdict.missed, startMs: 500),
          _j(2, TimingVerdict.late, startMs: 1000),
        ]),
      );

      expect(find.text('Replay'), findsOneWidget);
      // Two mistakes → two chips, one in measure 1, one in measure 2.
      expect(find.text('Missed'), findsOneWidget);
      expect(find.text('Mistimed'), findsOneWidget);
      expect(find.text('Measure 1'), findsWidgets);
      expect(find.text('Measure 2'), findsWidgets);

      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();
      expect(find.text('Replay'), findsNothing);
    });

    testWidgets('a clean run shows the no-mistakes message', (tester) async {
      await _openReplay(
        tester,
        _result([
          _j(0, TimingVerdict.perfect, startMs: 0),
          _j(1, TimingVerdict.good, startMs: 500),
        ]),
      );
      expect(find.text('No mistakes — nicely done'), findsOneWidget);
    });

    testWidgets('play then pause toggles the transport icon', (tester) async {
      await _openReplay(
        tester,
        _result([_j(0, TimingVerdict.missed, startMs: 0)]),
      );
      expect(find.byIcon(Icons.play_circle), findsOneWidget);
      await tester.tap(find.byIcon(Icons.play_circle));
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.byIcon(Icons.pause_circle), findsOneWidget);
      await tester.tap(find.byIcon(Icons.pause_circle));
      await tester.pump();
      expect(find.byIcon(Icons.play_circle), findsOneWidget);
    });

    testWidgets('playing scrubs through the score, sounds notes, and finishes', (
      tester,
    ) async {
      final audio = RecordingAudioService();
      await tester.pumpWidget(
        ProviderScope(
          overrides: [audioServiceProvider.overrideWithValue(audio)],
          child: localizedApp(
            Scaffold(
              body: Builder(
                builder: (context) => ElevatedButton(
                  onPressed: () => showMistakeReplay(
                    context,
                    _score,
                    _result([
                      _j(0, TimingVerdict.perfect, startMs: 0),
                      _j(1, TimingVerdict.missed, startMs: 500),
                      _j(2, TimingVerdict.late, startMs: 1000),
                    ]),
                  ),
                  child: const Text('go'),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('go'));
      await tester.pump();

      await tester.tap(find.byIcon(Icons.play_circle));
      // Advance in small steps (dt < the 100ms skip guard) past songEnd (1500ms).
      for (var i = 0; i < 40; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      // The player's own notes were sounded (not the score): the played
      // pitches (60 perfect, 62 late) sound; the missed pitch (61) does not.
      final pitches = audio.noteOns.map((e) => e.pitch).toSet();
      expect(pitches, contains(60));
      expect(pitches, contains(62));
      expect(pitches.contains(61), isFalse);
      // Reaching the end auto-pauses (back to the play icon).
      expect(find.byIcon(Icons.play_circle), findsOneWidget);
    });

    testWidgets('a mistimed note shows its direction and offset', (
      tester,
    ) async {
      await _openReplay(
        tester,
        _result(const [
          NoteJudgment(
            noteIndex: 0,
            pitch: 60,
            startMs: 0,
            waitMode: false,
            verdict: TimingVerdict.late,
            timingOffsetMs: 85,
            sustainRatio: 1,
          ),
          NoteJudgment(
            noteIndex: 1,
            pitch: 62,
            startMs: 500,
            waitMode: false,
            verdict: TimingVerdict.early,
            timingOffsetMs: -40,
            sustainRatio: 1,
          ),
        ]),
      );
      expect(find.text('Late 85 ms'), findsOneWidget);
      expect(find.text('Early 40 ms'), findsOneWidget);
    });

    testWidgets('the mistake bar scrolls along with the staff', (tester) async {
      // A long run with enough mistakes to overflow the bar horizontally.
      final judgments = [
        for (var i = 0; i < 12; i++)
          _j(i, TimingVerdict.missed, startMs: i * 500),
      ];
      final long = ReplayScore(
        notes: [
          for (var i = 0; i < 12; i++)
            TimedNote(pitch: 60 + i, startMs: i * 500, durationMs: 500),
        ],
        bpm: 120,
        songEndMs: 6000,
        keyFifths: 0,
        beats: 4,
        beatType: 4,
        measureStartMs: const [0, 2000, 4000],
      );
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            audioServiceProvider.overrideWithValue(RecordingAudioService()),
          ],
          child: localizedApp(
            Scaffold(
              body: Builder(
                builder: (context) => ElevatedButton(
                  onPressed: () =>
                      showMistakeReplay(context, long, _result(judgments)),
                  child: const Text('go'),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('go'));
      await tester.pump();

      final list = find.byKey(const ValueKey('replay-mistake-list'));
      double offset() => tester.widget<ListView>(list).controller!.offset;
      expect(offset(), 0);

      await tester.tap(find.byIcon(Icons.play_circle));
      for (var i = 0; i < 60; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      // The playhead has walked into later mistakes, so the bar followed it.
      expect(offset(), greaterThan(0));

      // Jumping back to the top of the run brings the bar back with it.
      await tester.tap(find.byIcon(Icons.pause_circle));
      await tester.pump();
      await tester.drag(find.byType(Slider), const Offset(-1000, 0));
      for (var i = 0; i < 20; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(offset(), 0);
    });

    testWidgets('tapping a mistake jumps to it without error', (tester) async {
      await _openReplay(
        tester,
        _result([
          _j(0, TimingVerdict.perfect, startMs: 0),
          _j(1, TimingVerdict.missed, startMs: 500),
        ]),
      );
      await tester.tap(find.text('Missed'));
      await tester.pump();
      expect(find.text('Replay'), findsOneWidget);
    });
  });
}
