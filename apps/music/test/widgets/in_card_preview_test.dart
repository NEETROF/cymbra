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
import 'package:music/services/audio_service.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/state/card_preview_notifier.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/widgets/in_card_preview.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/rating_fakes.dart';

List<Override> _overrides(RecordingAudioService audio) => [
  catalogServiceProvider.overrideWithValue(
    FakeDeckCatalogService(deckCorpus(1)),
  ),
  notationEngineProvider.overrideWithValue(FakeNotationEngine()),
  audioServiceProvider.overrideWithValue(audio),
];

Future<void> _pumpPreview(
  WidgetTester tester, {
  required RecordingAudioService audio,
  ValueChanged<double>? onProgress,
}) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: _overrides(audio),
      child: localizedApp(
        Center(
          child: SizedBox(
            width: 320,
            height: 200,
            child: InCardPreview(catalogId: 'c0', onProgress: onProgress),
          ),
        ),
      ),
    ),
  );
  // The score loads asynchronously; a couple of pumps resolve the future. Never
  // pumpAndSettle — the preview's ticker animates continuously.
  await tester.pump();
  await tester.pump();
}

void main() {
  test('cardPreviewScore loads and parses through the seams', () async {
    final c = ProviderContainer(
      overrides: [
        catalogServiceProvider.overrideWithValue(
          FakeDeckCatalogService(deckCorpus(1)),
        ),
        notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      ],
    );
    addTearDown(c.dispose);
    final score = await c.read(cardPreviewScoreProvider('c0').future);
    expect(score.notes, isNotEmpty);
    expect(score.songEndMs, greaterThan(0));
    expect(score.isEmpty, isFalse);
  });

  testWidgets('the preview sounds the score and never scores it', (
    tester,
  ) async {
    final audio = RecordingAudioService();
    await _pumpPreview(tester, audio: audio);
    // Advance the ticker across the opening notes.
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }
    // Read-only preview actually sounds the notes…
    expect(audio.noteOns, isNotEmpty);
    // …and never touches the performance scorer (no judging / no scoring).
    final container = ProviderScope.containerOf(
      tester.element(find.byType(InCardPreview)),
    );
    expect(container.read(performanceScorerProvider).active, isFalse);
    expect(container.read(performanceScorerProvider).lastResult, isNull);
  });

  testWidgets('the preview reports playback progress for the gate', (
    tester,
  ) async {
    var maxFraction = 0.0;
    final audio = RecordingAudioService();
    await _pumpPreview(
      tester,
      audio: audio,
      onProgress: (f) => maxFraction = f > maxFraction ? f : maxFraction,
    );
    // Auto-plays: after a few frames it has reported forward progress (> 0).
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }
    expect(maxFraction, greaterThan(0));
  });

  testWidgets('a failed preview unlocks rating (reports full progress)', (
    tester,
  ) async {
    var progress = 0.0;
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          catalogServiceProvider.overrideWithValue(
            FakeDeckCatalogService(deckCorpus(1)),
          ),
          // Parsing throws → the preview can't load.
          notationEngineProvider.overrideWithValue(
            FakeNotationEngine(parseError: Exception('boom')),
          ),
          audioServiceProvider.overrideWithValue(RecordingAudioService()),
        ],
        child: localizedApp(
          Center(
            child: SizedBox(
              width: 320,
              height: 200,
              child: InCardPreview(
                catalogId: 'c0',
                onProgress: (f) => progress = f,
              ),
            ),
          ),
        ),
      ),
    );
    for (var i = 0; i < 4; i++) {
      await tester.pump(const Duration(milliseconds: 16));
    }
    // The error is shown (EN test locale), and the gate is released so the user
    // isn't trapped behind an un-listenable preview.
    expect(find.text("Couldn't load the preview."), findsOneWidget);
    expect(progress, 1.0);
  });
}
