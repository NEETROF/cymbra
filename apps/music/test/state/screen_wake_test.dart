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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/screen_wake_service.dart';
import 'package:music/state/screen_wake.dart';

import 'screen_wake_test.mocks.dart';

@GenerateNiceMocks([MockSpec<ScreenWakeService>()])
ProviderContainer _container(ScreenWakeService service) {
  final c = ProviderContainer(
    overrides: [screenWakeServiceProvider.overrideWithValue(service)],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  group('ScreenWake counting', () {
    test('the first acquire holds the screen', () {
      final service = MockScreenWakeService();
      final c = _container(service);

      c.read(screenWakeProvider.notifier).acquire();

      verify(service.setEnabled(enabled: true)).called(1);
      expect(c.read(screenWakeProvider).shouldHold, isTrue);
    });

    test('a second acquire does not call the platform again', () {
      final service = MockScreenWakeService();
      final c = _container(service);
      final notifier = c.read(screenWakeProvider.notifier);

      notifier
        ..acquire()
        ..acquire();

      // A stack of surfaces is ONE platform call, not one per surface.
      verify(service.setEnabled(enabled: true)).called(1);
      expect(c.read(screenWakeProvider).holders, 2);
    });

    test('releasing one of two keeps the screen held', () {
      final service = MockScreenWakeService();
      final c = _container(service);
      final notifier = c.read(screenWakeProvider.notifier);

      notifier
        ..acquire()
        ..acquire()
        ..release();

      // The regression this exists for: the measure selector closing over the
      // player must not drop the player's hold.
      verifyNever(service.setEnabled(enabled: false));
      expect(c.read(screenWakeProvider).shouldHold, isTrue);
    });

    test('releasing the last one lets the screen sleep', () {
      final service = MockScreenWakeService();
      final c = _container(service);
      final notifier = c.read(screenWakeProvider.notifier);

      notifier
        ..acquire()
        ..acquire()
        ..release()
        ..release();

      verify(service.setEnabled(enabled: false)).called(1);
      expect(c.read(screenWakeProvider).shouldHold, isFalse);
    });

    test('an unbalanced release neither goes negative nor calls out', () {
      final service = MockScreenWakeService();
      final c = _container(service);
      final notifier = c.read(screenWakeProvider.notifier);

      notifier
        ..acquire()
        ..release()
        ..release()
        ..release();

      expect(c.read(screenWakeProvider).holders, 0);
      // A count driven negative would make the next acquire a silent no-op.
      verify(service.setEnabled(enabled: false)).called(1);

      clearInteractions(service);
      notifier.acquire();
      verify(service.setEnabled(enabled: true)).called(1);
    });
  });

  group('ScreenWake foreground', () {
    test('backgrounding drops a held screen', () {
      final service = MockScreenWakeService();
      final c = _container(service);
      final notifier = c.read(screenWakeProvider.notifier)..acquire();
      clearInteractions(service);

      notifier.setForeground(false);

      // Load-bearing on the desktops, where the hold is a process-level power
      // assertion that outlives losing focus.
      verify(service.setEnabled(enabled: false)).called(1);
      expect(c.read(screenWakeProvider).shouldHold, isFalse);
      // The surface is still mounted — only the foreground changed.
      expect(c.read(screenWakeProvider).holders, 1);
    });

    test('resuming re-takes the hold when a surface is still mounted', () {
      final service = MockScreenWakeService();
      final c = _container(service);
      final notifier = c.read(screenWakeProvider.notifier)..acquire();
      notifier.setForeground(false);
      clearInteractions(service);

      notifier.setForeground(true);

      verify(service.setEnabled(enabled: true)).called(1);
      expect(c.read(screenWakeProvider).shouldHold, isTrue);
    });

    test('resuming with no surface mounted takes nothing', () {
      final service = MockScreenWakeService();
      final c = _container(service);
      final notifier = c.read(screenWakeProvider.notifier);

      notifier
        ..setForeground(false)
        ..setForeground(true);

      verifyNever(service.setEnabled(enabled: anyNamed('enabled')));
      expect(c.read(screenWakeProvider).shouldHold, isFalse);
    });

    test('a surface mounted while backgrounded holds nothing until resume', () {
      final service = MockScreenWakeService();
      final c = _container(service);
      final notifier = c.read(screenWakeProvider.notifier);

      notifier
        ..setForeground(false)
        ..acquire();

      verifyNever(service.setEnabled(enabled: true));

      notifier.setForeground(true);
      verify(service.setEnabled(enabled: true)).called(1);
    });
  });

  group('WakelockPlusScreenWakeService', () {
    test('a refusing platform is swallowed, never thrown', () {
      // The Linux/Xvfb case: no screensaver service answers the D-Bus call. The
      // plugin is not registered in a VM test either, so the real call fails
      // here for the same reason it would there.
      const service = WakelockPlusScreenWakeService();

      expect(service.setEnabled(enabled: true), completes);
      expect(service.setEnabled(enabled: false), completes);
    });
  });
}
