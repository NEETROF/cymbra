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
import 'package:music/screens/drum_calibration_screen.dart';
import 'package:music/screens/midi_monitor_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/src/rust/api/musicxml.dart' show ScoreDocument;
import 'package:music/state/drum_calibration_notifier.dart';
import 'package:music/state/drum_input_mapping_notifier.dart';
import 'package:music/state/drum_kit.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/src/rust/api/midi.dart' show MidiEvent, MidiEventKind;

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

/// The score a document-bearing pump loads, so the pass has a kit to read.
const _entry = CatalogEntry(
  id: 'sample',
  title: 'Groove',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

// The calibration flow end to end (change: add-drum-input-calibration,
// tasks 6.5–6.8 and 7.1–7.3).

void main() {
  late FakeMidiService midi;
  late RecordingAudioService audio;
  late FakePreferencesService prefs;
  late ProviderContainer container;
  var stamp = 0;

  Future<void> pump(
    WidgetTester tester, {
    String? connected = 'Drum kit',
    Map<String, String>? storedPrefs,
    ScoreDocument? document,
  }) async {
    stamp = 0;
    audio = RecordingAudioService();
    // `echoTo` makes the fake behave like the engine, which sounds a live
    // stroke from its own MIDI callback (change: add-drum-input-mapping §8).
    // Without it the "strokes stay audible" assertion would be checking a
    // fiction rather than the arrangement the app actually ships.
    midi = FakeMidiService(
      ports: connected == null ? const [] : [connected],
      connected: connected,
      echoTo: audio,
    );
    prefs = FakePreferencesService(storedPrefs);
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(audio),
        preferencesServiceProvider.overrideWithValue(prefs),
        scoreCatalogProvider.overrideWithValue(const [_entry]),
        scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
        notationEngineProvider.overrideWithValue(
          FakeNotationEngine(document: document),
        ),
      ],
    );
    // The store restores from preferences on a future; the pass reads it.
    await container.read(drumInputMappingStoreProvider.notifier).restored;
    // With a document, the surface is reached the way it is in the app: over a
    // player holding a loaded score, which is the kit the pass reads.
    if (document != null) {
      container.read(selectedScoreProvider.notifier).select(_entry);
    }
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(const DrumCalibrationScreen()),
      ),
    );
    await tester.pump();
    await tester.pump();
    // The score loads asynchronously; let it land before anything asks what
    // the kit is.
    if (document != null) {
      for (var i = 0; i < 12; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
    }
  }

  /// A stroke on [pitch], stamped later than every stroke before it — the
  /// engine's own monotonic clock, which is what the stale-stroke rule reads.
  Future<void> strike(WidgetTester tester, int pitch) async {
    midi.emit(
      MidiEvent(
        kind: MidiEventKind.noteOn,
        pitch: pitch,
        velocity: 100,
        channel: 9,
        timestampMs: BigInt.from(stamp += 100),
      ),
    );
    await tester.pump();
    await tester.pump();
  }

  Future<void> tap(WidgetTester tester, String key) async {
    await tester.tap(find.byKey(Key(key)));
    await tester.pump();
    await tester.pump();
  }

  Future<void> teardown(WidgetTester tester) async {
    await tester.pumpWidget(const SizedBox());
    await tester.pumpAndSettle();
    container.dispose();
  }

  testWidgets('with no device there is nothing to calibrate', (tester) async {
    await pump(tester, connected: null);
    expect(find.byKey(const Key('calibration-no-device')), findsOneWidget);
    await teardown(tester);
  });

  testWidgets('an uncalibrated kit opens on the empty table', (tester) async {
    await pump(tester);
    expect(find.byKey(const Key('calibration-mapping-empty')), findsOneWidget);
    expect(find.byKey(const Key('calibration-start')), findsOneWidget);
    // Nothing to clear when there is nothing stored.
    expect(find.byKey(const Key('calibration-clear-all')), findsNothing);
    await teardown(tester);
  });

  testWidgets('calibrate three pieces, skip one, complete — and the stored '
      'mapping is exactly what was played', (tester) async {
    await pump(tester);
    await tap(tester, 'calibration-start');
    expect(find.byKey(const Key('calibration-prompt')), findsOneWidget);

    // The pass asks for the kick, the snare, then the hi-hat (design D7).
    await strike(tester, 12); // kick
    await strike(tester, 31); // snare
    await tap(tester, 'calibration-skip'); // this kit has no hi-hat pad
    // …then every remaining piece is skipped to reach the end.
    for (var i = 3; i < kCalibrationPieceOrder.length; i++) {
      await tap(tester, 'calibration-skip');
    }

    // Completed: the table is shown, and it holds exactly the two strokes.
    final stored = container
        .read(drumInputMappingStoreProvider.notifier)
        .forPort('Drum kit');
    expect(stored.byPiece, {kKickPieceId: 12, 'kitPieceSnare': 31});
    expect(find.byKey(const Key('calibration-finished')), findsOneWidget);
    expect(find.byKey(Key('calibration-row-$kKickPieceId')), findsOneWidget);
    expect(
      find.byKey(const Key('calibration-row-kitPieceSnare')),
      findsOneWidget,
    );
    // The skipped piece has no row — nothing was invented for it.
    expect(
      find.byKey(const Key('calibration-row-kitPieceHiHat')),
      findsNothing,
    );
    await teardown(tester);
  });

  testWidgets('abandoning leaves the stored mapping exactly as it was', (
    tester,
  ) async {
    await pump(tester);
    // A kit calibrated earlier.
    container
        .read(drumInputMappingStoreProvider.notifier)
        .setPiece('Drum kit', 'kitPieceSnare', 40);
    await tester.pump();

    await tap(tester, 'calibration-start');
    await strike(tester, 12);
    await strike(tester, 99);
    // Leaving mid-pass writes nothing.
    container.read(drumCalibrationProvider.notifier).abandon();
    await tester.pump();

    final stored = container
        .read(drumInputMappingStoreProvider.notifier)
        .forPort('Drum kit');
    expect(stored.byPiece, {'kitPieceSnare': 40});
    await teardown(tester);
  });

  testWidgets('a number another piece holds is reported, not overwritten', (
    tester,
  ) async {
    await pump(tester);
    await tap(tester, 'calibration-start');
    await strike(tester, 12); // the kick
    await strike(tester, 12); // …offered again for the snare

    expect(find.byKey(const Key('calibration-conflict')), findsOneWidget);
    // The step has not moved on, so nothing was silently reassigned.
    final state = container.read(drumCalibrationProvider);
    expect(state.currentPiece, 'kitPieceSnare');
    expect(state.recorded, {kKickPieceId: 12});

    // Striking again clears it and takes the next stroke.
    await tap(tester, 'calibration-strike-again');
    expect(find.byKey(const Key('calibration-conflict')), findsNothing);
    await strike(tester, 31);
    expect(container.read(drumCalibrationProvider).recorded, {
      kKickPieceId: 12,
      'kitPieceSnare': 31,
    });
    await teardown(tester);
  });

  testWidgets('reassigning moves the number off the piece that held it', (
    tester,
  ) async {
    await pump(tester);
    await tap(tester, 'calibration-start');
    await strike(tester, 12);
    await strike(tester, 12);
    await tap(tester, 'calibration-reassign');
    // One number is never claimed twice.
    expect(container.read(drumCalibrationProvider).recorded, {
      'kitPieceSnare': 12,
    });
    await teardown(tester);
  });

  testWidgets('the raw read-out is reachable from here (design D11)', (
    tester,
  ) async {
    // It left the settings, where it read as an alternative to calibrating.
    // This is where a drummer already is when the pass did not fix it — a pad
    // learned on the wrong number, or one that fires a second one.
    await pump(tester);
    expect(find.byKey(const Key('calibration-open-monitor')), findsOneWidget);
    await tap(tester, 'calibration-open-monitor');
    expect(find.byType(MidiMonitorScreen), findsOneWidget);
    await teardown(tester);
  });

  testWidgets('the pass asks for THIS score\'s kit, not the standard one '
      '(design D10)', (tester) async {
    // The fixture is a kick / snare / closed-hi-hat groove. A drummer opening
    // it is asked three questions, not twenty-three: the pieces they are about
    // to play, in the pass's own order.
    await pump(tester, document: sampleDrumDocument());
    await tap(tester, 'calibration-start');
    expect(container.read(drumCalibrationProvider).pieces, [
      kKickPieceId,
      'kitPieceSnare',
      'kitPieceHiHat',
    ]);

    await strike(tester, 12);
    await strike(tester, 31);
    await strike(tester, 22);
    // Three strokes end it — nothing to skip through, nothing to finish early.
    expect(find.byKey(const Key('calibration-finished')), findsOneWidget);
    expect(
      container
          .read(drumInputMappingStoreProvider.notifier)
          .forPort('Drum kit')
          .byPiece,
      {kKickPieceId: 12, 'kitPieceSnare': 31, 'kitPieceHiHat': 22},
    );
    await teardown(tester);
  });

  testWidgets('with no score to read a kit from, the standard kit is asked '
      'for', (tester) async {
    // The fallback that keeps the surface usable on its own — and the reason
    // every test below still walks the full list.
    await pump(tester);
    await tap(tester, 'calibration-start');
    expect(
      container.read(drumCalibrationProvider).pieces,
      kCalibrationPieceOrder,
    );
    await teardown(tester);
  });

  testWidgets('the zones a module triggers separately are learned, and stay '
      'the piece they belong to', (tester) async {
    // The gap the beta found (design D9): a kit sends its rim, its open hi-hat
    // and its pedal on numbers of their own, and a pass that only asked for
    // pieces left them unmappable — and therefore silent and inert.
    await pump(tester);
    await tap(tester, 'calibration-start');
    await strike(tester, 12); // kick
    await strike(tester, 31); // snare
    await strike(tester, 33); // …its rim
    await strike(tester, 22); // hi-hat, closed
    await strike(tester, 26); // …open
    await strike(tester, 21); // …pedal
    await tap(tester, 'calibration-finish');

    final stored = container
        .read(drumInputMappingStoreProvider.notifier)
        .forPort('Drum kit');
    expect(stored.byPiece, {
      kKickPieceId: 12,
      'kitPieceSnare': 31,
      kCrossStickPieceId: 33,
      'kitPieceHiHat': 22,
      kOpenHiHatPieceId: 26,
      kPedalHiHatPieceId: 21,
    });
    // Each lands on the number the app reasons in…
    expect(stored.translate(33), 37);
    expect(stored.translate(26), 46);
    expect(stored.translate(21), 44);
    // …and the two hi-hat strokes remain ONE piece, told apart only by their
    // articulation, exactly as a score written in 42s and 46s is.
    expect(samePiece(42, stored.translate(26)), isTrue);
    expect(isOpenHiHat(stored.translate(26)), isTrue);
    expect(isOpenHiHat(stored.translate(22)), isFalse);
    // The rim is still the snare to everything downstream — the flash, the gate
    // and the scorer never learn the pass asked for it separately.
    expect(samePiece(38, stored.translate(33)), isTrue);
    await teardown(tester);
  });

  testWidgets('the pass can be finished at the auxiliary pads, keeping what it '
      'learned', (tester) async {
    await pump(tester);
    await tap(tester, 'calibration-start');
    // Nothing recorded yet: "finish here" would be "stop" under another name.
    expect(find.byKey(const Key('calibration-finish')), findsNothing);

    await strike(tester, 12); // the kick
    expect(find.byKey(const Key('calibration-finish')), findsOneWidget);
    // A kit that ends at the china skips the rest of the standard kit…
    for (var i = 1; i < kCalibrationKitPieceOrder.length; i++) {
      await tap(tester, 'calibration-skip');
    }
    // …and is told what the remaining steps are before deciding.
    expect(find.byKey(const Key('calibration-aux-hint')), findsOneWidget);

    await tap(tester, 'calibration-finish');
    // Completed, not abandoned: the table is shown and the stroke was kept.
    expect(find.byKey(const Key('calibration-finished')), findsOneWidget);
    expect(
      container
          .read(drumInputMappingStoreProvider.notifier)
          .forPort('Drum kit')
          .byPiece,
      {kKickPieceId: 12},
    );
    await teardown(tester);
  });

  testWidgets('strokes stay audible throughout the pass', (tester) async {
    // A player who hears nothing while calibrating cannot tell a mis-mapped pad
    // from a disconnected kit. The player notifier is alive underneath (the
    // pass is a route pushed over it) and keeps sounding what arrives.
    await pump(tester);
    await tap(tester, 'calibration-start');
    final before = audio.calls.length;
    await strike(tester, 12);
    expect(
      audio.calls.length,
      greaterThan(before),
      reason: 'a stroke during the pass must still reach the audio seam',
    );
    await teardown(tester);
  });

  testWidgets('one entry is cleared without re-running the pass', (
    tester,
  ) async {
    await pump(tester);
    final store = container.read(drumInputMappingStoreProvider.notifier)
      ..setPiece('Drum kit', 'kitPieceSnare', 31)
      ..setPiece('Drum kit', kKickPieceId, 12);
    await tester.pump();
    expect(
      find.byKey(const Key('calibration-row-kitPieceSnare')),
      findsOneWidget,
    );

    await tester.tap(
      find.descendant(
        of: find.byKey(const Key('calibration-row-kitPieceSnare')),
        matching: find.byType(TextButton),
      ),
    );
    await tester.pump();
    // Only that entry went.
    expect(store.forPort('Drum kit').byPiece, {kKickPieceId: 12});
    expect(
      find.byKey(const Key('calibration-row-kitPieceSnare')),
      findsNothing,
    );
    expect(find.byKey(Key('calibration-row-$kKickPieceId')), findsOneWidget);
    await teardown(tester);
  });

  testWidgets('clearing the kit returns it to uncalibrated behaviour', (
    tester,
  ) async {
    await pump(tester);
    final store = container.read(drumInputMappingStoreProvider.notifier)
      ..setPiece('Drum kit', 'kitPieceSnare', 31);
    await tester.pump();

    await tap(tester, 'calibration-clear-all');
    expect(store.forPort('Drum kit').isEmpty, isTrue);
    // Uncalibrated means the identity — asserted on the translation, not on the
    // absence of a row.
    expect(store.forPort('Drum kit').translate(31), 31);
    expect(find.byKey(const Key('calibration-mapping-empty')), findsOneWidget);
    await teardown(tester);
  });

  testWidgets('a stored mapping survives a restart', (tester) async {
    // The store is seeded from what a previous session persisted, exactly as it
    // is on a cold start with the kit plugged in.
    await pump(tester);
    container
        .read(drumInputMappingStoreProvider.notifier)
        .setPiece('Drum kit', 'kitPieceSnare', 31);
    await tester.pump();
    final persisted = {...prefs.store};
    await teardown(tester);

    await pump(tester, storedPrefs: persisted);
    final store = container.read(drumInputMappingStoreProvider.notifier);
    expect(store.forPort('Drum kit').byPiece, {'kitPieceSnare': 31});
    expect(
      find.byKey(const Key('calibration-row-kitPieceSnare')),
      findsOneWidget,
    );
    await teardown(tester);
  });

  testWidgets('another device is remembered independently', (tester) async {
    await pump(tester);
    final store = container.read(drumInputMappingStoreProvider.notifier)
      ..setPiece('Drum kit', 'kitPieceSnare', 31)
      ..setPiece('Practice pad', 'kitPieceSnare', 40);
    await tester.pump();
    expect(store.forPort('Drum kit').translate(31), 38);
    expect(store.forPort('Drum kit').translate(40), 40);
    expect(store.forPort('Practice pad').translate(40), 38);
    await teardown(tester);
  });
}
