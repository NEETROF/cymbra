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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/audio_routing_service.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/audio_routing.dart';
import 'package:music/state/player_preferences.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';
import 'audio_routing_test.mocks.dart';

@GenerateNiceMocks([MockSpec<AudioRoutingService>()])
void main() {
  const builtin = AudioRoute(
    name: 'Built-in Output',
    kind: AudioRouteKind.builtin,
  );
  const iface = AudioRoute(name: 'Scarlett 2i2', kind: AudioRouteKind.usb);
  const speaker = AudioRoute(
    name: 'Kitchen Speaker',
    kind: AudioRouteKind.bluetooth,
  );

  late MockAudioRoutingService service;
  late StreamController<AudioRoute?> routeChanges;
  late FakePreferencesService prefs;

  /// A container wired to the mock, with the remembered output pre-seeded in
  /// storage when [remembered] is given.
  ProviderContainer makeContainer({String? remembered}) {
    prefs = FakePreferencesService(
      remembered == null
          ? null
          : {
              PlayerPreferences.prefsKey:
                  '{"audioOutput":"$remembered","outputOffsetMs":0}',
            },
    );
    final container = ProviderContainer(
      overrides: [
        audioRoutingServiceProvider.overrideWithValue(service),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
        preferencesServiceProvider.overrideWithValue(prefs),
      ],
    );
    addTearDown(container.dispose);
    return container;
  }

  /// Lets the notifier's async restore (and any pending selection) settle.
  Future<void> settle() async {
    for (var i = 0; i < 4; i++) {
      await Future<void>.delayed(Duration.zero);
    }
  }

  setUp(() {
    service = MockAudioRoutingService();
    routeChanges = StreamController<AudioRoute?>.broadcast();
    addTearDown(routeChanges.close);
    when(service.supportsDeviceSelection).thenReturn(true);
    when(service.routeChanges()).thenAnswer((_) => routeChanges.stream);
    when(service.listOutputs()).thenAnswer((_) async => [builtin, iface]);
    when(service.activeRoute()).thenAnswer((_) async => builtin);
    when(service.selectOutput(any)).thenAnswer((_) async {});
    when(service.presentRoutePicker()).thenAnswer((_) async {});
  });

  test('reads the device list and the active route on desktop', () async {
    final container = makeContainer();
    container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
    await settle();

    final state = container.read(audioRoutingProvider);
    expect(state.canSelectDevice, isTrue);
    expect(state.outputs, [builtin, iface]);
    expect(state.active, builtin);
    expect(state.isWireless, isFalse);
  });

  test('selecting a device applies it and remembers it', () async {
    final container = makeContainer();
    container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
    await settle();
    when(service.activeRoute()).thenAnswer((_) async => iface);

    await container
        .read(audioRoutingProvider.notifier)
        .selectOutput(iface.name);

    verify(service.selectOutput(iface.name)).called(1);
    expect(container.read(audioRoutingProvider).active, iface);
    expect(container.read(audioRoutingProvider).selectionFailed, isFalse);
    expect(container.read(playerPreferencesProvider).audioOutput, iface.name);
  });

  test(
    'a device that will not open reports a failure, keeping the current one',
    () async {
      final container = makeContainer();
      container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
      await settle();
      // The engine never gets there: the active route stays what it was.
      await container
          .read(audioRoutingProvider.notifier)
          .selectOutput(iface.name);

      final state = container.read(audioRoutingProvider);
      expect(state.selectionFailed, isTrue);
      expect(
        state.active,
        builtin,
        reason: 'working audio must not be traded for a broken device',
      );
    },
  );

  test('the failure is cleared once acknowledged', () async {
    final container = makeContainer();
    container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
    await settle();
    await container
        .read(audioRoutingProvider.notifier)
        .selectOutput(iface.name);
    expect(container.read(audioRoutingProvider).selectionFailed, isTrue);

    container.read(audioRoutingProvider.notifier).acknowledgeFailure();

    expect(container.read(audioRoutingProvider).selectionFailed, isFalse);
  });

  test('a remembered device is re-applied at startup', () async {
    final container = makeContainer(remembered: 'Scarlett 2i2');
    container.listen(
      playerPreferencesProvider,
      (_, _) {},
      fireImmediately: true,
    );
    container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
    await settle();

    verify(service.selectOutput('Scarlett 2i2')).called(1);
  });

  test(
    'an absent remembered device shows the device actually in use',
    () async {
      // The engine fell back: it reports the built-in output, not the request.
      final container = makeContainer(remembered: 'Ghost Interface');
      container.listen(
        playerPreferencesProvider,
        (_, _) {},
        fireImmediately: true,
      );
      container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
      await settle();

      expect(container.read(audioRoutingProvider).active, builtin);
      expect(container.read(audioRoutingProvider).selectionFailed, isFalse);
    },
  );

  test('a platform route change updates the displayed route', () async {
    final container = makeContainer();
    container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
    await settle();

    routeChanges.add(speaker);
    await settle();

    expect(container.read(audioRoutingProvider).active, speaker);
    expect(container.read(audioRoutingProvider).isWireless, isTrue);
  });

  test('a route change refreshes the device list', () async {
    const dongle = AudioRoute(
      name: 'USB-C Dongle',
      kind: AudioRouteKind.headphones,
    );
    final container = makeContainer();
    container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
    await settle();
    expect(container.read(audioRoutingProvider).outputs, [builtin, iface]);
    // A device was plugged in after startup: only the change event knows.
    when(
      service.listOutputs(),
    ).thenAnswer((_) async => [builtin, iface, dongle]);

    routeChanges.add(builtin);
    await settle();

    expect(container.read(audioRoutingProvider).outputs, [
      builtin,
      iface,
      dongle,
    ]);
    verifyNever(service.selectOutput(any));
  });

  test(
    'a route change re-applies the remembered device when it is present',
    () async {
      // The chosen device came back from a replug: it is in the list again, but
      // under a new platform id, so the pin must be re-applied — silently, since
      // nothing here is a user action to answer with a failure toast.
      final container = makeContainer(remembered: 'Scarlett 2i2');
      container.listen(
        playerPreferencesProvider,
        (_, _) {},
        fireImmediately: true,
      );
      container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
      await settle();
      clearInteractions(service);

      routeChanges.add(builtin);
      await settle();

      verify(service.selectOutput('Scarlett 2i2')).called(1);
      expect(container.read(audioRoutingProvider).selectionFailed, isFalse);
    },
  );

  test(
    're-pins even when the event already names the remembered device',
    () async {
      // On Android an unpinned stream's reported route is a ranked guess that
      // often *names* the remembered USB device while the sound actually
      // follows the system default. Trusting the report would skip exactly the
      // re-pin that is needed — so the re-pin must not consult it.
      final container = makeContainer(remembered: 'Scarlett 2i2');
      container.listen(
        playerPreferencesProvider,
        (_, _) {},
        fireImmediately: true,
      );
      container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
      await settle();
      clearInteractions(service);

      routeChanges.add(iface); // the event claims the remembered device
      await settle();

      verify(service.selectOutput('Scarlett 2i2')).called(1);
    },
  );

  test('a route change never re-pins a device that is still absent', () async {
    final container = makeContainer(remembered: 'Ghost Interface');
    container.listen(
      playerPreferencesProvider,
      (_, _) {},
      fireImmediately: true,
    );
    container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
    await settle();
    clearInteractions(service);

    routeChanges.add(builtin);
    await settle();

    verifyNever(service.selectOutput(any));
  });

  group('mobile', () {
    setUp(() {
      when(service.supportsDeviceSelection).thenReturn(false);
      when(service.listOutputs()).thenAnswer((_) async => const []);
      when(service.activeRoute()).thenAnswer((_) async => speaker);
    });

    test('offers no device list and refuses to fake a selection', () async {
      final container = makeContainer(remembered: 'Scarlett 2i2');
      container.listen(audioRoutingProvider, (_, _) {}, fireImmediately: true);
      await settle();

      expect(container.read(audioRoutingProvider).canSelectDevice, isFalse);
      expect(container.read(audioRoutingProvider).outputs, isEmpty);
      verifyNever(service.selectOutput(any));

      await container.read(audioRoutingProvider.notifier).selectOutput('X');
      verifyNever(service.selectOutput(any));
    });

    test(
      'a route change reports the new route and never selects a device',
      () async {
        // iOS: the OS owns the route. Even with a name persisted from another
        // platform, a route event must only update the display.
        final container = makeContainer(remembered: 'Scarlett 2i2');
        container.listen(
          playerPreferencesProvider,
          (_, _) {},
          fireImmediately: true,
        );
        container.listen(
          audioRoutingProvider,
          (_, _) {},
          fireImmediately: true,
        );
        await settle();

        routeChanges.add(builtin);
        await settle();

        expect(container.read(audioRoutingProvider).active, builtin);
        verifyNever(service.selectOutput(any));
      },
    );

    test(
      'presenting the OS picker re-reads the route it left active',
      () async {
        final container = makeContainer();
        container.listen(
          audioRoutingProvider,
          (_, _) {},
          fireImmediately: true,
        );
        await settle();
        when(service.activeRoute()).thenAnswer((_) async => builtin);

        await container
            .read(audioRoutingProvider.notifier)
            .presentRoutePicker();

        verify(service.presentRoutePicker()).called(1);
        expect(container.read(audioRoutingProvider).active, builtin);
      },
    );
  });
}
