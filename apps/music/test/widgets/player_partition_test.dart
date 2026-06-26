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
import 'package:music/painters/staff_painter.dart';
import 'package:music/painters/synthesia_painter.dart';
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
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
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

  group('hand selector (settings menu)', () {
    T painterOf<T>(WidgetTester tester) => tester
        .widgetList<CustomPaint>(find.byType(CustomPaint))
        .map((w) => w.painter)
        .whereType<T>()
        .first;

    /// Opens the gear end drawer (the screen's Ticker never settles, so pump
    /// explicitly past the open animation).
    Future<void> openSettings(WidgetTester tester) async {
      await tester.tap(find.byIcon(Icons.tune));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
    }

    /// Opens the drawer and drills into the "Hand" category.
    Future<void> openHandCategory(WidgetTester tester) async {
      await openSettings(tester);
      await tester.tap(find.text('Hand'));
      await tester.pump();
    }

    /// Dismisses the drawer by tapping the scrim left of the right-side panel.
    Future<void> closeSettings(WidgetTester tester) async {
      await tester.tapAt(const Offset(20, 400));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));
    }

    testWidgets('shows the current selection and dispatches the setter', (
      tester,
    ) async {
      final container = await _pumpPlayer(tester);
      // The two-staff sample makes the Hand category meaningful → it is offered.
      await openHandCategory(tester);
      expect(find.text('Both'), findsOneWidget); // current selection is listed

      await tester.tap(find.text('Left'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 600));

      expect(container.read(playerProvider).selectedHands, Hand.left);
      await _teardown(tester, container);
    });

    testWidgets('is reachable in every render mode', (tester) async {
      final container = await _pumpPlayer(tester);
      final notifier = container.read(playerProvider.notifier);
      for (final mode in RenderMode.values) {
        notifier.setMode(mode);
        await tester.pump();
        await openHandCategory(tester);
        expect(
          find.text('Left'),
          findsOneWidget,
          reason: 'hand option missing in $mode',
        );
        await closeSettings(tester);
      }
      await _teardown(tester, container);
    });

    testWidgets('is hidden for a single-staff piece (the demo)', (
      tester,
    ) async {
      // No score selected → the demo (all staff 1) loads, so there is no hand
      // to isolate and the Hand category is not offered.
      final container = await _pumpPlayer(tester, select: false);
      expect(container.read(playerProvider).hasMultipleStaves, isFalse);
      await openSettings(tester);
      expect(find.text('Hand'), findsNothing);
      // The other categories are still offered.
      expect(find.text('Keyboard size'), findsOneWidget);
      await _teardown(tester, container);
    });

    testWidgets('Staff and Synthesia exclude the unselected hand', (
      tester,
    ) async {
      final container = await _pumpPlayer(tester);
      final notifier = container.read(playerProvider.notifier);

      // Both: the painter sees the left-hand (staff 2) note.
      notifier.setMode(RenderMode.staff);
      await tester.pump();
      expect(
        painterOf<StaffPainter>(tester).notes.any((n) => n.staff >= 2),
        isTrue,
      );

      // Right: only staff-1 notes reach the painters, in every time-based mode.
      notifier.setSelectedHands(Hand.right);
      await tester.pump();
      final staff = painterOf<StaffPainter>(tester);
      expect(staff.notes, isNotEmpty);
      expect(staff.notes.every((n) => n.staff == 1), isTrue);

      notifier.setMode(RenderMode.synthesia);
      await tester.pump();
      final synth = painterOf<SynthesiaPainter>(tester);
      expect(synth.notes, isNotEmpty);
      expect(synth.notes.every((n) => n.staff == 1), isTrue);

      // Partition collapses to a single staff via the selection it is handed.
      notifier.setMode(RenderMode.partition);
      await tester.pump();
      expect(painterOf<PartitionPainter>(tester).selectedHands, Hand.right);
      await _teardown(tester, container);
    });
  });
}
