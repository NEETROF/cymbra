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
import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/audio_capture_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/src/rust/api/audio_input.dart' show InputRouteKind;
import 'package:music/state/input_calibration_notifier.dart';

import '../support/prefs_fakes.dart';
import 'input_calibration_notifier_test.mocks.dart';

/// The calibration flow (change: add-acoustic-piano-input): every exit maps to
/// a UI-consumable status, the store is keyed per route, and the measurement
/// is only ever the service's — a measured value, never a guess.
@GenerateNiceMocks([MockSpec<AudioCaptureService>()])
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const builtin = CaptureRoute(
    name: 'Built-in Microphone',
    kind: InputRouteKind.builtin,
    refusedBluetooth: false,
  );
  const airpods = CaptureRoute(
    name: 'AirPods',
    kind: InputRouteKind.bluetooth,
    refusedBluetooth: true,
  );

  late MockAudioCaptureService service;
  late FakePreferencesService prefs;
  late StreamController<CaptureRoute?> routes;

  ProviderContainer harness({Map<String, String>? seeded}) {
    service = MockAudioCaptureService();
    prefs = FakePreferencesService(seeded);
    routes = StreamController<CaptureRoute?>.broadcast();
    when(service.routeChanges()).thenAnswer((_) => routes.stream);
    when(service.activeRoute()).thenAnswer((_) async => builtin);
    when(
      service.permissionStatus(),
    ).thenAnswer((_) async => InputPermissionStatus.granted);
    final container = ProviderContainer(
      overrides: [
        audioCaptureServiceProvider.overrideWithValue(service),
        preferencesServiceProvider.overrideWithValue(prefs),
      ],
    );
    addTearDown(container.dispose);
    addTearDown(routes.close);
    return container;
  }

  Future<void> settle() async {
    // Let the microtask-restore and the stream listeners land.
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);
  }

  test('restores the per-route store and reads the active route', () async {
    final container = harness(
      seeded: {
        InputCalibration.prefsKey: jsonEncode({'Built-in Microphone': 72.5}),
      },
    );
    container.read(inputCalibrationProvider);
    await settle();

    final state = container.read(inputCalibrationProvider);
    expect(state.hydrated, isTrue);
    expect(state.route, builtin);
    expect(state.measuredMs, 72.5);
    expect(container.read(measuredInputOffsetMsProvider), 72.5);
  });

  test('a successful run stores the measurement for the route', () async {
    final container = harness();
    when(
      service.runCalibration(),
    ).thenAnswer((_) async => const CalibrationMeasurement(latencyMs: 84.0));
    container.read(inputCalibrationProvider);
    await settle();

    await container.read(inputCalibrationProvider.notifier).runCalibration();

    final state = container.read(inputCalibrationProvider);
    expect(state.status, CalibrationStatus.done);
    expect(state.measuredMs, 84.0);
    // Persisted under the route's name, so a different route reads nothing.
    expect(jsonDecode(prefs.store[InputCalibration.prefsKey]!), {
      'Built-in Microphone': 84.0,
    });
  });

  test(
    'an undetermined permission is requested; refusal ends the run',
    () async {
      final container = harness();
      when(
        service.permissionStatus(),
      ).thenAnswer((_) async => InputPermissionStatus.undetermined);
      when(service.requestPermission()).thenAnswer((_) async => false);
      container.read(inputCalibrationProvider);
      await settle();

      await container.read(inputCalibrationProvider.notifier).runCalibration();

      expect(
        container.read(inputCalibrationProvider).status,
        CalibrationStatus.permissionDenied,
      );
      verifyNever(service.runCalibration());
      expect(prefs.store[InputCalibration.prefsKey], isNull);
    },
  );

  test('a Bluetooth route is refused before any capture', () async {
    final container = harness();
    when(service.activeRoute()).thenAnswer((_) async => airpods);
    container.read(inputCalibrationProvider);
    await settle();

    await container.read(inputCalibrationProvider.notifier).runCalibration();

    expect(
      container.read(inputCalibrationProvider).status,
      CalibrationStatus.refusedBluetooth,
    );
    verifyNever(service.runCalibration());
  });

  test('an undetected click ends with guidance and stores nothing', () async {
    final container = harness();
    when(service.runCalibration()).thenAnswer((_) async => null);
    container.read(inputCalibrationProvider);
    await settle();

    await container.read(inputCalibrationProvider.notifier).runCalibration();

    expect(
      container.read(inputCalibrationProvider).status,
      CalibrationStatus.notDetected,
    );
    expect(prefs.store[InputCalibration.prefsKey], isNull);
  });

  test('a route change re-keys the measurement and lifts a refusal', () async {
    final container = harness(
      seeded: {
        InputCalibration.prefsKey: jsonEncode({'Built-in Microphone': 60.0}),
      },
    );
    when(service.activeRoute()).thenAnswer((_) async => airpods);
    container.read(inputCalibrationProvider);
    await settle();

    await container.read(inputCalibrationProvider.notifier).runCalibration();
    expect(
      container.read(inputCalibrationProvider).status,
      CalibrationStatus.refusedBluetooth,
    );
    // The Bluetooth mic has no stored entry: nothing to present.
    expect(container.read(measuredInputOffsetMsProvider), isNull);

    routes.add(builtin);
    await settle();

    final state = container.read(inputCalibrationProvider);
    // The refusal lifted with the route, and the stored measurement for the
    // returning route re-applies without any re-run.
    expect(state.status, CalibrationStatus.idle);
    expect(state.measuredMs, 60.0);
  });
}
