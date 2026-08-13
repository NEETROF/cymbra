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
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/practice_settings_store.dart';
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

/// Pumps the player over the four-measure fixture and closes the auto-opened
/// setup modal, leaving the transport bar reachable.
Future<ProviderContainer> _pump(
  WidgetTester tester, {
  FakePreferencesService? prefs,
}) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(
        FakeNotationEngine(document: sampleFourMeasureDocument()),
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
  await _frames(tester, 12);
  await tester.tap(find.byIcon(Icons.close));
  await _frames(tester);
  return container;
}

/// A player over the built-in DEMO score (no notation document, no measure
/// table), where measure navigation is meaningless.
Future<ProviderContainer> _pumpDemo(WidgetTester tester) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final container = ProviderContainer(
    overrides: [
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource(null)),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      notationProvider.overrideWith(() => _FixedNotation(const NotationData())),
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
    ],
  );
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const PlayerScreen(), locale: const Locale('en')),
    ),
  );
  await _frames(tester, 12);
  return container;
}

class _FixedNotation extends Notation {
  _FixedNotation(this._value);
  final NotationData _value;
  @override
  NotationData build() => _value;
}

Future<void> _frames(WidgetTester tester, [int count = 6]) async {
  for (var i = 0; i < count; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

/// Places the paused playhead at [elapsedMs] without letting the screen's
/// ticker drift it: advance in a free run, then pause.
void _seekPaused(ProviderContainer container, double elapsedMs) {
  final notifier = container.read(playerProvider.notifier);
  if (container.read(playerProvider).waitMode) notifier.toggleWaitMode();
  notifier.setPlaying(true);
  notifier.advance(elapsedMs);
  notifier.setPlaying(false);
}

/// Opens the measure-selection screen via the transport button's long-press.
Future<void> _openSelect(WidgetTester tester) async {
  await tester.longPress(find.byKey(const Key('transport-rewind')));
  await _frames(tester);
}

/// The centre of bar [index] on the selection canvas — the fixture lays out one
/// measure per system, so bar N sits on line N.
Offset _barCentre(WidgetTester tester, int index) {
  final box = tester.getRect(find.byKey(const Key('measure-select-canvas')));
  final painter = PartitionPainter(
    document: sampleFourMeasureDocument(),
    systems: const [],
  );
  final y = painter.systemTopY(index) + painter.systemStride / 2;
  return box.topLeft + Offset(box.width * 0.7, y);
}

/// The selection screen's title-bar label.
String _title(WidgetTester tester) =>
    tester.widget<Text>(find.byKey(const Key('measure-select-title'))).data!;

void main() {
  group('transport rewind button', () {
    testWidgets('a tap rewinds one bar; restart is untouched', (tester) async {
      final container = await _pump(tester);
      _seekPaused(container, 6000); // mid bar 2
      expect(container.read(playerProvider).elapsedMs, 6000);

      await tester.tap(find.byKey(const Key('transport-rewind')));
      await tester.pump();
      expect(container.read(playerProvider).elapsedMs, 4000);
      await tester.tap(find.byKey(const Key('transport-rewind')));
      await tester.pump();
      expect(container.read(playerProvider).elapsedMs, 0);

      // The restart-from-top button is still there, unchanged.
      expect(find.byIcon(Icons.skip_previous), findsOneWidget);
      await _teardown(tester, container);
    });

    testWidgets('disabled on the demo score (no measure table)', (
      tester,
    ) async {
      final container = await _pumpDemo(tester);
      final button = tester.widget<IconButton>(
        find.byKey(const Key('transport-rewind')),
      );
      expect(button.onPressed, isNull);

      // The long-press entry is equally inert: no selection screen opens.
      await _openSelect(tester);
      expect(find.byKey(const Key('measure-select-title')), findsNothing);
      await _teardown(tester, container);
    });
  });

  group('measure-selection screen', () {
    testWidgets('long-press pauses playback and opens the screen', (
      tester,
    ) async {
      final container = await _pump(tester);
      container.read(playerProvider.notifier).setPlaying(true);
      await tester.pump();

      await _openSelect(tester);
      expect(find.byKey(const Key('measure-select-title')), findsOneWidget);
      expect(find.byKey(const Key('measure-select-canvas')), findsOneWidget);
      expect(container.read(playerProvider).isPlaying, isFalse);
      // No keyboard on this screen — the engraving owns the viewport.
      expect(find.byKey(const Key('piano-keyboard')), findsNothing);
      await _teardown(tester, container);
    });

    testWidgets('two taps draft a normalized range; confirm applies it', (
      tester,
    ) async {
      final container = await _pump(tester);
      await _openSelect(tester);
      expect(_title(tester), 'Pick a passage');

      // Reversed order: bar 3 then bar 2 → normalized to 2–3.
      await tester.tapAt(_barCentre(tester, 2));
      await _frames(tester);
      expect(_title(tester), 'Bar 3 → …');
      // The live session is untouched while drafting.
      expect(container.read(playerProvider).hasPracticeRange, isFalse);

      await tester.tapAt(_barCentre(tester, 1));
      await _frames(tester);
      expect(_title(tester), 'Bars 2–3');
      expect(container.read(playerProvider).hasPracticeRange, isFalse);

      await tester.tap(find.byKey(const Key('measure-select-confirm')));
      await _frames(tester, 12); // let the pop transition finish
      // Back on the player, with the range applied as a selective run.
      expect(find.byKey(const Key('measure-select-title')), findsNothing);
      final data = container.read(playerProvider);
      expect(data.practiceStartMeasure, 1);
      expect(data.practiceEndMeasure, 2);
      expect(data.isSelectiveRun, isTrue);
      await _teardown(tester, container);
    });

    testWidgets('re-tapping starts a fresh draft', (tester) async {
      final container = await _pump(tester);
      await _openSelect(tester);

      await tester.tapAt(_barCentre(tester, 0));
      await _frames(tester);
      await tester.tapAt(_barCentre(tester, 1));
      await _frames(tester);
      expect(_title(tester), 'Bars 1–2');

      // A third tap begins a NEW draft from the tapped bar.
      await tester.tapAt(_barCentre(tester, 3));
      await _frames(tester);
      expect(_title(tester), 'Bar 4 → …');
      await _teardown(tester, container);
    });

    testWidgets('cancel leaves the session exactly as on entry', (
      tester,
    ) async {
      final container = await _pump(tester);
      _seekPaused(container, 6000);
      await _openSelect(tester);

      await tester.tapAt(_barCentre(tester, 0));
      await _frames(tester);
      await tester.tapAt(_barCentre(tester, 3));
      await _frames(tester);

      await tester.pageBack();
      await _frames(tester);
      final data = container.read(playerProvider);
      expect(data.hasPracticeRange, isFalse);
      expect(data.elapsedMs, 6000);
      expect(data.isPlaying, isFalse);
      await _teardown(tester, container);
    });

    testWidgets('pre-fills from the active range; whole-piece clears it', (
      tester,
    ) async {
      final container = await _pump(tester);
      container.read(playerProvider.notifier).setPracticeRange(1, 2);
      await _frames(tester);

      await _openSelect(tester);
      expect(_title(tester), 'Bars 2–3');

      await tester.tap(find.byKey(const Key('measure-select-whole')));
      await _frames(tester);
      final data = container.read(playerProvider);
      expect(data.hasPracticeRange, isFalse);
      expect(data.isSelectiveRun, isFalse);
      await _teardown(tester, container);
    });

    testWidgets('pre-fills from the saved practice settings on a full run', (
      tester,
    ) async {
      // The pre-fill the setup modal used to do (D7) now lives here.
      final prefs = FakePreferencesService();
      await PracticeSettingsStore(
        prefs,
      ).save('sample', const PracticeSettings(startMeasure: 1, endMeasure: 2));

      final container = await _pump(tester, prefs: prefs);
      expect(container.read(playerProvider).isSelectiveRun, isFalse);

      await _openSelect(tester);
      expect(_title(tester), 'Bars 2–3');
      // Still a draft: nothing applies until confirmed.
      expect(container.read(playerProvider).hasPracticeRange, isFalse);
      await _teardown(tester, container);
    });
  });
}
