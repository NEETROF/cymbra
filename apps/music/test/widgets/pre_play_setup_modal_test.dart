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
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

/// Pumps the player for [_entry] and leaves the pre-play setup modal open.
Future<ProviderContainer> _pumpWithModal(
  WidgetTester tester, {
  ScoreDocument? document,
}) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(
        FakeNotationEngine(document: document),
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
      child: localizedApp(const PlayerScreen()),
    ),
  );
  for (var i = 0; i < 12; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
  return container;
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

Future<void> _pumpFrames(WidgetTester tester) async {
  for (var i = 0; i < 6; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  testWidgets('opens on a loaded score, shows info, applies hand on Validate', (
    tester,
  ) async {
    final container = await _pumpWithModal(tester); // grand-staff (multi-staff)

    // Score info is shown.
    expect(find.text('Sample Piece'), findsOneWidget);
    expect(find.text('Tester'), findsOneWidget);
    // Multi-staff → the hand chooser is offered.
    expect(find.text('Play with'), findsOneWidget);
    expect(find.text('Left'), findsOneWidget);

    // Change the hand, then Validate.
    await tester.tap(find.text('Left'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, 'Play'));
    await _pumpFrames(tester);

    // Applied and closed.
    expect(container.read(playerProvider).selectedHands, Hand.left);
    expect(find.text('Play with'), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('close (X) keeps the current settings', (tester) async {
    final container = await _pumpWithModal(tester);
    expect(container.read(playerProvider).selectedHands, Hand.both);

    // Change the hand in the draft, then dismiss with the close button.
    await tester.tap(find.text('Left'));
    await tester.pump();
    await tester.tap(find.byIcon(Icons.close));
    await _pumpFrames(tester);

    // Not applied — still Both — and the modal is closed.
    expect(container.read(playerProvider).selectedHands, Hand.both);
    expect(find.text('Play with'), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('single-staff piece offers no hand chooser', (tester) async {
    final container = await _pumpWithModal(
      tester,
      document: sampleTieSlurDocument(), // staves: 1
    );

    // The modal is shown (Validate button present) but with no hand chooser.
    expect(find.widgetWithText(FilledButton, 'Play'), findsOneWidget);
    expect(find.text('Play with'), findsNothing);
    expect(find.text('Left'), findsNothing);
    await _teardown(tester, container);
  });
}
