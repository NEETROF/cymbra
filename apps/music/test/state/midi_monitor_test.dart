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
import 'package:music/state/midi_monitor.dart';

/// The monitor's reading of an incoming number (change:
/// add-drum-input-calibration).
///
/// The distinction that matters is not "which piece" but **inert or not**: a
/// stroke the app cannot place lights no pad, releases no gate and earns no
/// credit, and the player's only symptom is silence. These tests pin the three
/// ways a stroke can fail to land, and the two ways it can succeed.
void main() {
  /// A kit derived from a groove written with a closed hi-hat, an acoustic
  /// snare and a kick — the shape of almost every drum export.
  List<DrumLane> grooveLanes() => const [
    DrumLane(
      role: KitPieceRole.hiHat,
      gmNumbers: {42},
      labelKey: 'kitPieceHiHat',
    ),
    DrumLane(
      role: KitPieceRole.snare,
      gmNumbers: {38},
      labelKey: 'kitPieceSnare',
    ),
  ];

  MidiMonitorEntry read(
    int pitch, {
    bool percussion = true,
    List<DrumLane>? lanes,
    bool hasKick = true,
    int velocity = 100,
    int channel = 9,
    bool isNoteOn = true,
  }) => readMidiEvent(
    seq: 0,
    pitch: pitch,
    velocity: velocity,
    channel: channel,
    isNoteOn: isNoteOn,
    percussion: percussion,
    lanes: lanes ?? grooveLanes(),
    hasKick: hasKick,
  );

  group('a stroke that lands', () {
    test('a piece of this score names itself', () {
      final e = read(38);
      expect(e.resolution, MidiResolution.matchedPiece);
      expect(e.pieceLabelKey, 'kitPieceSnare');
      expect(e.isInert, isFalse);
    });

    test('the kick lands even though it has no lane', () {
      // The kick is the full-width bar, not a lane, so the layout alone cannot
      // answer for it — and reporting the one piece every groove has as
      // unmatched would make the monitor useless.
      final e = read(36);
      expect(e.resolution, MidiResolution.matchedPiece);
      expect(e.isInert, isFalse);
    });

    test('a different number for the same piece still lands', () {
      // The score writes its snare as 38; a module sending the electric 40 is
      // playing the same drum, and the gate and the scorer already treat it
      // that way. A monitor that reported a miss here would send a player
      // hunting a problem that does not exist.
      final e = read(40);
      expect(e.resolution, MidiResolution.matchedPiece);
      expect(e.pieceLabelKey, 'kitPieceSnare');
    });

    test('the open hi-hat lands on the closed hi-hat lane', () {
      expect(read(46).resolution, MidiResolution.matchedPiece);
    });
  });

  group('a stroke that does nothing', () {
    test('a real instrument this score does not use is inert', () {
      // 49 is a crash: the font will sound it, and nothing else will happen.
      final e = read(49);
      expect(e.resolution, MidiResolution.outsideThisKit);
      expect(e.gmName, 'Crash Cymbal 1');
      expect(e.isInert, isTrue);
    });

    test('a number outside the General MIDI map is inert and unnamed', () {
      // The beta report's shape: a reassigned pad sending a number the standard
      // has no name for. Not merely mis-sounded — invisible.
      final e = read(97);
      expect(e.resolution, MidiResolution.outsideTheMap);
      expect(e.gmName, isNull);
      expect(e.isInert, isTrue);
    });

    test('the two failures are distinguishable', () {
      // They need different fixes: one is "your kit plays a piece this groove
      // does not ask for", the other "the app does not know what you sent".
      expect(read(49).resolution, isNot(read(97).resolution));
    });

    test('the map boundaries are exact', () {
      expect(read(kGmPercussionLowest).gmName, isNotNull);
      expect(read(kGmPercussionHighest).gmName, isNotNull);
      expect(
        read(kGmPercussionLowest - 1).resolution,
        MidiResolution.outsideTheMap,
      );
      expect(
        read(kGmPercussionHighest + 1).resolution,
        MidiResolution.outsideTheMap,
      );
    });
  });

  group('a keyboard score', () {
    test('numbers are pitches, and the kit vocabulary does not apply', () {
      final e = read(60, percussion: false);
      expect(e.resolution, MidiResolution.pitched);
      expect(e.isInert, isFalse, reason: 'a pitch is never inert');
      expect(e.gmName, isNull);
    });
  });

  group('what is reported verbatim', () {
    test('the number, velocity and channel arrive untouched', () {
      final e = read(38, velocity: 37, channel: 5);
      expect(e.pitch, 38);
      expect(e.velocity, 37);
      expect(
        e.channel,
        5,
        reason: 'reported as received; interpretation stays channel-agnostic',
      );
    });

    test('the channel never changes the reading', () {
      // A kit transmitting anywhere but channel 10 must read identically —
      // the property a channel-based "fix" would break.
      for (final channel in [0, 5, 9, 15]) {
        expect(
          read(38, channel: channel).resolution,
          MidiResolution.matchedPiece,
        );
      }
    });

    test('a note-off is recorded, not swallowed', () {
      // A kit that sends none is a fact worth seeing; so is one that sends them
      // milliseconds after the attack.
      expect(read(38, isNoteOn: false).isNoteOn, isFalse);
    });
  });

  group('before a score is open', () {
    test('the standard map still answers', () {
      // The monitor is most useful exactly when nothing is loaded — a player
      // hunting a silent pad has no reason to have opened a groove first.
      final e = read(38, lanes: const [], hasKick: false);
      expect(e.gmName, 'Acoustic Snare');
      expect(e.resolution, MidiResolution.outsideThisKit);
    });
  });
}
