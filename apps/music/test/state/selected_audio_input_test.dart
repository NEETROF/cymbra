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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/audio_capture_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/src/rust/api/audio_input.dart' show InputRouteKind;
import 'package:music/state/input_calibration_notifier.dart';
import 'package:music/state/selected_audio_input.dart';

import '../support/prefs_fakes.dart';
import 'input_calibration_notifier_test.mocks.dart';

/// The desktop input-device selection (spec: Desktop Capture Device
/// Selection): persisted, applied through the seam, inert where unsupported —
/// and the calibration route follows it, so measurements stay keyed to the
/// device that actually answers.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const builtin = CaptureRoute(
    name: 'MacBook Pro Microphone',
    kind: InputRouteKind.other,
    refusedBluetooth: false,
  );
  const scarlett = CaptureRoute(
    name: 'Scarlett 2i2',
    kind: InputRouteKind.other,
    refusedBluetooth: false,
  );

  late MockAudioCaptureService service;
  late FakePreferencesService prefs;

  ProviderContainer harness({
    bool supports = true,
    Map<String, String>? seeded,
  }) {
    service = MockAudioCaptureService();
    prefs = FakePreferencesService(seeded);
    when(service.supportsDeviceSelection).thenReturn(supports);
    when(service.listInputs()).thenAnswer((_) async => [builtin, scarlett]);
    when(service.selectInput(any)).thenAnswer((_) async {});
    when(
      service.routeChanges(),
    ).thenAnswer((_) => const Stream<CaptureRoute?>.empty());
    when(service.activeRoute()).thenAnswer((_) async => builtin);
    final container = ProviderContainer(
      overrides: [
        audioCaptureServiceProvider.overrideWithValue(service),
        preferencesServiceProvider.overrideWithValue(prefs),
      ],
    );
    addTearDown(container.dispose);
    return container;
  }

  Future<void> settle() async {
    for (var i = 0; i < 3; i++) {
      await Future<void>.delayed(Duration.zero);
    }
  }

  test('restores and applies the persisted choice at launch', () async {
    final container = harness(
      seeded: {SelectedAudioInput.prefsKey: 'Scarlett 2i2'},
    );
    container.read(selectedAudioInputProvider);
    await settle();

    final state = container.read(selectedAudioInputProvider);
    expect(state.hydrated, isTrue);
    expect(state.selected, 'Scarlett 2i2');
    expect(state.inputs, [builtin, scarlett]);
    // Applied to the engine, which owns the absent-device fallback.
    verify(service.selectInput('Scarlett 2i2')).called(1);
  });

  test('selecting persists; the default clears the key', () async {
    final container = harness();
    container.read(selectedAudioInputProvider);
    await settle();

    final notifier = container.read(selectedAudioInputProvider.notifier);
    await notifier.select('Scarlett 2i2');
    expect(prefs.store[SelectedAudioInput.prefsKey], 'Scarlett 2i2');
    verify(service.selectInput('Scarlett 2i2')).called(1);

    await notifier.select(null);
    expect(prefs.store.containsKey(SelectedAudioInput.prefsKey), isFalse);
    verify(service.selectInput(null)).called(greaterThanOrEqualTo(1));
  });

  test('an unsupported platform stays inert', () async {
    final container = harness(
      supports: false,
      seeded: {SelectedAudioInput.prefsKey: 'Scarlett 2i2'},
    );
    container.read(selectedAudioInputProvider);
    await settle();

    final state = container.read(selectedAudioInputProvider);
    expect(state.supportsSelection, isFalse);
    expect(state.inputs, isEmpty);
    verifyNever(service.selectInput(any));
    verifyNever(service.listInputs());
  });

  test('the calibration route follows a selection change', () async {
    final container = harness();
    // Keep both providers alive so the calibration listener reacts.
    final sub = container.listen(inputCalibrationProvider, (_, _) {});
    addTearDown(sub.close);
    container.read(selectedAudioInputProvider);
    await settle();
    expect(container.read(inputCalibrationProvider).route, builtin);

    // The seam now resolves the pinned device as what capture would open.
    when(service.activeRoute()).thenAnswer((_) async => scarlett);
    await container
        .read(selectedAudioInputProvider.notifier)
        .select('Scarlett 2i2');
    await settle();

    expect(container.read(inputCalibrationProvider).route, scarlett);
  });
}
