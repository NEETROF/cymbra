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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
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
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/score_font.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

// The percussion readiness seam, widget-tested through the real PlayerScreen
// subtree (change: add-drum-audio-channel): opening a percussion score swaps
// to the kit through the AWAITED load, playback stays visual-only until that
// completion resolves true, and leaving the player restores the piano. A
// notifier test cannot see this seam — the font-follows-score reaction lives
// in the player subtree's dedicated listener widget — so the screen is driven
// with fixed pumps (never pumpAndSettle, per the player-ticker convention).

const _entry = CatalogEntry(
  id: 'drums-1',
  title: 'Groove',
  composer: 'Tester',
  assetPath: 'assets/scores/groove.musicxml',
  level: PracticeLevel.beginner,
);

Future<ProviderContainer> _pump(
  WidgetTester tester, {
  required RecordingAudioService audio,
  bool kitInCatalog = true,
  bool dismissModal = true,
}) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  final container = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const [_entry]),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(
        FakeNotationEngine(document: sampleDrumDocument()),
      ),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(audio),
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
      soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
      soundFontImporterProvider.overrideWithValue(FakeSoundFontImporter()),
      privateSoundFontServiceProvider.overrideWithValue(
        FakePrivateSoundFontService(),
      ),
      // The BUNDLED kit ships in every build, so "no kit anywhere" has to be
      // forced rather than inherited from an empty server listing: narrowing
      // the catalog to the keyboard fonts is what models a build (or a device)
      // where no percussion font resolves.
      if (!kitInCatalog)
        pianoCatalogProvider.overrideWith((ref) => builtInPianos),
      // With the kit "in the catalog" the bundled-kit id resolves (models the
      // asset having landed / being served); without it, the honest no-kit
      // visual-only outcome is exercised.
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(
          downloadable: kitInCatalog
              ? [
                  fakeDownloadPiano(
                    id: defaultKitId,
                    label: 'FluidR3 Drums',
                    family: SoundFamily.percussion,
                  ),
                ]
              : const [],
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
  // Dismiss the pre-play setup modal (at desktop size) so the player
  // underneath is reachable.
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
  await tester.pump();
  container.dispose();
}

