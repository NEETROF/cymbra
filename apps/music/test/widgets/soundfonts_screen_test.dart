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
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_preview_service.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/card_preview_notifier.dart' show CardPreviewScore;
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/player_data.dart' show TimedNote;
import 'package:music/state/sound_preview_sample.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

String _encode(List<PianoEntry> e) =>
    jsonEncode([for (final x in e) x.toJson()]);

/// A curator-rewards seam returning a fixed reward-shop list, so a catalog font can
/// be made a **locked** reward (cost > 0, redeemable, unowned) for the audition tests.
class _ShopFake implements CuratorRewardsService {
  const _ShopFake(this.items);
  final List<RewardShopItemView> items;

  @override
  Future<List<RewardShopItemView>> listShop() async => items;

  @override
  Future<CuratorRewardsView> getRewards() async => const CuratorRewardsView(
    lifetimePoints: 0,
    spendableBalance: 0,
    level: 0,
    levelFloor: 0,
    nextLevelAt: 50,
    totalRatings: 0,
    coverageContribution: 0,
    alignmentRate: 0,
    badges: [],
    recent: [],
  );

  @override
  Future<RedeemResultView> redeem(String rewardKey) async =>
      const RedeemResultView(owned: true, newBalance: 0);
}

RewardShopItemView _reward(
  String key, {
  required bool owned,
  int cost = 50,
  bool redeemable = true,
}) => RewardShopItemView(
  key: key,
  label: key,
  instrument: 'piano',
  license: 'CC0-1.0',
  attribution: '',
  pointCost: cost,
  redeemable: redeemable,
  owned: owned,
);

