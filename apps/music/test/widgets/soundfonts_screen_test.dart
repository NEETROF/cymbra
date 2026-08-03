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
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/soundfonts_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/card_preview_notifier.dart' show CardPreviewScore;
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/sound_preview_sample.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

String _encode(List<PianoEntry> e) =>
    jsonEncode([for (final x in e) x.toJson()]);

ProviderContainer _container({
  FakePreferencesService? prefs,
  FakeSoundFontImporter? importer,
  FakePrivateSoundFontService? private,
  RecordingAudioService? audio,
  List<PianoEntry>? serverFonts,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      soundFontImporterProvider.overrideWithValue(
        importer ?? FakeSoundFontImporter(),
      ),
      privateSoundFontServiceProvider.overrideWithValue(
        private ?? FakePrivateSoundFontService(),
      ),
      soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(downloadable: serverFonts ?? const []),
      ),
      audioServiceProvider.overrideWithValue(audio ?? RecordingAudioService()),
      // The audition sample is parsed via the native notation engine, which is
      // absent in unit tests — override it with a trivial (empty) score so the
      // audition wiring runs without FFI.
      soundPreviewSampleProvider.overrideWith(
        (ref) async => const CardPreviewScore(
          notes: [],
          rests: [],
          songEndMs: 0,
          bpm: 120,
          keyFifths: 0,
          beats: 4,
          beatType: 4,
          measureStartMs: [],
          startMs: 0,
        ),
      ),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(WidgetTester tester, ProviderContainer c) async {
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(const SoundFontsScreen()),
    ),
  );
  await tester.pumpAndSettle();
}

/// A synced user import (has a remoteId), persisted so the screen lists it.
PianoEntry _synced(String id, String label) => PianoEntry(
  id: id,
  label: label,
  kind: PianoKind.user,
  source: '/copied/$id.sf2',
  remoteId: 'remote-$id',
);

void main() {
  testWidgets('empty state when there are no imported sounds', (tester) async {
    await _pump(tester, _container());
    expect(find.textContaining('No imported sounds'), findsOneWidget);
  });

  testWidgets('lists the user imports', (tester) async {
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encode([_synced('a', 'My Grand')]),
    });
    await _pump(tester, _container(prefs: prefs));
    expect(find.text('My Grand'), findsOneWidget);
  });

  testWidgets('add drawer: choose a file, name it, and add', (tester) async {
    final importer = FakeSoundFontImporter(
      picked: PickedSoundFont(
        bytes: Uint8List.fromList('RIFF____sfbk'.codeUnits),
        suggestedLabel: 'Picked Font',
      ),
    );
    final c = _container(importer: importer);
    await _pump(tester, c);

    // Open the add drawer.
    await tester.tap(find.byIcon(Icons.library_add_outlined));
    await tester.pumpAndSettle();
    // Choose the file (fake picker returns the canned pick).
    await tester.tap(find.byIcon(Icons.folder_open));
    await tester.pumpAndSettle();
    expect(importer.pickCalls, 1);
    // The name prefilled from the file; confirm add.
    expect(find.text('Picked Font'), findsWidgets);
    await tester.tap(find.widgetWithText(FilledButton, 'Add'));
    await tester.pumpAndSettle();

    // Saved with the label, and the registry now has it.
    expect(importer.saved.single.label, 'Picked Font');
    final registry = c.read(importedSoundFontsProvider).requireValue;
    expect(registry.map((e) => e.label), contains('Picked Font'));
  });

  testWidgets('remove deletes the import after confirmation', (tester) async {
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encode([_synced('a', 'My Grand')]),
    });
    final private = FakePrivateSoundFontService(
      library: const [
        RemoteSoundFont(id: 'remote-a', label: 'My Grand', sizeBytes: 1),
      ],
    );
    final c = _container(prefs: prefs, private: private);
    await _pump(tester, c);

    await tester.tap(find.byIcon(Icons.delete_outline));
    await tester.pumpAndSettle();
    // Confirm in the dialog.
    await tester.tap(find.widgetWithText(FilledButton, 'Remove'));
    await tester.pumpAndSettle();

    expect(c.read(importedSoundFontsProvider).requireValue, isEmpty);
    expect(private.deleted, contains('remote-a'));
  });

  testWidgets('rename updates the label via the edit drawer', (tester) async {
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encode([_synced('a', 'Old Name')]),
    });
    final c = _container(prefs: prefs);
    await _pump(tester, c);

    await tester.tap(find.byIcon(Icons.edit_outlined));
    await tester.pumpAndSettle();
    // The drawer's name field (the screen also has a search field).
    await tester.enterText(
      find.descendant(
        of: find.byType(Drawer),
        matching: find.byType(TextField),
      ),
      'New Name',
    );
    await tester.tap(find.widgetWithText(FilledButton, 'Rename'));
    await tester.pumpAndSettle();

    final registry = c.read(importedSoundFontsProvider).requireValue;
    expect(registry.single.label, 'New Name');
  });

  testWidgets('a proposed font shows a status tag and hides propose', (
    tester,
  ) async {
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encode([_synced('a', 'My Grand')]),
    });
    // The server reports the font's proposal is pending review.
    final private = FakePrivateSoundFontService(
      library: const [
        RemoteSoundFont(
          id: 'remote-a',
          label: 'My Grand',
          sizeBytes: 1,
          proposalStatus: 'pending',
        ),
      ],
    );
    await _pump(tester, _container(prefs: prefs, private: private));

    // A status tag is shown, and the propose action is gone (already submitted).
    expect(find.text('Pending review'), findsOneWidget);
    expect(find.byIcon(Icons.publish_outlined), findsNothing);
  });

  testWidgets('tapping a sound loads its font to audition it', (tester) async {
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encode([_synced('a', 'My Grand')]),
    });
    final audio = RecordingAudioService();
    final c = _container(prefs: prefs, audio: audio);
    await _pump(tester, c);

    final loadsBefore = audio.loadedSoundFonts.length;
    // Tap the card body to audition (not an action icon).
    await tester.tap(find.text('My Grand'));
    await tester.pump();
    await tester.pump();

    // The sound's font was resolved and swapped into the synth.
    expect(audio.loadedSoundFonts.length, greaterThan(loadsBefore));
    // The card now offers a stop control; tap it so the looping ticker doesn't
    // outlive the test.
    await tester.tap(find.byIcon(Icons.stop_circle));
    await tester.pump();
  });
}
