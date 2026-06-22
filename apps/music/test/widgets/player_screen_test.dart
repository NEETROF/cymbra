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
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';

import '../support/fakes.dart';

void main() {
  late FakeMidiService midi;
  late ProviderContainer container;

  PlayerData state() => container.read(playerProvider);

  Future<void> pumpScreen(
    WidgetTester tester, {
    List<String> ports = const ['Piano'],
    String? connected = 'Piano',
    Size size = const Size(1600, 900),
  }) async {
    await tester.binding.setSurfaceSize(size);
    midi = FakeMidiService(ports: ports, connected: connected);
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      ],
    );
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: PlayerScreen()),
      ),
    );
    await tester.pump(); // flush score load + first rebuild
  }

  Future<void> teardownScreen(WidgetTester tester) async {
    await tester.pumpWidget(const SizedBox()); // unmount the screen
    await tester.pump(); // let the auto-dispose provider tear down its timer
    container.dispose();
    await midi.close();
    await tester.binding.setSurfaceSize(null);
  }

  testWidgets('renders title, tempo, and connected MIDI device', (
    tester,
  ) async {
    await pumpScreen(tester);
    expect(find.text('Cymbra Music'), findsOneWidget);
    expect(find.text('Tempo: 80'), findsOneWidget);
    expect(find.text('Piano'), findsWidgets);
    await teardownScreen(tester);
  });

  testWidgets('play/pause toggles the transport icon', (tester) async {
    await pumpScreen(tester);
    expect(find.byIcon(Icons.play_arrow), findsOneWidget);
    await tester.tap(find.byIcon(Icons.play_arrow));
    await tester.pump();
    expect(state().isPlaying, isTrue);
    expect(find.byIcon(Icons.pause), findsOneWidget);
    await teardownScreen(tester);
  });

  testWidgets('mode toggle switches Synthesia ⇄ Staff', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.text('Staff'));
    await tester.pump();
    expect(state().mode, RenderMode.staff);
    await teardownScreen(tester);
  });

  testWidgets('speed and wait-mode controls update state', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.add));
    await tester.pump();
    expect(state().speed, greaterThan(1.0));
    await tester.tap(find.byIcon(Icons.remove));
    await tester.pump();

    expect(state().waitMode, isTrue);
    await tester.tap(find.text('Wait'));
    await tester.pump();
    expect(state().waitMode, isFalse);
    await teardownScreen(tester);
  });

  testWidgets('computer keyboard fallback presses a key', (tester) async {
    await pumpScreen(tester);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyA); // C4
    await tester.pump();
    expect(state().activeNotes, contains(60));
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyA);
    await tester.pump();
    expect(state().activeNotes, isNot(contains(60)));
    await teardownScreen(tester);
  });

  testWidgets('MIDI indicator shows "No MIDI device" when none detected', (
    tester,
  ) async {
    await pumpScreen(tester, ports: const [], connected: null);
    expect(find.text('No MIDI device'), findsOneWidget);
    await teardownScreen(tester);
  });

  testWidgets('MIDI indicator shows a connecting state', (tester) async {
    await pumpScreen(tester, ports: ['Piano'], connected: null);
    expect(find.text('Piano (connecting…)'), findsOneWidget);
    await teardownScreen(tester);
  });

  testWidgets('fits a tablet-width (1024px) window without overflow', (
    tester,
  ) async {
    await pumpScreen(tester, size: const Size(1024, 768));
    expect(tester.takeException(), isNull);
    expect(find.text('Cymbra Music'), findsOneWidget);
    await teardownScreen(tester);
  });

  testWidgets('wait-mode overlay appears when the cascade is blocked', (
    tester,
  ) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.play_arrow)); // play, wait-mode on
    await tester.pump(const Duration(milliseconds: 16)); // one ticker frame
    await tester.pump(); // rebuild after blocked=true
    expect(state().blocked, isTrue);
    expect(find.byType(StaffPainter), findsNothing); // still synthesia mode
    expect(find.text('⏸  Play the expected note to continue'), findsOneWidget);
    await teardownScreen(tester);
  });
}
