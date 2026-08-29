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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/reward_shop_notifier.dart';
import 'package:music/state/selected_piano.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

/// A curator-rewards seam returning a fixed reward-shop list, so the selection
/// gate for locked reward fonts is exercisable.
class _ShopFake implements CuratorRewardsService {
  _ShopFake(this.items);

  /// Mutable: a lapse re-locks a font the account owned a moment ago.
  List<RewardShopItemView> items;

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
  required FakePreferencesService prefs,
  required RecordingAudioService audio,
  FakeSoundFontSource? source,
  FakeSoundFontImporter? importer,
  List<PianoEntry>? serverFonts,
  List<RewardShopItemView> shop = const [],
  _ShopFake? shopFake,
}) {
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      audioServiceProvider.overrideWithValue(audio),
      curatorRewardsServiceProvider.overrideWithValue(
        shopFake ?? _ShopFake(shop),
      ),
      soundFontSourceProvider.overrideWithValue(
        source ?? FakeSoundFontSource(),
      ),
      soundFontImporterProvider.overrideWithValue(
        importer ?? FakeSoundFontImporter(),
      ),
      // Private library seam: empty (offline-equivalent) so the registry stays
      // local and never hits the network during selection-restore tests.
      privateSoundFontServiceProvider.overrideWithValue(
        FakePrivateSoundFontService(),
      ),
      // The server's downloadable pianos (YDP/Salamander live here now, not in
      // the built-in list); defaults to the two CC-BY grands.
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(
          downloadable:
              serverFonts ??
              [
                fakeDownloadPiano(id: 'ydp-grand', label: 'YDP Grand Piano'),
                fakeDownloadPiano(
                  id: 'salamander-grand',
                  label: 'Salamander Grand Piano',
                ),
              ],
        ),
      ),
    ],
  );
  addTearDown(container.dispose);
  // Keep the notifiers alive for the test (they are keepAlive in production).
  container.listen(selectedPianoProvider, (_, _) {}, fireImmediately: true);
  container.listen(pianoCatalogProvider, (_, _) {}, fireImmediately: true);
  // Keep the reward shop alive so the selection gate sees its loaded value.
  container.listen(rewardShopProvider, (_, _) {}, fireImmediately: true);
  return container;
}

