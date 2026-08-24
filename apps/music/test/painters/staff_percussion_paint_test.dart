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

import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/notation_palette.dart';
import 'package:music/painters/smufl.dart';
import 'package:music/painters/staff_hit_index.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/src/rust/api/musicxml.dart' show HeadClass;
import 'package:music/state/notation_playback.dart';
import 'package:music/state/player_data.dart';

import '../support/notation_fakes.dart';
import '../support/test_fonts.dart';

/// Percussion engraving on the scrolling Staff (change:
/// add-drum-notation-render), asserted over the derived playback of the
/// `groove_ouvert` fixture mirror — pixel probes plus the painter's own hit
/// index, the house style for paint proofs.
///
/// The "parity" tests cover the same named facts the console painter's SVG
/// suite asserts (clef glyph, written positions, head classes), so a rule
/// change that lands on one side only breaks a named test on the other.
void main() {
  setUpAll(loadBravura);

  const width = 900;
  const height = 400;
  final lineGap = StaffPainter.staffLineGap(
    height: height.toDouble(),
    twoStaff: false,
  );
  final stepGap = lineGap / 2;

  ({ui.Image image, StaffHitIndex hits}) paintStaff({
    DerivedPlayback? derived,
    List<TimedNote>? notes,
    List<TimedRest>? rests,
    List<TimedNote>? tieContinuations,
    int keyFifths = 0,
    NotationPalette palette = NotationPalette.dark,
  }) {
    final d = derived ?? notationToTimedNotes(sampleOpenGrooveDocument());
    final hits = StaffHitIndex();
    final recorder = ui.PictureRecorder();
    StaffPainter(
      notes: notes ?? d.notes,
      rests: rests ?? d.rests,
      tieContinuations: tieContinuations ?? d.tieContinuations,
      elapsedMs: 0,
      activeNotes: const {},
      bpm: d.bpm,
      songEndMs: d.songEndMs,
      keyFifths: keyFifths,
      measureKeyFifths: d.measureKeyFifths,
      measureStartMs: d.measureStartMs,
      palette: palette,
      hitIndex: hits,
    ).paint(Canvas(recorder), Size(width.toDouble(), height.toDouble()));
    final image = recorder.endRecording().toImageSync(width, height);
    return (image: image, hits: hits);
  }

  Future<ByteData> bytes(ui.Image image) async =>
      (await image.toByteData(format: ui.ImageByteFormat.rawRgba))!;

  Color pixelOf(ByteData data, int x, int y) {
    final offset = (y * width + x) * 4;
    return Color.fromARGB(
      data.getUint8(offset + 3),
      data.getUint8(offset),
      data.getUint8(offset + 1),
      data.getUint8(offset + 2),
    );
  }

  /// Non-transparent pixels inside [box] — "ink", whatever its colour.
  int inkIn(ByteData data, Rect box) {
    var count = 0;
    for (var y = box.top.round(); y <= box.bottom.round(); y++) {
      for (var x = box.left.round(); x <= box.right.round(); x++) {
        if (x < 0 || y < 0 || x >= width || y >= height) continue;
        if (data.getUint8((y * width + x) * 4 + 3) > 40) count++;
      }
    }
    return count;
  }

  /// Channel distance for "reads as this colour" assertions.
  int dist(Color a, Color b) =>
      (((a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs()) * 255)
          .round();

  /// Centre of the first recorded note head for GM number [gm], optionally
  /// the occurrence at index [nth].
  Offset headCenter(StaffHitIndex hits, int gm, {int nth = 0}) {
    var seen = 0;
    for (final e in hits.entries) {
      final d = e.descriptor;
      if (d is NoteSymbol && d.pitch == gm) {
        if (seen++ == nth) return e.region.center;
      }
    }
    fail('no note head recorded for GM $gm (occurrence $nth)');
  }

  test('the Smufl clef mapping is explicit for percussion — never the '
      'default-to-treble fallback', () {
    expect(Smufl.clef('percussion'), Smufl.percussionClef);
    expect(Smufl.percussionClef, '\u{E069}'); // unpitchedPercussionClef1
    expect(Smufl.clef('percussion'), isNot(Smufl.clef('G')));
  });

  test('parity: the percussion clef opens the staff and no armature is drawn '
      'even when the file declares fifths = 2', () {
    final r = paintStaff(
      derived: notationToTimedNotes(sampleOpenGrooveDocument(keyFifths: 2)),
      keyFifths: 2,
    );
    final clefs = [
      for (final e in r.hits.entries)
        if (e.descriptor is ClefSymbol) (e.descriptor as ClefSymbol).sign,
    ];
    expect(clefs, ['percussion']);
    // No armature — and no key-change redraw — despite fifths = 2…
    expect(
      r.hits.entries.any((e) => e.descriptor is KeySignatureSymbol),
      isFalse,
    );
    // …while the time signature still draws as on any staff.
    expect(
      r.hits.entries.any((e) => e.descriptor is TimeSignatureSymbol),
      isTrue,
    );
  });

  test('control: a keyboard staff with the same painter DOES record its '
      'armature (the percussion assert is not vacuous)', () {
    final r = paintStaff(
      derived: notationToTimedNotes(sampleGrandStaffDocument()),
      keyFifths: 3,
    );
    expect(
      r.hits.entries.any((e) => e.descriptor is KeySignatureSymbol),
      isTrue,
    );
  });

  test('parity: snare and kick engrave at their written positions — snare C5 '
      'third space, kick F4 bottom space, hi-hat G5 above the top line', () {
    final r = paintStaff();
    // Use occurrences past the playhead so the positions are ordinary heads.
    final snare = headCenter(r.hits, 38);
    final kick = headCenter(r.hits, 36, nth: 1);
    final hat = headCenter(r.hits, 42, nth: 2);
    // C5 sits 4 diatonic steps above F4, G5 four above C5 — position comes
    // from the written placement, never from the GM number (36 > 35 yet the
    // kick draws BELOW the snare).
    expect(kick.dy - snare.dy, closeTo(4 * stepGap, 0.6));
    expect(snare.dy - hat.dy, closeTo(4 * stepGap, 0.6));
    // The kick is a note head on the staff, not a full-width bar: its
    // recorded region is head-sized, a small fraction of the canvas width.
    final kickRegion = r.hits.entries
        .firstWhere(
          (e) =>
              e.descriptor is NoteSymbol &&
              (e.descriptor as NoteSymbol).pitch == 36,
        )
        .region;
    expect(kickRegion.width, lessThan(width / 10));
  });

  test('parity: cymbals take x heads and drums oval heads — the x form '
      'covers visibly less of the head box than the filled oval', () async {
    final r = paintStaff();
    final data = await bytes(r.image);
    final snare = headCenter(r.hits, 38); // oval, eighth
    final hat = headCenter(r.hits, 42, nth: 2); // x, eighth
    Rect box(Offset c) => Rect.fromCenter(
      center: c,
      width: Smufl.noteheadWidth * lineGap + 2,
      height: lineGap - 2,
    );
    final oval = inkIn(data, box(snare));
    final x = inkIn(data, box(hat));
    expect(oval, greaterThan(0));
    expect(x, greaterThan(0));
    expect(
      x,
      lessThan(oval * 0.8),
      reason:
          'the hi-hat head must be the x form (two strokes), not a filled '
          'oval — got $x ink px vs the snare\'s $oval',
    );
  });

  test('the open mark rides the open hi-hat (GM 46) and is absent on the '
      'closed stroke (GM 42)', () async {
    final r = paintStaff();
    final data = await bytes(r.image);
    final open = headCenter(r.hits, 46);
    final closed = headCenter(r.hits, 42, nth: 2);
    Rect above(Offset c) => Rect.fromCenter(
      center: Offset(c.dx, c.dy - lineGap * 1.6),
      width: lineGap * 0.9,
      height: lineGap,
    );
    expect(
      inkIn(data, above(open)),
      greaterThan(0),
      reason: 'the open stroke carries the small circle above its head',
    );
    expect(
      inkIn(data, above(closed)),
      0,
      reason: 'a closed stroke carries no open mark',
    );
  });

  test('a voice-2 rest is displaced below the middle line in a two-voice '
      'measure', () {
    final r = paintStaff();
    // Recover the staff geometry from a known head: kick F4 sits one step
    // above the bottom line, so the middle line is 3 line gaps above it.
    final kick = headCenter(r.hits, 36, nth: 1);
    final midlineY = kick.dy - 3 * lineGap;
    final restCenters = [
      for (final e in r.hits.entries)
        if (e.descriptor is RestSymbol) e.region.center,
    ];
    expect(restCenters, isNotEmpty);
    for (final c in restCenters) {
      expect(
        c.dy,
        greaterThan(midlineY),
        reason:
            'the feet voice\'s rests sit below the middle line, clear of '
            'the voice-1 material',
      );
    }
  });

  test('hands are blue and feet amber on the engraved heads, per the voice '
      'convention', () async {
    final r = paintStaff();
    final data = await bytes(r.image);
    const palette = NotationPalette.dark;
    final snare = headCenter(r.hits, 38); // voice 1 → hands
    final kick = headCenter(r.hits, 36, nth: 1); // voice 2 → feet
    final snarePx = pixelOf(data, snare.dx.round(), snare.dy.round());
    final kickPx = pixelOf(data, kick.dx.round(), kick.dy.round());
    expect(
      dist(snarePx, palette.handRight) < dist(snarePx, palette.handLeft),
      isTrue,
      reason: 'a hand-struck head reads blue — got $snarePx',
    );
    expect(
      dist(kickPx, palette.handLeft) < dist(kickPx, palette.handRight),
      isTrue,
      reason: 'a foot-struck head reads amber — got $kickPx',
    );
  });

  test('the hands filter hides foot events without touching the staff '
      'furniture', () async {
    final derived = notationToTimedNotes(sampleOpenGrooveDocument());
    // The player's own filter model: hands (right) selected on a percussion
    // score — the painter receives what the player would pass it.
    final player = PlayerData(
      notes: derived.notes,
      rests: derived.rests,
      tieContinuations: derived.tieContinuations,
      isPercussion: true,
      selectedHands: Hand.right,
    );
    final r = paintStaff(
      derived: derived,
      notes: player.visibleNotes,
      rests: player.visibleRests,
      tieContinuations: player.visibleTieContinuations,
    );
    // No foot event survives…
    expect(
      r.hits.entries.any(
        (e) =>
            e.descriptor is NoteSymbol &&
            (e.descriptor as NoteSymbol).pitch == 36,
      ),
      isFalse,
    );
    // …while the staff furniture stays: percussion clef, time signature and
    // the five staff lines themselves.
    expect(
      [
        for (final e in r.hits.entries)
          if (e.descriptor is ClefSymbol) (e.descriptor as ClefSymbol).sign,
      ],
      ['percussion'],
    );
    expect(
      r.hits.entries.any((e) => e.descriptor is TimeSignatureSymbol),
      isTrue,
    );
    final data = await bytes(r.image);
    final hat = headCenter(r.hits, 42, nth: 2);
    final bottomLineY = (hat.dy + 9 * stepGap).round(); // G5 → bottom line
    expect(
      inkIn(
        data,
        Rect.fromLTRB(100, bottomLineY - 1.0, width - 100, bottomLineY + 1.0),
      ),
      greaterThan(300),
      reason: 'the staff lines stay drawn under the hands filter',
    );
  });

  test('the derived schedule carries the bridge head classes and the rest '
      'voices through to the painter inputs', () {
    final derived = notationToTimedNotes(sampleOpenGrooveDocument());
    // Cymbals x / xOpen, drums oval — verbatim from the (faked) bridge.
    expect(
      derived.notes.firstWhere((n) => n.pitch == 46).headClass,
      HeadClass.xOpen,
    );
    expect(
      derived.notes.firstWhere((n) => n.pitch == 42).headClass,
      HeadClass.x,
    );
    expect(
      derived.notes.firstWhere((n) => n.pitch == 36).headClass,
      HeadClass.oval,
    );
    // Rests carry their voice (3.1): the groove's rests are all voice 2.
    expect(derived.rests, isNotEmpty);
    expect(derived.rests.every((rest) => rest.voice == 2), isTrue);
  });
}
