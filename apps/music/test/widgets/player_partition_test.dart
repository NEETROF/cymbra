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
import 'package:music/painters/partition_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/notation_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

Future<ProviderContainer> _pumpPlayer(
  WidgetTester tester, {
  bool select = true,
}) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
    ],
  );
  if (select) container.read(selectedScoreProvider.notifier).select(_entry);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: const MaterialApp(home: PlayerScreen()),
    ),
  );
  for (var i = 0; i < 12; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
  return container;
}

/// Unmounts the player and disposes the container so the notifier's MIDI poll
/// timer is cancelled (otherwise the test ends with a pending timer).
Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

void main() {
  testWidgets('loads the selected score and derives its playback', (
    tester,
  ) async {
    final container = await _pumpPlayer(tester);
    final data = container.read(playerProvider);

    expect(data.title, 'Sample'); // sample document title
    expect(data.notes, isNotEmpty); // derived from the MusicXML, not the demo
    expect(data.score, isNull); // demo score not used
    expect(find.text('Now Playing: Sample'), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('Partition mode renders the engraved grand staff', (
    tester,
  ) async {
    final container = await _pumpPlayer(tester);

    container.read(playerProvider.notifier).setMode(RenderMode.partition);
    await tester.pump();

    final painter = tester
        .widgetList<CustomPaint>(find.byType(CustomPaint))
        .map((w) => w.painter)
        .whereType<PartitionPainter>()
        .first;
    expect(painter.document.staves, 2);
    expect(painter.systems, isNotEmpty);

    // The keyboard is still present in Partition mode.
    expect(find.byType(SegmentedButton<RenderMode>), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('all three render modes are offered', (tester) async {
    final container = await _pumpPlayer(tester);
    expect(find.text('Synthesia'), findsOneWidget);
    expect(find.text('Staff'), findsOneWidget);
    expect(find.text('Partition'), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('with no selection the demo score still loads', (tester) async {
    final container = await _pumpPlayer(tester, select: false);
    final data = container.read(playerProvider);
    expect(data.score, isNotNull); // fell back to the demo
    expect(data.notes, isNotEmpty);
    await _teardown(tester, container);
  });
}
