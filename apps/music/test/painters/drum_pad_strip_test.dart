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

import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/drum_pad_strip_painter.dart';
import 'package:music/state/drum_kit.dart';
import 'package:music/state/player_data.dart';

/// The pad strip as a **controller** (change: add-drum-input-mapping): where a
/// pointer lands (no dead gutters), and the one honest feedback state — a
/// time-based struck flash that claims nothing about correctness.
void main() {
  const size = Size(400, 120);
  final lanes = deriveDrumLanes([
    for (final gm in [42, 38, 49])
      TimedNote(pitch: gm, startMs: 0, durationMs: 100),
  ]);

  group('hit testing — the whole strip is live', () {
    int? at(double dx, double dy, {bool hasKick = true, int? laneCount}) =>
        DrumPadStripPainter.surfaceAt(
          Offset(dx, dy),
          size,
          laneCount: laneCount ?? lanes.length,
          hasKick: hasKick,
        );

    // The pads band is the top 70% when a pedal is drawn (pedalFraction .30).
    const padsHeight = 120 * (1 - DrumPadStripPainter.pedalFraction);

    test('a pointer hits the lane whose horizontal span contains it', () {
      expect(at(10, 10), 0);
      expect(at(200, 10), 1); // middle third
      expect(at(390, 10), 2);
    });

    test('a tap in the gutter between two drawn pads still strikes', () {
      // The painter insets each pad by 3 px, so 133.5 ± 2 falls between the
      // first pad's right edge and the second's left edge — decoration, never
      // a hit boundary: a swallowed tap would be a ghost stroke.
      const boundary = 400 / 3;
      expect(at(boundary - 1, 10), 0);
      expect(at(boundary + 1, 10), 1);
      // The very edges of the strip belong to the outermost lanes.
      expect(at(0, 0), 0);
      expect(at(399.9, padsHeight - 1), 2);
    });

    test('anywhere in the pedal band is the kick', () {
      expect(at(0, padsHeight), kPedalSurface);
      expect(at(200, padsHeight + 5), kPedalSurface);
      expect(at(399, 119.9), kPedalSurface);
      // Without a kick the pads own the full height instead.
      expect(at(200, 119, hasKick: false), 1);
    });

    test('nothing outside the strip, and nothing where no pad is drawn', () {
      expect(at(-1, 10), isNull);
      expect(at(10, -1), isNull);
      expect(at(400, 10), isNull);
      expect(at(10, 120), isNull);
      // An all-kick score draws only the pedal: the empty pads band strikes
      // nothing rather than inventing a lane.
      expect(at(10, 10, laneCount: 0), isNull);
      expect(at(10, padsHeight + 1, laneCount: 0), kPedalSurface);
    });
  });

  group('the struck flash — one state, decaying on its own clock', () {
    double intensity(double ageMs) =>
        DrumPadStripPainter.flashIntensity(struckMs: 1000, nowMs: 1000 + ageMs);

    test('full at the attack, decayed to nothing by the flash duration', () {
      expect(intensity(0), 1.0);
      expect(
        intensity(DrumPadStripPainter.flashDurationMs / 2),
        closeTo(0.5, 0.001),
      );
      expect(intensity(DrumPadStripPainter.flashDurationMs.toDouble()), 0.0);
      expect(intensity(DrumPadStripPainter.flashDurationMs * 10), 0.0);
      // A stamp in the future (clock skew) shows nothing rather than a
      // permanent glow.
      expect(intensity(-5), 0.0);
    });

    test('the decay depends on nothing but the age of the attack', () {
      // Not on the release — a note-off carries no feedback meaning — and not
      // on the playhead: two strokes of the same age look identical whatever
      // happened around them. This is what makes the flash honest: it says
      // "struck", never "right".
      final onOnset = DrumPadStripPainter.flashIntensity(
        struckMs: 5000,
        nowMs: 5060,
      );
      final farFromAny = DrumPadStripPainter.flashIntensity(
        struckMs: 90000,
        nowMs: 90060,
      );
      expect(onOnset, farFromAny);
    });
  });

  group('painting the flash', () {
    Future<ui.Image> paint({
      Map<int, double> struck = const {},
      double nowMs = 0,
    }) {
      final recorder = ui.PictureRecorder();
      DrumPadStripPainter(
        lanes: lanes,
        labels: const ['HH', 'SN', 'CR'],
        kickLabel: 'Kick',
        hasKick: true,
        struckMs: struck,
        nowMs: nowMs,
      ).paint(Canvas(recorder), size);
      return recorder.endRecording().toImage(
        size.width.toInt(),
        size.height.toInt(),
      );
    }

    Future<Color> pixel(ui.Image image, int x, int y) async {
      final data = (await image.toByteData(
        format: ui.ImageByteFormat.rawRgba,
      ))!;
      final offset = (y * size.width.toInt() + x) * 4;
      return Color.fromARGB(
        data.getUint8(offset + 3),
        data.getUint8(offset),
        data.getUint8(offset + 1),
        data.getUint8(offset + 2),
      );
    }

    test('a struck pad reads differently from its idle self, and its '
        'neighbours are untouched', () async {
      final idle = await paint();
      final struck = await paint(struck: {1: 1000}, nowMs: 1000);
      final idleSnare = await pixel(idle, 200, 30);
      final litSnare = await pixel(struck, 200, 30);
      expect(litSnare, isNot(idleSnare));
      // Only the struck pad changed — a stroke lights one surface.
      expect(await pixel(struck, 60, 30), await pixel(idle, 60, 30));
      expect(await pixel(struck, 340, 30), await pixel(idle, 340, 30));
    });

    test(
      'the pedal flashes like a pad, and an expired stamp does not',
      () async {
        final idle = await paint();
        final struckPedal = await paint(
          struck: {kPedalSurface: 1000},
          nowMs: 1000,
        );
        final expired = await paint(
          struck: {kPedalSurface: 1000},
          nowMs: 1000 + DrumPadStripPainter.flashDurationMs + 1,
        );
        const y = 100; // inside the pedal band
        expect(
          await pixel(struckPedal, 200, y),
          isNot(await pixel(idle, 200, y)),
        );
        expect(await pixel(expired, 200, y), await pixel(idle, 200, y));
      },
    );

    test('repaints while a flash is live, and stops once it has decayed', () {
      const base = DrumPadStripPainter(
        lanes: [],
        labels: [],
        kickLabel: 'Kick',
        hasKick: true,
        struckMs: {0: 1000},
        nowMs: 1050,
      );
      // Same struck table, a later frame: still flashing ⇒ repaint.
      expect(
        const DrumPadStripPainter(
          lanes: [],
          labels: [],
          kickLabel: 'Kick',
          hasKick: true,
          struckMs: {0: 1000},
          nowMs: 1100,
        ).shouldRepaint(base),
        isTrue,
      );
      // Both frames past the decay: the still strip does not repaint per frame.
      const settled = DrumPadStripPainter(
        lanes: [],
        labels: [],
        kickLabel: 'Kick',
        hasKick: true,
        struckMs: {0: 1000},
        nowMs: 9000,
      );
      expect(
        const DrumPadStripPainter(
          lanes: [],
          labels: [],
          kickLabel: 'Kick',
          hasKick: true,
          struckMs: {0: 1000},
          nowMs: 9100,
        ).shouldRepaint(settled),
        isFalse,
      );
    });
  });
}
