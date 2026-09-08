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

import 'package:flutter/foundation.dart' show debugPrint;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

part 'screen_wake_service.g.dart';

/// The platform's "don't let the screen sleep" request (change:
/// keep-play-surfaces-awake).
///
/// Behind a seam because it is a native plugin: unit and widget tests exercise
/// every rule about *when* the screen is held on the Dart VM with nothing native
/// loaded, and no widget or notifier imports `wakelock_plus` directly.
///
/// Nothing here decides policy. It is a dumb switch; the counting, the
/// foreground interaction and the flip detection all live in [ScreenWake].
abstract class ScreenWakeService {
  /// Ask the platform to hold the screen awake ([enabled] true) or let it sleep
  /// normally again. Idempotent: calling it twice with the same value is
  /// harmless, and it never throws — a platform that refuses is logged and
  /// ignored (keeping the screen lit is a comfort, not a precondition for
  /// practising).
  Future<void> setEnabled({required bool enabled});
}

/// Production [ScreenWakeService] over `wakelock_plus`.
///
/// The plugin picks the right primitive per platform — `FLAG_KEEP_SCREEN_ON` on
/// Android, `isIdleTimerDisabled` on the Apple platforms, and a process-level
/// power assertion on the three desktops. That last family is why [ScreenWake]
/// releases on background: a desktop assertion survives losing focus, so nothing
/// would drop it for us.
class WakelockPlusScreenWakeService implements ScreenWakeService {
  const WakelockPlusScreenWakeService();

  @override
  Future<void> setEnabled({required bool enabled}) async {
    try {
      await (enabled ? WakelockPlus.enable() : WakelockPlus.disable());
    } catch (e) {
      // A platform with nothing to talk to — notably Linux, where the
      // implementation calls the freedesktop screensaver over D-Bus and CI runs
      // the integration suite headless under Xvfb. Degrade to the device's
      // normal sleep behaviour; never fail the screen this was requested for.
      debugPrint('screen wakelock setEnabled($enabled) failed ($e); ignoring.');
    }
  }
}

/// Production screen-wake provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
ScreenWakeService screenWakeService(Ref ref) =>
    const WakelockPlusScreenWakeService();
