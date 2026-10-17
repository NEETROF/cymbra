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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/courses/course_manifest.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/placement_notifier.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;

import '../support/daily_fixtures.dart';
import '../support/prefs_fakes.dart';

const _placementListing = CourseListing(
  id: 'placement-piano',
  schemaVersion: 2,
  track: kPlacementTrack,
  sortOrder: 1,
  title: {'en': 'Find your starting point'},
);

ProviderContainer _container({
  Map<String, String> prefs = const {},
  bool withPlacement = true,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
      nowFnProvider.overrideWithValue(() => DateTime(2026, 10, 17, 9)),
      courseCatalogServiceProvider.overrideWithValue(
        DailyFakeCatalog([
          ...dailyListings,
          if (withPlacement) _placementListing,
        ]),
      ),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _tick() => Future<void>.delayed(const Duration(milliseconds: 20));

void main() {
  group('the catalogue and the placement track', () {
    test(
      'the path listing leaves the placement out; its own provider has it',
      () async {
        final c = _container();
        final path = await c.read(coursesProvider.future);
        expect(path.map((l) => l.id), isNot(contains('placement-piano')));
        expect(path, hasLength(dailyListings.length));
        final placement = await c.read(placementCourseProvider.future);
        expect(placement?.id, 'placement-piano');
      },
    );

    test('no placement published → null, nothing offered', () async {
      final c = _container(withPlacement: false);
      expect(await c.read(placementCourseProvider.future), isNull);
    });
  });

  group('the placement pointer', () {
    test('starts empty and not taken, then loaded', () async {
      final c = _container();
      expect(c.read(placementProvider).startUnit, isNull);
      await _tick();
      final p = c.read(placementProvider);
      expect(p.loaded, isTrue);
      expect(p.taken, isFalse);
    });

    test('set persists and marks the placement taken', () async {
      final prefs = FakePreferencesService();
      final c = ProviderContainer(
        overrides: [
          preferencesServiceProvider.overrideWithValue(prefs),
          nowFnProvider.overrideWithValue(() => DateTime(2026, 10, 17, 9)),
          courseCatalogServiceProvider.overrideWithValue(
            const DailyFakeCatalog(),
          ),
        ],
      );
      addTearDown(c.dispose);
      await c.read(placementProvider.notifier).set('u3');
      final p = c.read(placementProvider);
      expect(p.startUnit, 'u3');
      expect(p.taken, isTrue);
      expect(jsonDecode(prefs.store['placement.v1']!)['startUnit'], 'u3');
    });

    test('restores the pointer; a retake replaces it', () async {
      final c = _container(
        prefs: {
          'placement.v1': jsonEncode({
            'startUnit': 'u3',
            'takenAtMs': DateTime(2026, 10, 1).millisecondsSinceEpoch,
          }),
        },
      );
      c.read(placementProvider);
      await _tick();
      expect(c.read(placementProvider).startUnit, 'u3');
      await c.read(placementProvider.notifier).set('u2');
      expect(c.read(placementProvider).startUnit, 'u2');
    });

    test('clear starts from the beginning but stays taken', () async {
      final c = _container();
      await c.read(placementProvider.notifier).set('u3');
      await c.read(placementProvider.notifier).clear();
      final p = c.read(placementProvider);
      expect(p.startUnit, isNull);
      expect(p.taken, isTrue, reason: 'the offer does not come back');
    });

    test('the skipped units follow the pointer and the path', () async {
      final c = _container();
      final sub = c.listen(skippedUnitsProvider, (_, _) {});
      addTearDown(sub.close);
      await c.read(coursesProvider.future);
      await _tick();
      expect(c.read(skippedUnitsProvider), isEmpty);
      await c.read(placementProvider.notifier).set('u3');
      await _tick();
      expect(c.read(skippedUnitsProvider), {'u1', 'u2'});
      await c.read(placementProvider.notifier).clear();
      await _tick();
      expect(c.read(skippedUnitsProvider), isEmpty);
    });
  });
}
