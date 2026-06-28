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
import 'package:music/src/rust/api/musicxml.dart' show BeamState;
import 'package:music/state/player_data.dart';

import '../support/test_fonts.dart';

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

// A grand-staff run of beamed eighth notes (treble + bass) at 80 bpm
// (eighth = 375 ms), to verify the Staff painter ligatures beamed groups.
const _beamedStaffNotes = [
  TimedNote(
    pitch: 64,
    startMs: 0,
    durationMs: 375,
    staff: 1,
    beams: [BeamState.begin],
  ),
  TimedNote(
    pitch: 67,
    startMs: 375,
    durationMs: 375,
    staff: 1,
    beams: [BeamState.continue_],
  ),
  TimedNote(
    pitch: 72,
    startMs: 750,
    durationMs: 375,
    staff: 1,
    beams: [BeamState.continue_],
  ),
  TimedNote(
    pitch: 67,
    startMs: 1125,
    durationMs: 375,
    staff: 1,
    beams: [BeamState.end],
  ),
  TimedNote(
    pitch: 48,
    startMs: 0,
    durationMs: 375,
    staff: 2,
    clefSign: 'F',
    clefLine: 4,
    beams: [BeamState.begin],
  ),
  TimedNote(
    pitch: 52,
    startMs: 375,
    durationMs: 375,
    staff: 2,
    clefSign: 'F',
    clefLine: 4,
    beams: [BeamState.continue_],
  ),
  TimedNote(
    pitch: 55,
    startMs: 750,
    durationMs: 375,
    staff: 2,
    clefSign: 'F',
    clefLine: 4,
    beams: [BeamState.end],
  ),
];

// Two 3/4 measures (Minuet in G, bpm 90 → quarter 666 ms, eighth 333 ms) with a
// real measure table, so the bar lines must land on the measure boundaries
// (0, 2000, 4000) — not on a hardcoded 4-beat spacing.
const _threeFourMeasureStarts = [0, 2000, 4000];
const _threeFourNotes = [
  // Measure 1 (treble): D5 quarter, then G4 A4 B4 C5 beamed eighths.
  TimedNote(pitch: 74, startMs: 0, durationMs: 666),
  TimedNote(pitch: 67, startMs: 666, durationMs: 333, beams: [BeamState.begin]),
  TimedNote(
    pitch: 69,
    startMs: 1000,
    durationMs: 333,
    beams: [BeamState.continue_],
  ),
  TimedNote(
    pitch: 71,
    startMs: 1333,
    durationMs: 333,
    beams: [BeamState.continue_],
  ),
  TimedNote(pitch: 72, startMs: 1666, durationMs: 333, beams: [BeamState.end]),
  // Measure 2 (treble): D5, G4, G4 quarters.
  TimedNote(pitch: 74, startMs: 2000, durationMs: 666),
  TimedNote(pitch: 67, startMs: 2666, durationMs: 666),
  TimedNote(pitch: 67, startMs: 3333, durationMs: 666),
  // Bass quarters across both measures.
  TimedNote(
    pitch: 55,
    startMs: 0,
    durationMs: 666,
    staff: 2,
    clefSign: 'F',
    clefLine: 4,
  ),
  TimedNote(
    pitch: 59,
    startMs: 666,
    durationMs: 666,
    staff: 2,
    clefSign: 'F',
    clefLine: 4,
  ),
  TimedNote(
    pitch: 55,
    startMs: 1333,
    durationMs: 666,
    staff: 2,
    clefSign: 'F',
    clefLine: 4,
  ),
  TimedNote(
    pitch: 59,
    startMs: 2000,
    durationMs: 666,
    staff: 2,
    clefSign: 'F',
    clefLine: 4,
  ),
  TimedNote(
    pitch: 60,
    startMs: 2666,
    durationMs: 666,
    staff: 2,
    clefSign: 'F',
    clefLine: 4,
  ),
  TimedNote(
    pitch: 62,
    startMs: 3333,
    durationMs: 666,
    staff: 2,
    clefSign: 'F',
    clefLine: 4,
  ),
];

