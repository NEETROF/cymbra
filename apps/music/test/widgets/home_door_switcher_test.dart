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
import 'package:music/services/preferences_service.dart';
import 'package:music/state/drums_access.dart';
import 'package:music/state/home_door.dart';
import 'package:music/state/learn_door_access.dart';
import 'package:music/widgets/home_door_switcher.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';

final _switcher = find.byKey(const Key('home-door-switcher'));

Future<ProviderContainer> _pump(
  WidgetTester tester, {
  FakePreferencesService? prefs,
  bool learnDoor = true,
  bool drumsVisible = false,
  Size size = const Size(1400, 900),
}) async {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      learnDoorEnabledProvider.overrideWithValue(learnDoor),
      drumsEnabledProvider.overrideWithValue(drumsVisible),
    ],
  );
  addTearDown(c.dispose);
  // The view size is what MediaQuery (and so the device class) reads;
  // setSurfaceSize alone leaves it at the 800x600 default.
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1.0;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        const Scaffold(body: Center(child: HomeDoorSwitcher())),
        locale: const Locale('en'),
      ),
    ),
  );
  await tester.pump();
  await tester.pump();
  return c;
}

void main() {
  testWidgets('offers Play and Learn with labels on a large viewport, Play '
      'selected by default', (tester) async {
    await _pump(tester);
    expect(_switcher, findsOneWidget);
    expect(find.text('Play'), findsOneWidget);
    expect(find.text('Learn'), findsOneWidget);
    final button = tester.widget<SegmentedButton<HomeDoor>>(_switcher);
    expect(button.selected, {HomeDoor.play});
  });

  testWidgets('tapping Learn selects and persists the door', (tester) async {
    final prefs = FakePreferencesService();
    final c = await _pump(tester, prefs: prefs);
    await tester.tap(find.text('Learn'));
    await tester.pump();
    await tester.pump();
    expect(c.read(homeDoorChoiceProvider).door, HomeDoor.learn);
    expect(prefs.store[HomeDoorChoice.prefsKey], 'learn');
    final button = tester.widget<SegmentedButton<HomeDoor>>(_switcher);
    expect(button.selected, {HomeDoor.learn});
  });

  testWidgets('a phone viewport shows icon-only segments that keep their '
      'labels for assistive tech', (tester) async {
    await _pump(tester, size: const Size(844, 390));
    expect(_switcher, findsOneWidget);
    expect(find.text('Play'), findsNothing);
    expect(find.text('Learn'), findsNothing);
    expect(find.byIcon(Icons.play_circle_outline), findsOneWidget);
    expect(find.byIcon(Icons.school_outlined), findsOneWidget);
    expect(find.bySemanticsLabel('Learn'), findsWidgets);
  });

  testWidgets('under the drums context only Play is offered', (tester) async {
    await _pump(
      tester,
      drumsVisible: true,
      prefs: FakePreferencesService({
        'instrument_context': jsonEncode({
          'context': 'drums',
          'choiceOffered': true,
        }),
        HomeDoorChoice.prefsKey: 'learn',
      }),
    );
    expect(_switcher, findsOneWidget);
    expect(find.text('Play'), findsOneWidget);
    expect(find.text('Learn'), findsNothing);
    final button = tester.widget<SegmentedButton<HomeDoor>>(_switcher);
    expect(button.selected, {HomeDoor.play});
  });

  testWidgets('absent while the two-door home is switched off', (tester) async {
    await _pump(tester, learnDoor: false);
    expect(_switcher, findsNothing);
  });
}
