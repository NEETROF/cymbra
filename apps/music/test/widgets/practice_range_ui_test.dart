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
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

/// Pumps the player for a **four-measure** score (so a narrower range is really
/// narrower than the whole piece) and leaves the setup modal open.
Future<ProviderContainer> _pump(
  WidgetTester tester, {
  ScoreDocument? document,
  FakePreferencesService? prefs,
}) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(
        FakeNotationEngine(document: document ?? sampleFourMeasureDocument()),
      ),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
    ],
  );
  container.read(selectedScoreProvider.notifier).select(_entry);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const PlayerScreen(), locale: const Locale('en')),
    ),
  );
  for (var i = 0; i < 12; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
  return container;
}

Future<void> _pumpFrames(WidgetTester tester) async {
  for (var i = 0; i < 6; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

void main() {
  testWidgets('the setup modal carries no practice section', (tester) async {
    // The deliberate range-selection surface moved in-game (change:
    // add-in-game-measure-selection): the modal offers no run-type toggle, no
    // steppers, no passage step.
    final container = await _pump(tester);

    expect(find.byKey(const Key('practice-run-type')), findsNothing);
    expect(find.text('Section'), findsNothing);
    expect(find.byKey(const Key('practice-from')), findsNothing);
    expect(find.byKey(const Key('practice-score-strip')), findsNothing);

    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _pumpFrames(tester);

    final data = container.read(playerProvider);
    expect(data.hasPracticeRange, isFalse);
    expect(data.isSelectiveRun, isFalse);
    await _teardown(tester, container);
  });

  testWidgets('an armed range survives the setup modal', (tester) async {
    // The modal is range-neutral: opening and dismissing it — Apply or X —
    // never sets nor clears the active range.
    final container = await _pump(tester);
    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _pumpFrames(tester);

    container.read(playerProvider.notifier).setPracticeRange(1, 2);
    await _pumpFrames(tester);

    // Reopen in-game (the tune button), then Apply.
    await tester.tap(find.byIcon(Icons.tune));
    await _pumpFrames(tester);
    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _pumpFrames(tester);
    var data = container.read(playerProvider);
    expect(data.practiceStartMeasure, 1);
    expect(data.practiceEndMeasure, 2);
    expect(data.isSelectiveRun, isTrue);

    // Reopen again, dismiss with the X this time.
    await tester.tap(find.byIcon(Icons.tune));
    await _pumpFrames(tester);
    await tester.tap(find.byIcon(Icons.close));
    await _pumpFrames(tester);
    data = container.read(playerProvider);
    expect(data.practiceStartMeasure, 1);
    expect(data.practiceEndMeasure, 2);
    await _teardown(tester, container);
  });

  testWidgets('the stop button IS the passage indicator', (tester) async {
    // One indicator, not two: the exit from the loop also says which bars are
    // looping, so the transport carries no second chip.
    final container = await _pump(tester);
    await tester.tap(find.byIcon(Icons.close));
    await _pumpFrames(tester);
    expect(find.byKey(const Key('practice-stop-loop')), findsNothing);

    container.read(playerProvider.notifier).setPracticeRange(1, 3);
    await _pumpFrames(tester);
    final stop = find.byKey(const Key('practice-stop-loop'));
    expect(stop, findsOneWidget);
    // It TAKES THE TITLE'S PLACE rather than stacking above it, so the top bar
    // does not grow on a short phone-landscape viewport.
    expect(find.text('Cymbra Music'), findsNothing);
    expect(
      find.descendant(of: stop, matching: find.text('Bars 2–4')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: stop, matching: find.text('Stop the loop')),
      findsOneWidget,
    );

    // Back to a full run: the whole indicator disappears.
    container.read(playerProvider.notifier).clearPracticeRange();
    await _pumpFrames(tester);
    expect(stop, findsNothing);
    expect(find.text('Cymbra Music'), findsOneWidget); // the title is back
    await _teardown(tester, container);
  });

  testWidgets('tapping two bars on the score sets the range', (tester) async {
    final container = await _pump(tester);
    // Close the setup modal and switch to the engraved Partition view.
    await tester.tap(find.byIcon(Icons.close));
    await _pumpFrames(tester);
    container.read(playerProvider.notifier).setMode(RenderMode.partition);
    await _pumpFrames(tester);

    final canvas = find.byKey(const Key('partition-canvas'));
    expect(canvas, findsOneWidget);
    // The tap hint is offered while paused with no range chosen.
    expect(find.byKey(const Key('partition-tap-hint')), findsOneWidget);

    // The fixture lays out one measure per system, so bar N sits on line N.
    final box = tester.getRect(canvas);
    Offset barCentre(int index) {
      final painter = PartitionPainter(
        document: sampleFourMeasureDocument(),
        systems: const [],
      );
      final y = painter.systemTopY(index) + painter.systemStride / 2;
      // Right of the header (clef/key/time), inside the measure.
      return box.topLeft + Offset(box.width * 0.7, y);
    }

    await tester.tapAt(barCentre(1));
    await _pumpFrames(tester);
    // First tap only arms the selection — no range yet.
    expect(container.read(playerProvider).hasPracticeRange, isFalse);

    await tester.tapAt(barCentre(2));
    await _pumpFrames(tester);
    final data = container.read(playerProvider);
    expect(data.practiceStartMeasure, 1);
    expect(data.practiceEndMeasure, 2);
    expect(data.isSelectiveRun, isTrue);
    await _teardown(tester, container);
  });
}
