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

import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/score_catalog.dart';

import '../support/notation_fakes.dart';

void main() {
  group('scoreCatalog', () {
    final catalog = ProviderContainer().read(scoreCatalogProvider);

    test('has at least one entry per practice level', () {
      for (final level in PracticeLevel.values) {
        expect(
          catalog.where((e) => e.level == level),
          isNotEmpty,
          reason: 'expected an entry for ${level.label}',
        );
      }
    });

    test('every entry exposes the required fields', () {
      expect(catalog, isNotEmpty);
      for (final e in catalog) {
        expect(e.id, isNotEmpty);
        expect(e.title, isNotEmpty);
        expect(e.composer, isNotEmpty);
        expect(e.assetPath, contains('assets/scores/'));
        expect(PracticeLevel.values, contains(e.level));
      }
    });

    test('ids are unique', () {
      final ids = catalog.map((e) => e.id).toSet();
      expect(ids.length, catalog.length);
    });
  });

  group('scoreAssetSource override', () {
    test('a test can supply in-memory bytes without the bundle', () async {
      final fake = FakeScoreAssetSource(Uint8List.fromList(const [9, 8, 7]));
      final container = ProviderContainer(
        overrides: [scoreAssetSourceProvider.overrideWithValue(fake)],
      );
      addTearDown(container.dispose);

      final source = container.read(scoreAssetSourceProvider);
      final bytes = await source.load('assets/scores/x.musicxml');
      expect(bytes, [9, 8, 7]);
      expect(fake.loaded, ['assets/scores/x.musicxml']);
    });
  });

  group('selectedScore', () {
    test('starts null and records the selected entry', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      expect(container.read(selectedScoreProvider), isNull);
      final entry = container.read(scoreCatalogProvider).first;
      container.read(selectedScoreProvider.notifier).select(entry);
      expect(container.read(selectedScoreProvider), entry);
    });
  });
}
