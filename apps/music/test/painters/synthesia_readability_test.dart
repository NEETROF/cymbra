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
import 'package:music/painters/piano_layout.dart';
import 'package:music/painters/synthesia_painter.dart';
import 'package:music/state/player_data.dart';

const double _w = 400;
const double _h = 600;

Future<ByteData> _render(List<TimedNote> notes) async {
  final recorder = ui.PictureRecorder();
  SynthesiaPainter(
    layout: const PianoLayout(width: _w),
    notes: notes,
    elapsedMs: 1000,
    activeNotes: const {},
    lookAheadMs: 3000,
  ).paint(Canvas(recorder), const Size(_w, _h));
  final image = await recorder.endRecording().toImage(_w.toInt(), _h.toInt());
  final bytes = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
  image.dispose();
  return bytes!;
}

bool _differ(ByteData a, ByteData b) {
  if (a.lengthInBytes != b.lengthInBytes) return true;
  for (var i = 0; i < a.lengthInBytes; i++) {
    if (a.getUint8(i) != b.getUint8(i)) return true;
  }
  return false;
}

void main() {
  test(
    'a merged tie chain renders its sustain differently from its attack',
    () async {
      // Same note with and without the sustain split: the tail (slimmer, dimmer,
      // no halo) must change the pixels above the attack segment.
      const plain = TimedNote(pitch: 72, startMs: 1500, durationMs: 2000);
      const split = TimedNote(
        pitch: 72,
        startMs: 1500,
        durationMs: 2000,
        sustainFromMs: 2000,
      );
      expect(
        _differ(await _render(const [plain]), await _render(const [split])),
        isTrue,
        reason: 'the sustain tail must not render as a plain full bar',
      );
    },
  );

  test('abutting same-key repeats get a wider separation notch', () async {
    // One bar alone vs the same bar with a back-to-back repeat on the same
    // key. With the repeat, the first bar's release edge is carved wider
    // (9px instead of 3px), so the seam rows just under the bar's natural top
    // change — while the attack edge (bottom) stays put.
    const bar = TimedNote(pitch: 72, startMs: 1500, durationMs: 500);
    const repeat = TimedNote(pitch: 72, startMs: 2000, durationMs: 500);
    final alone = await _render(const [bar]);
    final repeated = await _render(const [bar, repeat]);

    // Geometry: pxPerMs = 600/3000 = 0.2 → bar bottom y=500, natural top
    // y=400. The widened notch empties rows ~403..408 of the bar's column.
    final r = const PianoLayout(width: _w).keyRect(72);
    bool regionDiffer(int y0, int y1) {
      for (var y = y0; y < y1; y++) {
        for (
          var x = r.left.ceil() + 2;
          x < (r.left + r.width).floor() - 2;
          x++
        ) {
          final i = (y * _w.toInt() + x) * 4;
          for (var c = 0; c < 4; c++) {
            if (alone.getUint8(i + c) != repeated.getUint8(i + c)) return true;
          }
        }
      }
      return false;
    }

    expect(
      regionDiffer(402, 409),
      isTrue,
      reason: 'the seam under the release edge must be carved wider',
    );
    // The attack edge never moves: in both renders the bar is still painted
    // just above the strike row (y=500) and the background starts right below
    // it. (Exact pixel equality is not expected — the bar's gradient rescales
    // with its trimmed height.)
    bool paintedAt(ByteData img, int y) {
      final x = (r.left + r.width / 2).round();
      final i = (y * _w.toInt() + x) * 4;
      // The navy background is dark; a painted bar row is far brighter.
      return img.getUint8(i) + img.getUint8(i + 1) + img.getUint8(i + 2) > 120;
    }

    for (final img in [alone, repeated]) {
      expect(paintedAt(img, 497), isTrue, reason: 'bar present at its attack');
      expect(paintedAt(img, 520), isFalse, reason: 'background below attack');
    }
  });
}