void main() {
  testWidgets(
    'opening a percussion score swaps to the kit through the awaited load and '
    'playback sounds drums once it resolves',
    (tester) async {
      final audio = RecordingAudioService();
      final c = await _pump(tester, audio: audio);

      // The kit went through the AWAITED swap (the readiness seam), resolved
      // from the remembered (bundled) kit id.
      expect(audio.awaitedLoads, ['/fake/soundfonts/$defaultKitId.sf2']);
      expect(c.read(scoreFontProvider), KitFontStatus.ready);

      // Free run: a drum score is offered Wait Mode since
      // add-drum-scoring, and the gate would freeze the playhead at the
      // first onset. What the audio routing does is the subject here.
      c.read(playerProvider.notifier)
        ..toggleWaitMode()
        ..setPlaying(true);
      for (var i = 0; i < 8; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }

      // Scheduled percussion notes sound as drum strokes, never as melodic
      // pitches.
      expect(audio.drumOns, isNotEmpty);
      expect(audio.noteOns, isEmpty);

      // Stopping silences everything (all-off covers both channels).
      c.read(playerProvider.notifier).setPlaying(false);
      expect(audio.allNotesOffCount, greaterThan(0));
      await _teardown(tester, c);
    },
  );

  testWidgets(
    'in Wait Mode the schedule does not strike the onset the player just '
    'played — one hit is one sound, never a flam',
    (tester) async {
      final audio = RecordingAudioService();
      final c = await _pump(tester, audio: audio);
      expect(c.read(scoreFontProvider), KitFontStatus.ready);
      final player = c.read(playerProvider.notifier);
      expect(c.read(playerProvider).waitMode, isTrue);

      player.startPlayback();
      for (var i = 0; i < 3; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      // Frozen on the opening onset — hi-hat in the hands, kick in the feet.
      expect(c.read(playerProvider).blocked, isTrue);
      final onset = c.read(playerProvider).onsetPitchesAt(0);
      expect(onset, isNotEmpty);

      audio.drumOns.clear();
      for (final gm in onset) {
        player.noteOn(gm);
      }
      // Each stroke sounded once, as the player made it.
      for (final gm in onset) {
        expect(audio.drumOns.where((d) => d.key == gm), hasLength(1));
      }

      // Releasing the gate walks the playhead through the written onset —
      // which the player has just played themselves. The bass drum sounding
      // "several times for one hit" was this second, scheduled attack landing
      // a frame behind the first.
      for (var i = 0; i < 3; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(c.read(playerProvider).blocked, isFalse);
      expect(c.read(playerProvider).elapsedMs, greaterThan(0));
      for (final gm in onset) {
        expect(
          audio.drumOns.where((d) => d.key == gm),
          hasLength(1),
          reason:
              'GM $gm was played by the player; the score must not repeat '
              'it — got ${audio.drumOns.map((d) => d.key).toList()}',
        );
      }
      await _teardown(tester, c);
    },
  );

  testWidgets(
    'playback is visual-only while the kit install is in flight, and sounds '
    'only after the completion resolves',
    (tester) async {
      final audio = RecordingAudioService()
        ..pendingAwaitedLoad = Completer<bool>();
      final c = await _pump(tester, audio: audio);

      // The swap is in flight: the player is NOT ready and playback advances
      // silently (visual-only) — nothing calls drumOn.
      expect(c.read(scoreFontProvider), KitFontStatus.loading);
      // Free run: a drum score is offered Wait Mode since
      // add-drum-scoring, and the gate would freeze the playhead at the
      // first onset. What the audio routing does is the subject here.
      c.read(playerProvider.notifier)
        ..toggleWaitMode()
        ..setPlaying(true);
      for (var i = 0; i < 8; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(c.read(playerProvider).elapsedMs, greaterThan(0));
      expect(audio.drumOns, isEmpty);
      expect(audio.noteOns, isEmpty);

      // The install lands: readiness resolves and later onsets sound.
      audio.pendingAwaitedLoad!.complete(true);
      for (var i = 0; i < 10; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(c.read(scoreFontProvider), KitFontStatus.ready);
      expect(audio.drumOns, isNotEmpty);
      expect(audio.noteOns, isEmpty);
      await _teardown(tester, c);
    },
  );

  testWidgets(
    'with no resolvable kit font the score stays honestly visual-only',
    (tester) async {
      final audio = RecordingAudioService();
      final c = await _pump(tester, audio: audio, kitInCatalog: false);

      // No kit anywhere (the bundled asset has not landed): the readiness gate
      // never resolves and nothing was swapped.
      expect(c.read(scoreFontProvider), KitFontStatus.unavailable);
      expect(audio.awaitedLoads, isEmpty);

      // Free run: a drum score is offered Wait Mode since
      // add-drum-scoring, and the gate would freeze the playhead at the
      // first onset. What the audio routing does is the subject here.
      c.read(playerProvider.notifier)
        ..toggleWaitMode()
        ..setPlaying(true);
      for (var i = 0; i < 8; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      // The playhead runs (the kit view's visual playback), silently.
      expect(c.read(playerProvider).elapsedMs, greaterThan(0));
      expect(audio.drumOns, isEmpty);
      expect(audio.noteOns, isEmpty);
      await _teardown(tester, c);
    },
  );

  testWidgets(
    'leaving the player restores the remembered piano through the existing '
    'load path',
    (tester) async {
      final audio = RecordingAudioService();
      final c = await _pump(tester, audio: audio);
      expect(c.read(scoreFontProvider), KitFontStatus.ready);

      // Leave the player: the dedicated listener reports the keyboard surface
      // and the controller restores the piano (the bundled default here).
      await tester.pumpWidget(const SizedBox());
      await tester.pump();
      await tester.pump();

      expect(c.read(scoreFontProvider), KitFontStatus.inactive);
      expect(
        audio.loadedSoundFonts,
        contains('/fake/soundfonts/$defaultPianoId.sf2'),
      );
      c.dispose();
    },
  );

  testWidgets('the setup modal labels the sound section as the kit picker on a '
      'percussion score', (tester) async {
    final audio = RecordingAudioService();
    final c = await _pump(tester, audio: audio, dismissModal: false);

    // The kit picker label replaces the piano one (fr/en/es/it keys; the
    // harness runs in English), and the offered font is the kit — the BUNDLED
    // one: the fake server listing carries the same stable id, which the
    // catalog deduplicates in the bundled entry's favour.
    expect(find.text('Drum kit sound'), findsOneWidget);
    expect(find.text('Piano sound'), findsNothing);
    expect(find.text('Standard Kit'), findsOneWidget);
    expect(find.text('Upright Piano KW'), findsNothing);
    await _teardown(tester, c);
  });

  testWidgets(
    'the kit picker shows the honest empty state when no kit font exists',
    (tester) async {
      final audio = RecordingAudioService();
      final c = await _pump(
        tester,
        audio: audio,
        kitInCatalog: false,
        dismissModal: false,
      );

      expect(find.text('Drum kit sound'), findsOneWidget);
      expect(find.text('No drum kit available'), findsOneWidget);
      await _teardown(tester, c);
    },
  );
}
