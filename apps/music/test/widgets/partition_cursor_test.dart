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
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/notation_fakes.dart';

const _entry = CatalogEntry(
  id: 'tall',
  title: 'Tall Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/tall.musicxml',
  level: PracticeLevel.beginner,
);

// The main (scrolling) engraving canvas — keyed so it stays unambiguous when the
// "next line" overlay (also a PartitionPainter) is on screen.
Finder _partitionPaint() => find.byKey(const Key('partition-canvas'));

void main() {
  testWidgets('Partition auto-scrolls to follow the playhead', (tester) async {
    await tester.binding.setSurfaceSize(const Size(1400, 900));
    final container = ProviderContainer(
      overrides: [
        scoreCatalogProvider.overrideWithValue(const [_entry]),
        scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
        notationEngineProvider.overrideWithValue(
          FakeNotationEngine(document: tallDocument(24)),
        ),
        midiServiceProvider.overrideWithValue(FakeMidiService()),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
      ],
    );
    container.read(selectedScoreProvider.notifier).select(_entry);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: PlayerScreen()),
      ),
    );
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }

    final notifier = container.read(playerProvider.notifier);
    notifier.setMode(RenderMode.partition);
    notifier.toggleWaitMode(); // off, so the playhead advances freely
    await tester.pump();

    // The engraved content starts at the top of the render area.
    final beforeY = tester.getTopLeft(_partitionPaint()).dy;

    notifier.togglePlay(); // play
    // 50ms steps: the ticker guard ignores dt >= 100ms, so advance with small steps.
    for (var i = 0; i < 160; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }

    expect(container.read(playerProvider).elapsedMs, greaterThan(0));
    // Scrolling moves the (tall) content upward, so its top is now higher.
    final afterY = tester.getTopLeft(_partitionPaint()).dy;
    expect(afterY, lessThan(beforeY));

    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    container.dispose();
  });

  testWidgets('hides the next-line overlay while the next line is on screen', (
    tester,
  ) async {
    await tester.binding.setSurfaceSize(const Size(1400, 900));
    final container = ProviderContainer(
      overrides: [
        scoreCatalogProvider.overrideWithValue(const [_entry]),
        scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
        notationEngineProvider.overrideWithValue(
          FakeNotationEngine(document: tallDocument(24)),
        ),
        midiServiceProvider.overrideWithValue(FakeMidiService()),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
      ],
    );
    container.read(selectedScoreProvider.notifier).select(_entry);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const MaterialApp(home: PlayerScreen()),
      ),
    );
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }

    final notifier = container.read(playerProvider.notifier);
    notifier.setMode(RenderMode.partition);
    notifier.toggleWaitMode(); // off
    await tester.pump();

    // At the very start (first half of the first measure) there is no overlay.
    expect(find.text('NEXT'), findsNothing);

    notifier.togglePlay();
    for (var i = 0; i < 40; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    // This viewport is tall enough to show the next line below the current one,
    // so the overlay must stay hidden (it would otherwise cover the score).
    expect(find.text('NEXT'), findsNothing);

    await tester.pumpWidget(const SizedBox());
    await tester.pump();
    container.dispose();
  });
}
