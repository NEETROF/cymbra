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
import 'package:music/services/preferences_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/widgets/sound_selector_field.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

// The instrument-sound combobox (SoundSelectorField), used at every play entry
// point (the pre-play popup and the rating deck). Driven in isolation over the
// fake catalog/import seams — no player, no backend.

String _encodeRegistry(List<PianoEntry> entries) =>
    jsonEncode([for (final e in entries) e.toJson()]);

ProviderContainer _container({
  FakePreferencesService? prefs,
  FakeSoundFontImporter? importer,
  List<PianoEntry>? serverFonts,
}) => ProviderContainer(
  overrides: [
    preferencesServiceProvider.overrideWithValue(
      prefs ?? FakePreferencesService(),
    ),
    soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
    soundFontImporterProvider.overrideWithValue(
      importer ?? FakeSoundFontImporter(),
    ),
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

Future<void> _pumpField(
  WidgetTester tester,
  ProviderContainer container, {
  String value = defaultPianoId,
  required ValueChanged<String> onChanged,
}) async {
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(
        Scaffold(
          body: Center(
            child: SoundSelectorField(value: value, onChanged: onChanged),
          ),
        ),
      ),
    ),
  );
  // Let the async catalog sources (server list + imports) resolve.
  for (var i = 0; i < 6; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

Future<void> _openDropdown(WidgetTester tester) async {
  await tester.tap(find.byType(DropdownButton<String>));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('lists the catalog sounds and offers import', (tester) async {
    final container = _container();
    addTearDown(container.dispose);
    await _pumpField(tester, container, onChanged: (_) {});

    await _openDropdown(tester);

    // The menu lists the built-in default + the server grands + the import item.
    expect(find.text('Upright Piano KW'), findsWidgets);
    expect(find.text('YDP Grand Piano'), findsOneWidget);
    expect(find.text('Salamander Grand Piano'), findsOneWidget);
    expect(find.text('Add SoundFont…'), findsOneWidget);
  });

  testWidgets('picking a sound reports its id via onChanged', (tester) async {
    final container = _container();
    addTearDown(container.dispose);
    String? picked;
    await _pumpField(tester, container, onChanged: (id) => picked = id);

    await _openDropdown(tester);
    await tester.tap(find.text('YDP Grand Piano').last);
    await tester.pumpAndSettle();

    expect(picked, 'ydp-grand');
  });

  testWidgets('surfaces the selected sound CC-BY attribution', (tester) async {
    final container = _container();
    addTearDown(container.dispose);
    await _pumpField(
      tester,
      container,
      value: 'salamander-grand',
      onChanged: (_) {},
    );

    // The required CC-BY credit is shown under the combobox for the selection.
    expect(find.textContaining('Alexander Holm'), findsOneWidget);
  });

  testWidgets('the add item runs the import and selects the imported font', (
    tester,
  ) async {
    final imported = fakeUserPiano(id: 'mine', label: 'My Imported Piano');
    final container = _container(
      importer: FakeSoundFontImporter(next: imported),
    );
    addTearDown(container.dispose);
    String? picked;
    await _pumpField(tester, container, onChanged: (id) => picked = id);

    await _openDropdown(tester);
    await tester.tap(find.text('Add SoundFont…').last);
    await tester.pumpAndSettle();

    expect(picked, 'mine');
  });

  testWidgets('imported sounds are removable from the manage sheet', (
    tester,
  ) async {
    final imported = fakeUserPiano(id: 'mine', label: 'My Imported Piano');
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encodeRegistry([imported]),
    });
    final container = _container(prefs: prefs);
    addTearDown(container.dispose);
    await _pumpField(tester, container, onChanged: (_) {});

    // The manage affordance appears once there is an import; it opens a sheet
    // that lists the import with a delete button.
    await tester.tap(find.byIcon(Icons.tune));
    await tester.pumpAndSettle();
    expect(find.text('My Imported Piano'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.delete_outline));
    await tester.pumpAndSettle();

    expect(container.read(importedSoundFontsProvider).requireValue, isEmpty);
  });
}
