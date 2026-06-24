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

// End-to-end test driving the REAL app: it builds and loads the native Rust
// library (cargokit) and exercises the genuine flutter_rust_bridge path
// (RustLib.init, parse_musicxml, layout_systems, midiEventStream). No MIDI
// hardware is required — the computer-keyboard fallback covers the input path.
// Run locally with `flutter test integration_test -d macos`; in CI it runs on
// the Linux desktop engine under Xvfb.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:music/main.dart';
import 'package:music/src/rust/frb_generated.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  testWidgets('library → score → plays, keyboard input, render modes', (
    tester,
  ) async {
    // The desktop/tablet-first UI is laid out for a realistic viewport; pin a
    // desktop size so the headless CI window (defaults to ~800x600) doesn't
    // overflow the dense top/transport bars.
    tester.view.physicalSize = const Size(1440, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(const ProviderScope(child: CymbraApp()));
    await tester.pump(const Duration(milliseconds: 100));

    // Boots into the score library; pick a bundled score.
    expect(find.text('Cymbra — Score Library'), findsOneWidget);
    final entry = find.text('Ode to Joy (theme)');
    expect(entry, findsOneWidget);
    await tester.tap(entry);

    // Let navigation + asset load + the real bridge parse/layout settle.
    for (var i = 0; i < 25; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }

    // Player chrome for the loaded score (parsed over the bridge).
    expect(find.text('Cymbra Music'), findsWidgets);
    expect(find.textContaining('Ode to Joy'), findsWidgets);

    // Transport: play.
    expect(find.byIcon(Icons.play_arrow), findsOneWidget);
    await tester.tap(find.byIcon(Icons.play_arrow));
    await tester.pump();
    expect(find.byIcon(Icons.pause), findsOneWidget);

    // Computer-keyboard fallback: press and release C4.
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyA);
    await tester.pump();
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyA);
    await tester.pump();

    // Cycle the three rendering modes: Synthesia → Staff → Partition → Synthesia.
    await tester.tap(find.text('Staff'));
    await tester.pump();
    await tester.tap(find.text('Partition'));
    await tester.pump(const Duration(milliseconds: 100));
    await tester.tap(find.text('Synthesia'));
    await tester.pump();
  });
}
