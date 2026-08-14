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
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/coaching_notifier.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/widgets/coach_layer.dart';
import 'package:music/widgets/coach_mark.dart';

import '../support/fakes.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

/// The player under the same coaching wiring as the real app: the coach layer is
/// stacked above the navigator, so it can spotlight controls inside the pre-play
/// dialog.
class _CoachedApp extends StatelessWidget {
  const _CoachedApp();

  @override
  Widget build(BuildContext context) => MaterialApp(
    localizationsDelegates: AppLocalizations.localizationsDelegates,
    supportedLocales: AppLocalizations.supportedLocales,
    locale: const Locale('en'),
    builder: (context, child) => Stack(children: [?child, const CoachLayer()]),
    home: const PlayerScreen(),
  );
}

Future<ProviderContainer> _pumpPlayer(
  WidgetTester tester, {
  required FakePreferencesService prefs,
}) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
    ],
  );
  container.read(selectedScoreProvider.notifier).select(_entry);
  await tester.pumpWidget(
    UncontrolledProviderScope(container: container, child: const _CoachedApp()),
  );
  await _frames(tester, 20);
  return container;
}

/// Unmounts the tree and disposes the container inside the test body: the
/// player's MIDI-status timer is cancelled on provider dispose, and the test
/// binding checks for pending timers before tear-downs run.
Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

/// The player runs a ticker, so the tree never settles — pump frames instead.
Future<void> _frames(WidgetTester tester, [int count = 10]) async {
  for (var i = 0; i < count; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

/// The rect the spotlight is currently cut out over, if any.
Rect? _hole(WidgetTester tester) => tester
    .widgetList<CoachMarkOverlay>(find.byType(CoachMarkOverlay))
    .firstOrNull
    ?.hole;

void main() {
  testWidgets('the first player visit walks the key controls in place', (
    tester,
  ) async {
    final prefs = FakePreferencesService();
    final container = await _pumpPlayer(tester, prefs: prefs);
    final registry = container.read(coachTargetRegistryProvider);

    // Step 1 — the piano sound, spotlighting the real control.
    expect(find.text('Choose your piano sound'), findsOneWidget);
    expect(find.text('1 / 4'), findsOneWidget);
    expect(_hole(tester), isNotNull);
    expect(
      _hole(tester)!.overlaps(registry.rectFor(CoachAnchor.pianoSound)!),
      isTrue,
    );

    // Step 2 — the connected MIDI instrument.
    await tester.tap(find.byKey(const Key('coach-mark-next')));
    await _frames(tester);
    expect(find.text('Your connected instrument'), findsOneWidget);
    expect(
      _hole(tester)!.overlaps(registry.rectFor(CoachAnchor.midiDevice)!),
      isTrue,
    );

    // Step 3 — the hand selection, still inside the setup dialog.
    await tester.tap(find.byKey(const Key('coach-mark-next')));
    await _frames(tester);
    expect(find.text('Right hand, left hand, or both'), findsOneWidget);
    expect(_hole(tester)!.overlaps(registry.rectFor(CoachAnchor.hands)!), true);

    // Step 4 — the transport measure rewind. Its control lives on the player
    // screen BEHIND the dialog, so advancing to it applies the setup drafts
    // and closes the dialog, revealing the spotlighted button.
    await tester.tap(find.byKey(const Key('coach-mark-next')));
    await _frames(tester);
    expect(find.text('Rewind and pick a passage'), findsOneWidget);
    expect(find.text('4 / 4'), findsOneWidget);
    expect(
      find.byKey(const Key('pre-play-primary')),
      findsNothing,
      reason: 'the setup dialog must close to reveal the transport bar',
    );
    expect(
      _hole(tester)!.overlaps(registry.rectFor(CoachAnchor.measureRewind)!),
      isTrue,
    );

    await tester.tap(find.byKey(const Key('coach-mark-next')));
    await _frames(tester);
    expect(find.byType(CoachMarkOverlay), findsNothing);
    expect(prefs.store[CoachHint.playerTour.prefsKey], 'true');
    await _teardown(tester, container);
  });

  testWidgets('it is skippable and never blocks playing', (tester) async {
    final prefs = FakePreferencesService();
    final container = await _pumpPlayer(tester, prefs: prefs);

    expect(find.byType(CoachMarkOverlay), findsOneWidget);
    await tester.tap(find.byKey(const Key('coach-mark-skip')));
    await _frames(tester);

    expect(find.byType(CoachMarkOverlay), findsNothing);
    // The setup surface is untouched and the user can start playing right away.
    expect(find.widgetWithText(FilledButton, 'Play'), findsOneWidget);
    await tester.tap(find.widgetWithText(FilledButton, 'Play'));
    await _frames(tester);
    expect(container.read(coachingProvider).tourRunning, isFalse);
    await _teardown(tester, container);
  });

  testWidgets('a returning player is not walked again', (tester) async {
    final container = await _pumpPlayer(
      tester,
      prefs: FakePreferencesService({CoachHint.playerTour.prefsKey: 'true'}),
    );
    expect(find.byType(CoachMarkOverlay), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('a replay armed from help runs on the next player visit', (
    tester,
  ) async {
    final prefs = FakePreferencesService({
      CoachHint.playerTour.prefsKey: 'true',
    });
    final container = await _pumpPlayer(tester, prefs: prefs);
    expect(find.byType(CoachMarkOverlay), findsNothing);

    // What the help surface does…
    container.read(coachingProvider.notifier).armPlayerTourReplay();
    // …takes effect when the setup surface is next opened.
    container.read(coachingProvider.notifier).startPlayerTour();
    await _frames(tester);

    expect(find.text('Choose your piano sound'), findsOneWidget);
    await _teardown(tester, container);
  });
}
