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

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/analytics/usage_actions.dart';
import 'package:music/analytics/usage_environment.dart';
import 'package:music/analytics/usage_event_record.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  tearDown(() => debugDefaultTargetPlatformOverride = null);

  test('usagePlatform maps each target platform', () {
    const cases = {
      TargetPlatform.iOS: 'ios',
      TargetPlatform.android: 'android',
      TargetPlatform.macOS: 'macos',
      TargetPlatform.windows: 'windows',
      TargetPlatform.linux: 'linux',
    };
    cases.forEach((platform, expected) {
      debugDefaultTargetPlatformOverride = platform;
      expect(usagePlatform(), expected);
    });
  });

  test('desktop platforms always classify as desktop', () {
    for (final p in [
      TargetPlatform.macOS,
      TargetPlatform.windows,
      TargetPlatform.linux,
    ]) {
      debugDefaultTargetPlatformOverride = p;
      expect(usageDeviceClass(), 'desktop');
    }
  });

  test('a mobile platform derives a valid non-desktop device class', () {
    debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
    expect(usageDeviceClass(), anyOf('phone', 'tablet'));
  });

  test(
    'UsageEventRecord round-trips through JSON (optional fields omitted)',
    () {
      const full = UsageEventRecord(
        id: 'id-1',
        action: UsageActions.playStart,
        variant: 'fall_note',
        subjectId: 'score-9',
        platform: 'ios',
        deviceClass: 'phone',
        appVersion: '1.2.3',
        locale: 'fr',
        occurredAtMs: 1718494200000,
      );
      final back = UsageEventRecord.fromJson(full.toJson());
      expect(back.id, full.id);
      expect(back.action, full.action);
      expect(back.variant, 'fall_note');
      expect(back.subjectId, 'score-9');
      expect(back.occurredAtMs, full.occurredAtMs);

      // Optional fields absent → omitted from JSON, decode back to null.
      const minimal = UsageEventRecord(
        id: 'id-2',
        action: UsageActions.authSignIn,
        platform: 'web',
        deviceClass: 'desktop',
        appVersion: '1.0.0',
        locale: 'en',
        occurredAtMs: 1,
      );
      final json = minimal.toJson();
      expect(json.containsKey('variant'), isFalse);
      expect(json.containsKey('subjectId'), isFalse);
      expect(UsageEventRecord.fromJson(json).variant, isNull);
    },
  );
}
