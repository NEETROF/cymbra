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
import 'package:music/screens/library_screen.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/notation_fakes.dart';

const _entries = [
  CatalogEntry(
    id: 'b1',
    title: 'Beginner Piece',
    composer: 'Composer A',
    assetPath: 'assets/scores/beginner/b1.musicxml',
    level: PracticeLevel.beginner,
  ),
  CatalogEntry(
    id: 'i1',
    title: 'Intermediate Piece',
    composer: 'Composer B',
    assetPath: 'assets/scores/intermediate/i1.musicxml',
    level: PracticeLevel.intermediate,
  ),
  CatalogEntry(
    id: 'a1',
    title: 'Advanced Piece',
    composer: 'Composer C',
    assetPath: 'assets/scores/advanced/a1.musicxml',
    level: PracticeLevel.advanced,
  ),
];

ProviderContainer _container() => ProviderContainer(
  overrides: [
    scoreCatalogProvider.overrideWithValue(_entries),
    scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
    notationEngineProvider.overrideWithValue(FakeNotationEngine()),
    midiServiceProvider.overrideWithValue(FakeMidiService()),
    scoreSourceProvider.overrideWithValue(FakeScoreSource()),
    audioServiceProvider.overrideWithValue(RecordingAudioService()),
  ],
);

/// Unmounts and disposes the container so the player's auto-dispose poll timer
/// is cancelled before the test ends.
Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

Future<void> _pump(WidgetTester tester, ProviderContainer container) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: const MaterialApp(home: LibraryScreen()),
    ),
  );
}

/// Pumps a bounded number of frames. The player runs a Ticker/Timer, so
/// `pumpAndSettle` would never settle — pump a fixed number of frames instead.
Future<void> _pumpFrames(WidgetTester tester, [int n = 12]) async {
  for (var i = 0; i < n; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  testWidgets('lists entries grouped by practice level', (tester) async {
    final container = _container();
    await _pump(tester, container);

    expect(find.text('Beginner'), findsOneWidget);
    expect(find.text('Intermediate'), findsOneWidget);
    expect(find.text('Advanced'), findsOneWidget);
    expect(find.text('Beginner Piece'), findsOneWidget);
    expect(find.text('Composer A'), findsOneWidget);
    expect(find.text('Advanced Piece'), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('tapping an entry selects it and opens the player screen', (
    tester,
  ) async {
    final container = _container();
    await _pump(tester, container);

    await tester.tap(find.text('Intermediate Piece'));
    await _pumpFrames(tester);

    expect(container.read(selectedScoreProvider)?.id, 'i1');
    expect(find.byType(PlayerScreen), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('back from the player screen returns to the library', (
    tester,
  ) async {
    final container = _container();
    await _pump(tester, container);

    await tester.tap(find.text('Beginner Piece'));
    await _pumpFrames(tester);
    expect(find.byType(PlayerScreen), findsOneWidget);

    // Tap the player's back button (wired because there is a route to pop).
    await tester.tap(find.byTooltip('Back to library'));
    await _pumpFrames(tester);

    expect(find.byType(LibraryScreen), findsOneWidget);
    expect(find.byType(PlayerScreen), findsNothing);
    await _teardown(tester, container);
  });
}
