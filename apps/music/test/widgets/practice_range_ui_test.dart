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
/// The audio fake of the most recent [_pump], so a test can assert what the
/// picker actually sounded.
late RecordingAudioService lastAudio;

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
      audioServiceProvider.overrideWithValue(
        lastAudio = RecordingAudioService(),
      ),
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

/// Scrolls [finder] into the modal's viewport, then taps it — the setup sheet
/// scrolls as one unit, so a control can start below the fold.
Future<void> _tap(WidgetTester tester, Finder finder) async {
  await tester.ensureVisible(finder);
  await tester.pump();
  await tester.tap(finder);
  await tester.pump();
}

/// Taps the +/− button of one measure stepper.
Future<void> _step(
  WidgetTester tester,
  String key, {
  required bool up,
  int times = 1,
}) async {
  for (var i = 0; i < times; i++) {
    await _tap(
      tester,
      find.descendant(
        of: find.byKey(Key(key)),
        matching: find.byIcon(up ? Icons.add : Icons.remove),
      ),
    );
  }
}

/// The bar number currently shown by one stepper (1-based, as displayed).
String _barShown(WidgetTester tester, String key) => tester
    .widgetList<Text>(
      find.descendant(of: find.byKey(Key(key)), matching: find.byType(Text)),
    )
    // [label, value]
    .last
    .data!;

