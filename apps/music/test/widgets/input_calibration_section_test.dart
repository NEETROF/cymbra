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

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/audio_capture_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/src/rust/api/audio_input.dart' show InputRouteKind;
import 'package:music/state/input_calibration_notifier.dart';
import 'package:music/widgets/input_calibration_section.dart';

import '../state/input_calibration_notifier_test.mocks.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';

/// The calibration section (change: add-acoustic-piano-input): route + stored
/// measurement, and one localized guidance line per failure state.
void main() {
  const builtin = CaptureRoute(
    name: 'Built-in Microphone',
    kind: InputRouteKind.builtin,
    refusedBluetooth: false,
  );
  const bluetooth = CaptureRoute(
    name: 'AirPods',
    kind: InputRouteKind.bluetooth,
    refusedBluetooth: true,
  );

  late MockAudioCaptureService service;

  Future<ProviderContainer> pumpSection(
    WidgetTester tester, {
    CaptureRoute? route = builtin,
    Map<String, String>? seeded,
  }) async {
    service = MockAudioCaptureService();
    when(
      service.routeChanges(),
    ).thenAnswer((_) => const Stream<CaptureRoute?>.empty());
    when(service.activeRoute()).thenAnswer((_) async => route);
    when(
      service.permissionStatus(),
    ).thenAnswer((_) async => InputPermissionStatus.granted);
    final container = ProviderContainer(
      overrides: [
        audioCaptureServiceProvider.overrideWithValue(service),
        preferencesServiceProvider.overrideWithValue(
          FakePreferencesService(seeded),
        ),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(
          const Scaffold(
            body: SingleChildScrollView(child: InputCalibrationSection()),
          ),
        ),
      ),
    );
    await tester.pump();
    await tester.pump();
    return container;
  }

  testWidgets('shows the route and its stored measurement', (tester) async {
    await pumpSection(
      tester,
      seeded: {
        InputCalibration.prefsKey: jsonEncode({'Built-in Microphone': 85.0}),
      },
    );

    expect(find.text('Built-in microphone'), findsOneWidget);
    expect(find.text('Measured delay: 85 ms'), findsOneWidget);
  });

  testWidgets('an uncalibrated route reads as such', (tester) async {
    await pumpSection(tester);

    expect(find.text('Not calibrated yet'), findsOneWidget);
  });

  testWidgets('the run button drives a calibration to done', (tester) async {
    when(
      service.runCalibration(),
    ).thenAnswer((_) async => const CalibrationMeasurement(latencyMs: 90.4));
    await pumpSection(tester);
    when(
      service.runCalibration(),
    ).thenAnswer((_) async => const CalibrationMeasurement(latencyMs: 90.4));

    await tester.tap(find.text('Calibrate'));
    await tester.pumpAndSettle();

    expect(find.text('Measured delay: 90 ms'), findsOneWidget);
  });

  testWidgets('a Bluetooth microphone shows the refusal copy', (tester) async {
    await pumpSection(tester, route: bluetooth);
    await tester.tap(find.text('Calibrate'));
    await tester.pumpAndSettle();

    expect(
      find.textContaining('Bluetooth microphones cannot be used'),
      findsOneWidget,
    );
    verifyNever(service.runCalibration());
  });

  testWidgets('a denied permission points to the system settings', (
    tester,
  ) async {
    await pumpSection(tester);
    when(
      service.permissionStatus(),
    ).thenAnswer((_) async => InputPermissionStatus.denied);

    await tester.tap(find.text('Calibrate'));
    await tester.pumpAndSettle();

    expect(
      find.textContaining('Allow it in your system settings'),
      findsOneWidget,
    );
  });

  testWidgets('an undetected run shows the environment guidance', (
    tester,
  ) async {
    await pumpSection(tester);
    when(service.runCalibration()).thenAnswer((_) async => null);

    await tester.tap(find.text('Calibrate'));
    await tester.pumpAndSettle();

    expect(find.textContaining('The sound was not detected'), findsOneWidget);
  });
}
