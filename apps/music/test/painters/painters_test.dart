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

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/piano_keyboard_painter.dart';
import 'package:music/painters/piano_layout.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/painters/synthesia_painter.dart';
import 'package:music/state/player_state.dart';

/// Renders [painter] into a keyed RepaintBoundary for golden comparison.
Widget _host(CustomPainter painter, Size size) => Directionality(
  textDirection: TextDirection.ltr,
  child: Center(
    child: RepaintBoundary(
      key: const Key('golden'),
      child: SizedBox.fromSize(
        size: size,
        child: CustomPaint(size: size, painter: painter),
      ),
    ),
  ),
);

const _layout = PianoLayout(width: 600);
const _notes = [
  TimedNote(pitch: 60, startMs: 0, durationMs: 500),
  TimedNote(pitch: 64, startMs: 500, durationMs: 500),
  TimedNote(pitch: 67, startMs: 1000, durationMs: 500),
];

void main() {
  // Golden tests are pixel comparisons and are not reliable across platforms
  // (generated on macOS, CI is Linux). They're tagged so the cross-platform CI
  // gate excludes them (`--exclude-tags golden`); run/refresh locally with
  // `flutter test --tags golden --update-goldens`. Painter paint() is still
  // covered in CI by the widget tests, which render all three painters.
  group('golden renders', () {
    testWidgets('piano keyboard', tags: 'golden', (tester) async {
      await tester.pumpWidget(
        _host(
          const PianoKeyboardPainter(layout: _layout, activeNotes: {60, 64}),
          const Size(600, 160),
        ),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/piano_keyboard.png'),
      );
    });

    testWidgets('synthesia waterfall', tags: 'golden', (tester) async {
      await tester.pumpWidget(
        _host(
          const SynthesiaPainter(
            layout: _layout,
            notes: _notes,
            elapsedMs: 250,
            activeNotes: {60},
          ),
          const Size(600, 400),
        ),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/synthesia.png'),
      );
    });

    testWidgets('staff', tags: 'golden', (tester) async {
      await tester.pumpWidget(
        _host(
          const StaffPainter(
            notes: _notes,
            elapsedMs: 250,
            activeNotes: {60},
            bpm: 80,
            songEndMs: 1500,
          ),
          const Size(600, 300),
        ),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/staff.png'),
      );
    });
  });

  group('shouldRepaint', () {
    test('keyboard repaints only when inputs change', () {
      const notes = {60};
      const a = PianoKeyboardPainter(layout: _layout, activeNotes: notes);
      const b = PianoKeyboardPainter(layout: _layout, activeNotes: notes);
      expect(a.shouldRepaint(b), isFalse);
      const c = PianoKeyboardPainter(layout: _layout, activeNotes: {61});
      expect(a.shouldRepaint(c), isTrue);
    });

    test('synthesia repaints when the playhead moves', () {
      const active = {60};
      const a = SynthesiaPainter(
        layout: _layout,
        notes: _notes,
        elapsedMs: 0,
        activeNotes: active,
      );
      const b = SynthesiaPainter(
        layout: _layout,
        notes: _notes,
        elapsedMs: 100,
        activeNotes: active,
      );
      expect(a.shouldRepaint(b), isTrue);
    });

    test('staff repaints when the playhead moves', () {
      const active = {60};
      const a = StaffPainter(
        notes: _notes,
        elapsedMs: 0,
        activeNotes: active,
        bpm: 80,
        songEndMs: 1500,
      );
      const b = StaffPainter(
        notes: _notes,
        elapsedMs: 100,
        activeNotes: active,
        bpm: 80,
        songEndMs: 1500,
      );
      expect(a.shouldRepaint(b), isTrue);
    });
  });
}
