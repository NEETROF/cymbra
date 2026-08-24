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

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/library_screen.dart';
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/drums_access.dart';
import 'package:music/state/instrument_context.dart';
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

/// Flippable visibility: models the `drums.enabled` snapshot resolving (or a
/// campaign opening/closing) live, which a fixed value override cannot.
final _visible = StateProvider((ref) => false);

String _stored(AppInstrument context, {bool offered = true}) =>
    jsonEncode({'context': context.name, 'choiceOffered': offered});

ProviderContainer _container({
  FakePreferencesService? prefs,
  bool drumsVisible = false,
  List<CatalogEntry>? catalog,
  ScoreDocument? document,
}) {
  final c = ProviderContainer(
    overrides: [
      // The REAL scoreCatalogProvider stays in force unless a test narrows it:
      // the bundled-drums listing gate is part of what these tests pin.
      if (catalog != null) scoreCatalogProvider.overrideWithValue(catalog),
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      drumsEnabledProvider.overrideWith((ref) => ref.watch(_visible)),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(
        FakeNotationEngine(document: document),
      ),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
    ],
  );
  c.read(_visible.notifier).state = drumsVisible;
  return c;
}

Future<void> _pump(
  WidgetTester tester,
  ProviderContainer container, {
  Size size = const Size(1400, 900),
}) async {
  await tester.binding.setSurfaceSize(size);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const LibraryScreen(), locale: const Locale('en')),
    ),
  );
  await _pumpFrames(tester, 8);
}

