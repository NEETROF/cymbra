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
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/practice_settings_store.dart';
import 'package:music/widgets/practice_range_picker.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

/// A [Notation] pinned to the four-measure fixture, so the player has a real
/// measure table without touching the byte sources.
class _FixedNotation extends Notation {
  _FixedNotation(this._value);
  final NotationData _value;
  @override
  NotationData build() => _value;
}

/// The audio fake of the most recent [_open], so a test can assert what the
/// picker actually sounded.
late RecordingAudioService lastAudio;

/// Pumps a bare host that opens the picker, and returns both the container and
/// the picker's result once it closes.
Future<(ProviderContainer, Future<bool>)> _open(
  WidgetTester tester, {
  FakePreferencesService? prefs,
  ScoreDocument? document,
}) async {
  final container = ProviderContainer(
    overrides: [
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource(null)),
      audioServiceProvider.overrideWithValue(
        lastAudio = RecordingAudioService(),
      ),
      notationProvider.overrideWith(
        () => _FixedNotation(
          NotationData(document: document ?? sampleFourMeasureDocument()),
        ),
      ),
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
    ],
  );
  // Keep the auto-dispose player alive for the test.
  container.listen(playerProvider, (_, _) {}, fireImmediately: true);

  late Future<bool> result;
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(
        Scaffold(
          body: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () => result = showPracticeRangePicker(context),
              child: const Text('go'),
            ),
          ),
        ),
        locale: const Locale('en'),
      ),
    ),
  );
  await _pumpFrames(tester);
  await tester.tap(find.text('go'));
  await _pumpFrames(tester);
  return (container, result);
}

/// Pumps a fixed number of frames — the player's 1 s MIDI-status timer never
/// settles, so `pumpAndSettle` would time out.
Future<void> _pumpFrames(WidgetTester tester) async {
  for (var i = 0; i < 8; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

/// Unmounts the host and disposes the container INSIDE the test body — the
/// player's periodic MIDI-status timer must be gone before the framework's
/// pending-timer check runs.
Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

Future<void> _tap(WidgetTester tester, Finder finder) async {
  await tester.ensureVisible(finder);
  await tester.pump();
  await tester.tap(finder);
  await tester.pump();
}

Future<void> _step(WidgetTester tester, String key, {required bool up}) => _tap(
  tester,
  find.descendant(
    of: find.byKey(Key(key)),
    matching: find.byIcon(up ? Icons.add : Icons.remove),
  ),
);

void main() {
  testWidgets('choosing a range starts a selective (unscored) run', (
    tester,
  ) async {
    final (container, result) = await _open(tester);

    expect(find.byKey(const Key('practice-range-picker')), findsOneWidget);
    expect(find.text('Practice a section'), findsOneWidget);
    // Bars 2 → 3.
    await _step(tester, 'practice-from', up: true);
    await _step(tester, 'practice-to', up: false);
    await _tap(tester, find.byKey(const Key('practice-picker-start')));
    await _pumpFrames(tester);

    expect(await result, isTrue);
    final data = container.read(playerProvider);
    expect(data.practiceStartMeasure, 1);
    expect(data.practiceEndMeasure, 2);
    expect(data.isSelectiveRun, isTrue);
    await _teardown(tester, container);
  });

  testWidgets('cancelling leaves the run untouched', (tester) async {
    final (container, result) = await _open(tester);

    await _tap(tester, find.widgetWithText(TextButton, 'Cancel'));
    await _pumpFrames(tester);

    expect(await result, isFalse);
    expect(container.read(playerProvider).hasPracticeRange, isFalse);
    await _teardown(tester, container);
  });

  testWidgets('the picker pre-fills from this score\'s saved settings', (
    tester,
  ) async {
    final prefs = FakePreferencesService();
    // Seed a saved selection for the piece the fixture resolves to.
    final store = PracticeSettingsStore(prefs);
    await store.save(
      'FourBars', // no catalog entry selected → the title is the key
      const PracticeSettings(startMeasure: 2, endMeasure: 3),
    );

    final (container, result) = await _open(tester, prefs: prefs);
    await _pumpFrames(tester);

    // Bars 3 → 4 (1-based) and looping off.
    expect(
      tester
          .widgetList<Text>(
            find.descendant(
              of: find.byKey(const Key('practice-from')),
              matching: find.byType(Text),
            ),
          )
          .last
          .data,
      '3',
    );

    await _tap(tester, find.byKey(const Key('practice-picker-start')));
    await _pumpFrames(tester);
    expect(await result, isTrue);
    final data = container.read(playerProvider);
    expect(data.practiceStartMeasure, 2);
    expect(data.practiceEndMeasure, 3);
    await _teardown(tester, container);
  });

  testWidgets('raising "from" past "to" pushes "to" along', (tester) async {
    final (container, _) = await _open(tester);

    // to: 4 → 2, then from: 1 → 3 (which drags "to" to 3).
    await _step(tester, 'practice-to', up: false);
    await _step(tester, 'practice-to', up: false);
    await _step(tester, 'practice-from', up: true);
    await _step(tester, 'practice-from', up: true);
    await _tap(tester, find.byKey(const Key('practice-picker-start')));
    await _pumpFrames(tester);

    final data = container.read(playerProvider);
    expect(data.practiceStartMeasure, 2);
    expect(data.practiceEndMeasure, 2);
    await _teardown(tester, container);
  });

  testWidgets('the picker auditions the chosen passage, and follows it', (
    tester,
  ) async {
    // Choosing bars is something you HEAR: opening the picker plays the passage,
    // and moving a bound re-auditions the NEW one instead of the old.
    final (container, _) = await _open(tester);

    // Opening the picker already sounds the passage from bar 1.
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
    await _step(tester, 'practice-from', up: true);
    await _step(tester, 'practice-from', up: true);
    await _step(tester, 'practice-to', up: true);
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
    final (container, _) = await _open(
      tester,
      document: sampleManyMeasureDocument(),
    );

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

    // Push "from" far to the right ("to" already opens on the LAST bar here);
    // the ribbon must follow the bound being moved.
    for (var i = 0; i < 12; i++) {
      await _step(tester, 'practice-from', up: true);
    }
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

  testWidgets('the picker opens on the saved passage, already scrolled', (
    tester,
  ) async {
    // The picker opens ALREADY holding the saved range, so no bound ever
    // "changes" — without an explicit scroll on open the ribbon sat at bar 1
    // while the passage was off-screen.
    final prefs = FakePreferencesService();
    final store = PracticeSettingsStore(prefs);
    await store.save(
      'ManyBars', // no catalog entry selected → the title is the key
      const PracticeSettings(startMeasure: 14, endMeasure: 17),
    );

    final (container, _) = await _open(
      tester,
      prefs: prefs,
      document: sampleManyMeasureDocument(),
    );
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
  });
}
