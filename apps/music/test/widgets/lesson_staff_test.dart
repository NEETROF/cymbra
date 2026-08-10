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
import 'package:music/courses/course_manifest.dart';
import 'package:music/courses/lesson_pitch.dart';
import 'package:music/courses/lesson_rhythm.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/state/note_label.dart';
import 'package:music/widgets/lesson_staff.dart';

LessonStaffElement note(String p, [NoteFigure fig = NoteFigure.quarter]) =>
    LessonStaffElement(pitch: LessonPitch.parse(p), fig: RhythmFigure(fig));

Widget host(Widget child, {Locale locale = const Locale('fr')}) => MaterialApp(
  localizationsDelegates: AppLocalizations.localizationsDelegates,
  supportedLocales: AppLocalizations.supportedLocales,
  locale: locale,
  home: Scaffold(body: Center(child: child)),
);

void main() {
  testWidgets('renders a treble staff with notes, labels and a rest', (
    tester,
  ) async {
    await tester.pumpWidget(
      host(
        LessonStaff(
          clef: LessonClef.treble,
          time: const LessonTimeSig(4, 4),
          elements: [
            note('C4'),
            note('E4', NoteFigure.half),
            const LessonStaffElement(
              fig: RhythmFigure(NoteFigure.quarter, rest: true),
            ),
            note('F#5', NoteFigure.eighth),
            note('A5', NoteFigure.whole),
          ],
          labels: true,
        ),
      ),
    );
    expect(find.byType(LessonStaff), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('renders a bass staff with a key signature, dots and a ghost', (
    tester,
  ) async {
    await tester.pumpWidget(
      host(
        LessonStaff(
          clef: LessonClef.bass,
          keyFifths: -3,
          elements: [
            note('G2'),
            LessonStaffElement(
              pitch: LessonPitch.parse('D3'),
              fig: const RhythmFigure(NoteFigure.half, dots: 1),
            ),
          ],
          ghost: LessonPitch.parse('B2'),
        ),
      ),
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets('renders a stacked chord with per-element colours', (
    tester,
  ) async {
    await tester.pumpWidget(
      host(
        LessonStaff(
          clef: LessonClef.treble,
          elements: [note('C4'), note('E4'), note('G4')],
          elementColors: const {0: Colors.green, 2: Colors.red},
          stacked: true,
        ),
      ),
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets('reports the tapped staff step when interactive', (tester) async {
    final steps = <int>[];
    const height = 126.0; // s = 14, bottom = 83.16
    await tester.pumpWidget(
      host(
        SizedBox(
          width: 300,
          child: LessonStaff(
            clef: LessonClef.treble,
            onTapStep: steps.add,
            height: height,
          ),
        ),
      ),
    );
    final origin = tester.getTopLeft(find.byKey(const Key('lesson-staff-tap')));
    final g = LessonStaff.geometryFor(height);
    // Tap exactly on the bottom line → step 0 (Mi4 in treble).
    await tester.tapAt(origin + Offset(150, g.bottom));
    // Tap one full space above it → step 2 (Sol4's line).
    await tester.tapAt(origin + Offset(150, g.bottom - g.s));
    // Tap below the staff → ledger territory, clamped in range.
    await tester.tapAt(origin + Offset(150, g.bottom + g.s));
    expect(steps, [0, 2, -2]);
  });

  test('stepAt clamps to the teachable range', () {
    expect(LessonStaff.stepAt(0, 126), lessThanOrEqualTo(14));
    expect(LessonStaff.stepAt(126, 126), greaterThanOrEqualTo(-6));
  });

  testWidgets('is not tappable without onTapStep', (tester) async {
    await tester.pumpWidget(
      host(LessonStaff(clef: LessonClef.treble, elements: [note('C4')])),
    );
    expect(find.byKey(const Key('lesson-staff-tap')), findsNothing);
  });
}
