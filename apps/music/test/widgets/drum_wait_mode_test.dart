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
import 'package:music/painters/drum_kit_art.dart';
import 'package:music/painters/drum_cascade_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/drum_kit.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

// Wait Mode for percussion (change: add-drum-scoring), driven through the
// PLAYER SEAM — the screen's Ticker calling `advance` — because a
// notifier-only test cannot see that seam at all.
//
// Fixture: `sampleOpenGrooveDocument` at 100 BPM / 4 divisions per quarter, so
// a quarter is 600 ms. Onset 1 (0 ms) is crash 49 + closed hat 42 in the hands
// and kick 36 in the feet; onset 2 (600 ms) is the closed hat 42 with the
// snare 38 — a two-piece onset, so it doubles as the "every stroke, in any
// order" case. The feet play again at 1200 ms.

const _entry = CatalogEntry(
  id: 'drums-wait',
  title: 'Groove ouvert',
  composer: 'Cymbra',
  assetPath: 'assets/scores/groove.musicxml',
  level: PracticeLevel.beginner,
);

/// Frames small enough for the notifier to accept them (it drops any delta of
/// 100 ms or more as a stall), pumped explicitly — never `pumpAndSettle`,
/// which would spin forever on the player's always-running Ticker.
Future<void> _frames(WidgetTester tester, {int count = 1}) async {
  for (var i = 0; i < count; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

Future<ProviderContainer> _pump(WidgetTester tester) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(
        FakeNotationEngine(document: sampleOpenGrooveDocument()),
      ),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
    ],
  );
  container.read(selectedScoreProvider.notifier).select(_entry);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const PlayerScreen()),
    ),
  );
  await _frames(tester, count: 12);
  final validate = find.widgetWithText(FilledButton, 'Play');
  if (validate.evaluate().isNotEmpty) {
    await tester.tap(validate);
    await _frames(tester, count: 6);
  }
  return container;
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

/// EXPERIMENT (drum-highway): the awaited pieces pulse on the DRAWN kit, so
/// the assertion reads the cascade painter that draws it.
DrumCascadePainter _strip(WidgetTester tester) =>
    tester
            .widgetList<CustomPaint>(
              find.byWidgetPredicate(
                (w) => w is CustomPaint && w.painter is DrumCascadePainter,
              ),
            )
            .first
            .painter!
        as DrumCascadePainter;

/// Where to tap to strike [surface] on the drawn kit, from the app's own
/// geometry rather than a copy of the layout arithmetic.
Offset _kitPoint(WidgetTester tester, List<DrumLane> lanes, int surface) {
  final rect = tester.getRect(find.byKey(const Key('drum-kit-surface')));
  final art = DrumKitArt(
    lanes: lanes,
    size: rect.size,
    top: rect.height * 0.66,
  );
  return rect.topLeft +
      (surface == kPedalSurface
          ? art.kickRect.center
          : art.pieceRect(surface).center);
}

