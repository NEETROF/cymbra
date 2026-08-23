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

TimedNote n(int gm, {int voice = 1, int startMs = 0}) =>
    TimedNote(pitch: gm, startMs: startMs, durationMs: 250, voice: voice);

List<String?> laneIds(List<DrumLane> lanes) => [
  for (final l in lanes) l.labelKey ?? l.gmName,
];

void main() {
  group('deriveDrumLanes — the sort rule and its invariant', () {
    test('a sparse groove gets its pieces only, the kick excluded', () {
      // Hi-hat + snare + kick → two lanes, no lane for the kick.
      final lanes = deriveDrumLanes([n(42), n(38), n(35), n(36)]);
      expect(laneIds(lanes), ['kitPieceHiHat', 'kitPieceSnare']);
    });

    test('position 1 is the time-keeper, 2 the snare; pieces append right', () {
      final sparse = deriveDrumLanes([n(42), n(38), n(36)]);
      final dense = deriveDrumLanes([
        n(42), n(38), n(36), // core
        n(50), n(45), // toms
        n(51), n(49), n(57), // ride + crashes
      ]);
      // The core keeps positions 1–2 in both; extras append to the right.
      expect(laneIds(sparse).sublist(0, 2), ['kitPieceHiHat', 'kitPieceSnare']);
      expect(laneIds(dense).sublist(0, 2), ['kitPieceHiHat', 'kitPieceSnare']);
      expect(laneIds(dense), [
        'kitPieceHiHat',
        'kitPieceSnare',
        'kitPieceTomHigh',
        'kitPieceTomLow',
        'kitPieceRide',
        'kitPieceCrash',
        'kitPieceCrash2',
      ]);
    });

    test('the ride takes position 1 when the score has no hi-hat', () {
      final lanes = deriveDrumLanes([n(51), n(38), n(36)]);
      expect(laneIds(lanes), ['kitPieceRide', 'kitPieceSnare']);
    });

    test('the ride joins the cymbals when a hi-hat is present', () {
      final lanes = deriveDrumLanes([n(42), n(38), n(51), n(45)]);
      expect(laneIds(lanes), [
        'kitPieceHiHat',
        'kitPieceSnare',
        'kitPieceTomLow',
        'kitPieceRide',
      ]);
    });

    test('a snare-less score closes ranks leftward', () {
      // No snare: the toms slide up next to the time-keeper — empty buckets
      // are skipped, never reserved as gaps.
      final lanes = deriveDrumLanes([n(42), n(45), n(41)]);
      expect(laneIds(lanes), [
        'kitPieceHiHat',
        'kitPieceTomLow',
        'kitPieceTomFloorLow',
      ]);
    });

    test('equivalent numbers share one lane', () {
      // Closed + open hi-hat; acoustic + electric snare + side stick.
      final lanes = deriveDrumLanes([n(42), n(46), n(38), n(40), n(37)]);
      expect(lanes, hasLength(2));
      expect(lanes[0].gmNumbers, {42, 46});
      expect(lanes[1].gmNumbers, {37, 38, 40});
    });

    test('toms order highest to lowest', () {
      final lanes = deriveDrumLanes([n(41), n(50), n(45), n(48)]);
      expect(laneIds(lanes), [
        'kitPieceTomHigh',
        'kitPieceTomHiMid',
        'kitPieceTomLow',
        'kitPieceTomFloorLow',
      ]);
    });

    test('no silent drop: every non-kick number present lands in exactly one '
        'lane, unmapped ones as generic pieces in the terminal bucket', () {
      // Cowbell 56, tambourine 54 and the pedal hi-hat 44 are outside the
      // named roles: each takes a generic lane AFTER the cymbals, in stable
      // ascending GM order. GM 44 is deliberately a lane (visible and
      // aim-able beats invisible-but-scheduled) until its bar encoding
      // exists.
      final present = [n(42), n(38), n(49), n(56), n(44), n(54), n(35)];
      final lanes = deriveDrumLanes(present);
      expect(laneIds(lanes), [
        'kitPieceHiHat',
        'kitPieceSnare',
        'kitPieceCrash',
        'Pedal Hi-Hat',
        'Tambourine',
        'Cowbell',
      ]);
      // Every non-kick number is represented exactly once.
      for (final note in present.where(
        (x) => !kKickGmNumbers.contains(x.pitch),
      )) {
        expect(laneIndexOf(lanes, note.pitch), isNotNull);
      }
      // The kick never consumes lane width.
      expect(laneIndexOf(lanes, 35), isNull);
      expect(laneIndexOf(lanes, 36), isNull);
    });

    test('an all-kick score has no lanes at all', () {
      expect(deriveDrumLanes([n(35), n(36)]), isEmpty);
    });
  });

  group('hands/feet classification', () {
    test('two-voice files split by voice — never by GM number', () {
      final notes = [n(42, voice: 1), n(36, voice: 2)];
      expect(spansMultipleVoices(notes), isTrue);
      expect(isFootNote(notes[0], multiVoice: true), isFalse);
      expect(isFootNote(notes[1], multiVoice: true), isTrue);
      // A hand-voiced kick (unusual but written) follows its voice.
      expect(isFootNote(n(36, voice: 1), multiVoice: true), isFalse);
    });

    test('single-voice files fall back to the GM number', () {
      final notes = [n(42), n(36), n(44)];
      expect(spansMultipleVoices(notes), isFalse);
      expect(isFootNote(notes[0], multiVoice: false), isFalse);
      expect(isFootNote(notes[1], multiVoice: false), isTrue); // kick
      expect(isFootNote(notes[2], multiVoice: false), isTrue); // pedal hi-hat
    });
  });

  test('the open hi-hat is a note variant, not a separate lane', () {
    expect(isOpenHiHat(46), isTrue);
    expect(isOpenHiHat(42), isFalse);
    final lanes = deriveDrumLanes([n(42), n(46)]);
    expect(lanes, hasLength(1)); // one hi-hat lane holds both strokes
  });
}
