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
import 'package:music/state/drum_input_mapping.dart';
import 'package:music/state/drum_kit.dart';

// The pure model behind the calibration pass (change:
// add-drum-input-calibration, tasks 4.1–4.3).

void main() {
  group('canonical numbers a piece translates to', () {
    test('every piece a calibration pass offers has a canonical number', () {
      // The property the whole mapping rests on: a recorded piece must have a
      // General MIDI number to translate INTO, or the entry is unusable.
      for (final id in kCalibrationPieceOrder) {
        expect(canonicalGmOfPiece(id), isNotNull, reason: id);
      }
    });

    test('the canonical number is the piece\'s first emission member', () {
      expect(canonicalGmOfPiece(kKickPieceId), 36); // Bass Drum 1, not 35
      expect(canonicalGmOfPiece('kitPieceSnare'), 38); // acoustic, not 40/37
      expect(canonicalGmOfPiece('kitPieceHiHat'), 42); // closed, not open
      expect(canonicalGmOfPiece('kitPieceRide'), 51); // ride 1, not the bell
    });

    test('a generic piece is its own number, an unknown one is null', () {
      expect(canonicalGmOfPiece('gm:56'), 56); // cowbell
      expect(canonicalGmOfPiece('gm:notANumber'), isNull);
      expect(canonicalGmOfPiece('kitPieceFromALaterBuild'), isNull);
    });

    test(
      'every canonical number is inside the General MIDI percussion map',
      () {
        // The invariant that makes a calibrated pad safe from silence, and the
        // reason the monitor's missing "will not sound" marker (task 3.3) is a
        // diagnostic refinement rather than a hole.
        //
        // A calibration translates whatever a pad sends into its piece's
        // canonical number. If every canonical number sits inside 35–81, then
        // after calibrating, every stroke lands on a number that ANY
        // General-MIDI-compliant drum font samples by definition — the shipped
        // kit included, whose measured range (27–87) contains the map whole. A
        // silent pad then cannot survive calibration; it can only survive NOT
        // being calibrated.
        //
        // Pinned rather than observed: add a piece tomorrow whose canonical
        // number falls outside the map — a generic `gm:` identity, say — and it
        // would silently reintroduce a pad that calibrates and still makes no
        // sound. This test is what stops that being discovered on a kit.
        for (final id in kCalibrationPieceOrder) {
          final gm = canonicalGmOfPiece(id)!;
          expect(
            gm,
            inInclusiveRange(kGmPercussionLowest, kGmPercussionHighest),
            reason: '$id translates to $gm, outside the standard map',
          );
          // …and the map names it, which is the same statement read from the
          // table the monitor reports through.
          expect(gmPercussionName(gm), isNotNull, reason: id);
        }
      },
    );

    test('a piece round-trips through its canonical number; a zone lands on '
        'the piece it belongs to', () {
      // A calibration target is either a piece — where the round trip is the
      // identity — or a **zone** of one (design D9), where it deliberately is
      // not: a rim stroke learned as 37 is still the snare to [drumPieceIdOf],
      // which is precisely what keeps the extra question free of consequences
      // downstream. The flash, the gate and the scorer never learn that the
      // pass asked for it separately.
      const zoneBelongsTo = {
        kCrossStickPieceId: 'kitPieceSnare',
        kOpenHiHatPieceId: 'kitPieceHiHat',
        kRideBellPieceId: 'kitPieceRide',
      };
      for (final id in kCalibrationPieceOrder) {
        expect(
          drumPieceIdOf(canonicalGmOfPiece(id)!),
          zoneBelongsTo[id] ?? id,
          reason: id,
        );
      }
      // The pedal "chick" is the exception among the zones: no hand strikes it,
      // so it is a piece of its own rather than part of the hi-hat.
      expect(drumPieceIdOf(44), kPedalHiHatPieceId);
    });
  });

  group('translate', () {
    test('an empty mapping is exactly the identity', () {
      final m = DrumInputMapping.empty;
      expect(m.isEmpty, isTrue);
      for (var n = 0; n <= 127; n++) {
        expect(m.translate(n), n, reason: 'number $n');
      }
      expect(m.translationTable, isEmpty);
    });

    test('a recorded number becomes its piece\'s canonical number', () {
      // The beta report's shape: a module sending 31 where the app expects a
      // snare, so the stroke is inaudible AND invisible until it is mapped.
      final m = DrumInputMapping(const {'kitPieceSnare': 31});
      expect(m.translate(31), 38);
    });

    test('everything the mapping does not cover passes through unchanged', () {
      final m = DrumInputMapping(const {'kitPieceSnare': 31});
      for (var n = 0; n <= 127; n++) {
        if (n == 31) continue;
        expect(m.translate(n), n, reason: 'number $n');
      }
    });

    test('translation is total and order-independent', () {
      const table = {
        kKickPieceId: 12,
        'kitPieceSnare': 31,
        'kitPieceHiHat': 22,
        'kitPieceTomHigh': 91,
      };
      final forward = DrumInputMapping(table);
      // The same entries inserted in the opposite order must translate
      // identically: the answer cannot depend on how the map was built.
      final reversed = DrumInputMapping({
        for (final key in table.keys.toList().reversed) key: table[key]!,
      });
      for (var n = 0; n <= 127; n++) {
        expect(forward.translate(n), reversed.translate(n), reason: 'n $n');
      }
      expect(forward.translate(12), 36);
      expect(forward.translate(91), 50);
    });

    test('a device already on the standard map translates to itself', () {
      // Calibrating a compliant module must be a no-op, not a re-shuffle.
      final m = DrumInputMapping(const {
        kKickPieceId: 36,
        'kitPieceSnare': 38,
        'kitPieceHiHat': 42,
      });
      for (var n = 0; n <= 127; n++) {
        expect(m.translate(n), n, reason: 'number $n');
      }
    });

    test('an entry for a piece this build does not know is skipped', () {
      // A table written by a later version: that entry degrades to
      // uncalibrated, the rest of the mapping still applies.
      final m = DrumInputMapping(const {
        'kitPieceFromTheFuture': 99,
        'kitPieceSnare': 31,
      });
      expect(m.translate(99), 99);
      expect(m.translate(31), 38);
    });

    test(
      'the engine table is the translation, in the callback\'s direction',
      () {
        final m = DrumInputMapping(const {
          'kitPieceSnare': 31,
          kKickPieceId: 12,
        });
        expect(m.translationTable, {31: 38, 12: 36});
      },
    );
  });

  group('conflicts', () {
    final mapping = DrumInputMapping(const {
      'kitPieceSnare': 31,
      'kitPieceHiHat': 22,
    });

    test('a number another piece already claims is a conflict', () {
      expect(
        mapping.conflictFor(31, forPiece: 'kitPieceTomHigh'),
        'kitPieceSnare',
      );
    });

    test('re-recording the same piece is not a conflict with itself', () {
      // A player striking the snare twice during the pass is correcting, not
      // colliding — the case that would otherwise make a re-take impossible.
      expect(mapping.conflictFor(31, forPiece: 'kitPieceSnare'), isNull);
    });

    test('a free number is not a conflict', () {
      expect(mapping.conflictFor(45, forPiece: 'kitPieceTomHigh'), isNull);
    });
  });

  group('editing', () {
    test('withPiece records, withoutPiece clears, neither disturbs the rest', () {
      final base = DrumInputMapping(const {'kitPieceSnare': 31});
      final added = base.withPiece('kitPieceHiHat', 22);
      expect(added.byPiece, {'kitPieceSnare': 31, 'kitPieceHiHat': 22});
      expect(added.translate(22), 42);

      final cleared = added.withoutPiece('kitPieceHiHat');
      expect(cleared.byPiece, {'kitPieceSnare': 31});
      // Cleared means uncalibrated for that piece — the number is itself again.
      expect(cleared.translate(22), 22);
      expect(cleared.translate(31), 38);

      // The originals are untouched (the value is immutable).
      expect(base.byPiece, {'kitPieceSnare': 31});
    });

    test('clearing every entry returns the device to uncalibrated', () {
      final m = DrumInputMapping(const {
        'kitPieceSnare': 31,
      }).withoutPiece('kitPieceSnare');
      expect(m.isEmpty, isTrue);
      expect(m.translate(31), 31);
    });

    test('re-recording a piece replaces its number', () {
      final m = DrumInputMapping(const {
        'kitPieceSnare': 31,
      }).withPiece('kitPieceSnare', 38);
      expect(m.byPiece, {'kitPieceSnare': 38});
      expect(m.translate(31), 31); // the old number no longer means anything
    });
  });

  group('serialisation (spec: local-preferences)', () {
    test('a round trip preserves every device independently', () {
      final mappings = {
        'TD-17': DrumInputMapping(const {
          'kitPieceSnare': 31,
          kKickPieceId: 12,
        }),
        'Practice pad': DrumInputMapping(const {'kitPieceSnare': 40}),
      };
      final back = decodeDrumInputMappings(encodeDrumInputMappings(mappings));
      expect(back.keys.toSet(), {'TD-17', 'Practice pad'});
      expect(back['TD-17']!.translate(31), 38);
      expect(back['Practice pad']!.translate(40), 38);
      // Connecting one does not disturb the other.
      expect(back['Practice pad']!.translate(31), 31);
    });

    test('an empty device table is not written at all', () {
      final encoded = encodeDrumInputMappings({
        'TD-17': DrumInputMapping.empty,
        'Kit': DrumInputMapping(const {'kitPieceSnare': 31}),
      });
      expect(decodeDrumInputMappings(encoded).keys, ['Kit']);
    });

    test('missing, empty or unparseable storage means no mapping', () {
      for (final raw in [null, '', 'not json', '[]', '42', '{']) {
        expect(decodeDrumInputMappings(raw), isEmpty, reason: '$raw');
      }
    });

    test('a malformed device entry is dropped, its neighbours survive', () {
      // One corrupt table must not cost a player the kit they calibrated.
      const raw = '{"Broken": "not a table", "Kit": {"kitPieceSnare": 31}}';
      final back = decodeDrumInputMappings(raw);
      expect(back.keys, ['Kit']);
      expect(back['Kit']!.translate(31), 38);
    });

    test('a number outside the 7-bit MIDI range is dropped', () {
      // No instrument can have sent it, so it is corruption, not an exotic pad.
      const raw = '{"Kit": {"kitPieceSnare": 300, "kitPieceHiHat": 22}}';
      final back = decodeDrumInputMappings(raw);
      expect(back['Kit']!.byPiece, {'kitPieceHiHat': 22});
    });

    test('an entry with a non-string piece or non-int number is dropped', () {
      const raw = '{"Kit": {"kitPieceSnare": "31", "kitPieceHiHat": 22}}';
      expect(decodeDrumInputMappings(raw)['Kit']!.byPiece, {
        'kitPieceHiHat': 22,
      });
    });
  });

  group('value semantics', () {
    test('equal tables are equal whatever order they were built in', () {
      final a = DrumInputMapping(const {
        'kitPieceSnare': 31,
        'kitPieceHiHat': 22,
      });
      final b = DrumInputMapping(const {
        'kitPieceHiHat': 22,
        'kitPieceSnare': 31,
      });
      expect(a, b);
      expect(a.hashCode, b.hashCode);
    });

    test('a different number makes a different mapping', () {
      expect(
        DrumInputMapping(const {'kitPieceSnare': 31}),
        isNot(DrumInputMapping(const {'kitPieceSnare': 32})),
      );
    });
  });
}
