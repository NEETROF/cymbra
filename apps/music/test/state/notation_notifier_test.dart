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
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/notation_fakes.dart';

/// Lets the provider rebuild (on selection change) and the async asset load /
/// parse settle.
Future<void> _flush() async {
  for (var i = 0; i < 5; i++) {
    await Future<void>.delayed(Duration.zero);
  }
}

void main() {
  late FakeScoreAssetSource source;
  late FakeNotationEngine engine;
  late ProviderContainer container;

  ProviderContainer build({FakeNotationEngine? withEngine}) {
    source = FakeScoreAssetSource();
    engine = withEngine ?? FakeNotationEngine();
    final c = ProviderContainer(
      overrides: [
        scoreAssetSourceProvider.overrideWithValue(source),
        notationEngineProvider.overrideWithValue(engine),
      ],
    );
    addTearDown(c.dispose);
    c.listen(notationProvider, (_, _) {}, fireImmediately: true);
    return c;
  }

  NotationData read() => container.read(notationProvider);
  CatalogEntry firstEntry() => container.read(scoreCatalogProvider).first;

  test('no selection → empty notation data', () async {
    container = build();
    await _flush();
    expect(read().document, isNull);
    expect(read().systems, isEmpty);
    expect(read().error, isNull);
  });

  test('selecting an entry loads bytes and populates notation data', () async {
    container = build();
    container.read(selectedScoreProvider.notifier).select(firstEntry());
    await _flush();

    expect(source.loaded, hasLength(1));
    expect(read().document, isNotNull);
    expect(read().hasDocument, isTrue);
    expect(read().systems, isNotEmpty);
    expect(read().error, isNull);
  });

  test('a parse failure sets the error and clears the document', () async {
    container = build(
      withEngine: FakeNotationEngine(parseError: Exception('bad xml')),
    );
    container.read(selectedScoreProvider.notifier).select(firstEntry());
    await _flush();

    expect(read().error, contains('bad xml'));
    expect(read().document, isNull);
    expect(read().hasDocument, isFalse);
  });

  test('setAvailableWidth re-lays out the cached document', () async {
    container = build();
    container.read(selectedScoreProvider.notifier).select(firstEntry());
    await _flush();
    final before = engine.layoutCalls;

    container.read(notationProvider.notifier).setAvailableWidth(1234);
    expect(read().availableWidth, 1234);
    expect(engine.layoutCalls, before + 1);
    expect(engine.lastWidth, 1234);
  });

  test('a sub-threshold width change does not re-lay out', () async {
    container = build();
    container.read(selectedScoreProvider.notifier).select(firstEntry());
    await _flush();
    container.read(notationProvider.notifier).setAvailableWidth(1000);
    final calls = engine.layoutCalls;

    container.read(notationProvider.notifier).setAvailableWidth(1003);
    expect(engine.layoutCalls, calls, reason: 'change below threshold');
  });
}
