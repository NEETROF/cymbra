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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/audio_routing_service.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/audio_routing.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/widgets/sound_output_section.dart';

import '../state/audio_routing_test.mocks.dart';
import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';

/// The sound-output section renders one shape over two platform realities, and
/// its wireless warning is driven by the route's **kind**, never its name
/// (change: add-audio-output-routing).
void main() {
  const builtin = AudioRoute(
    name: 'Built-in Output',
    kind: AudioRouteKind.builtin,
  );
  const iface = AudioRoute(name: 'Scarlett 2i2', kind: AudioRouteKind.usb);

  late MockAudioRoutingService service;
  late FakeMidiService midi;
  late ProviderContainer container;
  ProviderContainer? live;

  /// Unmounts the section and disposes the container **inside** the test body:
  /// the player notifier owns a periodic MIDI-status timer, which the framework
  /// checks for before any `addTearDown` would run.
  Future<void> disposeLive(WidgetTester tester) async {
    if (live == null) return;
    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    live!.dispose();
    live = null;
    await midi.close();
  }

  Future<void> pumpSection(
    WidgetTester tester, {
    required bool canSelectDevice,
    required AudioRoute? active,
    List<AudioRoute> outputs = const [],
    String? connectedMidi = 'Piano',
  }) async {
    await disposeLive(tester);
    service = MockAudioRoutingService();
    when(service.supportsDeviceSelection).thenReturn(canSelectDevice);
    when(
      service.routeChanges(),
    ).thenAnswer((_) => const Stream<AudioRoute?>.empty());
    when(service.listOutputs()).thenAnswer((_) async => outputs);
    when(service.activeRoute()).thenAnswer((_) async => active);
    when(service.selectOutput(any)).thenAnswer((_) async {});
    when(service.presentRoutePicker()).thenAnswer((_) async {});

    midi = FakeMidiService(
      ports: connectedMidi == null ? const [] : [connectedMidi],
      connected: connectedMidi,
    );
    container = ProviderContainer(
      overrides: [
        audioRoutingServiceProvider.overrideWithValue(service),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
        preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      ],
    );
    live = container;
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(
          const Scaffold(
            body: SingleChildScrollView(child: SoundOutputSection()),
          ),
        ),
      ),
    );
    // Let the notifier's async restore land.
    await tester.pump();
    await tester.pump();
  }

  Future<void> teardown(WidgetTester tester) => disposeLive(tester);

  testWidgets('desktop shows a device list', (tester) async {
    await pumpSection(
      tester,
      canSelectDevice: true,
      active: builtin,
      outputs: const [builtin, iface],
    );

    expect(find.byKey(const Key('sound-output-device')), findsOneWidget);
    expect(find.byKey(const Key('sound-output-route')), findsNothing);
    expect(find.text('Built-in Output'), findsWidgets);
    await teardown(tester);
  });

  testWidgets('selecting a device goes through the notifier', (tester) async {
    await pumpSection(
      tester,
      canSelectDevice: true,
      active: builtin,
      outputs: const [builtin, iface],
    );

    // The engine lands on the requested device, so the notifier settles at once.
    when(service.activeRoute()).thenAnswer((_) async => iface);
    await tester.tap(find.byKey(const Key('sound-output-device')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Scarlett 2i2').last);
    await tester.pumpAndSettle();

    verify(service.selectOutput('Scarlett 2i2')).called(1);
    expect(container.read(audioRoutingProvider).active, iface);
    await teardown(tester);
  });

  testWidgets('mobile shows the active route and a picker button', (
    tester,
  ) async {
    await pumpSection(
      tester,
      canSelectDevice: false,
      active: const AudioRoute(
        name: 'iPhone Speaker',
        kind: AudioRouteKind.builtin,
      ),
    );

    expect(find.byKey(const Key('sound-output-device')), findsNothing);
    expect(find.byKey(const Key('sound-output-route')), findsOneWidget);
    expect(find.text('iPhone Speaker'), findsOneWidget);

    await tester.tap(find.byKey(const Key('sound-output-route-picker')));
    await tester.pump();
    verify(service.presentRoutePicker()).called(1);
    await teardown(tester);
  });

  testWidgets('an unreported route degrades to a readable placeholder', (
    tester,
  ) async {
    await pumpSection(tester, canSelectDevice: false, active: null);

    expect(find.text('Unknown output'), findsOneWidget);
    await teardown(tester);
  });

  group('wireless warning', () {
    testWidgets('is shown for a Bluetooth route', (tester) async {
      await pumpSection(
        tester,
        canSelectDevice: false,
        active: const AudioRoute(
          name: 'Kitchen Speaker',
          kind: AudioRouteKind.bluetooth,
        ),
      );

      expect(
        find.byKey(const Key('sound-output-wireless-warning')),
        findsOneWidget,
      );
      expect(find.byKey(const Key('sound-output-offset')), findsOneWidget);
      await teardown(tester);
    });

    testWidgets('is not shown for USB or headphones', (tester) async {
      await pumpSection(tester, canSelectDevice: false, active: iface);
      expect(
        find.byKey(const Key('sound-output-wireless-warning')),
        findsNothing,
      );
      expect(find.byKey(const Key('sound-output-offset')), findsNothing);

      await pumpSection(
        tester,
        canSelectDevice: false,
        active: const AudioRoute(
          name: 'Headphones',
          kind: AudioRouteKind.headphones,
        ),
      );
      expect(
        find.byKey(const Key('sound-output-wireless-warning')),
        findsNothing,
      );
      await teardown(tester);
    });

    testWidgets('a name that reads wireless does not trigger it', (
      tester,
    ) async {
      // Kind, never the name: a USB device called "Bluetooth Speaker" is wired.
      await pumpSection(
        tester,
        canSelectDevice: false,
        active: const AudioRoute(
          name: 'Bluetooth Speaker',
          kind: AudioRouteKind.usb,
        ),
      );

      expect(
        find.byKey(const Key('sound-output-wireless-warning')),
        findsNothing,
      );
      await teardown(tester);
    });
  });

  group('output offset', () {
    testWidgets('suggests a starting value without applying it', (
      tester,
    ) async {
      await pumpSection(
        tester,
        canSelectDevice: false,
        active: const AudioRoute(
          name: 'Kitchen Speaker',
          kind: AudioRouteKind.bluetooth,
        ),
      );

      // Offered, not applied.
      expect(container.read(playerProvider).outputOffsetMs, 0);
      expect(
        find.byKey(const Key('sound-output-offset-suggestion')),
        findsOneWidget,
      );

      await tester.tap(find.byKey(const Key('sound-output-offset-suggestion')));
      await tester.pump();

      expect(
        container.read(playerProvider).outputOffsetMs,
        kSuggestedWirelessOffsetMs,
      );
      await teardown(tester);
    });
  });

  group('instrument-sounds-itself toggle', () {
    testWidgets('is enabled and switchable with an instrument connected', (
      tester,
    ) async {
      await pumpSection(tester, canSelectDevice: true, active: builtin);

      final tile = tester.widget<SwitchListTile>(
        find.byKey(const Key('instrument-sounds-itself')),
      );
      expect(tile.onChanged, isNotNull);

      await tester.tap(find.byKey(const Key('instrument-sounds-itself')));
      await tester.pump();
      expect(container.read(playerProvider).instrumentSoundsItself, isTrue);
      await teardown(tester);
    });

    testWidgets('is disabled with its reason when no MIDI port is connected', (
      tester,
    ) async {
      await pumpSection(
        tester,
        canSelectDevice: true,
        active: builtin,
        connectedMidi: null,
      );

      final tile = tester.widget<SwitchListTile>(
        find.byKey(const Key('instrument-sounds-itself')),
      );
      expect(tile.onChanged, isNull);
      expect(
        find.text('Connect a MIDI instrument to use this.'),
        findsOneWidget,
      );
      await teardown(tester);
    });
  });
}
