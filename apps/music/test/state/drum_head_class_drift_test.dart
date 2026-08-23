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
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/drum_kit.dart';
import 'package:music/state/player_data.dart';

import '../support/notation_fakes.dart';

/// Drift pinning between the two tables that share one fact (change:
/// add-drum-notation-render, task 6.1): the app's kit-view roles
/// (`drum_kit.dart` — the GAMEPLAY question: lanes and pads) and the shared
/// crate's engraved head classes (`HeadClass::of` in
/// `crates/musicxml-core/src/model.rs` — the ENGRAVING question: x vs oval).
/// The overlap — which pieces are cymbals — must never disagree silently.
///
/// A pure Dart test cannot call the Rust `HeadClass::of`, so the crate side
/// of the fact is pinned by the crate test
/// `head_classes_split_cymbals_from_drums_and_mark_the_open_hat`
/// (`crates/musicxml-core/src/lib.rs`), and THIS test pins the same fact on
/// the app side through [_expectedHeadClass] — a test-only mirror of the
/// crate's table (production code never re-derives it; painters consume the
/// bridged `Unpitched.headClass` verbatim). If the crate table changes, its
/// test and this mirror must change together, and this test then catches any
/// kit-view role (or notation fake) left behind.
///
/// The shared fact, explicitly: the cymbal sounds of the standard kit are
/// General MIDI (0-based) 42, 44, 46, 49, 51, 52, 53, 55, 57, 59 — hi-hats
/// (42/44/46, 46 being the marked open stroke), crashes (49/52/55/57) and
/// rides (51/53/59). Everything else, unresolved included, is a drum (oval).
const Set<int> _cymbalGm = {42, 44, 46, 49, 51, 52, 53, 55, 57, 59};
const int _openHiHatGm = 46;

/// The crate's classification, mirrored for assertion only.
HeadClass _expectedHeadClass(int? gm) {
  if (gm == _openHiHatGm) return HeadClass.xOpen;
  if (gm != null && _cymbalGm.contains(gm)) return HeadClass.x;
  return HeadClass.oval;
}

void main() {
  test('every GM number the kit view roles as a cymbal (hiHat/ride/crash) '
      'classifies as an x head, and no drum-role number does', () {
    // Lay out the full standard kit so every named piece takes its lane.
    final lanes = deriveDrumLanes([
      for (var gm = 35; gm <= 81; gm++)
        TimedNote(pitch: gm, startMs: 0, durationMs: 100),
    ]);
    expect(lanes, isNotEmpty);

    const cymbalRoles = {
      KitPieceRole.hiHat,
      KitPieceRole.ride,
      KitPieceRole.crash,
    };
    const drumRoles = {KitPieceRole.snare, KitPieceRole.tom};

    for (final lane in lanes) {
      for (final gm in lane.gmNumbers) {
        final headClass = _expectedHeadClass(gm);
        if (cymbalRoles.contains(lane.role)) {
          expect(
            headClass == HeadClass.x || headClass == HeadClass.xOpen,
            isTrue,
            reason:
                'GM $gm holds a cymbal lane (${lane.role}) but the head '
                'class table calls it $headClass — the gameplay and '
                'engraving tables disagree',
          );
        }
        if (drumRoles.contains(lane.role)) {
          expect(
            headClass,
            HeadClass.oval,
            reason:
                'GM $gm holds a drum lane (${lane.role}) but the head '
                'class table calls it $headClass — the gameplay and '
                'engraving tables disagree',
          );
        }
      }
    }
    // The kicks (no lane — they are the cascade's bar) are drums too.
    for (final gm in kKickGmNumbers) {
      expect(_expectedHeadClass(gm), HeadClass.oval);
    }
    // The one marked stroke is the open hi-hat, nothing else.
    expect(_expectedHeadClass(46), HeadClass.xOpen);
    expect(
      [
        for (var gm = 35; gm <= 81; gm++)
          if (_expectedHeadClass(gm) == HeadClass.xOpen) gm,
      ],
      [46],
    );
  });

  test('the notation fakes carry exactly the head classes the crate would '
      'bridge for their GM numbers', () {
    // The fakes set `headClass` explicitly (widget tests cannot run the
    // native parser); this pins them to the crate's table so a fixture can
    // never quietly teach the painters the wrong classes.
    final documents = {
      'sampleDrumDocument': sampleDrumDocument(),
      'sampleOpenGrooveDocument': sampleOpenGrooveDocument(),
      'samplePercussionSharedPositionDocument':
          samplePercussionSharedPositionDocument(),
    };
    var checked = 0;
    for (final entry in documents.entries) {
      for (final measure in entry.value.measures) {
        for (final note in measure.notes) {
          final unpitched = note.unpitched;
          if (unpitched == null) continue;
          checked++;
          expect(
            unpitched.headClass,
            _expectedHeadClass(unpitched.gmNumber),
            reason:
                '${entry.key}: GM ${unpitched.gmNumber} carries '
                '${unpitched.headClass} but the crate table says '
                '${_expectedHeadClass(unpitched.gmNumber)}',
          );
        }
      }
    }
    expect(checked, greaterThan(0));
  });
}
