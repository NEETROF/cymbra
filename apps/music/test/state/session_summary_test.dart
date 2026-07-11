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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/session_summary.dart';

NoteJudgment _onset({
  required int index,
  required bool waitMode,
  required TimingVerdict verdict,
  double sustain = 1.0,
}) => NoteJudgment(
  noteIndex: index,
  pitch: 60 + index,
  startMs: index * 500,
  waitMode: waitMode,
  verdict: verdict,
  timingOffsetMs: waitMode ? null : 0,
  reactionMs: waitMode ? 50 : null,
  sustainRatio: verdict == TimingVerdict.missed ? 0 : sustain,
);

NoteJudgment _wrong({required bool waitMode}) => NoteJudgment(
  noteIndex: -1,
  pitch: 61,
  startMs: 100,
  waitMode: waitMode,
  verdict: TimingVerdict.missed,
  wrong: true,
);

void main() {
  group('SessionResult.fromJudgments derivation', () {
    test('pure free run has only a free sub-score', () {
      final r = SessionResult.fromJudgments(
        pieceId: 'p1',
        title: 'Piece',
        hands: 'both',
        judgments: [
          _onset(index: 0, waitMode: false, verdict: TimingVerdict.perfect),
          _onset(index: 1, waitMode: false, verdict: TimingVerdict.good),
        ],
        bestCombo: 2,
        playedAtMs: 1000,
        speed: 1.0,
      );
      expect(r.runMode, RunMode.free);
      expect(r.freeSyncPct, isNotNull);
      expect(r.waitSyncPct, isNull);
      expect(r.freeOnsetCount, 2);
      expect(r.waitOnsetCount, 0);
      expect(r.verdictCounts[TimingVerdict.perfect], 1);
      expect(r.verdictCounts[TimingVerdict.good], 1);
    });

    test('pure wait run has only a wait sub-score', () {
      final r = SessionResult.fromJudgments(
        pieceId: 'p1',
        title: 'Piece',
        hands: 'both',
        judgments: [
          _onset(index: 0, waitMode: true, verdict: TimingVerdict.perfect),
        ],
        bestCombo: 1,
        playedAtMs: 0,
        speed: 1.0,
      );
      expect(r.runMode, RunMode.wait);
      expect(r.freeSyncPct, isNull);
      expect(r.waitSyncPct, isNotNull);
    });

    test('mixed run yields both sub-scores', () {
      final r = SessionResult.fromJudgments(
        pieceId: 'p1',
        title: 'Piece',
        hands: 'right',
        judgments: [
          _onset(index: 0, waitMode: false, verdict: TimingVerdict.perfect),
          _onset(index: 1, waitMode: true, verdict: TimingVerdict.good),
          _wrong(waitMode: false),
        ],
        bestCombo: 1,
        playedAtMs: 0,
        speed: 0.75,
      );
      expect(r.runMode, RunMode.mixed);
      expect(r.freeSyncPct, isNotNull);
      expect(r.waitSyncPct, isNotNull);
      expect(r.wrongNotes, 1);
      // The wrong note lowers the free sub-score below the wait sub-score.
      expect(r.freeSyncPct!, lessThan(r.waitSyncPct!));
    });

    test('overall percentage is defined and in range', () {
      final r = SessionResult.fromJudgments(
        pieceId: 'p1',
        title: 'Piece',
        hands: 'both',
        judgments: const [],
        bestCombo: 0,
        playedAtMs: 0,
        speed: 1.0,
      );
      expect(r.overallSyncPct, 100.0);
      expect(r.runMode, RunMode.free);
    });
  });

  group('JSON round-trip', () {
    test('preserves all fields including per-mode sub-scores', () {
      final r = SessionResult.fromJudgments(
        pieceId: 'sonata',
        title: 'Sonata',
        hands: 'both',
        judgments: [
          _onset(index: 0, waitMode: false, verdict: TimingVerdict.perfect),
          _onset(index: 1, waitMode: true, verdict: TimingVerdict.late,
              sustain: 0.5),
          _wrong(waitMode: true),
        ],
        bestCombo: 3,
        playedAtMs: 123456,
        speed: 1.25,
      );

      final restored = SessionResult.fromJson(
        // Force a plain map (as if decoded from storage).
        Map<String, dynamic>.from(r.toJson()),
      );

      expect(restored.pieceId, r.pieceId);
      expect(restored.title, r.title);
      expect(restored.hands, r.hands);
      expect(restored.runMode, r.runMode);
      expect(restored.overallSyncPct, closeTo(r.overallSyncPct, 1e-9));
      expect(restored.freeSyncPct, closeTo(r.freeSyncPct!, 1e-9));
      expect(restored.waitSyncPct, closeTo(r.waitSyncPct!, 1e-9));
      expect(restored.freeOnsetCount, r.freeOnsetCount);
      expect(restored.waitOnsetCount, r.waitOnsetCount);
      expect(restored.wrongNotes, r.wrongNotes);
      expect(restored.bestCombo, r.bestCombo);
      expect(restored.playedAtMs, r.playedAtMs);
      expect(restored.speed, r.speed);
      expect(restored.verdictCounts, r.verdictCounts);
      expect(restored.notes.length, r.notes.length);
      expect(restored.notes.last.wrong, isTrue);
      expect(restored.notes.first.verdict, TimingVerdict.perfect);
    });

    test('absent sub-score stays null through JSON', () {
      final r = SessionResult.fromJudgments(
        pieceId: 'p',
        title: 'T',
        hands: 'left',
        judgments: [
          _onset(index: 0, waitMode: false, verdict: TimingVerdict.good),
        ],
        bestCombo: 1,
        playedAtMs: 0,
        speed: 1.0,
      );
      final restored = SessionResult.fromJson(r.toJson());
      expect(restored.waitSyncPct, isNull);
      expect(restored.freeSyncPct, isNotNull);
    });
  });
}
