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
import 'package:music/screens/player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/selected_piano.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

const _entry = CatalogEntry(
  id: 'sample',
  title: 'Sample Piece',
  composer: 'Tester',
  assetPath: 'assets/scores/beginner/sample.musicxml',
  level: PracticeLevel.beginner,
);

String _encodeRegistry(List<PianoEntry> entries) =>
    jsonEncode([for (final e in entries) e.toJson()]);

Future<ProviderContainer> _pumpPlayer(
  WidgetTester tester, {
  FakePreferencesService? prefs,
  FakeSoundFontImporter? importer,
  List<PianoEntry>? serverFonts,
}) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
      soundFontImporterProvider.overrideWithValue(
        importer ?? FakeSoundFontImporter(),
      ),
      // The server's downloadable grands (now server-listed, not hardcoded).
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(
          downloadable:
              serverFonts ??
              [
                fakeDownloadPiano(
                  id: 'ydp-grand',
                  label: 'YDP Grand Piano',
                  attribution: 'Roberto / Zenph Studios',
                ),
                fakeDownloadPiano(
                  id: 'salamander-grand',
                  label: 'Salamander Grand Piano',
                  attribution: 'Alexander Holm',
                ),
              ],
        ),
      ),
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
  final validate = find.widgetWithText(FilledButton, 'Play');
  if (validate.evaluate().isNotEmpty) {
    await tester.tap(validate);
    for (var i = 0; i < 6; i++) {
      await tester.pump(const Duration(milliseconds: 50));
    }
  }
  return container;
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

Future<void> _openPianoCategory(WidgetTester tester) async {
  await tester.tap(find.byIcon(Icons.tune));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 300));
  await tester.tap(find.text('Piano sound'));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}

void main() {
  testWidgets('lists the built-in pianos and the active one is default', (
    tester,
  ) async {
    final container = await _pumpPlayer(tester);
    await _openPianoCategory(tester);

    expect(find.text('Upright Piano KW'), findsOneWidget);
    expect(find.text('YDP Grand Piano'), findsOneWidget);
    expect(find.text('Salamander Grand Piano'), findsOneWidget);
    // CC-BY attribution is surfaced in-app.
    expect(find.textContaining('Alexander Holm'), findsOneWidget);
    // The default is the active selection.
    expect(container.read(selectedPianoProvider), defaultPianoId);
    // The import action is offered.
    expect(find.text('Add SoundFont…'), findsOneWidget);

    await _teardown(tester, container);
  });

  testWidgets('tapping a piano changes and persists the selection', (
    tester,
  ) async {
    final prefs = FakePreferencesService();
    final container = await _pumpPlayer(tester, prefs: prefs);
    await _openPianoCategory(tester);

    await tester.tap(find.text('YDP Grand Piano'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(container.read(selectedPianoProvider), 'ydp-grand');
    expect(prefs.store[SelectedPiano.prefsKey], 'ydp-grand');

    await _teardown(tester, container);
  });

  testWidgets('an imported piano is listed with a remove affordance', (
    tester,
  ) async {
    final imported = fakeUserPiano(id: 'mine', label: 'My Imported Piano');
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encodeRegistry([imported]),
    });
    final container = await _pumpPlayer(
      tester,
      prefs: prefs,
      importer: FakeSoundFontImporter(),
    );
    await _openPianoCategory(tester);

    expect(find.text('My Imported Piano'), findsOneWidget);
    // Imported group header and the remove (delete) button are present.
    expect(find.text('Imported'), findsOneWidget);
    expect(find.byIcon(Icons.delete_outline), findsOneWidget);

    // Removing it drops it from the list.
    await tester.tap(find.byIcon(Icons.delete_outline));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('My Imported Piano'), findsNothing);
    expect(container.read(importedSoundFontsProvider).requireValue, isEmpty);

    await _teardown(tester, container);
  });
}