/// Chooses "Section" and advances to the passage step (the controls moved there).
Future<void> _openPassageStep(WidgetTester tester) async {
  await tester.tap(find.text('Section'));
  await tester.pump();
  await tester.tap(find.byKey(const Key('pre-play-primary')));
  for (var i = 0; i < 6; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  testWidgets('setup modal defaults to a full run and applies no range', (
    tester,
  ) async {
    final container = await _pump(tester);

    expect(find.text('Practice'), findsOneWidget);
    expect(find.text('Whole piece'), findsOneWidget);
    expect(find.text('Section'), findsOneWidget);
    // Full run is the default, so the range steppers are not offered.
    expect(find.byKey(const Key('practice-from')), findsNothing);

    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _pumpFrames(tester);

    final data = container.read(playerProvider);
    expect(data.hasPracticeRange, isFalse);
    expect(data.isSelectiveRun, isFalse);
    await _teardown(tester, container);
  });

  testWidgets('choosing Section reveals the steppers and applies the range', (
    tester,
  ) async {
    final container = await _pump(tester);

    await _openPassageStep(tester);
    expect(find.byKey(const Key('practice-from')), findsOneWidget);
    expect(find.byKey(const Key('practice-to')), findsOneWidget);
    // A "passage" is seeded NARROW (bars 1 → 2), never the whole piece: a
    // whole-piece passage would silently start a SCORED run.
    expect(_barShown(tester, 'practice-from'), '1');
    expect(_barShown(tester, 'practice-to'), '2');

    // Bars 2 → 3.
    await _step(tester, 'practice-from', up: true);
    await _step(tester, 'practice-to', up: true);

    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _pumpFrames(tester);

    final data = container.read(playerProvider);
    expect(data.practiceStartMeasure, 1);
    expect(data.practiceEndMeasure, 2);
    expect(data.isSelectiveRun, isTrue);
    // The playhead moved to the range's first measure (4000 ms at ♩=60).
    expect(data.elapsedMs, 4000);
    await _teardown(tester, container);
  });

  testWidgets('raising "from" past "to" pushes "to" along', (tester) async {
    final container = await _pump(tester);

    await _openPassageStep(tester);
    // to: 4 → 2, then from: 1 → 3 (which drags "to" to 3).
    await _step(tester, 'practice-to', up: false, times: 2);
    await _step(tester, 'practice-from', up: true, times: 2);

    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _pumpFrames(tester);

    final data = container.read(playerProvider);
    expect(data.practiceStartMeasure, 2);
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

  testWidgets('saved settings pre-fill the picker on reopen', (tester) async {
    final prefs = FakePreferencesService();
    final container = await _pump(tester, prefs: prefs);

    // Choose bars 2–3 with 4 repeats, then apply.
    await _openPassageStep(tester);
    await _step(tester, 'practice-from', up: true);
    await _step(tester, 'practice-to', up: true);
    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _pumpFrames(tester);
    await _teardown(tester, container);

    // Reopen the score on the SAME storage: a full run by default, but the
    // picker is pre-filled with what was last drilled.
    final reopened = await _pump(tester, prefs: prefs);
    expect(reopened.read(playerProvider).isSelectiveRun, isFalse);
    await _openPassageStep(tester);
    // Bars 2 → 3 (1-based) — the range is what a reopen restores.
    expect(_barShown(tester, 'practice-from'), '2');
    expect(_barShown(tester, 'practice-to'), '3');
    await _teardown(tester, reopened);
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

  testWidgets('a single-measure score offers no practice section', (
    tester,
  ) async {
    // The grand-staff fixture has one measure — a range would be meaningless.
    final container = await _pump(tester, document: sampleGrandStaffDocument());
    expect(find.text('Practice'), findsNothing);
    expect(find.text('Whole piece'), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('the setup sheet picks the range ON the score ribbon', (
    tester,
  ) async {
    // The point of the change: a musician recognises the passage by how it
    // looks, so the modal shows the music and the range is chosen by tapping
    // the first and last bar — not by reading blind bar numbers. The ribbon is
    // horizontal (the Portée), so the whole piece fits without scrolling and a
    // bar is identified by its X span alone.
    final container = await _pump(tester);
    await _openPassageStep(tester);

    final strip = find.byKey(const Key('practice-score-strip'));
    expect(
      strip,
      findsOneWidget,
      reason: 'the music itself must be the picker',
    );
    expect(
      find.text('Tap the first bar of the passage on the score'),
      findsOneWidget,
    );

    // Move the draft away from its seeded bars 1–2, so the taps below are
    // proven to be what sets the range.
    await _step(tester, 'practice-from', up: true, times: 2);
    expect(_barShown(tester, 'practice-from'), '3');

    await tester.ensureVisible(strip);
    await _pumpFrames(tester);
    // The ribbon maps time to X, so bars run left to right. Rather than
    // recomputing the painter's internal geometry, tap well inside the ribbon at
    // two clearly separated points and assert the ORDERED range they produce —
    // what the picker promises, without coupling the test to the layout maths.
    final box = tester.getRect(strip);
    Offset at(double fraction) =>
        Offset(box.left + box.width * fraction, box.center.dy);

    await tester.tapAt(at(0.30));
    await _pumpFrames(tester);
    // First tap arms the selection and the prompt moves on to the end bar.
    expect(find.text('Now tap the last bar'), findsOneWidget);

    await tester.tapAt(at(0.80));
    await _pumpFrames(tester);
    final from = int.parse(_barShown(tester, 'practice-from'));
    final to = int.parse(_barShown(tester, 'practice-to'));
    expect(from, lessThan(to), reason: 'the pick must come out ordered');
    expect(from, isNot(3), reason: 'the taps, not the steppers, set the range');

    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _pumpFrames(tester);
    final data = container.read(playerProvider);
    expect(data.practiceStartMeasure, from - 1);
    expect(data.practiceEndMeasure, to - 1);
    expect(data.isSelectiveRun, isTrue);
    await _teardown(tester, container);
  });

  testWidgets('"Section" turns the action into Next and opens a second step', (
    tester,
  ) async {
    final container = await _pump(tester);

    // Step 1: the general settings. The action starts the run outright.
    expect(find.widgetWithText(FilledButton, 'Play'), findsOneWidget);
    expect(find.byKey(const Key('practice-score-strip')), findsNothing);
    expect(find.byKey(const Key('practice-from')), findsNothing);

    // Choosing "Section" only re-labels the action — nothing starts yet.
    await tester.tap(find.text('Section'));
    await tester.pump();
    expect(find.widgetWithText(FilledButton, 'Next'), findsOneWidget);
    expect(container.read(playerProvider).hasPracticeRange, isFalse);

    // Step 2: the passage settings own the modal, score picker included.
    await tester.tap(find.byKey(const Key('pre-play-primary')));
    await _pumpFrames(tester);
    expect(find.byKey(const Key('practice-score-strip')), findsOneWidget);
    expect(find.byKey(const Key('practice-from')), findsOneWidget);
    // The general settings are out of the way on this step.
    expect(find.text('Metronome'), findsNothing);
    expect(find.widgetWithText(FilledButton, 'Play'), findsOneWidget);

    // Back returns to the general settings with the choice intact.
    await tester.tap(find.byKey(const Key('practice-step-back')));
    await _pumpFrames(tester);
    expect(find.text('Metronome'), findsOneWidget);
    expect(find.widgetWithText(FilledButton, 'Next'), findsOneWidget);

    // Reverting to the whole piece drops the step entirely.
    await tester.tap(find.text('Whole piece'));
    await _pumpFrames(tester);
    expect(find.widgetWithText(FilledButton, 'Play'), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('the picker auditions the chosen passage, and follows it', (
    tester,
  ) async {
    // Choosing bars is something you HEAR: opening the picker plays the passage,
    // and moving a bound re-auditions the NEW one instead of the old.
    final container = await _pump(tester);
    await _openPassageStep(tester);

    // Opening the step already sounds the seeded passage (bars 1-2).
    await tester.pump(const Duration(milliseconds: 600));
    expect(
      lastAudio.noteOns,
      isNotEmpty,
      reason: 'opening the picker must audition the passage',
    );

    // The fixture is C5 / D5 / E5 / F5, one per bar — so what sounds says
    // exactly which bars were auditioned.
    final openingPitches = lastAudio.noteOns.map((n) => n.pitch).toSet();
    expect(openingPitches, contains(72), reason: 'bar 1 is C5');

    lastAudio.noteOns.clear();
    // Move the passage to bars 3-4 and let it play.
    await _step(tester, 'practice-from', up: true, times: 2);
    await _step(tester, 'practice-to', up: true, times: 2);
    await tester.pump(const Duration(milliseconds: 900));

    final movedPitches = lastAudio.noteOns.map((n) => n.pitch).toSet();
    expect(
      movedPitches,
      isNot(contains(72)),
      reason: 'the audition must follow the new range, not replay bar 1',
    );
    expect(movedPitches, isNotEmpty);
    await _teardown(tester, container);
  });

  testWidgets('the ribbon auto-scrolls to the bound being moved', (
    tester,
  ) async {
    // With enough bars the ribbon is wider than its viewport, so a bar moved far
    // to the right is off-screen until the picker follows it. Four bars fit
    // without scrolling, which is why this needs its own fixture.
    final container = await _pump(
      tester,
      document: sampleManyMeasureDocument(),
    );
    await _openPassageStep(tester);

    // The horizontal ribbon is the only horizontally-scrolling view here.
    ScrollableState ribbon() => tester
        .stateList<ScrollableState>(find.byType(Scrollable))
        .firstWhere((s) => s.position.axisDirection == AxisDirection.right);

    expect(
      ribbon().position.maxScrollExtent,
      greaterThan(0),
      reason: 'the fixture must actually be scrollable',
    );
    final before = ribbon().position.pixels;

    // Push "to" far to the right; the ribbon must follow it.
    await _step(tester, 'practice-to', up: true, times: 12);
    // Not pumpAndSettle: the audition timer keeps ticking, so nothing ever
    // settles. Pump long enough for the scroll animation instead.
    for (var i = 0; i < 20; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }

    expect(
      ribbon().position.pixels,
      greaterThan(before),
      reason: 'moving a bound must bring it into view',
    );
    await _teardown(tester, container);
  });

  testWidgets(
    'reopening the picker shows the saved passage, already scrolled',
    (tester) async {
      // The picker opens ALREADY holding the saved range, so no bound ever
      // "changes" — without an explicit scroll on open the ribbon sat at bar 1
      // while the passage was off-screen.
      final prefs = FakePreferencesService();
      final doc = sampleManyMeasureDocument();
      var container = await _pump(tester, document: doc, prefs: prefs);
      await _openPassageStep(tester);
      await _step(tester, 'practice-from', up: true, times: 14);
      await _step(tester, 'practice-to', up: true, times: 14);
      await tester.tap(find.byKey(const Key('pre-play-primary')));
      await _pumpFrames(tester);
      expect(container.read(playerProvider).practiceStartMeasure, 14);
      await _teardown(tester, container);

      // Reopen the same score on the same storage.
      container = await _pump(tester, document: doc, prefs: prefs);
      await _openPassageStep(tester);
      for (var i = 0; i < 20; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }

      final ribbon = tester
          .stateList<ScrollableState>(find.byType(Scrollable))
          .firstWhere((s) => s.position.axisDirection == AxisDirection.right);
      expect(
        ribbon.position.pixels,
        greaterThan(0),
        reason: 'the saved passage must be in view on open, not bar 1',
      );
      await _teardown(tester, container);
    },
  );
}