ProviderContainer _container({
  FakePreferencesService? prefs,
  FakeSoundFontImporter? importer,
  FakePrivateSoundFontService? private,
  RecordingAudioService? audio,
  List<PianoEntry>? serverFonts,
  CardPreviewScore? sample,
  FakeSoundFontSource? source,
  FakeSoundFontPreviewService? preview,
  List<RewardShopItemView> shop = const [],
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
      soundFontSourceProvider.overrideWithValue(
        source ?? FakeSoundFontSource(),
      ),
      soundFontPreviewServiceProvider.overrideWithValue(
        preview ?? FakeSoundFontPreviewService(),
      ),
      curatorRewardsServiceProvider.overrideWithValue(_ShopFake(shop)),
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(downloadable: serverFonts ?? const []),
      ),
      audioServiceProvider.overrideWithValue(audio ?? RecordingAudioService()),
      // The audition sample is parsed via the native notation engine, which is
      // absent in unit tests — override it with a trivial (empty) score so the
      // audition wiring runs without FFI.
      soundPreviewSampleProvider.overrideWith(
        (ref) async =>
            sample ??
            const CardPreviewScore(
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

  testWidgets(
    'every font carries a family badge — kits and pianos alike, unfiltered '
    '(change: add-drum-audio-channel)',
    (tester) async {
      // The hub is not scoped to a score: both families are LISTED (never
      // filtered) and the family is made visible as a badge instead.
      final prefs = FakePreferencesService({
        ImportedSoundFonts.prefsKey: _encode([
          _synced('a', 'My Grand'),
          const PianoEntry(
            id: 'k',
            label: 'My Kit',
            kind: PianoKind.user,
            source: '/copied/k.sf2',
            family: SoundFamily.percussion,
            remoteId: 'remote-k',
          ),
        ]),
      });
      await _pump(
        tester,
        _container(
          prefs: prefs,
          serverFonts: [
            fakeDownloadPiano(
              id: 'street-kit',
              label: 'Street Kit',
              family: SoundFamily.percussion,
            ),
          ],
        ),
      );

      expect(find.text('My Grand'), findsOneWidget);
      expect(find.text('My Kit'), findsOneWidget);
      expect(find.text('Street Kit'), findsOneWidget);
      // Localized family badges: the three kits (the bundled Standard Kit,
      // the import and the server one) read "Drums"; the bundled piano and
      // the imported grand read "Piano".
      expect(find.text('Standard Kit'), findsOneWidget);
      expect(find.text('Drums'), findsNWidgets(3));
      expect(find.text('Piano'), findsNWidgets(2));
    },
  );

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

  testWidgets('propose dialog: licence picker + attest, submit marks pending', (
    tester,
  ) async {
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

    // Open the propose dialog from the card action.
    await tester.tap(find.byIcon(Icons.publish_outlined));
    await tester.pumpAndSettle();
    // The licence combobox shows a description for the default (CC0) selection.
    expect(find.textContaining('Public domain'), findsOneWidget);

    // Check the mandatory attestation, then submit.
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Propose'));
    await tester.pumpAndSettle();

    // The proposal went out with the chosen licence, and the card is now pending.
    expect(private.proposed.single.license, 'CC0-1.0');
    final e = c
        .read(importedSoundFontsProvider)
        .requireValue
        .firstWhere((x) => x.id == 'a');
    expect(e.proposalStatus, 'pending');
  });

  testWidgets('accepted and rejected proposals show their status tag', (
    tester,
  ) async {
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encode([
        _synced('a', 'Accepted One'),
        _synced('b', 'Rejected One'),
      ]),
    });
    final private = FakePrivateSoundFontService(
      library: const [
        RemoteSoundFont(
          id: 'remote-a',
          label: 'Accepted One',
          sizeBytes: 1,
          proposalStatus: 'accepted',
        ),
        RemoteSoundFont(
          id: 'remote-b',
          label: 'Rejected One',
          sizeBytes: 1,
          proposalStatus: 'rejected',
          rejectionReason: 'poor samples',
        ),
      ],
    );
    await _pump(tester, _container(prefs: prefs, private: private));

    expect(find.text('Accepted'), findsOneWidget);
    expect(find.text('Rejected'), findsOneWidget);
    // The rejected card shows the moderator's motive (change:
    // add-soundfont-uploader-attribution)…
    expect(find.text('Rejected: poor samples'), findsOneWidget);
    // …and only the REJECTED font may be re-proposed (accepted may not).
    expect(find.byIcon(Icons.publish_outlined), findsOneWidget);
  });

  testWidgets('re-proposing a rejected font requires a justification', (
    tester,
  ) async {
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encode([_synced('b', 'Rejected One')]),
    });
    final private = FakePrivateSoundFontService(
      library: const [
        RemoteSoundFont(
          id: 'remote-b',
          label: 'Rejected One',
          sizeBytes: 1,
          proposalStatus: 'rejected',
          rejectionReason: 'bad licence',
        ),
      ],
    );
    final c = _container(prefs: prefs, private: private);
    await _pump(tester, c);

    // Re-open the propose dialog from the rejected card.
    await tester.tap(find.byIcon(Icons.publish_outlined));
    await tester.pumpAndSettle();
    // The moderator's rejection reason shows on the card AND in the dialog.
    expect(find.text('Rejected: bad licence'), findsNWidgets(2));

    // Attesting alone is NOT enough — submit stays disabled without a
    // justification. (The dialog content scrolls; bring the checkbox on screen.)
    await tester.ensureVisible(find.byType(CheckboxListTile));
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pumpAndSettle();
    FilledButton submit() => tester.widget<FilledButton>(
      find.widgetWithText(FilledButton, 'Propose'),
    );
    expect(submit().onPressed, isNull);

    // Filling the justification enables submit; the note rides the proposal.
    await tester.ensureVisible(find.byType(TextField).last);
    await tester.enterText(
      find.byType(TextField).last,
      'now relicensed under CC0',
    );
    await tester.pumpAndSettle();
    expect(submit().onPressed, isNotNull);
    await tester.tap(find.widgetWithText(FilledButton, 'Propose'));
    await tester.pumpAndSettle();

    expect(
      private.proposed.single.resubmissionNote,
      'now relicensed under CC0',
    );
    // The card flips back to pending and the stale reason is gone.
    final e = c
        .read(importedSoundFontsProvider)
        .requireValue
        .firstWhere((x) => x.id == 'b');
    expect(e.proposalStatus, 'pending');
    expect(e.proposalRejectionReason, isNull);
  });

  testWidgets('a catalog font shows its contributor credit when present', (
    tester,
  ) async {
    await _pump(
      tester,
      _container(
        serverFonts: [
          fakeDownloadPiano(
            id: 'community-grand',
            label: 'Community Grand',
            attribution: 'Sample Author',
            contributorCredit: 'alice',
          ),
          fakeDownloadPiano(id: 'plain-grand', label: 'Plain Grand'),
        ],
      ),
    );
    // The opt-in credit shows alongside (not replacing) the licence attribution…
    expect(find.text('Proposed by @alice'), findsOneWidget);
    expect(find.text('CC-BY 3.0 · Sample Author'), findsOneWidget);
    // …and a font without a credit shows none.
    expect(find.textContaining('Proposed by'), findsOneWidget);
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

  testWidgets('auditioning a sound plays the sample, then stops', (
    tester,
  ) async {
    final audio = RecordingAudioService();
    const sample = CardPreviewScore(
      notes: [TimedNote(pitch: 60, startMs: 0, durationMs: 400)],
      rests: [],
      songEndMs: 400,
      bpm: 120,
      keyFifths: 0,
      beats: 4,
      beatType: 4,
      measureStartMs: [0],
      startMs: 0,
    );
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encode([_synced('a', 'Mine')]),
    });
    final c = _container(prefs: prefs, audio: audio, sample: sample);
    await _pump(tester, c);

    // Tap the card → its font loads and the sample starts playing.
    await tester.tap(find.text('Mine'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 60)); // seed the playhead
    await tester.pump(const Duration(milliseconds: 60)); // cross the note onset
    expect(audio.loadedSoundFonts, isNotEmpty);
    expect(audio.noteOns.map((n) => n.pitch), contains(60));

    // Tapping again stops the audition (flushes voices).
    await tester.tap(find.text('Mine'));
    await tester.pump();
    expect(audio.allNotesOffCount, greaterThan(0));
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

  // --- Locked fonts audition via the preview clip (change:
  //     add-soundfont-entitlement-previews) ---

  testWidgets('a locked reward font auditions via its preview clip, never '
      'downloading the font', (tester) async {
    final source = FakeSoundFontSource();
    final preview = FakeSoundFontPreviewService(available: {'grand'});
    final c = _container(
      // The catalog reports a preview exists, so the control is a play button.
      serverFonts: [
        fakeDownloadPiano(id: 'grand', label: 'Grand Piano', hasPreview: true),
      ],
      shop: [_reward('grand', owned: false)],
      source: source,
      preview: preview,
    );
    await _pump(tester, c);

    // Tap the locked catalog card to audition.
    await tester.tap(find.text('Grand Piano'));
    await tester.pump();

    // Auditioned via the server preview clip…
    expect(preview.auditioned, contains('grand'));
    // …and the locked font's raw `.sf2` bytes were NEVER resolved/downloaded.
    // (The selected default piano is resolved on entry to capture a restore path;
    // that's unrelated — what matters is that 'grand' is never resolved.)
    expect(source.resolved.map((e) => e.id), isNot(contains('grand')));
    // The card now shows a stop control (playing).
    expect(find.byIcon(Icons.stop_circle), findsOneWidget);
  });

  testWidgets('a locked font with no preview greys the play control up front', (
    tester,
  ) async {
    final source = FakeSoundFontSource();
    final preview = FakeSoundFontPreviewService(available: const {});
    final c = _container(
      // The catalog reports no preview (hasPreview defaults to false).
      serverFonts: [fakeDownloadPiano(id: 'grand', label: 'Grand Piano')],
      shop: [_reward('grand', owned: false)],
      source: source,
      preview: preview,
    );
    await _pump(tester, c);

    // Greyed UP FRONT — no tap needed (the catalog said there is no preview).
    expect(find.byIcon(Icons.music_off_outlined), findsOneWidget);

    // The control is disabled: a tap does nothing (no audition, no download).
    await tester.tap(find.text('Grand Piano'));
    await tester.pump();
    expect(preview.auditioned, isEmpty);
    expect(source.resolved.map((e) => e.id), isNot(contains('grand')));
  });

  // A costed font the shop does not offer yet is STILL costed: the server refuses
  // its bytes on the cost alone, so it must read as locked and audition via its
  // clip. It previously read as free, and tapping it tried to download the font.
  testWidgets('a costed font that is not redeemable yet is locked and shows '
      'coming soon instead of an unlock button', (tester) async {
    final source = FakeSoundFontSource();
    final preview = FakeSoundFontPreviewService(available: {'grand'});
    final c = _container(
      serverFonts: [
        fakeDownloadPiano(id: 'grand', label: 'Grand Piano', hasPreview: true),
      ],
      shop: [_reward('grand', owned: false, redeemable: false)],
      source: source,
      preview: preview,
    );
    await _pump(tester, c);

    // Costed → the price shows, but there is no unlock action yet.
    expect(find.byKey(const Key('soundfont-unlock')), findsNothing);

    // And it is auditioned via the clip — the raw font is never downloaded.
    await tester.tap(find.text('Grand Piano'));
    await tester.pump();
    expect(preview.auditioned, contains('grand'));
    expect(source.resolved.map((e) => e.id), isNot(contains('grand')));
  });

  testWidgets('an owned reward font still auditions via the local synth', (
    tester,
  ) async {
    final source = FakeSoundFontSource();
    final preview = FakeSoundFontPreviewService(available: {'grand'});
    final c = _container(
      serverFonts: [fakeDownloadPiano(id: 'grand', label: 'Grand Piano')],
      // Owned → not locked → the full local path is used.
      shop: [_reward('grand', owned: true)],
      source: source,
      preview: preview,
    );
    await _pump(tester, c);

    await tester.tap(find.text('Grand Piano'));
    await tester.pump();
    await tester.pump();

    // Resolved locally (owned is entitled to the bytes); the preview clip path is
    // not used for an owned font.
    expect(source.resolved.map((e) => e.id), contains('grand'));
    expect(preview.auditioned, isEmpty);
    // Stop the looping ticker so it doesn't outlive the test.
    await tester.tap(find.byIcon(Icons.stop_circle));
    await tester.pump();
  });
}