void main() {
  testWidgets('the gate freezes at the first onset and releases only when '
      'every required stroke has landed', (tester) async {
    final c = await _pump(tester);
    final player = c.read(playerProvider.notifier);
    expect(c.read(playerProvider).waitMode, isTrue);

    player.startPlayback();
    await _frames(tester, count: 3);
    // Frozen on the opening onset: crash + hat + kick.
    expect(c.read(playerProvider).blocked, isTrue);
    expect(c.read(playerProvider).elapsedMs, 0);
    expect(c.read(playerProvider).onsetPitchesAt(0), {49, 42, 36});

    // A partial onset keeps blocking — in any order, every stroke is needed.
    player.noteOn(42);
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isTrue);
    player.noteOn(36); // the kick gates via the bar like any stroke
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isTrue);

    player.noteOn(49);
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isFalse);
    expect(c.read(playerProvider).elapsedMs, greaterThan(0));
    await _teardown(tester, c);
  });

  testWidgets('a stroke of an equivalent number releases the gate', (
    tester,
  ) async {
    final c = await _pump(tester);
    final player = c.read(playerProvider.notifier);
    player.startPlayback();
    await _frames(tester, count: 3);
    expect(c.read(playerProvider).blocked, isTrue);

    // The kit sends 35 where the score writes 36 — one pedal.
    player
      ..noteOn(42)
      ..noteOn(49)
      ..noteOn(35);
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isFalse);
    await _teardown(tester, c);
  });

  testWidgets('the wrong piece keeps blocking and is judged an extra note', (
    tester,
  ) async {
    final c = await _pump(tester);
    final player = c.read(playerProvider.notifier);
    player.startPlayback();
    await _frames(tester, count: 3);

    player.noteOn(47); // a tom: this groove contains none
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isTrue);
    expect(c.read(playerProvider).elapsedMs, 0);
    // The scorer saw it, and saw it as wrong.
    final hits = c.read(performanceScorerProvider).recentHits;
    expect(hits, isNotEmpty);
    expect(hits.last.wrong, isTrue);
    await _teardown(tester, c);
  });

  testWidgets('a strike EARLIER than the tolerance does not pre-satisfy the '
      'next onset — a stroke is an attack, never a hold', (tester) async {
    final c = await _pump(tester);
    final player = c.read(playerProvider.notifier);
    player.startPlayback();
    await _frames(tester, count: 3);

    // Clear onset 1 (0 ms) and let the playhead travel toward onset 2 (600 ms).
    player
      ..noteOn(42)
      ..noteOn(49)
      ..noteOn(36);
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isFalse);
    final travelling = c.read(playerProvider).elapsedMs;
    expect(travelling, greaterThan(0));
    expect(travelling, lessThan(600));

    // Strike the UPCOMING onset's hi-hat early, and hold it (never released).
    player.noteOn(42);
    expect(c.read(playerProvider).activeNotes, contains(42));
    await _frames(tester, count: 16); // the playhead reaches 600 ms

    // The gate is waiting all the same: the stroke was played hundreds of
    // milliseconds before the onset — far outside [kStrokeToleranceMs] — so
    // it satisfied nothing, and the still-held pad does not walk it through.
    expect(c.read(playerProvider).elapsedMs, 600);
    expect(c.read(playerProvider).blocked, isTrue);
    await _frames(tester, count: 4);
    expect(c.read(playerProvider).blocked, isTrue);
    expect(c.read(playerProvider).elapsedMs, 600);

    // The onset's OTHER piece alone does not release it either — the early
    // hi-hat is still owed.
    player.noteOn(38);
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isTrue);

    // A fresh strike at the gate is what releases it.
    player.noteOn(42);
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isFalse);
    await _teardown(tester, c);
  });

  testWidgets('a stroke a hair early still validates the onset, and is spent '
      'doing it', (tester) async {
    final c = await _pump(tester);
    final player = c.read(playerProvider.notifier);
    player.startPlayback();
    await _frames(tester, count: 3);

    // Clear onset 1 (0 ms) and run toward onset 2 (600 ms: hat 42 + snare 38).
    player
      ..noteOn(42)
      ..noteOn(49)
      ..noteOn(36);
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isFalse);

    // Stop the playhead INSIDE the tolerance window, before the onset.
    while (c.read(playerProvider).elapsedMs < 600 - kStrokeToleranceMs + 10) {
      await _frames(tester);
    }
    final early = c.read(playerProvider).elapsedMs;
    expect(early, lessThan(600));
    expect(600 - early, lessThanOrEqualTo(kStrokeToleranceMs));

    // Both strokes, played a hair before the beat — as a drummer plays.
    player
      ..noteOn(42)
      ..noteOn(38);
    await _frames(tester, count: 6);

    // The playhead walked straight through the onset: nothing was demanded a
    // second time.
    expect(c.read(playerProvider).blocked, isFalse);
    expect(c.read(playerProvider).elapsedMs, greaterThan(600));
    // And both strokes were SPENT crossing it, so neither can be credited to
    // a later onset as well.
    expect(c.read(playerProvider).strokeAtMs, isEmpty);
    await _teardown(tester, c);
  });

  testWidgets('with the hands selected a kick-only onset has an empty required '
      'set and the gate advances', (tester) async {
    final c = await _pump(tester);
    final player = c.read(playerProvider.notifier)
      ..setSelectedHands(Hand.right); // the hands
    await _frames(tester);
    player.startPlayback();
    await _frames(tester, count: 3);

    // No foot event is awaited anywhere in the run.
    expect(
      c
          .read(playerProvider)
          .visibleNotes
          .any((n) => kKickGmNumbers.contains(n.pitch)),
      isFalse,
    );
    expect(c.read(playerProvider).onsetPitchesAt(0), {49, 42});
    // The hands alone release the opening onset — the hidden foot never gates.
    player
      ..noteOn(42)
      ..noteOn(49);
    await _frames(tester, count: 2);
    expect(c.read(playerProvider).blocked, isFalse);
    await _teardown(tester, c);
  });

  testWidgets('with the feet selected, hand onsets are skipped and only the '
      'foot gates', (tester) async {
    final c = await _pump(tester);
    final player = c.read(playerProvider.notifier)
      ..setSelectedHands(Hand.left); // the feet
    await _frames(tester);
    player.startPlayback();
    await _frames(tester, count: 3);

    expect(c.read(playerProvider).onsetPitchesAt(0), {36});
    // Onset 2 (600 ms) is a hand stroke: with the feet selected there is
    // nothing there to wait for, so the playhead runs straight past it and on
    // toward the next kick (1200 ms).
    player.noteOn(36);
    await _frames(tester, count: 16);
    expect(c.read(playerProvider).elapsedMs, greaterThan(600));
    expect(c.read(playerProvider).elapsedMs, lessThanOrEqualTo(1200));
    await _teardown(tester, c);
  });

  group('the indicator lives on the pad strip', () {
    testWidgets('the expected pads (and the pedal) pulse while blocked and '
        'stop on release', (tester) async {
      final c = await _pump(tester);
      final player = c.read(playerProvider.notifier);
      final data = c.read(playerProvider);
      final lanes = data.presentedDrumLanes;
      final hatPad = laneIndexOf(lanes, 42);
      final crashPad = laneIndexOf(lanes, 49);

      player.startPlayback();
      await _frames(tester, count: 4);
      expect(c.read(playerProvider).blocked, isTrue);

      // Exactly the awaited surfaces are outlined — the pads of the pieces the
      // onset requires, plus the pedal because a kick is required.
      final blocked = _strip(tester);
      expect(blocked.expectedSurfaces, {hatPad, crashPad, kPedalSurface});
      // …and the outline is breathing.
      expect(blocked.waitPulse, greaterThan(0));

      // No overlay and no banner over the play surface: the strip IS the
      // indicator (and the pitch-naming reading aid has nothing to say about a
      // kit, so it stays away too).
      expect(find.byKey(const Key('reading-aid-figure')), findsNothing);
      expect(find.byType(Dialog), findsNothing);

      // Releasing the gate stops the pulse: the outline returns to steady.
      player
        ..noteOn(42)
        ..noteOn(49)
        ..noteOn(36);
      await _frames(tester, count: 2);
      expect(c.read(playerProvider).blocked, isFalse);
      expect(_strip(tester).waitPulse, 0);
      await _teardown(tester, c);
    });

    testWidgets('a hidden limb is never shown as expected', (tester) async {
      final c = await _pump(tester);
      final player = c.read(playerProvider.notifier)
        ..setSelectedHands(Hand.right);
      await _frames(tester);
      player.startPlayback();
      await _frames(tester, count: 4);
      expect(_strip(tester).expectedSurfaces, isNot(contains(kPedalSurface)));
      await _teardown(tester, c);
    });

    testWidgets('tapping the expected pad releases the gate', (tester) async {
      // The strip is a real controller: the gate is satisfiable with nothing
      // but the pointer, on a device with no kit attached.
      final c = await _pump(tester);
      c.read(playerProvider.notifier).startPlayback();
      await _frames(tester, count: 4);
      expect(c.read(playerProvider).blocked, isTrue);

      final lanes = c.read(playerProvider).presentedDrumLanes;
      for (var i = 0; i < lanes.length; i++) {
        await tester.tapAt(_kitPoint(tester, lanes, i));
        await _frames(tester);
      }
      await tester.tapAt(_kitPoint(tester, lanes, kPedalSurface));
      await _frames(tester, count: 2);
      expect(c.read(playerProvider).blocked, isFalse);
      await _teardown(tester, c);
    });
  });

  testWidgets('the free-run countdown behaves for percussion exactly as for a '
      'keyboard score', (tester) async {
    final c = await _pump(tester);
    final player = c.read(playerProvider.notifier);

    // Wait Mode start: no countdown — the gate already gives unlimited ready
    // time at the first onset.
    player.startPlayback();
    await _frames(tester);
    expect(c.read(playerProvider).countdownMs, 0);

    // Free-run start from the top: the 3…2…1 is armed.
    player
      ..setPlaying(false)
      ..toggleWaitMode()
      ..restart();
    await _frames(tester);
    player.startPlayback();
    await _frames(tester);
    expect(c.read(playerProvider).countdownMs, greaterThan(0));
    await _teardown(tester, c);
  });
}
