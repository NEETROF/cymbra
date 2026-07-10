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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/preferences_service.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/prefs_fakes.dart';

void main() {
  group('FakePreferencesService', () {
    test('a written value round-trips', () async {
      final prefs = FakePreferencesService();
      await prefs.setString('k', 'v');
      expect(await prefs.getString('k'), 'v');
    });

    test('a missing key reads as null', () async {
      final prefs = FakePreferencesService();
      expect(await prefs.getString('never-written'), isNull);
    });

    test('remove deletes the value', () async {
      final prefs = FakePreferencesService({'k': 'v'});
      await prefs.remove('k');
      expect(await prefs.getString('k'), isNull);
    });

    test(
      'value survives a "relaunch" (a new service over the same store)',
      () async {
        final first = FakePreferencesService();
        await first.setString('k', 'v');
        final second = FakePreferencesService(first.store);
        expect(await second.getString('k'), 'v');
      },
    );
  });

  group('SharedPreferencesService', () {
    setUp(() => SharedPreferences.setMockInitialValues({}));

    test('round-trips through on-device storage', () async {
      final prefs = SharedPreferencesService();
      expect(await prefs.getString('k'), isNull);
      await prefs.setString('k', 'v');
      expect(await prefs.getString('k'), 'v');
      await prefs.remove('k');
      expect(await prefs.getString('k'), isNull);
    });

    test('reads a value persisted before this launch', () async {
      SharedPreferences.setMockInitialValues({'k': 'stored'});
      final prefs = SharedPreferencesService();
      expect(await prefs.getString('k'), 'stored');
    });
  });
}