void main() {
  // Golden tests are pixel comparisons and are not reliable across platforms
  // (generated on macOS, CI is Linux). They're tagged so the cross-platform CI
  // gate excludes them (`--exclude-tags golden`); run/refresh locally with
  // `flutter test --tags golden --update-goldens`. Painter paint() is still
  // covered in CI by the widget tests, which render all three painters.
  group('golden renders', () {
    setUpAll(loadBravura);
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

    testWidgets('piano keyboard feedback states', tags: 'golden', (
      tester,
    ) async {
      await tester.pumpWidget(
        _host(
          // 60 correct (green), 64 expected (teal), 62 pressed-only (purple),
          // 61 expected black key (teal + outline).
          const PianoKeyboardPainter(
            layout: _layout,
            activeNotes: {60, 62},
            requiredNotes: {60, 64, 61},
          ),
          const Size(600, 160),
        ),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/piano_keyboard_feedback.png'),
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

    testWidgets('staff beamed grand staff', tags: 'golden', (tester) async {
      await tester.pumpWidget(
        _host(
          const StaffPainter(
            notes: _beamedStaffNotes,
            elapsedMs: 0,
            activeNotes: {},
            bpm: 80,
            songEndMs: 1500,
            keyFifths: 3, // armature: 3 sharps
          ),
          const Size(700, 360),
        ),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/staff_beamed.png'),
      );
    });

    // Single-hand collapse: feeding the Staff painter only one hand's notes (as
    // the screen does via `visibleNotes`) draws a lone, recentred staff with the
    // kept hand's clef/armature — right hand keeps the treble, left the bass.
    testWidgets('staff right-hand only (collapsed)', tags: 'golden', (
      tester,
    ) async {
      await tester.pumpWidget(
        _host(
          StaffPainter(
            notes: _beamedStaffNotes.where((n) => n.staff == 1).toList(),
            elapsedMs: 0,
            activeNotes: const {},
            bpm: 80,
            songEndMs: 1500,
            keyFifths: 3,
          ),
          const Size(700, 360),
        ),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/staff_right_only.png'),
      );
    });

    testWidgets('staff left-hand only (collapsed)', tags: 'golden', (
      tester,
    ) async {
      await tester.pumpWidget(
        _host(
          StaffPainter(
            notes: _beamedStaffNotes.where((n) => n.staff >= 2).toList(),
            elapsedMs: 0,
            activeNotes: const {},
            bpm: 80,
            songEndMs: 1500,
            keyFifths: 3,
          ),
          const Size(700, 360),
        ),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/staff_left_only.png'),
      );
    });

    // A 3/4 grand staff: the bar lines must align with the real measure starts
    // (from measureStartMs), so notes sit inside their measure instead of falling
    // on a bar drawn at a hardcoded 4-beat spacing.
    testWidgets('staff 3/4 measure bars align with measures', tags: 'golden', (
      tester,
    ) async {
      await tester.pumpWidget(
        _host(
          const StaffPainter(
            notes: _threeFourNotes,
            elapsedMs: 0,
            activeNotes: {},
            bpm: 90,
            songEndMs: 6000,
            keyFifths: 1, // G major
            beats: 3,
            beatType: 4,
            measureStartMs: _threeFourMeasureStarts,
          ),
          const Size(1400, 320),
        ),
      );
      await expectLater(
        find.byKey(const Key('golden')),
        matchesGoldenFile('goldens/staff_threefour.png'),
      );
    });
  });

  // Non-golden render so the measureStartMs bar path is exercised by the
  // cross-platform CI gate (goldens are excluded there).
  testWidgets('staff draws 3/4 bars from measureStartMs without error', (
    tester,
  ) async {
    await tester.pumpWidget(
      _host(
        const StaffPainter(
          notes: _threeFourNotes,
          elapsedMs: 500,
          activeNotes: {},
          bpm: 90,
          songEndMs: 6000,
          beats: 3,
          beatType: 4,
          measureStartMs: _threeFourMeasureStarts,
        ),
        const Size(1400, 320),
      ),
    );
    expect(tester.takeException(), isNull);
  });

  // Non-golden render so the three-state paint() branches are exercised by the
  // cross-platform CI gate (goldens are excluded there).
  testWidgets('staff beams a grand-staff group without error', (tester) async {
    await tester.pumpWidget(
      _host(
        const StaffPainter(
          notes: _beamedStaffNotes,
          elapsedMs: 0,
          activeNotes: {64},
          bpm: 80,
          songEndMs: 1500,
        ),
        const Size(700, 360),
      ),
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets('keyboard renders all feedback states without error', (
    tester,
  ) async {
    await tester.pumpWidget(
      _host(
        const PianoKeyboardPainter(
          layout: _layout,
          activeNotes: {60, 62}, // 60 correct, 62 pressed-only
          requiredNotes: {60, 64, 61}, // 64 expected white, 61 expected black
        ),
        const Size(600, 160),
      ),
    );
    expect(tester.takeException(), isNull);
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

    test('keyboard repaints when requiredNotes change', () {
      const active = {60};
      const a = PianoKeyboardPainter(
        layout: _layout,
        activeNotes: active,
        requiredNotes: {62},
      );
      const b = PianoKeyboardPainter(
        layout: _layout,
        activeNotes: active,
        requiredNotes: {64},
      );
      expect(a.shouldRepaint(b), isTrue);
    });

    test('keyboard repaints when the displayed range changes', () {
      const a = PianoKeyboardPainter(layout: _layout, activeNotes: {60});
      const wider = PianoLayout(width: 600, lowPitch: 21, highPitch: 108);
      const b = PianoKeyboardPainter(layout: wider, activeNotes: {60});
      expect(a.shouldRepaint(b), isTrue);
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
