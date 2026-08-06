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
import 'package:flutter_timezone/flutter_timezone.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'timezone_service.g.dart';

/// The device's IANA timezone name (change: add-push-notifications, task 4.4).
///
/// Reported to the server so a schedule expressed in *local* hours (e.g. an 20:00
/// reminder) fires at the right moment for each user. Dart core exposes only an
/// abbreviation and a UTC offset, neither of which survives DST, so this reads the
/// real zone name from the platform. Behind a provider so state is testable
/// without the native plugin.
abstract class TimezoneService {
  /// The IANA name (e.g. `Europe/Paris`), or `null` when the platform cannot
  /// report one — the server then keeps whatever it already had.
  Future<String?> current();
}

/// Production [TimezoneService] over `flutter_timezone`.
class PlatformTimezoneService implements TimezoneService {
  const PlatformTimezoneService();

  @override
  Future<String?> current() async {
    try {
      final name = await FlutterTimezone.getLocalTimezone();
      return name.isEmpty ? null : name;
    } catch (_) {
      // Unsupported platform / plugin unavailable: leave the stored zone alone.
      return null;
    }
  }
}

/// Production timezone-service provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
TimezoneService timezoneService(Ref ref) => const PlatformTimezoneService();
