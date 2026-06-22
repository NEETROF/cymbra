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
// (RustLib.init, demoScore, midiEventStream). No MIDI hardware is required —
// the computer-keyboard fallback covers the input path. Run locally with
// `flutter test integration_test -d macos`; in CI it runs on the Linux desktop
// engine under Xvfb.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:music/main.dart';
import 'package:music/src/rust/frb_generated.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  testWidgets('boots, plays, accepts keyboard input, toggles render mode', (
    tester,
  ) async {
    // The desktop/tablet-first UI is laid out for a realistic viewport; pin a
    // desktop size so the headless CI window (defaults to ~800x600) doesn't
    // overflow the dense top/transport bars.
    tester.view.physicalSize = const Size(1440, 900);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(const CymbraApp());
    await tester.pump(const Duration(milliseconds: 100));

    // App chrome from the real demo score loaded over the bridge.
    expect(find.text('Cymbra Music'), findsWidgets);
    expect(find.text('Tempo: 80'), findsOneWidget);

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

    // Switch rendering mode Synthesia → Staff → Synthesia.
    await tester.tap(find.text('Staff'));
    await tester.pump();
    await tester.tap(find.text('Synthesia'));
    await tester.pump();
  });
}
