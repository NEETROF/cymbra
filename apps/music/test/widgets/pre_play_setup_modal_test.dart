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

ProviderContainer _container({ScoreDocument? document}) => ProviderContainer(
  overrides: [
    scoreCatalogProvider.overrideWithValue(const [_entry]),
    scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
    notationEngineProvider.overrideWithValue(
      FakeNotationEngine(document: document),
    ),
    midiServiceProvider.overrideWithValue(FakeMidiService()),
    scoreSourceProvider.overrideWithValue(FakeScoreSource()),
    audioServiceProvider.overrideWithValue(RecordingAudioService()),
  ],
);

/// Pumps the whole player for [_entry] and leaves the setup modal open — used to
/// exercise the modal's behaviour (the notation loads so hands are derived).
Future<ProviderContainer> _pumpWithModal(
  WidgetTester tester, {
  ScoreDocument? document,
  Size size = const Size(1400, 900),
}) async {
  await tester.binding.setSurfaceSize(size);
  final container = _container(document: document);
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
