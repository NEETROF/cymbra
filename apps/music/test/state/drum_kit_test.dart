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

  group('emission — the number a pad strikes (add-drum-input-mapping)', () {
    /// The emitted number of the lane holding [gm] on a score made of [score].
    int emitted(List<int> score, int gm) {
      final lanes = deriveDrumLanes([for (final p in score) n(p)]);
      return emittedGmOfLane(lanes[laneIndexOf(lanes, gm)!]);
    }

    test('a pad emits its piece\'s first canonical member', () {
      // Hi-hat: closed before open. Snare: acoustic, then electric, then side
      // stick. Ride: the ride itself before the second ride and the bell.
      expect(emitted([42, 46, 38, 40, 37, 51, 59, 53], 42), 42);
      expect(emitted([42, 46, 38, 40, 37, 51, 59, 53], 38), 38);
      expect(emitted([42, 46, 38, 40, 37, 51, 59, 53], 51), 51);
    });

    test('the canonical member the score does NOT use is skipped', () {
      // A score written with the electric snare only emits 40, never the
      // absent canonical 38 — the stroke stays inside the file's vocabulary.
      expect(emitted([42, 40], 40), 40);
      // The side stick alone: the last canonical member, still emitted.
      expect(emitted([42, 37], 37), 37);
      // An open-only hi-hat emits 46 — the one case the strip emits the open
      // stroke, because it is the only hi-hat number the score has.
      expect(emitted([46, 38], 46), 46);
      // The bell alone speaks for the ride lane.
      expect(emitted([53, 38], 53), 53);
    });

    test('a generic (terminal-bucket) piece emits its single number', () {
      expect(emitted([42, 38, 56], 56), 56); // cowbell
      expect(emitted([42, 38, 44], 44), 44); // pedal hi-hat
    });

    test('every emitted number is a member of the lane it came from', () {
      // The property the struck-flash lookup and the future matcher both rest
      // on: emission can never leave the lane that produced it.
      final score = [42, 46, 37, 40, 41, 45, 50, 53, 59, 49, 52, 55, 57, 54];
      final lanes = deriveDrumLanes([for (final p in score) n(p)]);
      expect(lanes, isNotEmpty);
      for (final lane in lanes) {
        final gm = emittedGmOfLane(lane);
        expect(lane.gmNumbers, contains(gm));
        expect(laneIndexOf(lanes, gm), lanes.indexOf(lane));
      }
    });

    test('emission is deterministic for a given score', () {
      final score = [40, 38, 46, 42, 53, 51];
      final first = [
        for (final l in deriveDrumLanes([for (final p in score) n(p)]))
          emittedGmOfLane(l),
      ];
      // Same score, notes in another order: the same numbers, no runtime
      // cleverness and no dependence on how the set was built.
      final shuffled = [42, 51, 38, 53, 46, 40];
      final second = [
        for (final l in deriveDrumLanes([for (final p in shuffled) n(p)]))
          emittedGmOfLane(l),
      ];
      expect(first, [42, 38, 51]);
      expect(second, first);
      // A hand-built lane (not derived) emits by the table too, never by set
      // iteration order.
      expect(
        emittedGmOfLane(
          const DrumLane(
            role: KitPieceRole.snare,
            gmNumbers: {37, 40, 38},
            labelKey: 'kitPieceSnare',
          ),
        ),
        38,
      );
    });

    test('the kick pedal emits 36, or 35 for a score that only writes 35', () {
      expect(emittedKickGm({35, 36}), 36);
      expect(emittedKickGm({36}), 36);
      expect(emittedKickGm({35}), 35);
      // No kick at all: nothing to emit (and no pedal is drawn).
      expect(emittedKickGm(const {}), isNull);
      // The ordered list and the classification set agree on membership.
      expect(kKickEmissionOrder.toSet(), kKickGmNumbers);
    });
  });

  group('struck-surface resolution (add-drum-input-mapping)', () {
    final lanes = deriveDrumLanes([n(42), n(46), n(38), n(36)]);

    test('a number resolves to the pad of the lane that collapses it', () {
      expect(struckSurfaceOf(lanes, 42, hasPedal: true), 0);
      // The open stroke lights the SAME pad as the closed one — one lane.
      expect(struckSurfaceOf(lanes, 46, hasPedal: true), 0);
      expect(struckSurfaceOf(lanes, 38, hasPedal: true), 1);
    });

    test('either kick number resolves to the pedal', () {
      expect(struckSurfaceOf(lanes, 36, hasPedal: true), kPedalSurface);
      expect(struckSurfaceOf(lanes, 35, hasPedal: true), kPedalSurface);
      // The pedal sentinel can never collide with a pad index.
      expect(kPedalSurface, lessThan(0));
    });

    test('a number outside the score\'s kit resolves to nothing', () {
      // A crash over a crash-less groove: free play, no pad to flash, no
      // error — and a kick on a kickless score has no pedal to flash either.
      expect(struckSurfaceOf(lanes, 49, hasPedal: true), isNull);
      expect(struckSurfaceOf(lanes, 36, hasPedal: false), isNull);
    });
  });

  group('the early-stroke window (beta fix: the stale stroke)', () {
    test(
      'a stroke inside the window answers the onset, one outside does not',
      () {
        expect(strokeAnswersOnset(strokeMs: 590, nowMs: 600, speed: 1), isTrue);
        expect(
          strokeAnswersOnset(
            strokeMs: 600 - kStrokeToleranceMs - 1,
            nowMs: 600,
            speed: 1,
          ),
          isFalse,
        );
      },
    );

    test('a stroke stamped AHEAD of the playhead is stale, never early', () {
      // What a restart / loop wrap / rewind leaves behind: the stroke was
      // made at 40 s, the playhead is back at the top. Its age is negative,
      // so no upper bound alone would reject it — and the run would open
      // with its first onset already satisfied by nobody.
      expect(strokeAnswersOnset(strokeMs: 40000, nowMs: 0, speed: 1), isFalse);
      // Even one millisecond ahead: the window has no left-hand slack.
      expect(strokeAnswersOnset(strokeMs: 601, nowMs: 600, speed: 1), isFalse);
      // The exact instant still counts (a stroke landing on the onset).
      expect(strokeAnswersOnset(strokeMs: 600, nowMs: 600, speed: 1), isTrue);
    });

    test('the window widens with the transport, in both readings', () {
      final wide = 600 - strokeToleranceMsAt(2) + 1;
      expect(strokeAnswersOnset(strokeMs: wide, nowMs: 600, speed: 2), isTrue);
      expect(strokeAnswersOnset(strokeMs: wide, nowMs: 600, speed: 1), isFalse);
      // …and a stale stamp stays stale at any speed.
      expect(strokeAnswersOnset(strokeMs: 40000, nowMs: 0, speed: 2), isFalse);
    });
  });

  group('per-piece focus (change: add-practice-focus-controls)', () {
    // A groove with a hi-hat written both closed and open, a snare written as
    // the acoustic 38 AND the electric 40 plus a side stick, a crash and a
    // kick — every collapsed case the kit model already decided.
    final groove = [n(42), n(46), n(38), n(40), n(37), n(49), n(36)];

    test('the piece list is the pad strip order, with the kick last', () {
      final lanes = deriveDrumLanes(groove);
      final ids = kitPieceIdsOf(lanes, hasKick: true);
      expect(ids, [
        'kitPieceHiHat',
        'kitPieceSnare',
        'kitPieceCrash',
        kKickPieceId,
      ]);
      // Each lane's identity is the identity of its own members — no second
      // notion of "one piece" was invented for focus.
      for (final lane in lanes) {
        for (final gm in lane.gmNumbers) {
          expect(drumPieceIdOf(gm), drumPieceIdOfLane(lane));
        }
      }
    });

    test('a kickless score offers no kick piece', () {
      final lanes = deriveDrumLanes([n(42), n(38)]);
      expect(kitPieceIdsOf(lanes, hasKick: false), [
        'kitPieceHiHat',
        'kitPieceSnare',
      ]);
    });

    test('muting a piece mutes every number that collapses onto it', () {
      const muted = {'kitPieceHiHat'};
      // One pad, one answer: closed and open hi-hat go together.
      expect(isDrumPieceInFocus(42, muted), isFalse);
      expect(isDrumPieceInFocus(46, muted), isFalse);
      // …and nothing else moves.
      expect(isDrumPieceInFocus(38, muted), isTrue);
      expect(isDrumPieceInFocus(36, muted), isTrue);
    });

    test(
      'the snare collapses its acoustic, electric and side-stick numbers',
      () {
        const muted = {'kitPieceSnare'};
        for (final gm in [37, 38, 40]) {
          expect(isDrumPieceInFocus(gm, muted), isFalse, reason: 'GM $gm');
        }
      },
    );

    test('the kick collapses both of its numbers', () {
      const muted = {kKickPieceId};
      expect(isDrumPieceInFocus(35, muted), isFalse);
      expect(isDrumPieceInFocus(36, muted), isFalse);
    });

    test('an empty muted set puts everything in focus', () {
      for (var gm = 27; gm <= 87; gm++) {
        expect(isDrumPieceInFocus(gm, const {}), isTrue, reason: 'GM $gm');
      }
    });

    test('a piece the kit model never enumerated reads as in focus', () {
      // Free-play territory: a stroke on a number outside the score's kit must
      // never be filtered out by a selection that says nothing about it.
      expect(isDrumPieceInFocus(56, const {'kitPieceHiHat'}), isTrue);
    });

    test('muting the last piece in focus restores the whole kit', () {
      final all = kitPieceIdsOf(deriveDrumLanes(groove), hasKick: true);
      var muted = <String>{};
      // Mute every piece but the last one…
      for (final id in all.take(all.length - 1)) {
        muted = mutedAfterMuting(muted, id, all);
      }
      expect(muted, hasLength(all.length - 1));
      // …and the last mute empties the selection instead of the session.
      expect(mutedAfterMuting(muted, all.last, all), isEmpty);
    });

    test('soloing from the full kit isolates one piece', () {
      final all = kitPieceIdsOf(deriveDrumLanes(groove), hasKick: true);
      final muted = mutedAfterSoloing(const {}, 'kitPieceSnare', all);
      expect(
        muted,
        containsAll(['kitPieceHiHat', 'kitPieceCrash', kKickPieceId]),
      );
      expect(muted.contains('kitPieceSnare'), isFalse);
    });

    test('soloing a second piece isolates THAT one', () {
      // It used to add to the selection, which is not what "solo" says on the
      // row it is invoked from — and adding is what the piece's own checkbox
      // is for.
      final all = kitPieceIdsOf(deriveDrumLanes(groove), hasKick: true);
      var muted = mutedAfterSoloing(const {}, 'kitPieceHiHat', all);
      muted = mutedAfterSoloing(muted, 'kitPieceSnare', all);
      final focused = all.where((id) => !muted.contains(id)).toList();
      expect(focused, ['kitPieceSnare']);
    });

    test('soloing from a partial selection still isolates', () {
      // The shape the report came in: with some pieces already unchecked, solo
      // appeared to do nothing at all.
      final all = kitPieceIdsOf(deriveDrumLanes(groove), hasKick: true);
      final muted = mutedAfterSoloing(
        const {'kitPieceCrash'},
        'kitPieceSnare',
        all,
      );
      final focused = all.where((id) => !muted.contains(id)).toList();
      expect(focused, ['kitPieceSnare']);
    });

    test('soloing the piece already alone changes nothing', () {
      final all = kitPieceIdsOf(deriveDrumLanes(groove), hasKick: true);
      final muted = mutedAfterSoloing(const {}, 'kitPieceHiHat', all);
      expect(mutedAfterSoloing(muted, 'kitPieceHiHat', all), muted);
    });
  });
}
