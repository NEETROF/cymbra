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
import 'package:music/courses/course_manifest.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/preferences_service.dart';

import '../support/prefs_fakes.dart';

/// Hand fake of the gRPC seam (special case: a network client seam), toggleable
/// to simulate the server being unreachable.
class FakeCourseCatalogService implements CourseCatalogService {
  FakeCourseCatalogService({
    this.listings = const [],
    this.manifests = const {},
    this.offline = false,
  });

  List<CourseListing> listings;
  Map<String, String> manifests;
  bool offline;

  @override
  Future<List<CourseListing>> listCourses() async {
    if (offline) throw StateError('offline');
    return listings;
  }

  @override
  Future<String?> getCourseManifestJson(String id) async {
    if (offline) throw StateError('offline');
    return manifests[id];
  }
}

ProviderContainer _container(
  FakeCourseCatalogService service,
  FakePreferencesService prefs,
) {
  final c = ProviderContainer(
    overrides: [
      courseCatalogServiceProvider.overrideWithValue(service),
      preferencesServiceProvider.overrideWithValue(prefs),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

const _listing = CourseListing(
  id: 'reading-the-staff',
  schemaVersion: 1,
  track: 'solfege',
  level: 'beginner',
  sortOrder: 1,
  title: {'en': 'Reading the staff', 'fr': 'Lire la portée'},
);

void main() {
  test('courses lists from the server and refreshes the cache', () async {
    final prefs = FakePreferencesService();
    final service = FakeCourseCatalogService(listings: const [_listing]);
    final c = _container(service, prefs);

    final list = await c.read(coursesProvider.future);
    expect(list, hasLength(1));
    expect(list.first.id, 'reading-the-staff');
    // The listing was cached for offline use.
    expect(
      prefs.store.keys.any((k) => k.startsWith('courses.listing')),
      isTrue,
    );
  });

  test(
    'courses falls back to the cache when the server is unreachable',
    () async {
      final prefs = FakePreferencesService();
      // First, a successful fetch populates the cache…
      await _container(
        FakeCourseCatalogService(listings: const [_listing]),
        prefs,
      ).read(coursesProvider.future);

      // …then, offline, the same cache is served (same prefs, failing service).
      final offline = await _container(
        FakeCourseCatalogService(offline: true),
        prefs,
      ).read(coursesProvider.future);
      expect(offline, hasLength(1));
      expect(offline.first.id, 'reading-the-staff');
      expect(resolveInline(offline.first.title, 'fr'), 'Lire la portée');
    },
  );

  test('courses yields an empty list offline with no cache', () async {
    final list = await _container(
      FakeCourseCatalogService(offline: true),
      FakePreferencesService(),
    ).read(coursesProvider.future);
    expect(list, isEmpty);
  });

  test('courseManifest fetches, parses and caches the blob', () async {
    final prefs = FakePreferencesService();
    final service = FakeCourseCatalogService(
      manifests: {
        'c1':
            '{"schemaVersion":1,"id":"c1","blocks":['
            '{"type":"text","text":{"en":"hi"}}]}',
      },
    );
    final m = await _container(
      service,
      prefs,
    ).read(courseManifestProvider('c1').future);
    expect(m, isNotNull);
    expect(m!.blocks, hasLength(1));
    expect(m.blocks.first, isA<TextBlock>());
    expect(prefs.store.containsKey('courses.manifest.v1.c1'), isTrue);
  });

  test('courseManifest serves the cached blob when offline', () async {
    final prefs = FakePreferencesService();
    const raw = '{"schemaVersion":1,"id":"c1","blocks":[]}';
    await _container(
      FakeCourseCatalogService(manifests: const {'c1': raw}),
      prefs,
    ).read(courseManifestProvider('c1').future);

    final offline = await _container(
      FakeCourseCatalogService(offline: true),
      prefs,
    ).read(courseManifestProvider('c1').future);
    expect(offline, isNotNull);
    expect(offline!.id, 'c1');
  });

  test('courseManifest is null for an unknown course', () async {
    final m = await _container(
      FakeCourseCatalogService(manifests: const {}),
      FakePreferencesService(),
    ).read(courseManifestProvider('nope').future);
    expect(m, isNull);
  });
}