/// Bounded frames: the player runs a Ticker, so pumpAndSettle never settles.
Future<void> _pumpFrames(WidgetTester tester, [int n = 8]) async {
  for (var i = 0; i < n; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

final _switcher = find.byKey(const Key('instrument-switcher'));
final _modal = find.byKey(const Key('instrument-choice-modal'));

void main() {
  testWidgets('while drums are invisible the home is untouched: no switcher, '
      'no modal, no drum scores listed', (tester) async {
    final container = _container();
    await _pump(tester, container);

    expect(_switcher, findsNothing);
    expect(_modal, findsNothing);
    // The bundled grooves ship in the binary but appear in no listing.
    expect(find.text('Rock basique'), findsNothing);
    expect(find.text('Ode to Joy (theme)'), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('visibility brings the switcher and the one-time choice; '
      'choosing drums seeds the drum home', (tester) async {
    final prefs = FakePreferencesService();
    final container = _container(prefs: prefs, drumsVisible: true);
    await _pump(tester, container);

    expect(_switcher, findsOneWidget);
    expect(_modal, findsOneWidget);

    await tester.tap(find.byKey(const Key('instrument-choice-drums')));
    await _pumpFrames(tester);
    expect(_modal, findsNothing);
    // The home now seeds drums: grooves by level, the piano repertoire away.
    expect(find.text('Rock basique'), findsOneWidget);
    expect(find.text('Premiers pas'), findsOneWidget);
    expect(find.text('Ode to Joy (theme)'), findsNothing);
    // Both the choice and the offered marker were persisted.
    expect(prefs.store[InstrumentContext.prefsKey], contains('drums'));
    expect(prefs.store[InstrumentContext.prefsKey], contains('true'));
    await _teardown(tester, container);
  });

  testWidgets('the choice is offered at most once per installation — a '
      'relaunch after a dismissal stays quiet', (tester) async {
    final prefs = FakePreferencesService();
    final container = _container(prefs: prefs, drumsVisible: true);
    await _pump(tester, container);
    expect(_modal, findsOneWidget);
    // Answering keyboard (≈ dismissing) still consumes the one offer.
    await tester.tap(find.byKey(const Key('instrument-choice-keyboard')));
    await _pumpFrames(tester);
    expect(_modal, findsNothing);
    expect(find.text('Ode to Joy (theme)'), findsOneWidget);
    await _teardown(tester, container);

    // Relaunch over the same device store: no re-offer, switcher still there.
    final second = _container(prefs: prefs, drumsVisible: true);
    await _pump(tester, second);
    expect(_modal, findsNothing);
    expect(_switcher, findsOneWidget);
    await _teardown(tester, second);
  });

  testWidgets('a visibility flip landing mid-play defers the offer to the '
      'return home — never a dialog over the player', (tester) async {
    final container = _container();
    await _pump(tester, container);

    // Open a score (drums still invisible), reach the player.
    await tester.tap(find.text('Ode to Joy (theme)'));
    await _pumpFrames(tester, 12);
    await tester.tap(find.widgetWithText(FilledButton, 'Play'));
    await _pumpFrames(tester);
    expect(find.byType(PlayerScreen), findsOneWidget);

    // The flag snapshot resolves NOW, mid-play: no dialog appears here.
    container.read(_visible.notifier).state = true;
    await _pumpFrames(tester);
    expect(_modal, findsNothing);

    // Returning to the home is where the offer lands.
    await tester.tap(find.byTooltip('Back to library'));
    await _pumpFrames(tester);
    expect(find.byType(LibraryScreen), findsOneWidget);
    expect(_modal, findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('the switcher re-seeds the home in both directions', (
    tester,
  ) async {
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: _stored(AppInstrument.drums),
    });
    final container = _container(prefs: prefs, drumsVisible: true);
    await _pump(tester, container);
    expect(find.text('Rock basique'), findsOneWidget);

    // → keyboard: the piano repertoire returns, grooves leave.
    await tester.tap(
      find.descendant(of: _switcher, matching: find.text('Piano')),
    );
    await _pumpFrames(tester);
    expect(find.text('Ode to Joy (theme)'), findsOneWidget);
    expect(find.text('Rock basique'), findsNothing);

    // → drums again.
    await tester.tap(
      find.descendant(of: _switcher, matching: find.text('Drums')),
    );
    await _pumpFrames(tester);
    expect(find.text('Rock basique'), findsOneWidget);
    expect(find.text('Ode to Joy (theme)'), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('losing visibility falls back to the keyboard home without '
      'rewriting the choice; drums reapply when it returns', (tester) async {
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: _stored(AppInstrument.drums),
    });
    final container = _container(prefs: prefs, drumsVisible: true);
    await _pump(tester, container);
    expect(find.text('Rock basique'), findsOneWidget);

    // The campaign closes: today's home, no switcher — and no rewrite.
    container.read(_visible.notifier).state = false;
    await _pumpFrames(tester);
    expect(_switcher, findsNothing);
    expect(find.text('Ode to Joy (theme)'), findsOneWidget);
    expect(find.text('Rock basique'), findsNothing);
    expect(prefs.store[InstrumentContext.prefsKey], contains('drums'));

    // It reopens: the drum home is back with no user action, and no re-offer
    // (the marker was already set).
    container.read(_visible.notifier).state = true;
    await _pumpFrames(tester);
    expect(find.text('Rock basique'), findsOneWidget);
    expect(_modal, findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('an empty drums context invites — never a bare empty screen — '
      'and the invitation switches back', (tester) async {
    // A catalog with no percussion entries at all (e.g. a build without the
    // bundled grooves): the drums context has nothing to show.
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: _stored(AppInstrument.drums),
    });
    final container = _container(
      prefs: prefs,
      drumsVisible: true,
      catalog: const [
        CatalogEntry(
          id: 'p1',
          title: 'Piano Piece',
          composer: 'X',
          assetPath: 'assets/scores/beginner/p1.musicxml',
          level: PracticeLevel.beginner,
        ),
      ],
    );
    await _pump(tester, container);

    expect(find.text('No drum scores here yet'), findsOneWidget);
    await tester.tap(find.byKey(const Key('drums-empty-switch')));
    await _pumpFrames(tester);
    expect(find.text('Piano Piece'), findsOneWidget);
    expect(prefs.store[InstrumentContext.prefsKey], contains('keyboard'));
    await _teardown(tester, container);
  });

  testWidgets('phone and tablet: the switcher, the modal and the drum home '
      'hold on small viewports', (tester) async {
    for (final size in const [Size(390, 844), Size(820, 1180)]) {
      final container = _container(drumsVisible: true);
      await _pump(tester, container, size: size);
      // The modal renders within the viewport (an overflow fails the frame).
      expect(_modal, findsOneWidget);
      await tester.tap(find.byKey(const Key('instrument-choice-drums')));
      await _pumpFrames(tester);
      expect(_switcher, findsOneWidget);
      expect(find.text('Rock basique'), findsOneWidget);
      await _teardown(tester, container);
    }
  });

  testWidgets('a bundled drum score opens from the home into the cascade '
      'player with the pad strip', (tester) async {
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: _stored(AppInstrument.drums),
    });
    final container = _container(
      prefs: prefs,
      drumsVisible: true,
      document: sampleDrumDocument(),
    );
    await _pump(tester, container);

    await tester.tap(find.text('Premiers pas'));
    await _pumpFrames(tester, 12);
    // Through the pre-play setup (hands/feet, inverted kit) to the player.
    await tester.tap(find.widgetWithText(FilledButton, 'Play'));
    await _pumpFrames(tester);

    expect(find.byType(PlayerScreen), findsOneWidget);
    expect(find.byKey(const Key('pad-strip')), findsOneWidget);
    expect(find.byKey(const Key('onscreen-keyboard')), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('nothing in the player path writes the context: a keyboard '
      'score opened directly (the deep-link shape) under the drums context', (
    tester,
  ) async {
    final prefs = FakePreferencesService({
      InstrumentContext.prefsKey: _stored(AppInstrument.drums),
    });
    final container = _container(prefs: prefs, drumsVisible: true);
    container.read(instrumentContextProvider); // hydrate the stored record
    // Straight to the player, bypassing the home — an incoming link must not
    // reconfigure the app.
    const entry = CatalogEntry(
      id: 'kb',
      title: 'Keyboard Piece',
      composer: 'X',
      assetPath: 'assets/scores/beginner/kb.musicxml',
      level: PracticeLevel.beginner,
    );
    container.read(selectedScoreProvider.notifier).select(entry);
    await tester.binding.setSurfaceSize(const Size(1400, 900));
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(const PlayerScreen(), locale: const Locale('en')),
      ),
    );
    await _pumpFrames(tester, 12);
    final play = find.widgetWithText(FilledButton, 'Play');
    if (play.evaluate().isNotEmpty) {
      await tester.tap(play);
      await _pumpFrames(tester);
    }
    final before = Map.of(prefs.store);
    await _pumpFrames(tester); // let it play a stretch

    expect(
      container.read(instrumentContextProvider).context,
      AppInstrument.drums,
    );
    expect(prefs.store, before); // not a single write from the player path
    expect(prefs.store[InstrumentContext.prefsKey], contains('drums'));
    await _teardown(tester, container);
  });
}
