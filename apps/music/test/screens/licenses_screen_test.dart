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

import 'package:music/screens/licenses_screen.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

Future<void> _pump(WidgetTester tester, EdgeInsets padding) async {
  await tester.pumpWidget(
    MaterialApp(
      home: MediaQuery(
        data: MediaQueryData(padding: padding, viewPadding: padding),
        child: const LicensesScreen(),
      ),
    ),
  );
  await tester.pump();
}

void main() {
  // The app is landscape-locked, so the iPhone sensor housing sits on a side.
  // Flutter's LicensePage pads only the top, which put the back button and the
  // package list under the camera.
  testWidgets('landscape notch: the page clears the side inset', (
    tester,
  ) async {
    const inset = EdgeInsets.only(left: 44, right: 44);
    await _pump(tester, inset);

    final page = tester.getTopLeft(find.byType(LicensePage));
    expect(page.dx, greaterThanOrEqualTo(44));
    expect(
      tester.getTopRight(find.byType(LicensePage)).dx,
      lessThanOrEqualTo(tester.getSize(find.byType(MaterialApp)).width - 44),
    );
  });

  // Only the horizontal insets are consumed here: LicensePage owns an AppBar
  // that must keep padding the status bar itself.
  testWidgets('the top inset is left to the page app bar', (tester) async {
    await _pump(tester, const EdgeInsets.only(top: 47, left: 44));
    expect(tester.getTopLeft(find.byType(LicensePage)).dy, 0);
  });

  testWidgets('no notch: the page fills the window', (tester) async {
    await _pump(tester, EdgeInsets.zero);
    expect(tester.getTopLeft(find.byType(LicensePage)).dx, 0);
  });
}
