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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/painters/drum_cascade_painter.dart';
import 'package:music/painters/partition_painter.dart';
import 'package:music/painters/staff_painter.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/src/rust/api/musicxml.dart' show ScoreDocument;
import 'package:music/state/drum_kit.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/player_preferences.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';

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

  testWidgets('Wait Mode stays NOT offered for percussion — the '
      'add-drum-scoring interim is untouched by the mode re-offer', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester);
    expect(find.text('Wait'), findsNothing);
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

  testWidgets('pads are display-only: a tap produces no note, no state', (
    tester,
  ) async {
    final c = await _pumpPercussion(tester);
    final before = c.read(playerProvider);
    await tester.tap(find.byKey(const Key('pad-strip')), warnIfMissed: false);
    await tester.pump(const Duration(milliseconds: 50));
    final after = c.read(playerProvider);
    expect(after.activeNotes, isEmpty);
    expect(after.activeNotes, before.activeNotes);
    // And nothing reached the synth either (change: add-drum-audio-channel):
    // the one-shot drum verbs exist for scheduled playback, but the pads stay
    // silent until add-drum-input-mapping wires input to them.
    final audio = c.read(audioServiceProvider) as RecordingAudioService;
    expect(audio.drumOns, isEmpty);
    expect(audio.noteOns, isEmpty);
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
