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

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/screens/pre_play_setup_modal.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/drum_input_mapping.dart';
import 'package:music/state/drum_input_mapping_notifier.dart';
import 'package:music/state/drum_kit.dart';
import 'package:music/state/player_preferences.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

ProviderContainer _container({ScoreDocument? document, String? midiPort}) =>
    ProviderContainer(
      overrides: [
        scoreCatalogProvider.overrideWithValue(const [_entry]),
        scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
        notationEngineProvider.overrideWithValue(
          FakeNotationEngine(document: document),
        ),
        midiServiceProvider.overrideWithValue(
          FakeMidiService(
            ports: midiPort == null ? const [] : [midiPort],
            connected: midiPort,
          ),
        ),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
      ],
    );

/// Pumps the whole player for [_entry] and leaves the setup modal open — used to
/// exercise the modal's behaviour (the notation loads so hands are derived).
Future<ProviderContainer> _pumpWithModal(
  WidgetTester tester, {
  ScoreDocument? document,
  String? midiPort,
  Size size = const Size(1400, 900),
}) async {
  await tester.binding.setSurfaceSize(size);
  final container = _container(document: document, midiPort: midiPort);
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
  return container;
}

/// Shows the setup modal over a bare host (not the whole player), so a layout
/// check isn't perturbed by the player chrome behind it.
Future<ProviderContainer> _pumpModalIsolated(
  WidgetTester tester, {
  required Size size,
}) async {
  await tester.binding.setSurfaceSize(size);
  final container = _container();
  container.read(selectedScoreProvider.notifier).select(_entry);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const _ModalHost(), locale: const Locale('en')),
    ),
  );
  for (var i = 0; i < 12; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
  return container;
}

/// A bare host that opens the setup modal after the first frame.
class _ModalHost extends StatefulWidget {
  const _ModalHost();
  @override
  State<_ModalHost> createState() => _ModalHostState();
}

class _ModalHostState extends State<_ModalHost> {
  bool _shown = false;
  @override
  Widget build(BuildContext context) {
    if (!_shown) {
      _shown = true;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) showPrePlaySetup(context);
      });
    }
    return const Scaffold(body: SizedBox.shrink());
  }
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

