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
import 'package:music/courses/course_manifest.dart';
import 'package:music/screens/open_score.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/state/score_catalog.dart';

/// A lookup that knows one catalogue piece and fails on anything else.
class _FakeLookup implements CatalogScoreLookup {
  @override
  Future<CatalogHit> getCatalogScore(String catalogId) async {
    if (catalogId != 'cat-1') throw StateError('unknown $catalogId');
    return const CatalogHit(
      id: 'cat-1',
      title: 'Catalogue Minuet',
      composer: 'Someone',
      level: PracticeLevel.intermediate,
      license: 'CC0',
      source: 'pdmx',
    );
  }
}

const _bundled = CatalogEntry(
  id: 'b1',
  title: 'Bundled Piece',
  composer: 'X',
  assetPath: 'assets/scores/beginner/b1.musicxml',
  level: PracticeLevel.beginner,
);

void main() {
  // change: add-music-piece-lessons
  late ProviderContainer container;
  late WidgetRef ref;

  Future<void> pump(WidgetTester tester) async {
    container = ProviderContainer(
      overrides: [
        scoreCatalogProvider.overrideWithValue(const [_bundled]),
        catalogScoreLookupProvider.overrideWithValue(_FakeLookup()),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: Consumer(
          builder: (context, r, _) {
            ref = r;
            return const SizedBox();
          },
        ),
      ),
    );
  }

  testWidgets('a bundled reference resolves to the shipped entry', (
    tester,
  ) async {
    await pump(tester);
    final entry = await resolvePieceEntry(
      ref,
      const PieceScoreRef.bundled('b1'),
    );
    expect(entry, _bundled);
  });

  testWidgets('an unknown bundled id resolves to nothing', (tester) async {
    await pump(tester);
    expect(
      await resolvePieceEntry(ref, const PieceScoreRef.bundled('nope')),
      isNull,
    );
  });

  testWidgets('a catalogue reference is looked up and mapped to an entry', (
    tester,
  ) async {
    await pump(tester);
    final entry = await resolvePieceEntry(
      ref,
      const PieceScoreRef.catalog('cat-1'),
    );
    expect(entry, isNotNull);
    expect(entry!.catalogId, 'cat-1');
    expect(entry.title, 'Catalogue Minuet');
  });

  testWidgets('a failing catalogue lookup resolves to nothing, never throws', (
    tester,
  ) async {
    await pump(tester);
    expect(
      await resolvePieceEntry(ref, const PieceScoreRef.catalog('gone')),
      isNull,
    );
  });
}
