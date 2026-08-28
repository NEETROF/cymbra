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

import 'package:flutter/foundation.dart'
    show debugDefaultTargetPlatformOverride;
import 'dart:math' as math;

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show LogicalKeyboardKey;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/drum_cascade_painter.dart';
import 'package:music/painters/drum_kit_art.dart';
import 'package:music/painters/partition_painter.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/painters/drum_pad_strip_painter.dart';
import 'package:music/painters/drum_highway_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/src/rust/api/musicxml.dart' show ScoreDocument;
import 'package:music/state/drum_kit.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/player_preferences.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

const _entry = CatalogEntry(
  id: 'drums-1',
  title: 'Groove',
  composer: 'Tester',
  assetPath: 'assets/scores/groove.musicxml',
  level: PracticeLevel.beginner,
);

Future<ProviderContainer> _pumpPercussion(
  WidgetTester tester, {
  Size size = const Size(1400, 900),
  bool dismissModal = true,
  bool kitReady = false,
  ScoreDocument? document,
  FakePreferencesService? prefs,
}) async {
  await tester.binding.setSurfaceSize(size);
  final container = ProviderContainer(
    overrides: [
      if (prefs != null && !kitReady)
        preferencesServiceProvider.overrideWithValue(prefs),
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(
        FakeNotationEngine(document: document ?? sampleDrumDocument()),
      ),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      // A resolvable kit, so the readiness gate the screen's ScoreFontListener
      // opens lets strokes actually sound (change: add-drum-audio-channel).
      // Off by default: the display-side tests do not need the synth.
      if (kitReady) ...[
        preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
        soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
        soundFontImporterProvider.overrideWithValue(FakeSoundFontImporter()),
        privateSoundFontServiceProvider.overrideWithValue(
          FakePrivateSoundFontService(),
        ),
        soundFontCatalogServiceProvider.overrideWithValue(
          FakeSoundFontCatalogService(
            downloadable: [
              fakeDownloadPiano(
                id: defaultKitId,
                label: 'Kit',
                family: SoundFamily.percussion,
              ),
            ],
          ),
        ),
      ],
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
  // Dismiss the pre-play setup modal so the player underneath is reachable.
  if (dismissModal) {
    final validate = find.widgetWithText(FilledButton, 'Play');
    if (validate.evaluate().isNotEmpty) {
      await tester.tap(validate);
      for (var i = 0; i < 6; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
    }
  }
  return container;
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

/// Where to tap to strike [surface] (a lane index, or [kPedalSurface] for the
/// kick) on the drawn kit — computed with the SAME geometry the painter used,
/// so these taps cannot drift from what is on screen.
Offset _kitPoint(
  WidgetTester tester,
  List<DrumLane> lanes,
  int surface, {
  bool stage = false,
}) {
  final rect = tester.getRect(find.byKey(const Key('drum-kit-surface')));
  final art = DrumKitArt(
    lanes: lanes,
    size: rect.size,
    // The cascade reserves its bottom third for the kit; the stage's band
    // starts at its hit line.
    top: rect.height * (stage ? 0.62 : 0.66),
  );
  final local = surface == kPedalSurface
      ? art.kickRect.center
      : art.pieceRect(surface).center;
  return rect.topLeft + local;
}

void main() {
  testWidgets('EXPERIMENT (drum-highway): the stage is a fourth mode, offered '
      'for a percussion score only', (tester) async {
    final c = await _pumpPercussion(tester);

    // Four segments on a drum score: cascade, stage, staff, partition.
    expect(find.text('Stage'), findsOneWidget);
    await tester.tap(find.text('Stage'));
    for (var i = 0; i < 6; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(c.read(playerProvider).mode, RenderMode.stage);
    // It draws the highway, not the flat cascade.
    expect(
      find.byWidgetPredicate(
        (w) => w is CustomPaint && w.painter is DrumHighwayPainter,
      ),
      findsOneWidget,
    );
    expect(
      find.byWidgetPredicate(
        (w) => w is CustomPaint && w.painter is DrumCascadePainter,
      ),
      findsNothing,
    );
    await _teardown(tester, c);
  });

  testWidgets('a percussion score routes to the cascade, which draws its own '
      'kit instead of a strip', (tester) async {
    final c = await _pumpPercussion(tester);
    final data = c.read(playerProvider);

    expect(data.isPercussion, isTrue);
    // The layout was derived ONCE: hi-hat then snare, the kick excluded.
    expect(
      [for (final l in data.drumLanes) l.labelKey],
      ['kitPieceHiHat', 'kitPieceSnare'],
    );
    // EXPERIMENT (drum-highway): the kit is drawn INSIDE the play surface,
    // and the strip of labelled rectangles under it is gone — it said the
    // same thing twice and put what you strike away from what you read.
    expect(find.byKey(const Key('drum-kit-surface')), findsOneWidget);
    expect(find.byKey(const Key('pad-strip')), findsNothing);
    expect(find.byKey(const Key('onscreen-keyboard')), findsNothing);
    // …and the cascade is the mode in force.
    expect(data.mode, RenderMode.synthesia);
    await _teardown(tester, c);
  });

  testWidgets('the mode toggle offers the full set — cascade, Staff and '
      'Partition — and the cascade stays the default on load '
      '(add-drum-notation-render)', (tester) async {
    final c = await _pumpPercussion(tester);
    // The same mode set a keyboard score gets on this device…
    expect(find.text('Staff'), findsOneWidget);
    expect(find.text('Partition'), findsOneWidget);
    expect(find.text('Cascade'), findsOneWidget);
    // …with the cascade still the default presentation.
    expect(c.read(playerProvider).mode, RenderMode.synthesia);
    expect(
      find.byWidgetPredicate(
        (w) => w is CustomPaint && w.painter is DrumCascadePainter,
      ),
      findsOneWidget,
    );
    await _teardown(tester, c);
  });

  testWidgets('Wait Mode is offered for percussion exactly as for a keyboard '
      'score (the add-drum-kit-view interim, lifted)', (tester) async {
    final c = await _pumpPercussion(tester);
    // The transport's Wait toggle is present, and loading a drum score no
    // longer forces the mode off: the pads satisfy the gate and the matcher
    // judges what they satisfy (change: add-drum-scoring).
    expect(find.text('Wait'), findsOneWidget);
    expect(c.read(playerProvider).waitMode, isTrue);
    c.read(playerProvider.notifier).toggleWaitMode();
    await tester.pump(const Duration(milliseconds: 50));
    expect(c.read(playerProvider).waitMode, isFalse);
    await _teardown(tester, c);
  });

  testWidgets('switching to Staff and Partition engraves notation and '
      'switching back preserves playback state', (tester) async {
    final c = await _pumpPercussion(tester);
    final notifier = c.read(playerProvider.notifier);
    notifier.setPlaying(true);
    for (var i = 0; i < 4; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    final wasPlaying = c.read(playerProvider).isPlaying;
    final elapsedBefore = c.read(playerProvider).elapsedMs;

    // Staff mode: the scrolling staff replaces the cascade, and the DRAWN
    // KIT takes the band under it — the staff engraves notes but offers
    // nothing to aim at, and one score must not teach two pictures of the
    // same instrument.
    await tester.tap(find.text('Staff'));
    await tester.pump(const Duration(milliseconds: 50));
    expect(find.byKey(const Key('drum-kit')), findsOneWidget);
    expect(
      find.byWidgetPredicate(
        (w) => w is CustomPaint && w.painter is StaffPainter,
      ),
      findsWidgets,
    );
    expect(
      find.byWidgetPredicate(
        (w) => w is CustomPaint && w.painter is DrumCascadePainter,
      ),
      findsNothing,
    );

    // Partition mode: the engraved canvas appears.
    await tester.tap(find.text('Partition'));
    for (var i = 0; i < 3; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(find.byKey(const Key('partition-canvas')), findsOneWidget);
    // …and the Partition draws NO kit: a printed page is read, not aimed at.
    expect(find.byKey(const Key('drum-kit')), findsNothing);
    expect(
      find.byWidgetPredicate(
        (w) => w is CustomPaint && w.painter is PartitionPainter,
      ),
      findsWidgets,
    );

    // Back to the cascade: playback state survived the round trip like it
    // does for a keyboard score.
    await tester.tap(find.text('Cascade'));
    await tester.pump(const Duration(milliseconds: 50));
    expect(
      find.byWidgetPredicate(
        (w) => w is CustomPaint && w.painter is DrumCascadePainter,
      ),
      findsOneWidget,
    );
    final data = c.read(playerProvider);
    expect(data.isPlaying, wasPlaying);
    expect(data.elapsedMs, greaterThanOrEqualTo(elapsedBefore));
    await _teardown(tester, c);
  });

  testWidgets('the measure-select screen resolves written measures on a '
      'percussion score (smoke)', (tester) async {
    final c = await _pumpPercussion(
      tester,
      document: sampleOpenGrooveDocument(),
    );
    await tester.longPress(find.byKey(const Key('transport-rewind')));
    for (var i = 0; i < 6; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    expect(find.byKey(const Key('measure-select-canvas')), findsOneWidget);

    // One measure per system (fake layout): tap the middle of each line.
    final box = tester.getRect(find.byKey(const Key('measure-select-canvas')));
    final painter = PartitionPainter(
      document: sampleOpenGrooveDocument(),
      systems: const [],
    );
    Offset bar(int index) =>
        box.topLeft +
        Offset(
          box.width * 0.7,
          painter.systemTopY(index) + painter.systemStride / 2,
        );
    await tester.tapAt(bar(0));
    for (var i = 0; i < 3; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    await tester.tapAt(bar(1));
    for (var i = 0; i < 3; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    await tester.tap(find.byKey(const Key('measure-select-confirm')));
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    final data = c.read(playerProvider);
    expect(data.practiceStartMeasure, 0);
    expect(data.practiceEndMeasure, 1);
    await _teardown(tester, c);
  });

  // --- The pads as a controller (change: add-drum-input-mapping) ---------

  testWidgets('a pad tap emits its lane\'s stroke: it sounds as a one-shot '
      'and the pad flashes', (tester) async {
    final c = await _pumpPercussion(tester, kitReady: true);
    final audio = c.read(audioServiceProvider) as RecordingAudioService;
    // The groove's lanes are hi-hat then snare: strike the drawn snare.
    await tester.tapAt(
      _kitPoint(tester, c.read(playerProvider).presentedDrumLanes, 1),
    );
    await tester.pump(const Duration(milliseconds: 16));

    final data = c.read(playerProvider);
    // The score writes its snare as the acoustic 38, so that is what the pad
    // emits — through the very entry point a key press uses.
    expect(audio.drumOns.map((e) => e.key), [38]);
    expect(audio.noteOns, isEmpty); // never the pitched piano voice
    expect(data.struckSurfacesMs.keys, [1]); // the snare pad flashes
    // …and the painter is actually handed the flash.
    final painter =
        tester
                .widgetList<CustomPaint>(
                  find.descendant(
                    of: find.byKey(const Key('drum-kit-surface')),
                    matching: find.byType(CustomPaint),
                  ),
                )
                .first
                .painter
            as DrumCascadePainter;
    expect(painter.struckMs.keys, [1]);
    expect(
      DrumPadStripPainter.flashIntensity(
        struckMs: painter.struckMs[1]!,
        nowMs: painter.nowMs,
      ),
      greaterThan(0),
    );
    await _teardown(tester, c);
  });

  testWidgets('the kit has no dead gaps: a tap between two drums still '
      'strikes the nearer one, and the band under them is all kick', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester, kitReady: true);
    final audio = c.read(audioServiceProvider) as RecordingAudioService;
    final lanes = c.read(playerProvider).presentedDrumLanes;
    final surface = tester.getRect(find.byKey(const Key('drum-kit-surface')));
    // Between the hi-hat and the snare, just above the kick band: the gap is
    // styling, not a hit boundary. A swallowed tap here is a ghost stroke.
    final hat = _kitPoint(tester, lanes, 0);
    final snare = _kitPoint(tester, lanes, 1);
    await tester.tapAt(
      Offset((hat.dx + snare.dx) / 2 - 1, math.min(hat.dy, snare.dy)),
    );
    await tester.pump(const Duration(milliseconds: 16));
    expect(audio.drumOns.map((e) => e.key), [42]); // the nearer piece

    // The kick's band runs edge to edge under the drums.
    final kickY = _kitPoint(tester, lanes, kPedalSurface).dy;
    await tester.tapAt(Offset(surface.left + 4, kickY));
    await tester.pump(const Duration(milliseconds: 16));
    await tester.tapAt(Offset(surface.right - 4, kickY));
    await tester.pump(const Duration(milliseconds: 16));
    expect(audio.drumOns.map((e) => e.key), [42, 36, 36]);
    expect(
      c.read(playerProvider).struckSurfacesMs.keys,
      contains(kPedalSurface),
    );
    await _teardown(tester, c);
  });

  testWidgets('multi-touch: two drums struck together both emit, and a '
      'two-finger roll on ONE drum emits every stroke', (tester) async {
    final c = await _pumpPercussion(tester, kitReady: true);
    final audio = c.read(audioServiceProvider) as RecordingAudioService;
    final lanes = c.read(playerProvider).presentedDrumLanes;
    Offset pad(int lane) => _kitPoint(tester, lanes, lane);

    // Two different drums at once.
    final hat = await tester.startGesture(pad(0));
    final snare = await tester.startGesture(pad(1));
    await tester.pump(const Duration(milliseconds: 16));
    expect(audio.drumOns.map((e) => e.key), [42, 38]);
    await hat.up();
    await snare.up();
    await tester.pump(const Duration(milliseconds: 16));

    // A roll: alternating fingers on the SAME pad, each pressing while the
    // other is still down. Every pointer-down is a fresh stroke — no
    // keyboard-style retrigger exclusivity, because one-shots have no
    // sustained voice an extra attack could steal.
    audio.drumOns.clear();
    final left = await tester.startGesture(pad(1));
    await tester.pump(const Duration(milliseconds: 16));
    final right = await tester.startGesture(pad(1) + const Offset(6, 0));
    await tester.pump(const Duration(milliseconds: 16));
    await left.up();
    final again = await tester.startGesture(pad(1));
    await tester.pump(const Duration(milliseconds: 16));
    await right.up();
    await again.up();
    await tester.pump(const Duration(milliseconds: 16));
    expect(audio.drumOns.map((e) => e.key), [38, 38, 38]);
    await _teardown(tester, c);
  });

  testWidgets('pads play while stopped and during playback alike', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester, kitReady: true);
    final audio = c.read(audioServiceProvider) as RecordingAudioService;
    final snare = _kitPoint(
      tester,
      c.read(playerProvider).presentedDrumLanes,
      1,
    );

    expect(c.read(playerProvider).isPlaying, isFalse);
    await tester.tapAt(snare);
    await tester.pump(const Duration(milliseconds: 16));
    expect(audio.drumOns, hasLength(1));

    c.read(playerProvider.notifier).setPlaying(true);
    for (var i = 0; i < 3; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
    audio.drumOns.clear();
    await tester.tapAt(snare);
    await tester.pump(const Duration(milliseconds: 16));
    expect(audio.drumOns.map((e) => e.key), contains(38));
    await _teardown(tester, c);
  });

  testWidgets('the struck flash animates on its own clock while playback is '
      'stopped', (tester) async {
    final c = await _pumpPercussion(tester);
    DrumCascadePainter painter() =>
        tester
                .widgetList<CustomPaint>(
                  find.byWidgetPredicate(
                    (w) => w is CustomPaint && w.painter is DrumCascadePainter,
                  ),
                )
                .first
                .painter
            as DrumCascadePainter;

    await tester.tapAt(
      _kitPoint(tester, c.read(playerProvider).presentedDrumLanes, 1),
    );
    await tester.pump(const Duration(milliseconds: 16));
    expect(c.read(playerProvider).isPlaying, isFalse);
    final first = painter();
    await tester.pump(const Duration(milliseconds: 16));
    final second = painter();
    // A fresh painter on the next frame with no playhead moving at all: the
    // kit repaints on the flash's clock, not on playback frames.
    expect(identical(first, second), isFalse);
    expect(second.struckMs.keys, [1]);
    await _teardown(tester, c);
  });

  testWidgets('a stroke lights the kit and changes nothing about what falls', (
    tester,
  ) async {
    // EXPERIMENT (drum-highway): the kit moved INTO the cascade, so the
    // painter does receive the stroke state — what must stay true is the
    // division itself: striking a drum lights that drum and leaves the
    // falling surface exactly as it was.
    final c = await _pumpPercussion(tester);
    DrumCascadePainter cascade() =>
        tester
                .widgetList<CustomPaint>(
                  find.byWidgetPredicate(
                    (w) => w is CustomPaint && w.painter is DrumCascadePainter,
                  ),
                )
                .first
                .painter
            as DrumCascadePainter;

    final before = cascade();
    await tester.tapAt(
      _kitPoint(tester, c.read(playerProvider).presentedDrumLanes, 1),
    );
    await tester.pump(const Duration(milliseconds: 16));
    // The struck drum lights up…
    expect(c.read(playerProvider).struckSurfacesMs.keys, [1]);
    final after = cascade();
    expect(after.struckMs.keys, [1]);
    // …and every input the FALLING surface is drawn from is unchanged across
    // the stroke: the same division the keyboard makes between its keys and
    // the waterfall, now inside one painter.
    expect(after.elapsedMs, before.elapsedMs);
    expect(after.lanes, before.lanes);
    expect(after.multiVoice, before.multiVoice);
    expect(after.lookAheadMs, before.lookAheadMs);
    expect(
      [for (final n in after.notes) (n.pitch, n.startMs, n.durationMs)],
      [for (final n in before.notes) (n.pitch, n.startMs, n.durationMs)],
    );
    await _teardown(tester, c);
  });

  testWidgets('the assist keys stay silent on a percussion score', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester, kitReady: true);
    final audio = c.read(audioServiceProvider) as RecordingAudioService;
    // The keys shortcut the gate Wait Mode uses, and no percussion gate
    // exists until add-drum-scoring: the input path is real now, the
    // judgment they satisfy is not.
    for (final key in [LogicalKeyboardKey.keyA, LogicalKeyboardKey.keyZ]) {
      await tester.sendKeyDownEvent(key);
      await tester.pump(const Duration(milliseconds: 16));
      await tester.sendKeyUpEvent(key);
      await tester.pump(const Duration(milliseconds: 16));
    }
    expect(audio.drumOns, isEmpty);
    expect(audio.noteOns, isEmpty);
    expect(c.read(playerProvider).activeNotes, isEmpty);
    await _teardown(tester, c);
  });

  testWidgets('the inverted-kit setting reverses the PRESENTED order only', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester);
    final notifier = c.read(playerProvider.notifier);
    final before = c.read(playerProvider);
    expect(before.invertedKit, isFalse);

    notifier.setInvertedKit(enabled: true);
    await tester.pump();
    final after = c.read(playerProvider);
    // Presentation reverses — for the cascade AND the pad strip, which both
    // read presentedDrumLanes…
    expect(
      after.presentedDrumLanes,
      before.presentedDrumLanes.reversed.toList(),
    );
    // …while the derived layout and the notes (interpretation data) are
    // untouched, and the choice is persisted. (Semantic comparison: the
    // ticker may rebuild the state between reads, so identity is too strict.)
    expect(after.drumLanes, before.drumLanes);
    expect(
      [for (final n in after.notes) (n.pitch, n.startMs, n.voice)],
      [for (final n in before.notes) (n.pitch, n.startMs, n.voice)],
    );
    expect(c.read(playerPreferencesProvider).invertedKit, isTrue);
    await _teardown(tester, c);
  });

  // The counterpart of the former "hands / feet selection splits by the voice
  // convention" test, at the grain that replaced it (change:
  // add-practice-focus-controls): the same two selections a drummer used to
  // reach through the limb control are still expressible — "everything but the
  // kick" and "the kick alone" — and now every other subset is too.
  testWidgets('per-piece focus splits the kit at the piece grain', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester);
    final notifier = c.read(playerProvider.notifier);
    expect(c.read(playerProvider).hasDrumPiecesToFocus, isTrue);

    // Muting the kick: no bar survives the filter, so the cascade draws none —
    // what "hands only" used to mean, said directly.
    notifier.muteDrumPiece(kKickPieceId);
    final hands = c.read(playerProvider).visibleNotes;
    expect(hands, isNotEmpty);
    expect(hands.any((n) => kKickGmNumbers.contains(n.pitch)), isFalse);

    // Soloing the kick from an existing selection ADDS it back rather than
    // isolating it (design D2) — the additive rule, from the inside.
    notifier.soloDrumPiece(kKickPieceId);
    expect(c.read(playerProvider).mutedDrumPieces, isEmpty);

    // Soloing from the full kit isolates: the former "feet only".
    notifier.soloDrumPiece(kKickPieceId);
    final feet = c.read(playerProvider).visibleNotes;
    expect(feet, isNotEmpty);
    expect(feet.every((n) => kKickGmNumbers.contains(n.pitch)), isTrue);

    // …and the whole kit comes back.
    notifier.clearDrumFocus();
    expect(c.read(playerProvider).mutedDrumPieces, isEmpty);
    expect(
      c.read(playerProvider).visibleNotes.length,
      greaterThan(feet.length),
    );
    await _teardown(tester, c);
  });

  // The property the limb filter established and focus inherits: what is not
  // drawn is not awaited and not judged, all three from `visibleNotes`.
  testWidgets('a muted piece is not drawn, not awaited and not scored', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester);
    final notifier = c.read(playerProvider.notifier);
    notifier.muteDrumPiece(kKickPieceId);
    final data = c.read(playerProvider);
    expect(
      data.visibleNotes.any((n) => kKickGmNumbers.contains(n.pitch)),
      isFalse,
    );
    // The gate's required set is derived from the same source, so no onset can
    // ever wait on a piece the session does not draw.
    for (final gm in data.expectedKeys) {
      expect(kKickGmNumbers.contains(gm), isFalse);
    }
    // …and a restricted run is marked as one, so it is never submitted.
    expect(data.isFocusRestrictedRun, isTrue);
    await _teardown(tester, c);
  });

  // Focus states what is ASKED of the player, never what they hear from their
  // own kit (the `add-drum-input-mapping` 4.5 property, restated at the new
  // grain).
  testWidgets('a muted piece still sounds and still flashes when struck', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester);
    final notifier = c.read(playerProvider.notifier);
    notifier.muteDrumPiece(kKickPieceId);
    final kick = c.read(playerProvider).kickEmissionGm!;
    notifier.noteOn(kick, source: NoteSource.midiDevice);
    // The pedal is still a controller surface (it is read from `notes`, never
    // `visibleNotes`) and it still takes the stroke's flash stamp.
    expect(c.read(playerProvider).hasKickPedal, isTrue);
    expect(
      c.read(playerProvider).struckSurfacesMs.containsKey(kPedalSurface),
      isTrue,
    );
    await _teardown(tester, c);
  });

  testWidgets('the phone layout draws the kit and inherits the phone mode '
      'set: cascade + Staff, Partition remapped away', (tester) async {
    // Phone landscape: the same routing holds on the small form factor — the
    // strip follows the phone keyboard-height policy and the toggle is
    // icon-only. Mode parity is device-relative (the spec's phrasing): a
    // phone offers a keyboard score cascade + Staff and remaps Partition to
    // Staff, so a percussion score gets exactly that. The modal is dismissed
    // at desktop size first: it has a pre-existing overflow on phone
    // viewports (tracked separately) that would fail the frames before the
    // player is reachable.
    final c = await _pumpPercussion(tester);
    // A desktop host always classes as desktop: simulate the phone the way
    // the player-screen suite does — mobile platform + phone-sized view.
    debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
    try {
      tester.view.devicePixelRatio = 1.0;
      tester.view.physicalSize = const Size(844, 390);
      await tester.binding.setSurfaceSize(const Size(844, 390));
      for (var i = 0; i < 4; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      // The kit is drawn in the play surface on a phone too; the strip is
      // gone from the cascade there as everywhere else.
      expect(find.byKey(const Key('drum-kit-surface')), findsOneWidget);
      expect(find.byKey(const Key('pad-strip')), findsNothing);
      expect(find.byKey(const Key('onscreen-keyboard')), findsNothing);
      expect(find.byIcon(Icons.music_note), findsOneWidget); // Staff segment
      expect(find.byIcon(Icons.article), findsNothing); // no Partition here
      await _teardown(tester, c);
    } finally {
      debugDefaultTargetPlatformOverride = null;
      tester.view.resetPhysicalSize();
      tester.view.resetDevicePixelRatio();
    }
  });

  testWidgets('a drum score opens in the mode the player last read drums in', (
    tester,
  ) async {
    // The memory is per family (see player_preferences_test): here we only pin
    // that a LOADED percussion score is seeded from it, rather than always
    // opening on the cascade as it did before.
    final prefs = FakePreferencesService()
      ..store[PlayerPreferences.prefsKey] = jsonEncode({
        'percussionMode': 'stage',
        'keyboardMode': 'staff',
      });
    final c = await _pumpPercussion(tester, prefs: prefs);
    await tester.pump(const Duration(milliseconds: 50));
    expect(c.read(playerProvider).mode, RenderMode.stage);
    await _teardown(tester, c);
  });

  testWidgets(
    'with nothing remembered a drum score still opens on the cascade',
    (tester) async {
      final c = await _pumpPercussion(tester);
      expect(c.read(playerProvider).mode, RenderMode.synthesia);
      await _teardown(tester, c);
    },
  );

  testWidgets('the setup modal swaps the range chooser for the kit layout '
      'and the hand selector for per-piece focus', (tester) async {
    final c = await _pumpPercussion(tester, dismissModal: false);
    // The range apparatus does not apply to a drum kit: no keyboard-size
    // section; the inverted-kit switch takes its place, labelled by the
    // kit's layout, never handedness.
    expect(find.byKey(const Key('inverted-kit-switch')), findsOneWidget);
    expect(find.text('Inverted kit'), findsOneWidget);
    expect(
      find.byWidgetPredicate((w) => w is DropdownButton<KeyboardRangeMode>),
      findsNothing,
    );
    // The hand selector is gone entirely (change:
    // add-practice-focus-controls) — a drum part is written on one staff.
    expect(find.text('Right hand'), findsNothing);
    expect(find.text('Hands'), findsNothing);
    expect(find.text('Feet'), findsNothing);
    // The focus control takes its place, one row per piece of THIS score's
    // kit, in the pad strip's order.
    final data = c.read(playerProvider);
    expect(data.hasDrumPiecesToFocus, isTrue);
    for (final id in data.kitPieceIds) {
      expect(find.byKey(Key('drum-focus-$id')), findsOneWidget, reason: id);
    }
    // "Whole kit" is offered only once something is muted.
    expect(find.byKey(const Key('drum-focus-all')), findsNothing);
    await tester.tap(
      find.descendant(
        of: find.byKey(Key('drum-focus-${data.kitPieceIds.first}')),
        matching: find.byType(Checkbox),
      ),
    );
    await tester.pump();
    expect(c.read(playerProvider).mutedDrumPieces, {data.kitPieceIds.first});
    expect(find.byKey(const Key('drum-focus-all')), findsOneWidget);
    await _teardown(tester, c);
  });
}