void main() {
  test('seeds the bundled default synchronously before restore', () async {
    final container = _container(
      prefs: FakePreferencesService(),
      audio: RecordingAudioService(),
    );
    // Read before pumping the queue: the async restore has not run yet.
    expect(container.read(selectedPianoProvider), defaultPianoId);
    // Let the async restore finish before the container is torn down.
    await pumpEventQueue();
  });

  test('selecting a piano loads its SoundFont and persists the id', () async {
    final audio = RecordingAudioService();
    final prefs = FakePreferencesService();
    final source = FakeSoundFontSource();
    final container = _container(prefs: prefs, audio: audio, source: source);
    await pumpEventQueue();

    await container.read(selectedPianoProvider.notifier).select('ydp-grand');

    expect(container.read(selectedPianoProvider), 'ydp-grand');
    // The right bytes were resolved and handed to the engine.
    expect(source.resolved.map((e) => e.id), contains('ydp-grand'));
    expect(audio.loadedSoundFonts, contains('/fake/soundfonts/ydp-grand.sf2'));
    // The choice is persisted so it survives a relaunch.
    expect(prefs.store[SelectedPiano.prefsKey], 'ydp-grand');
  });

  test('a persisted selection is restored and loaded on launch', () async {
    final audio = RecordingAudioService();
    final container = _container(
      prefs: FakePreferencesService({SelectedPiano.prefsKey: 'ydp-grand'}),
      audio: audio,
    );
    await pumpEventQueue();

    expect(container.read(selectedPianoProvider), 'ydp-grand');
    expect(audio.loadedSoundFonts, contains('/fake/soundfonts/ydp-grand.sf2'));
  });

  test(
    'the default is not reloaded on launch (init already loaded it)',
    () async {
      // A restored *default* selection needs no swap — audio_init loaded it.
      final audio = RecordingAudioService();
      final container = _container(
        prefs: FakePreferencesService({SelectedPiano.prefsKey: defaultPianoId}),
        audio: audio,
      );
      await pumpEventQueue();

      expect(container.read(selectedPianoProvider), defaultPianoId);
      expect(audio.loadedSoundFonts, isEmpty);
    },
  );

  test(
    'an unknown persisted id falls back to the default and re-persists',
    () async {
      final prefs = FakePreferencesService({
        SelectedPiano.prefsKey: 'ghost-piano',
      });
      final container = _container(
        prefs: prefs,
        audio: RecordingAudioService(),
      );
      await pumpEventQueue();

      expect(container.read(selectedPianoProvider), defaultPianoId);
      expect(prefs.store[SelectedPiano.prefsKey], defaultPianoId);
    },
  );

  test('a failing source falls back to the default without crashing', () async {
    final audio = RecordingAudioService();
    final prefs = FakePreferencesService({SelectedPiano.prefsKey: 'ydp-grand'});
    // The persisted download piano cannot be fetched.
    final source = FakeSoundFontSource(failIds: {'ydp-grand'});
    final container = _container(prefs: prefs, audio: audio, source: source);
    await pumpEventQueue();

    expect(container.read(selectedPianoProvider), defaultPianoId);
    expect(prefs.store[SelectedPiano.prefsKey], defaultPianoId);
    // It fell back to loading the bundled default's bytes.
    expect(
      audio.loadedSoundFonts,
      contains('/fake/soundfonts/$defaultPianoId.sf2'),
    );
  });

  test('selecting while audio is a no-op still persists the choice', () async {
    // Model "audio unavailable" with a source that resolves fine but audio that
    // records (never throws): the choice must still persist and be applied.
    final audio = RecordingAudioService(failInit: true);
    final prefs = FakePreferencesService();
    final container = _container(prefs: prefs, audio: audio);
    await pumpEventQueue();

    await container
        .read(selectedPianoProvider.notifier)
        .select('salamander-grand');

    expect(container.read(selectedPianoProvider), 'salamander-grand');
    expect(prefs.store[SelectedPiano.prefsKey], 'salamander-grand');
  });

  test(
    'removing the selected imported piano falls back to the default',
    () async {
      final audio = RecordingAudioService();
      final prefs = FakePreferencesService();
      final imported = fakeUserPiano(id: 'user-x');
      final importer = FakeSoundFontImporter(next: imported);
      final container = _container(
        prefs: prefs,
        audio: audio,
        importer: importer,
      );
      await pumpEventQueue();

      // Import, select it, then remove it.
      await container
          .read(importedSoundFontsProvider.notifier)
          .importSoundFont();
      await container.read(selectedPianoProvider.notifier).select('user-x');
      expect(container.read(selectedPianoProvider), 'user-x');

      await container
          .read(importedSoundFontsProvider.notifier)
          .remove('user-x');
      await pumpEventQueue();

      // The catalog listener drops the vanished selection back to the default.
      expect(container.read(selectedPianoProvider), defaultPianoId);
      expect(prefs.store[SelectedPiano.prefsKey], defaultPianoId);
      // The copied file was deleted.
      expect(importer.deleted.map((e) => e.id), contains('user-x'));
    },
  );

  test('a locked reward font is not selectable until it is owned', () async {
    final container = _container(
      prefs: FakePreferencesService(),
      audio: RecordingAudioService(),
      serverFonts: [fakeDownloadPiano(id: 'reward-grand', label: 'Reward')],
      shop: [_reward('reward-grand', owned: false)],
    );
    // Load the reward shop so the selection gate can see it.
    await container.read(rewardShopProvider.future);
    await pumpEventQueue();

    // Locked (costed, unowned) → selecting it is refused; stays on the default.
    await container.read(selectedPianoProvider.notifier).select('reward-grand');
    expect(container.read(selectedPianoProvider), defaultPianoId);
  });

  // A costed font that the shop does not offer yet ("coming later") is still
  // COSTED, so it must stay locked: the server's entitlement gate keys on the cost
  // alone and would refuse its bytes with a 404. Consulting `redeemable` here used
  // to make it selectable, which then failed on the download.
  test(
    'a costed font that is not redeemable yet is still not selectable',
    () async {
      final container = _container(
        prefs: FakePreferencesService(),
        audio: RecordingAudioService(),
        serverFonts: [fakeDownloadPiano(id: 'soon-grand', label: 'Soon')],
        shop: [_reward('soon-grand', owned: false, redeemable: false)],
      );
      await container.read(rewardShopProvider.future);
      await pumpEventQueue();

      await container.read(selectedPianoProvider.notifier).select('soon-grand');
      expect(container.read(selectedPianoProvider), defaultPianoId);
    },
  );

  test(
    'a font that becomes locked again is dropped as the active instrument',
    () async {
      // The lapse case: premium (or a redemption) let the font be selected, the
      // plan ended, the withdrawal took its file back and the shop offers it
      // again. Keeping it selected would claim a sound the app cannot load.
      final shop = _ShopFake([_reward('reward-grand', owned: true)]);
      final prefs = FakePreferencesService();
      final container = _container(
        prefs: prefs,
        audio: RecordingAudioService(),
        serverFonts: [fakeDownloadPiano(id: 'reward-grand', label: 'Reward')],
        shopFake: shop,
      );
      await container.read(rewardShopProvider.future);
      await pumpEventQueue();
      await container
          .read(selectedPianoProvider.notifier)
          .select('reward-grand');
      expect(container.read(selectedPianoProvider), 'reward-grand');

      // Re-locked: the shop now says costed and unowned.
      shop.items = [_reward('reward-grand', owned: false)];
      container.invalidate(rewardShopProvider);
      await container.read(rewardShopProvider.future);
      await pumpEventQueue();

      expect(container.read(selectedPianoProvider), defaultPianoId);
      expect(prefs.store[SelectedPiano.prefsKey], defaultPianoId);
    },
  );

  test('an owned reward font is selectable', () async {
    final container = _container(
      prefs: FakePreferencesService(),
      audio: RecordingAudioService(),
      serverFonts: [fakeDownloadPiano(id: 'reward-grand', label: 'Reward')],
      shop: [_reward('reward-grand', owned: true)],
    );
    await container.read(rewardShopProvider.future);
    await pumpEventQueue();

    await container.read(selectedPianoProvider.notifier).select('reward-grand');
    expect(container.read(selectedPianoProvider), 'reward-grand');
  });
}