Future<void> _pumpFrames(WidgetTester tester) async {
  for (var i = 0; i < 6; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  testWidgets('opens on a loaded score, shows info, applies hand on Validate', (
    tester,
  ) async {
    final container = await _pumpWithModal(tester); // grand-staff (multi-staff)

    // Score info is shown.
    expect(find.text('Sample Piece'), findsOneWidget);
    expect(find.text('Tester'), findsOneWidget);
    // Multi-staff → the hand chooser is offered.
    expect(find.text('Play with'), findsOneWidget);
    expect(find.text('Left'), findsOneWidget);

    // Change the hand, then Validate.
    await tester.tap(find.text('Left'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, 'Play'));
    await _pumpFrames(tester);

    // Applied and closed.
    expect(container.read(playerProvider).selectedHands, Hand.left);
    expect(find.text('Play with'), findsNothing);
    await _teardown(tester, container);
  });

  // Task 4.3: the keyboard control this change deliberately does NOT touch,
  // asserted rather than inspected (change: add-practice-focus-controls).
  testWidgets('a keyboard score keeps the hand selector and is offered no '
      'per-piece focus control', (tester) async {
    final container = await _pumpWithModal(tester);
    expect(find.text('Play with'), findsOneWidget);
    expect(find.text('Left'), findsOneWidget);
    expect(find.text('Right'), findsOneWidget);
    expect(find.text('Both'), findsOneWidget);
    // Nothing percussion-shaped reaches a keyboard score.
    final data = container.read(playerProvider);
    expect(data.hasDrumPiecesToFocus, isFalse);
    expect(data.kitPieceIds, isEmpty);
    expect(data.isFocusRestrictedRun, isFalse);
    expect(find.byKey(const Key('drum-focus-all')), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('one MIDI-input door per score: the pass on percussion, the '
      'monitor on a keyboard', (tester) async {
    // Change: add-drum-input-calibration (designs D8 and D11). The mapping
    // states "this pad is the snare", and the seam that applies it is the
    // identity on anything that is not a percussion score: offering the pass on
    // a keyboard score would promise a calibration that provably does nothing.
    //
    // The monitor is not offered beside it on a percussion score either — it
    // moved one level down, into the calibration surface. Two entries here read
    // as alternatives, and only one of them repairs anything. On a keyboard
    // score, where there is no pass, it stays: it is then the only answer to
    // "nothing is arriving at all".
    final keyboard = await _pumpWithModal(tester);
    expect(keyboard.read(playerProvider).isPercussion, isFalse);
    expect(find.byKey(const Key('open-midi-monitor')), findsOneWidget);
    expect(find.byKey(const Key('open-drum-calibration')), findsNothing);
    await _teardown(tester, keyboard);

    final drums = await _pumpWithModal(tester, document: sampleDrumDocument());
    expect(drums.read(playerProvider).isPercussion, isTrue);
    expect(find.byKey(const Key('open-drum-calibration')), findsOneWidget);
    expect(find.byKey(const Key('open-midi-monitor')), findsNothing);
    await _teardown(tester, drums);
  });

  testWidgets('the settings name what this score has yet to teach the kit, and '
      'say so when nothing is left', (tester) async {
    // Change: add-drum-input-calibration (design D10). The question before
    // playing is not "is my kit calibrated" but "will everything this groove
    // asks me to hit be understood" — so the answer is listed under the action
    // that fixes it, in the score's own terms.
    final container = await _pumpWithModal(
      tester,
      document: sampleDrumDocument(),
      midiPort: 'Drum kit',
    );
    // The fixture writes a kick, a snare and a closed hi-hat; nothing is
    // learned yet, so all three are named.
    expect(container.read(playerProvider).calibrationTargets, [
      kKickPieceId,
      'kitPieceSnare',
      'kitPieceHiHat',
    ]);
    expect(find.byKey(const Key('calibration-missing')), findsOneWidget);
    expect(find.byKey(const Key('calibration-complete')), findsNothing);
    expect(find.textContaining('Snare'), findsWidgets);

    // Learn two of the three: the line shrinks to what is actually left.
    final store = container.read(drumInputMappingStoreProvider.notifier)
      ..setPiece('Drum kit', kKickPieceId, 12)
      ..setPiece('Drum kit', 'kitPieceSnare', 31);
    await _pumpFrames(tester);
    expect(
      tester.widget<Text>(find.byKey(const Key('calibration-missing'))).data,
      contains('Hi-hat'),
    );
    expect(
      tester.widget<Text>(find.byKey(const Key('calibration-missing'))).data,
      isNot(contains('Snare')),
    );

    // Learn the last one and the line turns into the positive statement.
    store.setPiece('Drum kit', 'kitPieceHiHat', 22);
    await _pumpFrames(tester);
    expect(find.byKey(const Key('calibration-missing')), findsNothing);
    expect(find.byKey(const Key('calibration-complete')), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('a piece the kit was said not to have is named as not awaited '
      '(design D13)', (tester) async {
    // The safety net for the answer that turns the gate off: "this kit has
    // none" must be readable BEFORE playing, or a player who tapped through the
    // pass would find Wait Mode quietly waiting for nothing.
    final container = await _pumpWithModal(
      tester,
      document: sampleDrumDocument(),
      midiPort: 'Drum kit',
    );
    container
        .read(drumInputMappingStoreProvider.notifier)
        .save(
          'Drum kit',
          DrumInputMapping(
            const {kKickPieceId: 12, 'kitPieceSnare': 31},
            absent: const {'kitPieceHiHat'},
          ),
        );
    await _pumpFrames(tester);

    // Nothing left to learn, so no "not calibrated" line…
    expect(find.byKey(const Key('calibration-missing')), findsNothing);
    // …but the hi-hat is not silently dropped either.
    final absent = tester.widget<Text>(
      find.byKey(const Key('calibration-absent')),
    );
    expect(absent.data, contains('Hi-hat'));
    // And the flat "everything is calibrated" claim gives way to it.
    expect(find.byKey(const Key('calibration-complete')), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('a piece the kit does not have is greyed out and disabled in the '
      'pieces-practised list (design D13)', (tester) async {
    // The run already neither awaits nor judges it, so no choice on this row
    // could change anything: checking a box cannot put a pad back on the
    // instrument. Stating that beats offering a dead control.
    final container = await _pumpWithModal(
      tester,
      document: sampleDrumDocument(),
      midiPort: 'Drum kit',
    );
    container
        .read(drumInputMappingStoreProvider.notifier)
        .save(
          'Drum kit',
          DrumInputMapping(
            const {'kitPieceSnare': 31},
            absent: const {'kitPieceHiHat'},
          ),
        );
    await _pumpFrames(tester);

    Checkbox boxOf(String pieceId) => tester.widget<Checkbox>(
      find.descendant(
        of: find.byKey(Key('drum-focus-$pieceId')),
        matching: find.byType(Checkbox),
      ),
    );
    // Unchecked and dead — and the reason is on the row, in the words the pass
    // asked the question with.
    expect(boxOf('kitPieceHiHat').onChanged, isNull);
    expect(boxOf('kitPieceHiHat').value, isFalse);
    expect(
      tester
          .widget<TextButton>(
            find.descendant(
              of: find.byKey(const Key('drum-focus-kitPieceHiHat')),
              matching: find.byType(TextButton),
            ),
          )
          .onPressed,
      isNull,
    );
    // Every other piece keeps its controls.
    expect(boxOf('kitPieceSnare').onChanged, isNotNull);
    expect(boxOf('kitPieceSnare').value, isTrue);
    await _teardown(tester, container);
  });

  testWidgets('close (X) keeps the current settings', (tester) async {
    final container = await _pumpWithModal(tester);
    expect(container.read(playerProvider).selectedHands, Hand.both);

    // Change the hand in the draft, then dismiss with the close button.
    await tester.tap(find.text('Left'));
    await tester.pump();
    await tester.tap(find.byIcon(Icons.close));
    await _pumpFrames(tester);

    // Not applied — still Both — and the modal is closed.
    expect(container.read(playerProvider).selectedHands, Hand.both);
    expect(find.text('Play with'), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('offers the three reading-aid levels, applies on Validate', (
    tester,
  ) async {
    final container = await _pumpWithModal(tester);

    // Preselected from the persisted level — names the note by default.
    expect(container.read(playerProvider).readingAid, NoteReadingAid.name);
    expect(find.text('Note names'), findsOneWidget);
    expect(find.text('Off'), findsOneWidget);
    expect(find.text('Note name'), findsOneWidget);
    expect(find.text('Name + rhythm'), findsOneWidget);

    await tester.tap(find.text('Name + rhythm'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, 'Play'));
    await _pumpFrames(tester);

    expect(
      container.read(playerProvider).readingAid,
      NoteReadingAid.nameAndRhythm,
    );
    await _teardown(tester, container);
  });

  testWidgets('close (X) keeps the current reading-aid level', (tester) async {
    final container = await _pumpWithModal(tester);

    await tester.tap(find.text('Off'));
    await tester.pump();
    await tester.tap(find.byIcon(Icons.close));
    await _pumpFrames(tester);

    expect(container.read(playerProvider).readingAid, NoteReadingAid.name);
    await _teardown(tester, container);
  });

  testWidgets('score size chooser applies to the shared prefs on Validate', (
    tester,
  ) async {
    final container = await _pumpWithModal(tester);
    // Nothing stored yet — the chooser preselects the form-factor default
    // (medium on this tablet-class surface) without persisting it.
    expect(container.read(playerPreferencesProvider).scoreSize, isNull);

    // Pick Large, then Validate.
    await tester.ensureVisible(find.text('Large'));
    await tester.tap(find.text('Large'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, 'Play'));
    await _pumpFrames(tester);

    expect(
      container.read(playerPreferencesProvider).scoreSize,
      ScoreSize.large,
    );
    await _teardown(tester, container);
  });

  testWidgets('score theme chooser applies to the shared prefs on Validate', (
    tester,
  ) async {
    final container = await _pumpWithModal(tester);
    expect(
      container.read(playerPreferencesProvider).notationTheme,
      NotationTheme.dark,
    );

    await tester.ensureVisible(find.text('Paper'));
    await tester.tap(find.text('Paper'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, 'Play'));
    await _pumpFrames(tester);

    expect(
      container.read(playerPreferencesProvider).notationTheme,
      NotationTheme.paper,
    );
    await _teardown(tester, container);
  });

  testWidgets('single-staff piece offers no hand chooser', (tester) async {
    final container = await _pumpWithModal(
      tester,
      document: sampleTieSlurDocument(), // staves: 1
    );

    // The modal is shown (Validate button present) but with no hand chooser.
    expect(find.widgetWithText(FilledButton, 'Play'), findsOneWidget);
    expect(find.text('Play with'), findsNothing);
    expect(find.text('Left'), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('phone landscape lays out without overflow and stays scrollable', (
    tester,
  ) async {
    // isPhoneLayout needs a mobile platform + a small MediaQuery. MediaQuery.size
    // comes from the view (not setSurfaceSize), so set the view too (dpr 1 ⇒
    // logical == physical). Reset before the body ends — the framework asserts
    // debug vars are clear before tearDown, so addTearDown would be too late.
    debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
    tester.view.physicalSize = const Size(812, 375);
    tester.view.devicePixelRatio = 1.0;
    try {
      // A short landscape phone viewport: a render overflow would throw here.
      final container = await _pumpModalIsolated(
        tester,
        size: const Size(812, 375),
      );

      // The many controls don't fit a short landscape at once, so the body
      // scrolls as one unit with the Play button pinned. Every control is in the
      // tree and reachable (scrolled into view), and the sound picker is full
      // width (so its dropdown menu isn't cramped).
      expect(find.byType(SingleChildScrollView), findsWidgets);
      expect(find.widgetWithText(FilledButton, 'Play'), findsOneWidget);
      for (final label in const [
        'Piano sound',
        'Left', // hands (multi-staff)
        'Metronome',
        'Tempo',
        'MIDI device',
        'Keyboard size',
        'Score size',
      ]) {
        await tester.ensureVisible(find.text(label).first);
        expect(find.text(label), findsWidgets, reason: '$label unreachable');
      }
      await _teardown(tester, container);
    } finally {
      debugDefaultTargetPlatformOverride = null;
      tester.view.resetPhysicalSize();
      tester.view.resetDevicePixelRatio();
    }
  });
}
