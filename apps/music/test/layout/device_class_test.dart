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
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/layout/device_class.dart';

void main() {
  group('deviceClassForShortestSide', () {
    test('a phone landscape shortest side (375) is a phone', () {
      expect(deviceClassForShortestSide(375), DeviceClass.phone);
    });

    test('just below the phone breakpoint is still a phone', () {
      expect(deviceClassForShortestSide(599), DeviceClass.phone);
    });

    test('a tablet landscape shortest side (768) is a tablet', () {
      expect(deviceClassForShortestSide(768), DeviceClass.tablet);
    });

    test('the phone breakpoint (600) becomes a tablet', () {
      expect(deviceClassForShortestSide(600), DeviceClass.tablet);
    });

    test('the desktop breakpoint (900) becomes a desktop', () {
      expect(deviceClassForShortestSide(900), DeviceClass.desktop);
    });

    test('a large shortest side (1200) is a desktop', () {
      expect(deviceClassForShortestSide(1200), DeviceClass.desktop);
    });
  });

  group('deviceClassOf', () {
    // Pumps a widget at [size] on [platform] and returns the resolved class.
    // Resets the platform override before returning (the widget-test framework
    // asserts all foundation debug vars are unset at the end of the test body,
    // which runs before tearDown — so an addTearDown reset would be too late).
    Future<DeviceClass> resolve(
      WidgetTester tester, {
      required Size size,
      required TargetPlatform platform,
    }) async {
      debugDefaultTargetPlatformOverride = platform;
      late DeviceClass resolved;
      try {
        await tester.pumpWidget(
          MediaQuery(
            data: MediaQueryData(size: size),
            child: Builder(
              builder: (context) {
                resolved = context.deviceClass;
                return const SizedBox();
              },
            ),
          ),
        );
      } finally {
        debugDefaultTargetPlatformOverride = null;
      }
      return resolved;
    }

    testWidgets('phone landscape on a mobile platform is a phone', (
      tester,
    ) async {
      expect(
        await resolve(
          tester,
          size: const Size(812, 375),
          platform: TargetPlatform.iOS,
        ),
        DeviceClass.phone,
      );
    });

    testWidgets('tablet landscape on a mobile platform is a tablet', (
      tester,
    ) async {
      expect(
        await resolve(
          tester,
          size: const Size(1024, 768),
          platform: TargetPlatform.iOS,
        ),
        DeviceClass.tablet,
      );
    });

    testWidgets('desktop platform is always desktop, even at phone size', (
      tester,
    ) async {
      expect(
        await resolve(
          tester,
          size: const Size(812, 375),
          platform: TargetPlatform.macOS,
        ),
        DeviceClass.desktop,
      );
    });
  });
}
