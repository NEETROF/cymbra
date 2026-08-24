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
import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show LogicalKeyboardKey;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/drum_cascade_painter.dart';
import 'package:music/painters/partition_painter.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/painters/drum_pad_strip_painter.dart';
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
}) async {
  await tester.binding.setSurfaceSize(size);
  final container = ProviderContainer(
    overrides: [
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

void main() {
  testWidgets('a percussion score routes to the cascade and the pad strip', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester);
    final data = c.read(playerProvider);

    expect(data.isPercussion, isTrue);
    // The layout was derived ONCE: hi-hat then snare, the kick excluded.
    expect(
      [for (final l in data.drumLanes) l.labelKey],
      ['kitPieceHiHat', 'kitPieceSnare'],
    );
    // The pad strip replaces the on-screen keyboard…
    expect(find.byKey(const Key('pad-strip')), findsOneWidget);
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
    expect(find.text('Synthesia'), findsOneWidget);
    // …with the cascade (Synthesia's slot) still the default presentation.
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

    // Staff mode: the scrolling staff replaces the cascade.
    await tester.tap(find.text('Staff'));
    await tester.pump(const Duration(milliseconds: 50));
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
    expect(
      find.byWidgetPredicate(
        (w) => w is CustomPaint && w.painter is PartitionPainter,
      ),
      findsWidgets,
    );

    // Back to the cascade: playback state survived the round trip like it
    // does for a keyboard score.
    await tester.tap(find.text('Synthesia'));
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
    final strip = tester.getRect(find.byKey(const Key('pad-strip')));
    // The groove's lanes are hi-hat then snare: tap the middle of the snare.
    await tester.tapAt(
      strip.topLeft + Offset(strip.width * 0.75, strip.height * 0.3),
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
        tester.widget<CustomPaint>(find.byKey(const Key('pad-strip'))).painter
            as DrumPadStripPainter;
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

  testWidgets('the whole strip is live: a tap in the gutter between two pads '
      'still strikes, and the pedal band is all kick', (tester) async {
    final c = await _pumpPercussion(tester, kitReady: true);
    final audio = c.read(audioServiceProvider) as RecordingAudioService;
    final strip = tester.getRect(find.byKey(const Key('pad-strip')));
    // Exactly on the boundary between the two drawn pads — the 3 px inset is
    // styling, not a hit boundary. A swallowed tap here is a ghost stroke.
    await tester.tapAt(
      strip.topLeft + Offset(strip.width / 2 - 1, strip.height * 0.2),
    );
    await tester.pump(const Duration(milliseconds: 16));
    expect(audio.drumOns.map((e) => e.key), [42]); // the hi-hat's span

    // The pedal band: kick from edge to edge, whatever the horizontal aim.
    await tester.tapAt(strip.topLeft + Offset(4, strip.height * 0.95));
    await tester.pump(const Duration(milliseconds: 16));
    await tester.tapAt(
      strip.topLeft + Offset(strip.width - 4, strip.height * 0.85),
    );
    await tester.pump(const Duration(milliseconds: 16));
    expect(audio.drumOns.map((e) => e.key), [42, 36, 36]);
    expect(
      c.read(playerProvider).struckSurfacesMs.keys,
      contains(kPedalSurface),
    );
    await _teardown(tester, c);
  });

  testWidgets('multi-touch: two pads struck together both emit, and a '
      'two-finger roll on ONE pad emits every stroke', (tester) async {
    final c = await _pumpPercussion(tester, kitReady: true);
    final audio = c.read(audioServiceProvider) as RecordingAudioService;
    final strip = tester.getRect(find.byKey(const Key('pad-strip')));
    Offset pad(double fraction) =>
        strip.topLeft + Offset(strip.width * fraction, strip.height * 0.3);

    // Two different pads at once.
    final hat = await tester.startGesture(pad(0.25));
    final snare = await tester.startGesture(pad(0.75));
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
    final left = await tester.startGesture(pad(0.7));
    await tester.pump(const Duration(milliseconds: 16));
    final right = await tester.startGesture(pad(0.8));
    await tester.pump(const Duration(milliseconds: 16));
    await left.up();
    final again = await tester.startGesture(pad(0.7));
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
    final strip = tester.getRect(find.byKey(const Key('pad-strip')));
    final snare =
        strip.topLeft + Offset(strip.width * 0.75, strip.height * 0.3);

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
    final strip = tester.getRect(find.byKey(const Key('pad-strip')));
    DrumPadStripPainter painter() =>
        tester.widget<CustomPaint>(find.byKey(const Key('pad-strip'))).painter
            as DrumPadStripPainter;

    await tester.tapAt(
      strip.topLeft + Offset(strip.width * 0.75, strip.height * 0.3),
    );
    await tester.pump(const Duration(milliseconds: 16));
    expect(c.read(playerProvider).isPlaying, isFalse);
    final first = painter();
    await tester.pump(const Duration(milliseconds: 16));
    final second = painter();
    // A fresh painter on the next frame with no playhead moving at all: the
    // strip repaints on the flash's clock, not on playback frames.
    expect(identical(first, second), isFalse);
    expect(second.struckMs.keys, [1]);
    await _teardown(tester, c);
  });

  testWidgets('feedback lives on the controller: the cascade receives no '
      'stroke state at all', (tester) async {
    final c = await _pumpPercussion(tester);
    final strip = tester.getRect(find.byKey(const Key('pad-strip')));
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
      strip.topLeft + Offset(strip.width * 0.75, strip.height * 0.3),
    );
    await tester.pump(const Duration(milliseconds: 16));
    // The pad flashed…
    expect(c.read(playerProvider).struckSurfacesMs.keys, [1]);
    // …and every input the cascade paints from is byte-identical across the
    // stroke — it is handed no stroke state at all, so it cannot react to
    // one. The same division the keyboard makes between its keys and the
    // waterfall.
    final after = cascade();
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

  testWidgets('hands / feet selection splits by the voice convention', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester);
    final notifier = c.read(playerProvider.notifier);
    final data = c.read(playerProvider);
    expect(data.hasHandsAndFeet, isTrue);

    // Hands (right): the kick disappears — no foot event survives the filter,
    // so the cascade draws no bar.
    notifier.setSelectedHands(Hand.right);
    final hands = c.read(playerProvider).visibleNotes;
    expect(hands, isNotEmpty);
    expect(hands.any((n) => kKickGmNumbers.contains(n.pitch)), isFalse);

    // Feet (left): only the kick remains — the hand lanes empty out.
    notifier.setSelectedHands(Hand.left);
    final feet = c.read(playerProvider).visibleNotes;
    expect(feet, isNotEmpty);
    expect(feet.every((n) => kKickGmNumbers.contains(n.pitch)), isTrue);
    await _teardown(tester, c);
  });

  testWidgets('the phone layout keeps the strip and inherits the phone mode '
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
      expect(find.byKey(const Key('pad-strip')), findsOneWidget);
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

  testWidgets('the setup modal swaps the range chooser for the kit layout '
      'and labels the selector hands / feet', (tester) async {
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
    // The selector reads hands / feet despite the single staff.
    expect(find.text('Hands'), findsOneWidget);
    expect(find.text('Feet'), findsOneWidget);
    expect(find.text('Right hand'), findsNothing);
    await _teardown(tester, c);
  });
}
