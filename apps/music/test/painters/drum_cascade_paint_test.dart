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
import 'package:music/painters/drum_cascade_painter.dart';
import 'package:music/state/drum_kit.dart';
import 'package:music/state/player_data.dart';
import 'package:music/theme/cymbra_theme.dart';

/// Pixel-probe proof of the cascade's paint order (change: add-drum-kit-view).
///
/// The defect this pins is only observable on a COINCIDENCE: a kick bar
/// painted over the note layer hides the hand note exactly where the foot and
/// the hand land together — the very information the bar exists to convey —
/// and a score where they never align renders perfectly with the layers
/// inverted. So the probe uses a kick sharing its onset with a snare stroke
/// and samples real pixels, robust across platforms where a golden is not.
void main() {
  const width = 400;
  const height = 300;

  Future<ui.Image> paintCascade(List<TimedNote> notes) async {
    final lanes = deriveDrumLanes(notes);
    final recorder = ui.PictureRecorder();
    final canvas = Canvas(recorder);
    DrumCascadePainter(
      lanes: lanes,
      notes: notes,
      multiVoice: spansMultipleVoices(notes),
      elapsedMs: 0,
      lookAheadMs: 3000,
    ).paint(canvas, const Size(width * 1.0, height * 1.0));
    return recorder.endRecording().toImage(width, height);
  }

  Future<Color> pixel(ui.Image image, int x, int y) async {
    final data = (await image.toByteData(format: ui.ImageByteFormat.rawRgba))!;
    final offset = (y * width + x) * 4;
    return Color.fromARGB(
      data.getUint8(offset + 3),
      data.getUint8(offset),
      data.getUint8(offset + 1),
      data.getUint8(offset + 2),
    );
  }

  /// Channel distance between two colours, for "reads as" assertions.
  int dist(Color a, Color b) =>
      (((a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs()) * 255)
          .round();

  test('a coinciding kick and hand stroke: the note stays legible over the '
      'bar, and the bar stays visible away from the notes', () async {
    // Two lanes (hi-hat, snare). At t=1500ms: a kick AND a snare stroke share
    // the onset; the hi-hat lane is empty there, so the bar crosses it in the
    // clear.
    final notes = [
      TimedNote(pitch: 42, startMs: 0, durationMs: 200, voice: 1),
      TimedNote(pitch: 38, startMs: 1500, durationMs: 300, voice: 1),
      TimedNote(pitch: 36, startMs: 1500, durationMs: 200, voice: 2),
    ];
    final image = await paintCascade(notes);

    // The onset's hit-line y: bottom = height - (start/lookAhead)*height.
    const onsetY = height - (1500 / 3000) * height; // 150
    // Sample INSIDE the snare note's body, just above its attack edge.
    final snareLaneCenterX = (width * 3 / 4).round(); // lane 2 of 2
    final noteProbe = await pixel(image, snareLaneCenterX, onsetY.round() - 4);
    // Sample the bar where nothing sits over it: the hi-hat lane's clear
    // stretch at the same instant.
    final hiHatLaneCenterX = (width * 1 / 4).round();
    final barProbe = await pixel(image, hiHatLaneCenterX, onsetY.round() - 3);
    final background = await pixel(image, hiHatLaneCenterX, 10);

    // The note reads as the HAND colour — not the amber bar, not background:
    // the note was painted over the bar, so blue dominates at its center.
    expect(
      dist(noteProbe, CymbraColors.handRight) <
          dist(noteProbe, CymbraColors.handLeft),
      isTrue,
      reason:
          'the coinciding hand note must stay legible over the kick bar '
          '(paint order inverted?) — got $noteProbe',
    );
    // The bar is clearly visible across the empty lane (not background).
    expect(
      dist(barProbe, background) > 40,
      isTrue,
      reason: 'the kick bar must stay visible away from the notes',
    );
    // And it reads amber-ish (the foot colour), not blue.
    expect(
      dist(barProbe, CymbraColors.handLeft) <
          dist(barProbe, CymbraColors.handRight),
      isTrue,
      reason: 'the bar carries the foot colour',
    );
  });

  test('the open hi-hat renders hollow: its lane center stays background '
      'while a closed stroke fills it', () async {
    final notes = [
      // Same lane, two instants: closed at 1500ms, open at 600ms.
      TimedNote(pitch: 42, startMs: 1500, durationMs: 300, voice: 1),
      TimedNote(pitch: 46, startMs: 600, durationMs: 300, voice: 1),
    ];
    final image = await paintCascade(notes);
    const closedY = height - (1500 / 3000) * height - 8;
    const openY = height - (600 / 3000) * height - 8;
    final laneX = (width / 2).round(); // single lane spans the width
    final closedProbe = await pixel(image, laneX, closedY.round());
    final openProbe = await pixel(image, laneX, openY.round());
    final background = await pixel(image, laneX, 5);

    expect(
      dist(closedProbe, background) > 40,
      isTrue,
      reason: 'a closed stroke is a filled note',
    );
    expect(
      dist(openProbe, background) < 40,
      isTrue,
      reason:
          'an open stroke is hollow — its center shows the background '
          'through the outline',
    );
  });
}
