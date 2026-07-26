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
import 'package:music/widgets/app_snackbar.dart';

void main() {
  testWidgets('a new message replaces the current snackbar (no stacking)', (
    tester,
  ) async {
    late ScaffoldMessengerState messenger;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Builder(
            builder: (context) {
              messenger = ScaffoldMessenger.of(context);
              return const SizedBox();
            },
          ),
        ),
      ),
    );

    showAppSnackBar(messenger, 'First');
    await tester.pump();
    expect(find.text('First'), findsOneWidget);

    showAppSnackBar(messenger, 'Second');
    await tester.pump();
    // The first is cleared immediately — only the latest shows.
    expect(find.text('First'), findsNothing);
    expect(find.text('Second'), findsOneWidget);
  });

  testWidgets('the snackbar can be dismissed manually with the close button', (
    tester,
  ) async {
    // Tall enough that the snackbar (and its close button) sit within the
    // viewport, so the tap hit-tests.
    await tester.binding.setSurfaceSize(const Size(800, 1200));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    late ScaffoldMessengerState messenger;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Builder(
            builder: (context) {
              messenger = ScaffoldMessenger.of(context);
              return const SizedBox();
            },
          ),
        ),
      ),
    );

    showAppSnackBar(messenger, 'Closable');
    await tester.pumpAndSettle(); // let the entry animation finish
    expect(find.text('Closable'), findsOneWidget);

    // showCloseIcon renders a close button.
    await tester.tap(find.byIcon(Icons.close));
    await tester.pumpAndSettle();
    expect(find.text('Closable'), findsNothing);
  });
}
