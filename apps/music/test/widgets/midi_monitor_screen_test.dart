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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/midi_monitor_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/state/midi_monitor.dart';
import 'package:music/state/midi_monitor_notifier.dart';
import 'package:music/src/rust/api/midi.dart' show MidiEvent;
import 'package:music/state/player_notifier.dart';

import '../support/fakes.dart';
import '../support/localized.dart';

void main() {
  late FakeMidiService midi;
  late RecordingAudioService audio;
  late ProviderContainer container;

  Future<void> pump(
    WidgetTester tester, {
    List<String> ports = const ['Drum kit'],
    String? connected = 'Drum kit',
  }) async {
    midi = FakeMidiService(ports: ports, connected: connected);
    audio = RecordingAudioService();
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(audio),
      ],
    );
    // The monitor reads the loaded score's kit off the player, so the player has
    // to be alive — as it is in the app, where this screen is opened from it.
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(const MidiMonitorScreen()),
      ),
    );
    // Twice: the player loads its score on a future, and a half-loaded player
    // leaves that future pending into teardown.
    await tester.pump();
    await tester.pump();
  }

  /// Emit and let the event land. The fake's broadcast controller delivers on a
  /// microtask, so a single `pump` paints the frame *before* the listener has
  /// run — the monitor would look inert when it is merely one turn behind.
  Future<void> emit(WidgetTester tester, MidiEvent event) async {
    midi.emit(event);
    await tester.pump();
    await tester.pump();
  }

  Future<void> teardown(WidgetTester tester) async {
    await tester.pumpWidget(const SizedBox());
    // Settle before disposing: the framework asserts no timer outlives the tree,
    // and the player schedules its score load on one.
    await tester.pumpAndSettle();
    container.dispose();
    await midi.close();
  }

  testWidgets('a stroke appears with the number that was sent', (tester) async {
    await pump(tester);
    await emit(tester, noteOnEvent(38));
    expect(find.text('38'), findsOneWidget);
    await teardown(tester);
  });

  testWidgets('the raw number is shown even when the app cannot place it', (
    tester,
  ) async {
    // The whole point: a stroke the app has no name for must still be visible,
    // with the number the player can read back to us.
    await pump(tester);
    await emit(tester, noteOnEvent(97));

    expect(find.text('97'), findsOneWidget);
    await teardown(tester);
  });

  testWidgets('waiting and no-device are different states', (tester) async {
    await pump(tester);
    expect(find.byKey(const Key('midi-monitor-empty')), findsOneWidget);
    final waiting = tester
        .widget<Text>(find.byKey(const Key('midi-monitor-empty')))
        .data;
    await teardown(tester);

    await pump(tester, ports: const [], connected: null);
    final noDevice = tester
        .widget<Text>(find.byKey(const Key('midi-monitor-empty')))
        .data;
    expect(noDevice, isNot(waiting));
    await teardown(tester);
  });

  testWidgets('clearing empties the read-out for the next stroke', (
    tester,
  ) async {
    // The actual workflow: clear, hit the one pad you are hunting, read it.
    await pump(tester);
    await emit(tester, noteOnEvent(38));
    await emit(tester, noteOnEvent(42));
    expect(find.byKey(const Key('midi-monitor-list')), findsOneWidget);

    await tester.tap(find.byKey(const Key('midi-monitor-clear')));
    await tester.pump();

    expect(find.byKey(const Key('midi-monitor-empty')), findsOneWidget);
    await teardown(tester);
  });

  testWidgets('the newest stroke is on top', (tester) async {
    await pump(tester);
    await emit(tester, noteOnEvent(38));
    await emit(tester, noteOnEvent(42));

    final first = tester.getTopLeft(find.text('42'));
    final second = tester.getTopLeft(find.text('38'));
    expect(
      first.dy,
      lessThan(second.dy),
      reason: 'the stroke a player is hunting is the one they just played',
    );
    await teardown(tester);
  });

  testWidgets('the history is bounded', (tester) async {
    await pump(tester);
    for (var i = 0; i < kMidiMonitorCapacity + 20; i++) {
      midi.emit(noteOnEvent(35 + (i % 40)));
    }
    await tester.pump();
    await tester.pump();

    expect(
      container.read(midiMonitorProvider),
      hasLength(kMidiMonitorCapacity),
      reason: 'a live read-out, not a log left open overnight',
    );
    await teardown(tester);
  });

  testWidgets('observing does not disturb the player', (tester) async {
    // The property the shared stream exists to make true: a second reader must
    // not cost the first one its events.
    await pump(tester);
    final before = container.read(playerProvider).activeNotes.length;
    await emit(tester, noteOnEvent(60));

    expect(
      container.read(playerProvider).activeNotes.length,
      greaterThan(before),
      reason: 'the player saw the same event the monitor did',
    );
    expect(container.read(midiMonitorProvider), isNotEmpty);
    await teardown(tester);
  });
}
