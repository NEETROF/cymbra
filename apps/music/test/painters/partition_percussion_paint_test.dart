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
import 'package:music/painters/partition_painter.dart';
import 'package:music/painters/smufl.dart';
import 'package:music/painters/staff_hit_index.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/drum_kit.dart' show kKickPieceId;
import 'package:music/state/player_data.dart' show Hand;

import '../support/notation_fakes.dart';
import '../support/test_fonts.dart';

/// Percussion engraving in the Partition (engraved) mode (change:
/// add-drum-notation-render), over the `groove_ouvert` fixture mirror —
/// pixel probes plus the painter's hit index, the house style.
///
/// The "parity" tests cover the same named facts the console painter's SVG
/// suite asserts (clef glyph, written positions, head classes), so a rule
/// change that lands on one side only breaks a named test on the other.
void main() {
  setUpAll(loadBravura);

  const width = 600;
  const height = 420;
  const s = 12.0; // the painter's default staff space

  ({ui.Image image, StaffHitIndex hits, PartitionPainter painter})
  paintPartition(
    ScoreDocument document, {
    Hand selectedHands = Hand.both,
    Set<String> mutedDrumPieces = const {},
    NotationPalette palette = NotationPalette.dark,
    List<MeasureHit>? hitRects,
  }) {
    final hits = StaffHitIndex();
    final painter = PartitionPainter(
      document: document,
      systems: FakeNotationEngine().layout(document, width.toDouble()),
      selectedHands: selectedHands,
      mutedDrumPieces: mutedDrumPieces,
      palette: palette,
      hitIndex: hits,
      hitRects: hitRects,
    );
    final recorder = ui.PictureRecorder();
    painter.paint(Canvas(recorder), Size(width.toDouble(), height.toDouble()));
    final image = recorder.endRecording().toImageSync(width, height);
    return (image: image, hits: hits, painter: painter);
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

  int dist(Color a, Color b) =>
      (((a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs()) * 255)
          .round();

  List<Offset> headCenters(StaffHitIndex hits, int gm) => [
    for (final e in hits.entries)
      if (e.descriptor is NoteSymbol &&
          (e.descriptor as NoteSymbol).pitch == gm)
        e.region.center,
  ];

  Offset headCenter(StaffHitIndex hits, int gm, {int nth = 0}) {
    final centers = headCenters(hits, gm);
    expect(
      centers.length,
      greaterThan(nth),
      reason: 'no head recorded for GM $gm (occurrence $nth)',
    );
    return centers[nth];
  }

  test('parity: the percussion clef opens every system and no armature or '
      'key-change naturals draw even when the file declares fifths = 2', () {
    final r = paintPartition(sampleOpenGrooveDocument(keyFifths: 2));
    final clefSigns = [
      for (final e in r.hits.entries)
        if (e.descriptor is ClefSymbol) (e.descriptor as ClefSymbol).sign,
    ];
    expect(clefSigns, isNotEmpty);
    expect(clefSigns.toSet(), {'percussion'});
    expect(
      r.hits.entries.any((e) => e.descriptor is KeySignatureSymbol),
      isFalse,
    );
    // The time signature still draws as on any staff.
    expect(
      r.hits.entries.any((e) => e.descriptor is TimeSignatureSymbol),
      isTrue,
    );
  });

  test('control: a keyboard partition DOES record its armature (the '
      'percussion assert is not vacuous)', () {
    final r = paintPartition(sampleGrandStaffDocument());
    expect(
      r.hits.entries.any((e) => e.descriptor is KeySignatureSymbol),
      isTrue,
    );
  });

  test('parity: snare and kick engrave at their written positions, never '
      'from the GM number', () {
    final r = paintPartition(sampleOpenGrooveDocument());
    final snare = headCenter(r.hits, 38);
    final kick = headCenter(r.hits, 36);
    final hat = headCenter(r.hits, 42, nth: 1); // an eighth, past the chord
    // C5 sits 4 diatonic steps above F4, G5 four above C5.
    expect(kick.dy - snare.dy, closeTo(4 * (s / 2), 0.6));
    expect(snare.dy - hat.dy, closeTo(4 * (s / 2), 0.6));
  });

  test('parity: cymbals take x heads and drums oval heads', () async {
    final r = paintPartition(sampleOpenGrooveDocument());
    final data = await bytes(r.image);
    final snare = headCenter(r.hits, 38); // oval eighth
    final hat = headCenter(r.hits, 42, nth: 1); // x eighth
    Rect box(Offset c) => Rect.fromCenter(
      center: c,
      width: Smufl.noteheadWidth * s + 2,
      height: s - 2,
    );
    final oval = inkIn(data, box(snare));
    final x = inkIn(data, box(hat));
    expect(oval, greaterThan(0));
    expect(x, greaterThan(0));
    expect(
      x,
      lessThan(oval * 0.8),
      reason:
          'the hi-hat head must be the x form, not a filled oval — got '
          '$x ink px vs the snare\'s $oval',
    );
  });

  test('the open mark rides GM 46 and is absent on GM 42', () async {
    final r = paintPartition(sampleOpenGrooveDocument());
    final data = await bytes(r.image);
    final open = headCenter(r.hits, 46);
    final closed = headCenter(r.hits, 42, nth: 1);
    Rect above(Offset c) => Rect.fromCenter(
      center: Offset(c.dx, c.dy - s * 1.6),
      width: s * 0.9,
      height: s,
    );
    expect(inkIn(data, above(open)), greaterThan(0));
    expect(inkIn(data, above(closed)), 0);
  });

  test('a voice-2 rest is displaced below the middle line in a two-voice '
      'measure; a single-voice measure keeps the midline', () {
    final r = paintPartition(sampleOpenGrooveDocument());
    final hat = headCenter(r.hits, 42, nth: 1); // G5, 2.5 gaps above midline
    final midlineY = hat.dy + 2.5 * s;
    final restCenters = [
      for (final e in r.hits.entries)
        if (e.descriptor is RestSymbol) e.region.center,
    ];
    expect(restCenters, isNotEmpty);
    for (final c in restCenters) {
      expect(c.dy, greaterThan(midlineY));
    }

    // Single-voice control: the same rest with no second voice sits ON the
    // midline (its record centre is half a space above the baseline).
    final single = paintPartition(_singleVoicePercussionWithRest());
    final singleHat = headCenter(single.hits, 42);
    final singleMidline = singleHat.dy + 2.5 * s;
    final singleRest = single.hits.entries
        .firstWhere((e) => e.descriptor is RestSymbol)
        .region
        .center;
    expect(singleRest.dy, closeTo(singleMidline - 0.5 * s, 1.0));
  });

  test('bare notes stem by voice — voice 1 up, voice 2 down — and a shared '
      'same-position onset offsets the second head so neither hides', () async {
    final doc = samplePercussionSharedPositionDocument();
    final r = paintPartition(doc);
    final data = await bytes(r.image);
    final tom = headCenter(r.hits, 41); // voice 1, no explicit stem
    final kick = headCenter(r.hits, 36); // voice 2, no explicit stem
    // Same written position (F4): the second head steps right of the first.
    expect(kick.dy, closeTo(tom.dy, 0.6));
    expect(
      (kick.dx - tom.dx).abs(),
      greaterThan(Smufl.noteheadWidth * s * 0.9),
      reason: 'coinciding same-position heads must not overprint',
    );
    // Voice 1 stems up from the head's right anchor…
    expect(
      inkIn(
        data,
        Rect.fromCenter(
          center: Offset(tom.dx + 0.59 * s, tom.dy - 2 * s),
          width: 4,
          height: s,
        ),
      ),
      greaterThan(0),
      reason: 'voice 1 (hands) stems up by default',
    );
    // …voice 2 stems down from the left anchor.
    expect(
      inkIn(
        data,
        Rect.fromCenter(
          center: Offset(kick.dx - 0.59 * s, kick.dy + 2 * s),
          width: 4,
          height: s,
        ),
      ),
      greaterThan(0),
      reason: 'voice 2 (feet) stems down by default',
    );
  });

  test('hands blue, feet amber — and the Paper theme keeps its darkened '
      'hand palette on engraved percussion', () async {
    for (final palette in [NotationPalette.dark, NotationPalette.paper]) {
      final r = paintPartition(sampleOpenGrooveDocument(), palette: palette);
      final data = await bytes(r.image);
      final snare = headCenter(r.hits, 38); // voice 1 → hands
      final kick = headCenter(r.hits, 36); // voice 2 → feet
      final snarePx = pixelOf(data, snare.dx.round(), snare.dy.round());
      final kickPx = pixelOf(data, kick.dx.round(), kick.dy.round());
      expect(
        dist(snarePx, palette.handRight) < dist(snarePx, palette.handLeft),
        isTrue,
        reason: 'hands read blue under $palette — got $snarePx',
      );
      expect(
        dist(kickPx, palette.handLeft) < dist(kickPx, palette.handRight),
        isTrue,
        reason: 'feet read amber under $palette — got $kickPx',
      );
    }
  });

  // The engraved Partition draws from the DOCUMENT, not from `visibleNotes`,
  // so it is the one surface that re-applies the filter itself — and it takes
  // the selection as data so it applies the same rule rather than a second one
  // (change: add-practice-focus-controls, task 3.5).
  test('muting a piece never blanks the canvas: the staff, clef and time '
      'signature stay drawn while that piece hides', () async {
    final doc = sampleOpenGrooveDocument();
    final both = paintPartition(doc);
    final hands = paintPartition(doc, mutedDrumPieces: const {kKickPieceId});
    // Foot events hidden…
    expect(headCenters(hands.hits, 36), isEmpty);
    expect(headCenters(hands.hits, 42), isNotEmpty);
    // …no staff collapsed: the single percussion staff keeps its height and
    // its furniture.
    expect(
      hands.painter.heightFor(width.toDouble()),
      both.painter.heightFor(width.toDouble()),
    );
    expect(
      {
        for (final e in hands.hits.entries)
          if (e.descriptor is ClefSymbol) (e.descriptor as ClefSymbol).sign,
      },
      {'percussion'},
    );
    expect(
      hands.hits.entries.any((e) => e.descriptor is TimeSignatureSymbol),
      isTrue,
    );
    // The five staff lines are still stroked: the hi-hat head sits 4.5 line
    // gaps above the bottom line — probe that row for a long run of ink.
    final data = await bytes(hands.image);
    final hat = headCenter(hands.hits, 42);
    final bottomLineY = hat.dy + 4.5 * s;
    expect(
      inkIn(
        data,
        Rect.fromLTRB(20, bottomLineY - 1, width - 20, bottomLineY + 1),
      ),
      greaterThan(300),
      reason: 'selecting hands must never blank the percussion staff',
    );
  });

  test('measure-select geometry: taps on a percussion partition resolve '
      'written measures through the recorded hit rects', () {
    final rects = <MeasureHit>[];
    paintPartition(sampleOpenGrooveDocument(), hitRects: rects);
    expect({for (final h in rects) h.measure}, {0, 1});
    final second = rects.firstWhere((h) => h.measure == 1);
    expect(measureAtPosition(rects, second.rect.center), 1);
    final first = rects.firstWhere((h) => h.measure == 0);
    expect(measureAtPosition(rects, first.rect.center), 0);
  });

  test(
    'the engraved filter collapses a piece exactly as the kit model does',
    () async {
      final doc = sampleOpenGrooveDocument();
      // Muting the hi-hat takes the closed AND the open stroke — one pad, one
      // answer — while the snare and the kick stay engraved.
      final noHat = paintPartition(
        doc,
        mutedDrumPieces: const {'kitPieceHiHat'},
      );
      expect(headCenters(noHat.hits, 42), isEmpty);
      expect(headCenters(noHat.hits, 46), isEmpty);
      expect(headCenters(noHat.hits, 36), isNotEmpty);
      // An empty selection engraves the whole kit.
      final all = paintPartition(doc);
      expect(headCenters(all.hits, 42), isNotEmpty);
      expect(headCenters(all.hits, 36), isNotEmpty);
    },
  );
}

/// A single-voice percussion measure with a rest, for the midline control:
/// hi-hat eighths around a quarter rest, all voice 1.
ScoreDocument _singleVoicePercussionWithRest() => ScoreDocument(
  instruments: const [
    InstrumentDecl(id: 'P1-I42', name: 'Closed Hi-Hat', gmNumber: 42),
  ],
  playOrder: const [],
  meta: const ScoreMeta(title: 'SingleVoice', composer: 'Tester'),
  staves: 1,
  attributes: const Attributes(
    divisions: 4,
    clefs: [Clef(staff: 1, sign: ClefSign.percussion, line: 2)],
    keyFifths: 0,
    time: TimeSignature(beats: 4, beatType: 4),
  ),
  measures: [
    NotationMeasure(
      repeats: noRepeats,
      index: 0,
      clefs: const [],
      keyFifths: 0,
      minWidth: 160,
      directions: const [],
      notes: [
        for (var i = 0; i < 4; i++)
          noteEvent(
            positionDivisions: i * 2,
            durationDivisions: 2,
            noteType: 'eighth',
            stem: StemDir.up,
            unpitched: const Unpitched(
              displayStep: 'G',
              displayOctave: 5,
              gmNumber: 42,
              headClass: HeadClass.x,
            ),
            instrumentId: 'P1-I42',
          ),
        noteEvent(
          positionDivisions: 8,
          isRest: true,
          durationDivisions: 4,
          noteType: 'quarter',
        ),
        noteEvent(
          positionDivisions: 12,
          durationDivisions: 4,
          noteType: 'quarter',
          stem: StemDir.up,
          unpitched: const Unpitched(
            displayStep: 'G',
            displayOctave: 5,
            gmNumber: 42,
            headClass: HeadClass.x,
          ),
          instrumentId: 'P1-I42',
        ),
      ],
    ),
  ],
);
