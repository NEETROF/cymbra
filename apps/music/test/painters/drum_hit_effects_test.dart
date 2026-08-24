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
import 'package:music/painters/drum_hit_effects_painter.dart';
import 'package:music/state/drum_kit.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';

/// Pixel-probe proof of where a percussion hit spark lands (change:
/// add-drum-scoring): a hand stroke on ITS lane at the hit line, a kick across
/// the FULL-WIDTH bar. A spark on the wrong column would tell the player they
/// aimed somewhere they did not, which is the one thing hit feedback must
/// never do — and the defect is invisible on a one-lane score.
void main() {
  const width = 400;
  const height = 300;
  // Two lanes: hi-hat (centre x = 100) then snare (centre x = 300).
  final lanes = deriveDrumLanes(const [
    TimedNote(pitch: 42, startMs: 0, durationMs: 100),
    TimedNote(pitch: 38, startMs: 0, durationMs: 100),
    TimedNote(pitch: 36, startMs: 0, durationMs: 100),
  ]);

  Future<ui.Image> paintHit(int pitch, {bool wrong = false}) async {
    final recorder = ui.PictureRecorder();
    final canvas = Canvas(recorder);
    DrumHitEffectsPainter(
      lanes: lanes,
      hits: [
        HitEffect(
          pitch: pitch,
          verdict: TimingVerdict.perfect,
          wrong: wrong,
          atMs: 0,
        ),
      ],
      // Half-way through the 600 ms fade: the spark is at full spread and
      // still clearly opaque.
      nowMs: 300,
    ).paint(canvas, const Size(width * 1.0, height * 1.0));
    return recorder.endRecording().toImage(width, height);
  }

  Future<int> alphaAt(ui.Image image, int x, int y) async {
    final data = (await image.toByteData(format: ui.ImageByteFormat.rawRgba))!;
    return data.getUint8((y * width + x) * 4 + 3);
  }

  test('a hand stroke sparks on its own lane and nowhere else', () async {
    final image = await paintHit(38);
    // Just above the hit line, in the snare lane: lit.
    expect(await alphaAt(image, 300, 295), greaterThan(0));
    // The hi-hat lane, at the same height: untouched.
    expect(await alphaAt(image, 100, 295), 0);
    // And nothing is drawn up in the falling-note area.
    expect(await alphaAt(image, 300, 40), 0);
  });

  test('the hi-hat sparks on the hi-hat lane', () async {
    final image = await paintHit(42);
    expect(await alphaAt(image, 100, 295), greaterThan(0));
    expect(await alphaAt(image, 300, 295), 0);
  });

  test('a kick sparks across the full-width bar', () async {
    final image = await paintHit(36);
    // The bar is a note in a different shape: the spark spans every column.
    expect(await alphaAt(image, 8, 295), greaterThan(0));
    expect(await alphaAt(image, 100, 295), greaterThan(0));
    expect(await alphaAt(image, 300, 295), greaterThan(0));
    expect(await alphaAt(image, width - 8, 295), greaterThan(0));
    // …and stays at the hit line rather than washing the surface.
    expect(await alphaAt(image, 200, 200), 0);
  });

  test('the other kick number sparks on the bar too', () async {
    final image = await paintHit(35);
    expect(await alphaAt(image, 200, 295), greaterThan(0));
  });

  test('an equivalent number sparks on its piece lane', () async {
    // The score writes no 40 anywhere, yet the electric snare is a snare: the
    // spark resolves at the same grain the scorer bound it with.
    final image = await paintHit(40);
    expect(await alphaAt(image, 300, 295), greaterThan(0));
    expect(await alphaAt(image, 100, 295), 0);
  });

  test('a stroke of a piece the layout does not present draws nothing', () {
    // No lane, no bar — free play sparks nowhere rather than guessing.
    expect(pieceLaneIndexOf(lanes, 47), isNull);
  });

  test('a faded-out hit draws nothing at all', () async {
    final recorder = ui.PictureRecorder();
    DrumHitEffectsPainter(
      lanes: lanes,
      hits: const [
        HitEffect(
          pitch: 38,
          verdict: TimingVerdict.perfect,
          wrong: false,
          atMs: 0,
        ),
      ],
      nowMs: 5000, // long past the fade
    ).paint(Canvas(recorder), const Size(width * 1.0, height * 1.0));
    final image = await recorder.endRecording().toImage(width, height);
    expect(await alphaAt(image, 300, 295), 0);
  });
}
