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
import 'package:music/widgets/course_diagram.dart';

Future<void> _pump(WidgetTester tester, String id) => tester.pumpWidget(
  MaterialApp(
    home: Scaffold(
      body: Center(
        child: SizedBox(width: 300, child: CourseDiagram(id: id)),
      ),
    ),
  ),
);

void main() {
  testWidgets('renders (paints) every built-in diagram id without error', (
    tester,
  ) async {
    for (final id in kCourseDiagramIds) {
      await _pump(tester, id);
      await tester.pump();
      expect(find.byType(CustomPaint), findsWidgets, reason: id);
      expect(tester.takeException(), isNull, reason: id);
    }
  });

  testWidgets('an unknown diagram id shows a neutral placeholder', (
    tester,
  ) async {
    await _pump(tester, 'not-a-real-diagram');
    await tester.pump();
    // The placeholder is an icon, not a painted staff.
    expect(find.byType(Icon), findsOneWidget);
  });
}
