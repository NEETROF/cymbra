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
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/services/curator_rewards_service.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/reward_shop_notifier.dart';
import 'package:music/widgets/sound_selector_field.dart';

import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

// The instrument-sound combobox (SoundSelectorField), used at every play entry
// point (the pre-play popup and the rating deck). Driven in isolation over the
// fake catalog/import seams — no player, no backend.

String _encodeRegistry(List<PianoEntry> entries) =>
    jsonEncode([for (final e in entries) e.toJson()]);

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

ProviderContainer _container({
  FakePreferencesService? prefs,
  FakeSoundFontImporter? importer,
  List<PianoEntry>? serverFonts,
  List<RewardShopItemView> shop = const [],

  /// Replaces the whole resolved catalog — the way to model a build where a
  /// family has no font at all, now that the kit is bundled in every build.
  List<PianoEntry>? catalogOverride,
}) => ProviderContainer(
  overrides: [
    if (catalogOverride != null)
      pianoCatalogProvider.overrideWith((ref) => catalogOverride),
    preferencesServiceProvider.overrideWithValue(
      prefs ?? FakePreferencesService(),
    ),
    curatorRewardsServiceProvider.overrideWithValue(_ShopFake(shop)),
    soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
    soundFontImporterProvider.overrideWithValue(
      importer ?? FakeSoundFontImporter(),
    ),
    // Private library seam: empty (offline-equivalent) so the registry stays
    // local — the widget tests exercise import/remove, not server sync.
    privateSoundFontServiceProvider.overrideWithValue(
      FakePrivateSoundFontService(),
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
  SoundFamily family = SoundFamily.keyboard,
  required ValueChanged<String> onChanged,
}) async {
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(
        Scaffold(
          body: Center(
            child: SoundSelectorField(
              value: value,
              family: family,
              onChanged: onChanged,
            ),
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
  testWidgets('lists the catalog sounds', (tester) async {
    final container = _container();
    addTearDown(container.dispose);
    await _pumpField(tester, container, onChanged: (_) {});

    await _openDropdown(tester);

    // The menu lists the built-in default + the server grands. (Importing and
    // managing sounds now lives on the dedicated management screen, not here.)
    expect(find.text('Upright Piano KW'), findsWidgets);
    expect(find.text('YDP Grand Piano'), findsOneWidget);
    expect(find.text('Salamander Grand Piano'), findsOneWidget);
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

  testWidgets('a user-imported sound is listed and selectable', (tester) async {
    final imported = fakeUserPiano(id: 'mine', label: 'My Imported Piano');
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encodeRegistry([imported]),
    });
    final container = _container(prefs: prefs);
    addTearDown(container.dispose);
    String? picked;
    await _pumpField(tester, container, onChanged: (id) => picked = id);

    await _openDropdown(tester);
    await tester.tap(find.text('My Imported Piano').last);
    await tester.pumpAndSettle();

    expect(picked, 'mine');
  });

  // Family scoping (change: add-drum-audio-channel): the picker offers only
  // the LOADED SCORE's family — a percussion score lists kits, a keyboard
  // score pianos — and an empty family shows the honest no-kit state.
  group('family scoping', () {
    testWidgets('a percussion score lists kit fonts only', (tester) async {
      final container = _container(
        serverFonts: [
          fakeDownloadPiano(id: 'ydp-grand', label: 'YDP Grand Piano'),
          fakeDownloadPiano(
            id: 'street-kit',
            label: 'Street Kit',
            family: SoundFamily.percussion,
          ),
        ],
      );
      addTearDown(container.dispose);
      await _pumpField(
        tester,
        container,
        value: 'street-kit',
        family: SoundFamily.percussion,
        onChanged: (_) {},
      );

      await _openDropdown(tester);

      expect(find.text('Street Kit'), findsWidgets);
      expect(find.text('YDP Grand Piano'), findsNothing);
      expect(find.text('Upright Piano KW'), findsNothing);
    });

    testWidgets('a keyboard score never lists kit fonts', (tester) async {
      final container = _container(
        serverFonts: [
          fakeDownloadPiano(id: 'ydp-grand', label: 'YDP Grand Piano'),
          fakeDownloadPiano(
            id: 'street-kit',
            label: 'Street Kit',
            family: SoundFamily.percussion,
          ),
        ],
      );
      addTearDown(container.dispose);
      await _pumpField(tester, container, onChanged: (_) {});

      await _openDropdown(tester);

      expect(find.text('Upright Piano KW'), findsWidgets);
      expect(find.text('YDP Grand Piano'), findsOneWidget);
      expect(find.text('Street Kit'), findsNothing);
    });

    testWidgets('an empty family shows the no-kit hint, not a crash', (
      tester,
    ) async {
      // The bundled kit ships in every build, so an EMPTY percussion family
      // has to be forced: narrowing the catalog to the keyboard fonts models
      // a build (or a device) where no kit resolves.
      final container = _container(
        serverFonts: const [],
        catalogOverride: builtInPianos,
      );
      addTearDown(container.dispose);
      await _pumpField(
        tester,
        container,
        family: SoundFamily.percussion,
        onChanged: (_) {},
      );

      expect(find.text('No drum kit available'), findsOneWidget);
    });
  });

  testWidgets('a locked reward font is shown locked and cannot be picked', (
    tester,
  ) async {
    // After a lapse the font is costed and unowned again: `select` refuses it,
    // so the list must say so instead of swallowing the tap in silence.
    final container = _container(
      serverFonts: [fakeDownloadPiano(id: 'reward-grand', label: 'Reward')],
      shop: const [
        RewardShopItemView(
          key: 'reward-grand',
          label: 'Reward',
          instrument: 'piano',
          license: 'CC0-1.0',
          attribution: '',
          pointCost: 50,
          redeemable: true,
          owned: false,
        ),
      ],
    );
    addTearDown(container.dispose);
    await container.read(rewardShopProvider.future);

    String? picked;
    await _pumpField(tester, container, onChanged: (id) => picked = id);
    await tester.tap(find.byType(DropdownButton<String>));
    await tester.pumpAndSettle();

    // Listed, but marked with a lock and disabled — a tap must not read as a
    // choice the app then silently drops.
    expect(find.byIcon(Icons.lock_outline), findsWidgets);
    final locked = tester
        .widgetList<DropdownMenuItem<String>>(
          find.byType(DropdownMenuItem<String>),
        )
        .firstWhere((i) => i.value == 'reward-grand');
    expect(locked.enabled, isFalse);
    await tester.tap(find.text('Reward').last);
    await tester.pumpAndSettle();
    expect(picked, isNull);
  });
}
