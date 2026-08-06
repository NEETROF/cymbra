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

// MANUAL-DEMO entry point (change: add-soundfont-uploader-attribution) — NOT part
// of the shipping app. Runs the real SoundFontsScreen on a device/simulator with
// the SAME in-memory fakes the widget tests use, so the new attribution UI is
// visible without a backend:
//   flutter run -d <device> -t test/demo/soundfont_demo_main.dart
// Shows: the public contributor credit on a catalog font, the rejection reason on
// a rejected import, and the justification-gated re-propose dialog.

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
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
import 'package:music/state/sound_preview_sample.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

/// A curator-rewards fake: empty shop (no locked fonts in this demo).
class _ShopFake implements CuratorRewardsService {
  const _ShopFake();

  @override
  Future<List<RewardShopItemView>> listShop() async => const [];

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

void main() {
  // Portrait-only, like the shipping app (the simulator may remember a
  // landscape orientation from a previous session).
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setPreferredOrientations([DeviceOrientation.portraitUp]);
  // A synced private import whose proposal was REJECTED with a moderator motive:
  // the card shows the reason and offers a justification-gated re-propose.
  final registry = jsonEncode([
    PianoEntry(
      id: 'my-upright',
      label: 'Mon piano droit',
      kind: PianoKind.user,
      source: '/copied/my-upright.sf2',
      remoteId: 'remote-my-upright',
    ).toJson(),
  ]);
  final prefs = FakePreferencesService({ImportedSoundFonts.prefsKey: registry});
  final private = FakePrivateSoundFontService(
    library: const [
      RemoteSoundFont(
        id: 'remote-my-upright',
        label: 'Mon piano droit',
        sizeBytes: 4 * 1024 * 1024,
        proposalStatus: 'rejected',
        rejectionReason: 'Échantillons bruités au-dessus de C6 — re-exportez',
      ),
    ],
  );
  runApp(
    ProviderScope(
      overrides: [
        preferencesServiceProvider.overrideWithValue(prefs),
        soundFontImporterProvider.overrideWithValue(FakeSoundFontImporter()),
        privateSoundFontServiceProvider.overrideWithValue(private),
        soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
        soundFontPreviewServiceProvider.overrideWithValue(
          FakeSoundFontPreviewService(
            available: const {'community-grand', 'warm-upright'},
          ),
        ),
        curatorRewardsServiceProvider.overrideWithValue(const _ShopFake()),
        soundFontCatalogServiceProvider.overrideWithValue(
          FakeSoundFontCatalogService(
            downloadable: [
              // A community font whose uploader opted into a public profile →
              // "Proposé par @alice" alongside its licence attribution.
              fakeDownloadPiano(
                id: 'community-grand',
                label: 'Community Grand',
                license: 'CC-BY 4.0',
                attribution: 'Sample Author',
                contributorCredit: 'alice',
                hasPreview: true,
              ),
              // Same catalog, private (or seeded) uploader → no credit.
              fakeDownloadPiano(
                id: 'warm-upright',
                label: 'Warm Upright',
                license: 'CC0-1.0',
                hasPreview: true,
              ),
            ],
          ),
        ),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
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
      child: localizedApp(const SoundFontsScreen(), locale: const Locale('fr')),
    ),
  );
}
