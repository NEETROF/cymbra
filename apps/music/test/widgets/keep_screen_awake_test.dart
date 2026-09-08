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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/screen_wake_service.dart';
import 'package:music/state/screen_wake.dart';
import 'package:music/widgets/keep_screen_awake.dart';

import 'keep_screen_awake_test.mocks.dart';

@GenerateNiceMocks([MockSpec<ScreenWakeService>()])
void main() {
  late MockScreenWakeService service;
  late ProviderContainer container;

  setUp(() {
    service = MockScreenWakeService();
    container = ProviderContainer(
      overrides: [screenWakeServiceProvider.overrideWithValue(service)],
    );
    addTearDown(container.dispose);
  });

  Future<void> pump(WidgetTester tester, Widget home) => tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MaterialApp(home: home),
    ),
  );

  testWidgets('mounting takes the hold', (tester) async {
    await pump(tester, const KeepScreenAwake(child: SizedBox()));

    verify(service.setEnabled(enabled: true)).called(1);
    expect(container.read(screenWakeProvider).holders, 1);
  });

  testWidgets('disposing releases it', (tester) async {
    await pump(tester, const KeepScreenAwake(child: SizedBox()));
    clearInteractions(service);

    await pump(tester, const SizedBox());

    verify(service.setEnabled(enabled: false)).called(1);
    expect(container.read(screenWakeProvider).holders, 0);
  });

  testWidgets('popping back to a non-play screen releases it', (tester) async {
    final key = GlobalKey<NavigatorState>();
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp(
          navigatorKey: key,
          home: const Scaffold(body: Text('library')),
        ),
      ),
    );
    unawaitedPush(key);
    await tester.pumpAndSettle();
    expect(container.read(screenWakeProvider).holders, 1);

    key.currentState!.pop();
    await tester.pumpAndSettle();

    expect(container.read(screenWakeProvider).holders, 0);
    verify(service.setEnabled(enabled: false)).called(1);
  });

  testWidgets('a surface opened over another keeps the hold on pop', (
    tester,
  ) async {
    final key = GlobalKey<NavigatorState>();
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp(
          navigatorKey: key,
          home: const KeepScreenAwake(child: Scaffold(body: Text('player'))),
        ),
      ),
    );
    unawaitedPush(key);
    await tester.pumpAndSettle();
    expect(container.read(screenWakeProvider).holders, 2);
    clearInteractions(service);

    key.currentState!.pop();
    await tester.pumpAndSettle();

    // The player is still on screen underneath — its hold must survive.
    expect(container.read(screenWakeProvider).holders, 1);
    verifyNever(service.setEnabled(enabled: false));
  });
}

/// Pushes a second play surface without awaiting the route's completion.
void unawaitedPush(GlobalKey<NavigatorState> key) {
  key.currentState!.push(
    MaterialPageRoute<void>(
      builder: (_) =>
          const KeepScreenAwake(child: Scaffold(body: Text('measures'))),
    ),
  );
}
