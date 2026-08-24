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
import 'package:music/state/drum_kit.dart';
import 'package:music/state/player_data.dart';

// Stroke identity (change: add-drum-scoring): what satisfies a written drum
// note, decided ONCE at the kit piece's grain and keyed to the STATIC
// named-piece table — never to the score-derived lane layout.

TimedNote n(int gm) => TimedNote(pitch: gm, startMs: 0, durationMs: 100);

void main() {
  group('the named groups inter-satisfy', () {
    test('snare {37, 38, 40} is one piece', () {
      for (final written in const [37, 38, 40]) {
        for (final incoming in const [37, 38, 40]) {
          expect(
            samePiece(written, incoming),
            isTrue,
            reason: 'snare $incoming should satisfy a written $written',
          );
        }
      }
    });

    test('hi-hat {42, 46} is one piece — binding, not articulation', () {
      expect(samePiece(42, 46), isTrue);
      expect(samePiece(46, 42), isTrue);
      // The verdict cap lives elsewhere; identity never gates on the variant.
      expect(sameStrokeArticulation(46, 42), isFalse);
      expect(sameStrokeArticulation(42, 46), isFalse);
      expect(sameStrokeArticulation(42, 42), isTrue);
      expect(sameStrokeArticulation(46, 46), isTrue);
    });

    test('ride {51, 53, 59} is one time-keeping cymbal', () {
      for (final written in const [51, 53, 59]) {
        for (final incoming in const [51, 53, 59]) {
          expect(samePiece(written, incoming), isTrue);
        }
      }
    });

    test('kick {35, 36} is one pedal', () {
      expect(samePiece(35, 36), isTrue);
      expect(samePiece(36, 35), isTrue);
      // …and the kick is nobody else's piece.
      expect(samePiece(36, 38), isFalse);
      expect(samePiece(36, 41), isFalse);
    });
  });

  group('separate aim points stay separate', () {
    test('a china does not satisfy a written crash', () {
      expect(samePiece(49, 52), isFalse);
      expect(samePiece(52, 49), isFalse);
      // Nor do the other accent cymbals collapse into one another.
      expect(samePiece(49, 55), isFalse);
      expect(samePiece(55, 57), isFalse);
    });

    test('one tom does not satisfy another', () {
      const toms = [41, 43, 45, 47, 48, 50];
      for (final a in toms) {
        for (final b in toms) {
          expect(samePiece(a, b), a == b);
        }
      }
    });

    test('a terminal-bucket number matches only itself', () {
      // The pedal hi-hat "chick" (44), the cowbell (56), the tambourine (54):
      // no named group, so each is a piece of its own.
      for (final gm in const [44, 54, 56, 80]) {
        expect(samePiece(gm, gm), isTrue);
        for (final other in const [42, 38, 51, 36, 49]) {
          expect(samePiece(gm, other), isFalse);
          expect(samePiece(other, gm), isFalse);
        }
      }
      expect(samePiece(44, 42), isFalse); // the pedal is not the hand stroke
      expect(samePiece(54, 56), isFalse);
    });

    test('a hand piece never satisfies the foot bar', () {
      expect(samePiece(36, 42), isFalse);
      expect(samePiece(38, 35), isFalse);
    });
  });

  group('equivalence reads the table, not the derived layout', () {
    // A score written entirely in acoustic snares (38), closed hats (42) and
    // Bass Drum 1 (36): 40, 37 and 35 appear NOWHERE in it.
    final lanes = deriveDrumLanes([n(42), n(38), n(36)]);

    test('a number absent from the score still matches its piece', () {
      expect(laneIndexOf(lanes, 40), isNull, reason: 'no lane holds a 40');
      expect(samePiece(38, 40), isTrue);
      expect(samePiece(38, 37), isTrue);
      expect(samePiece(36, 35), isTrue);
    });

    test('the pad feedback resolves at the same grain as the judgment', () {
      // The electric snare lights the snare pad even though the score never
      // writes a 40 — the same resolution the scorer binds it with.
      final snareLane = laneIndexOf(lanes, 38);
      expect(pieceLaneIndexOf(lanes, 40), snareLane);
      expect(pieceLaneIndexOf(lanes, 37), snareLane);
      expect(struckSurfaceOf(lanes, 40, hasPedal: true), snareLane);
      // The open hat lights the hi-hat pad, as it always did.
      expect(pieceLaneIndexOf(lanes, 46), laneIndexOf(lanes, 42));
      // Either kick number is the pedal.
      expect(struckSurfaceOf(lanes, 35, hasPedal: true), kPedalSurface);
      // A piece the score does not use still has nothing to flash.
      expect(struckSurfaceOf(lanes, 49, hasPedal: true), isNull);
      expect(pieceLaneIndexOf(lanes, 47), isNull);
    });

    test('a written note always resolves to its own lane', () {
      // The exact and the piece-grained lookups agree on every written note,
      // which is why the painters may keep using the exact one.
      for (final gm in const [42, 38]) {
        expect(pieceLaneIndexOf(lanes, gm), laneIndexOf(lanes, gm));
      }
    });
  });

  group('piece identity is a total, stable function', () {
    test('every General MIDI percussion number resolves to something', () {
      final ids = <String>{};
      for (var gm = 35; gm <= 81; gm++) {
        final id = drumPieceIdOf(gm);
        expect(id, isNotEmpty);
        ids.add(id);
      }
      // The collapses actually happen: 47 numbers, fewer pieces.
      expect(ids.length, lessThan(47));
    });

    test('identity is symmetric and reflexive', () {
      for (var a = 35; a <= 81; a++) {
        expect(samePiece(a, a), isTrue);
        for (var b = 35; b <= 81; b++) {
          expect(samePiece(a, b), samePiece(b, a));
        }
      }
    });
  });

  group('the gate reads the same identity as the scorer', () {
    const written = 38;
    final data = PlayerData(
      isPercussion: true,
      notes: [n(written)],
      songEndMs: 1000,
      waitMode: true,
    );

    test('strokeSatisfies is samePiece on a percussion score', () {
      for (var gm = 35; gm <= 81; gm++) {
        expect(
          data.strokeSatisfies(written, gm),
          samePiece(written, gm),
          reason: 'gate and matcher disagree on $gm',
        );
      }
    });

    test('and plain pitch equality on a keyboard score', () {
      final keyboard = data.copyWith(isPercussion: false);
      expect(keyboard.strokeSatisfies(60, 60), isTrue);
      expect(keyboard.strokeSatisfies(60, 61), isFalse);
      // The drum groups mean nothing there: 38 and 40 are two pitches.
      expect(keyboard.strokeSatisfies(38, 40), isFalse);
    });

    test('an equivalent stroke latches the REQUIRED number', () {
      // The gate's release check compares against the score's own vocabulary,
      // so what is latched is the written 38, never the incoming 40.
      expect(data.onsetPitchesSatisfiedBy(40, 0), {written});
      expect(data.onsetPitchesSatisfiedBy(37, 0), {written});
      expect(data.onsetPitchesSatisfiedBy(47, 0), isEmpty);
    });
  });
}
