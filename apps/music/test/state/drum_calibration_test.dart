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
import 'package:music/state/drum_calibration.dart';
import 'package:music/state/drum_kit.dart';

// The calibration pass as a pure state machine (change:
// add-drum-input-calibration, tasks 6.1–6.4).

const _pieces = [kKickPieceId, 'kitPieceSnare', 'kitPieceHiHat'];

/// A pass that has begun. `CalibrationState` starts **idle** — the surface
/// shows the stored table then, and a stroke must not silently record — so
/// every test here opens with `start`.
CalibrationState _fresh() =>
    const CalibrationState(pieces: _pieces).start(atMs: 0);

void main() {
  group('walking the pass', () {
    test('a pass that has not begun records nothing', () {
      // The table is on screen then, and a player checking an entry must be
      // able to play their kit without it being taken as an answer.
      const idle = CalibrationState(pieces: _pieces);
      expect(idle.outcome, CalibrationOutcome.idle);
      expect(idle.isRunning, isFalse);
      expect(idle.afterStroke(31, atMs: 100).recorded, isEmpty);
      expect(idle.skip(atMs: 100).index, 0);
    });

    test('it opens on the first piece with nothing recorded', () {
      final s = _fresh();
      expect(s.currentPiece, kKickPieceId);
      expect(s.recorded, isEmpty);
      expect(s.isRunning, isTrue);
      expect(s.step, 0);
      expect(s.total, 3);
    });

    test('a stroke is recorded against the piece being asked for', () {
      final s = _fresh().afterStroke(12, atMs: 100);
      expect(s.recorded, {kKickPieceId: 12});
      expect(s.currentPiece, 'kitPieceSnare');
    });

    test('answering the last step completes the pass', () {
      final s = _fresh()
          .afterStroke(12, atMs: 100)
          .afterStroke(31, atMs: 200)
          .afterStroke(22, atMs: 300);
      expect(s.outcome, CalibrationOutcome.completed);
      expect(s.currentPiece, isNull);
      expect(s.mapping.byPiece, {
        kKickPieceId: 12,
        'kitPieceSnare': 31,
        'kitPieceHiHat': 22,
      });
      // …and the mapping it built translates.
      expect(s.mapping.translate(31), 38);
    });

    test('a completed pass ignores everything that follows', () {
      final done = _fresh()
          .afterStroke(12, atMs: 100)
          .afterStroke(31, atMs: 200)
          .afterStroke(22, atMs: 300);
      expect(done.afterStroke(99, atMs: 400).recorded, done.recorded);
      expect(done.skip(atMs: 400).recorded, done.recorded);
      expect(done.abandon().outcome, CalibrationOutcome.completed);
    });
  });

  group('the stale-stroke rule (design D4)', () {
    test('a stroke stamped before the step armed is discarded', () {
      // The Wait-Mode lesson, restated: a timestamp only means something
      // relative to the moment that is asking. Here the player hit a pad while
      // the previous step was up; it must not answer this one.
      final s = _fresh().afterStroke(
        12,
        atMs: 500,
      ); // arms the next step at 500
      expect(s.currentPiece, 'kitPieceSnare');

      final stale = s.afterStroke(31, atMs: 400);
      expect(stale.recorded, {kKickPieceId: 12}, reason: 'nothing recorded');
      expect(stale.currentPiece, 'kitPieceSnare', reason: 'still asking');
    });

    test('a stroke stamped exactly at the arming instant is stale too', () {
      final s = _fresh().afterStroke(12, atMs: 500);
      expect(s.afterStroke(31, atMs: 500).recorded, {kKickPieceId: 12});
    });

    test('the next fresh stroke is taken', () {
      final s = _fresh()
          .afterStroke(12, atMs: 500)
          .afterStroke(31, atMs: 400) // stale, ignored
          .afterStroke(31, atMs: 600); // the one the player just played
      expect(s.recorded, {kKickPieceId: 12, 'kitPieceSnare': 31});
    });

    test('skipping re-arms, so a stroke played during the skipped step is '
        'not carried into the next one', () {
      final s = _fresh().skip(atMs: 500);
      expect(s.afterStroke(31, atMs: 450).recorded, isEmpty);
      expect(s.afterStroke(31, atMs: 550).recorded, {'kitPieceSnare': 31});
    });
  });

  group('conflicts (design D4)', () {
    CalibrationState withKickOn12() => _fresh().afterStroke(12, atMs: 100);

    test('a number another piece holds is reported, not recorded', () {
      final s = withKickOn12().afterStroke(12, atMs: 200);
      expect(
        s.conflict,
        const CalibrationConflict(number: 12, heldBy: kKickPieceId),
      );
      // The step has NOT advanced and nothing new was recorded.
      expect(s.currentPiece, 'kitPieceSnare');
      expect(s.recorded, {kKickPieceId: 12});
    });

    test('strike-again clears it and waits for a fresh stroke', () {
      final s = withKickOn12()
          .afterStroke(12, atMs: 200)
          .strikeAgain(atMs: 250);
      expect(s.conflict, isNull);
      expect(s.currentPiece, 'kitPieceSnare');
      // And the re-arm holds: a stroke from before the dismissal is stale.
      expect(s.afterStroke(31, atMs: 240).recorded, {kKickPieceId: 12});
      expect(s.afterStroke(31, atMs: 300).recorded, {
        kKickPieceId: 12,
        'kitPieceSnare': 31,
      });
    });

    test(
      'reassign moves the number and takes it from the piece that held it',
      () {
        final s = withKickOn12().afterStroke(12, atMs: 200).reassign(atMs: 250);
        // One number can never be claimed twice — the kick loses its entry.
        expect(s.recorded, {'kitPieceSnare': 12});
        expect(s.conflict, isNull);
        expect(s.currentPiece, 'kitPieceHiHat');
      },
    );

    test('re-striking the SAME piece is not a conflict with itself', () {
      // A player correcting their own answer for the step in front of them.
      final s = _fresh().afterStroke(12, atMs: 100).back(atMs: 150);
      expect(s.currentPiece, kKickPieceId);
      final again = s.afterStroke(12, atMs: 200);
      expect(again.conflict, isNull);
      expect(again.recorded, {kKickPieceId: 12});
    });
  });

  group('skipping, stepping back and abandoning', () {
    test('a skipped step records nothing and the pass continues', () {
      final s = _fresh().skip(atMs: 100);
      expect(s.recorded, isEmpty);
      expect(s.currentPiece, 'kitPieceSnare');
      expect(s.isRunning, isTrue);
    });

    test('skipping every step completes with an empty mapping', () {
      final s = _fresh().skip(atMs: 100).skip(atMs: 200).skip(atMs: 300);
      expect(s.outcome, CalibrationOutcome.completed);
      expect(s.mapping.isEmpty, isTrue);
    });

    test('back drops what that step had learned', () {
      final s = _fresh()
          .afterStroke(12, atMs: 100)
          .afterStroke(31, atMs: 200)
          .back(atMs: 250);
      expect(s.currentPiece, 'kitPieceSnare');
      // Its entry is gone, so re-striking it cannot collide with itself.
      expect(s.recorded, {kKickPieceId: 12});
    });

    test('back at the first step does nothing', () {
      final s = _fresh();
      expect(s.back(atMs: 100).index, 0);
    });

    test('abandoning ends the pass with nothing to store', () {
      final s = _fresh().afterStroke(12, atMs: 100).abandon();
      expect(s.outcome, CalibrationOutcome.abandoned);
      expect(s.isRunning, isFalse);
      // What it had learned is unreachable by design: the caller stores on
      // COMPLETION only, so the previous mapping stands.
      expect(s.afterStroke(31, atMs: 200).recorded, {kKickPieceId: 12});
    });
  });

  group('the pieces a pass offers (design D7)', () {
    test('the standard kit, round the kit as a drummer sits at it', () {
      // The fixed order, not the loaded score's: a mapping describes hardware,
      // and a groove with no toms must not leave a kit unable to map its toms.
      expect(kCalibrationPieceOrder.first, kKickPieceId);
      expect(kCalibrationPieceOrder[1], 'kitPieceSnare');
      expect(kCalibrationPieceOrder[2], 'kitPieceHiHat');
      expect(kCalibrationPieceOrder, contains('kitPieceRide'));
      expect(kCalibrationPieceOrder, contains('kitPieceCrash'));
      // No duplicates — a piece asked for twice would collide with itself.
      expect(
        kCalibrationPieceOrder.toSet().length,
        kCalibrationPieceOrder.length,
      );
    });

    test('a full pass over the standard kit completes', () {
      var s = CalibrationState(pieces: kCalibrationPieceOrder).start(atMs: 0);
      var at = 0;
      for (var i = 0; i < kCalibrationPieceOrder.length; i++) {
        s = s.afterStroke(20 + i, atMs: at += 100);
      }
      expect(s.outcome, CalibrationOutcome.completed);
      expect(s.recorded.length, kCalibrationPieceOrder.length);
    });
  });
}
