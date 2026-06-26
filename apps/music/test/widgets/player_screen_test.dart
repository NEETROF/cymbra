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
import 'package:music/painters/piano_layout.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';

import '../support/fakes.dart';

void main() {
  late FakeMidiService midi;
  late ProviderContainer container;

  PlayerData state() => container.read(playerProvider);

  /// Global position of the center of [pitch]'s key on the on-screen keyboard.
  /// [y] picks the vertical band: ~120 is the white-only region, ~30 the black
  /// band. Mirrors the layout the screen builds from the keyboard width and the
  /// current keyboard bounds.
  Offset keyPos(WidgetTester tester, int pitch, {double y = 120}) {
    final rect = tester.getRect(find.byKey(const Key('onscreen-keyboard')));
    final bounds = state().keyboardBounds;
    final layout = PianoLayout(
      width: rect.width,
      lowPitch: bounds.low,
      highPitch: bounds.high,
    );
    return rect.topLeft + Offset(layout.centerX(pitch), y);
  }

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
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
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

  testWidgets('renders title, tempo, and MIDI status (no device name)', (
    tester,
  ) async {
    await pumpScreen(tester);
    expect(find.text('Cymbra Music'), findsOneWidget);
    expect(find.text('Tempo: 80'), findsOneWidget);
    // The status chip shows the connection state, not the device name (that's
    // listed in the settings menu instead).
    expect(find.text('Connected'), findsOneWidget);
    expect(find.text('Piano'), findsNothing);
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

  testWidgets('settings menu › keyboard size updates the range mode', (
    tester,
  ) async {
    await pumpScreen(tester);
    // Defaults to the full 88-key piano.
    expect(state().keyboardRange, KeyboardRangeMode.keys88);

    // The screen runs a Ticker (never settles), so pump explicitly rather than
    // pumpAndSettle. 300ms lets the drawer open animation finish. Master-detail:
    // open the gear (end drawer) → pick the "Keyboard size" category → pick Auto.
    await tester.tap(find.byIcon(Icons.tune));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    await tester.tap(find.text('Keyboard size'));
    await tester.pump();
    await tester.tap(find.text('Auto (fit piece)'));
    await tester.pump();

    expect(state().keyboardRange, KeyboardRangeMode.auto);
    await teardownScreen(tester);
  });

  testWidgets('settings menu › MIDI device selects a port', (tester) async {
    await pumpScreen(tester, ports: ['Piano', 'Synth'], connected: 'Piano');
    await tester.tap(find.byIcon(Icons.tune));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    await tester.tap(find.text('MIDI device'));
    await tester.pump();
    await tester.tap(find.text('Synth'));
    await tester.pump();

    expect(state().connectedDevice, 'Synth');
    await teardownScreen(tester);
  });

  testWidgets('settings drawer pauses playback and resumes on close', (
    tester,
  ) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.play_arrow));
    await tester.pump();
    expect(state().isPlaying, isTrue);

    // Opening the end drawer pauses the session.
    await tester.tap(find.byIcon(Icons.tune));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    expect(state().isPlaying, isFalse);

    // Closing it (tap the scrim left of the right-side drawer) restores play.
    await tester.tapAt(const Offset(20, 400));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));
    expect(state().isPlaying, isTrue);
    await teardownScreen(tester);
  });

  testWidgets('right-correct assist key plays the expected right-hand note', (
    tester,
  ) async {
    // Demo notes are staff 1 (right hand); C4 (60) is due at t=0.
    await pumpScreen(tester);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyZ);
    await tester.pump();
    expect(state().activeNotes, contains(60));
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyZ);
    await tester.pump();
    expect(state().activeNotes, isNot(contains(60)));
    await teardownScreen(tester);
  });

  testWidgets('near-miss assist key plays a nearby wrong note', (tester) async {
    await pumpScreen(tester);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyS); // right near-miss
    await tester.pump();
    final active = state().activeNotes;
    expect(active, isNotEmpty);
    expect(active, isNot(contains(60))); // never the expected note
    expect(active.every((p) => (p - 60).abs() <= 3), isTrue);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyS);
    await tester.pump();
    expect(state().activeNotes, isEmpty);
    await teardownScreen(tester);
  });

  testWidgets('right-correct assist key satisfies Wait Mode', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.play_arrow)); // play, wait-mode on
    await tester.pump(const Duration(milliseconds: 16));
    await tester.pump();
    expect(state().blocked, isTrue);

    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyZ);
    await tester.pump(const Duration(milliseconds: 16)); // advance unblocks
    await tester.pump();
    expect(state().blocked, isFalse);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyZ);
    await teardownScreen(tester);
  });

  testWidgets('near-miss assist key does not satisfy Wait Mode', (
    tester,
  ) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.play_arrow));
    await tester.pump(const Duration(milliseconds: 16));
    await tester.pump();
    expect(state().blocked, isTrue);

    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyS); // right near-miss
    await tester.pump(const Duration(milliseconds: 16));
    await tester.pump();
    expect(state().blocked, isTrue); // wrong note → still blocked
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyS);
    await teardownScreen(tester);
  });

  testWidgets('former pitch keys no longer produce notes', (tester) async {
    await pumpScreen(tester);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyD); // was E4
    await tester.pump();
    expect(state().activeNotes, isEmpty);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyD);
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
    expect(find.text('Connecting…'), findsOneWidget);
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

  testWidgets('on-screen key press/release toggles the note', (tester) async {
    await pumpScreen(tester);
    final gesture = await tester.startGesture(keyPos(tester, 60)); // C4
    await tester.pump();
    expect(state().activeNotes, contains(60));
    await gesture.up();
    await tester.pump();
    expect(state().activeNotes, isNot(contains(60)));
    await teardownScreen(tester);
  });

  testWidgets('multi-touch holds two keys and releases independently', (
    tester,
  ) async {
    await pumpScreen(tester);
    final g1 = await tester.startGesture(keyPos(tester, 60)); // C4
    final g2 = await tester.startGesture(keyPos(tester, 62)); // D4
    await tester.pump();
    expect(state().activeNotes, containsAll(<int>[60, 62]));

    // Releasing one pointer note-offs only its pitch.
    await g1.up();
    await tester.pump();
    expect(state().activeNotes, isNot(contains(60)));
    expect(state().activeNotes, contains(62));

    await g2.up();
    await tester.pump();
    expect(state().activeNotes, isNot(contains(62)));
    await teardownScreen(tester);
  });

  testWidgets('on-screen play satisfies the Wait Mode gate', (tester) async {
    await pumpScreen(tester);
    await tester.tap(find.byIcon(Icons.play_arrow)); // play, wait-mode on
    await tester.pump(const Duration(milliseconds: 16)); // one ticker frame
    await tester.pump();
    expect(state().blocked, isTrue); // waiting for C4 (60)

    final gesture = await tester.startGesture(keyPos(tester, 60));
    await tester.pump(const Duration(milliseconds: 16)); // advance unblocks
    await tester.pump();
    expect(state().blocked, isFalse);
    await gesture.up();
    await teardownScreen(tester);
  });

  testWidgets('keyboard responds in every render mode', (tester) async {
    await pumpScreen(tester);
    for (final mode in RenderMode.values) {
      container.read(playerProvider.notifier).setMode(mode);
      await tester.pump();
      final gesture = await tester.startGesture(keyPos(tester, 60));
      await tester.pump();
      expect(state().activeNotes, contains(60), reason: 'mode $mode');
      await gesture.up();
      await tester.pump();
      expect(state().activeNotes, isNot(contains(60)), reason: 'mode $mode');
    }
    await teardownScreen(tester);
  });
}
