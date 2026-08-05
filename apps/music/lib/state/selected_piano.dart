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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../analytics/usage_actions.dart';
import '../services/audio_service.dart';
import '../services/preferences_service.dart';
import '../services/soundfont_catalog_service.dart';
import '../services/soundfont_source.dart';
import 'imported_soundfonts.dart';
import 'piano_catalog.dart';
import 'reward_shop_notifier.dart';
import 'usage_tracking_notifier.dart';

part 'selected_piano.g.dart';

/// The id of the piano the synthesizer uses, persisted across launches.
///
/// Seeded synchronously with the bundled default (so the first frame is valid
/// and startup never blocks), then reconciled against the persisted choice: a
/// stored id still in the catalog wins and is loaded via [SoundFontSource] +
/// [AudioService]; an unknown stored id falls back to the default and is
/// re-persisted. Loading is degradable — if the chosen SoundFont's bytes cannot
/// be obtained, it falls back to the default (non-fatal) rather than crashing.
@Riverpod(keepAlive: true)
class SelectedPiano extends _$SelectedPiano {
  /// Preferences key under which the selected piano id lives.
  static const String prefsKey = 'selected_piano';

  /// True once startup restore finished, so the catalog listener only treats a
  /// disappearing selection as a genuine removal (not the empty startup window
  /// before imports are restored).
  bool _ready = false;

  @override
  String build() {
    _restore();
    // React to catalog changes: if the selected piano is removed from the
    // catalog (an imported one deleted), fall back to the default and re-apply.
    // A sibling notifier never pokes this one imperatively — it listens instead.
    ref.listen(pianoCatalogProvider, (previous, next) {
      if (_ready && !next.any((e) => e.id == state)) {
        _apply(defaultPianoId, persist: true);
      }
    });
    return defaultPianoId;
  }

  Future<void> _restore() async {
    // Wait for the async catalog sources (the server download list and the
    // imported registry) so a selected download/imported font is not mistaken for
    // an unknown id during the empty startup window.
    try {
      await ref.read(serverSoundFontsProvider.future);
    } catch (_) {
      // Continue with whatever the catalog holds.
    }
    try {
      await ref.read(importedSoundFontsProvider.future);
    } catch (_) {
      // Continue with whatever the catalog holds.
    }

    String? stored;
    try {
      stored = await ref.read(preferencesServiceProvider).getString(prefsKey);
    } catch (_) {
      _ready = true;
      return; // storage unavailable → keep the default
    }

    final catalog = ref.read(pianoCatalogProvider);
    String id = defaultPianoId;
    if (stored != null && catalog.any((e) => e.id == stored)) {
      id = stored; // promoted to non-null
    } else if (stored != null) {
      // Self-heal: an unknown stored id resolves to the default; re-persist so
      // storage matches what is active.
      await _persist(defaultPianoId);
    }

    state = id;
    _ready = true;

    // The default is already loaded at startup by [AudioService.init]; only a
    // non-default restored selection needs an explicit swap.
    if (id != defaultPianoId) {
      await _load(id);
    }
  }

  /// Selects [id] (must be in the catalog), persists it, and swaps the synth's
  /// SoundFont. A no-op for an unknown id.
  Future<void> select(String id) async {
    if (id == state) return;
    final catalog = ref.read(pianoCatalogProvider);
    if (!catalog.any((e) => e.id == id)) return;
    // A costed reward font stays auditionable but must be UNLOCKED before it can
    // be used as the active instrument (change: add-curation-rewards) — otherwise
    // the point cost is bypassable. `redeem` (a grant) flips `owned`, then this
    // succeeds.
    final reward = ref.read(rewardShopItemsByKeyProvider)[id];
    if (reward != null &&
        reward.pointCost > 0 &&
        reward.redeemable &&
        !reward.owned) {
      return; // locked → not selectable until redeemed
    }
    await _apply(id, persist: true);
    // Usage telemetry (change: add-feature-usage-analytics): the piano *setting*
    // changed (category only — the chosen piano is never recorded).
    unawaited(
      ref
          .read(usageTrackingNotifierProvider.notifier)
          .record(
            UsageActions.settingsChange,
            variant: UsageVariants.pianoType,
          ),
    );
  }

  /// Sets the selection to [id], optionally persisting, and loads its SoundFont.
  Future<void> _apply(String id, {required bool persist}) async {
    state = id;
    if (persist) await _persist(id);
    await _load(id);
  }

  /// Resolves the entry's SoundFont and swaps it in. On failure, falls back to
  /// the bundled default (non-fatal) so a missing download/import never breaks
  /// playback.
  Future<void> _load(String id) async {
    final entry = _entry(id);
    try {
      final path = await ref.read(soundFontSourceProvider).resolve(entry);
      await ref.read(audioServiceProvider).loadSoundFont(path);
    } catch (_) {
      // Already the default; nothing more to fall back to.
      if (id == defaultPianoId) return;
      state = defaultPianoId;
      await _persist(defaultPianoId);
      try {
        final path = await ref
            .read(soundFontSourceProvider)
            .resolve(defaultPiano);
        await ref.read(audioServiceProvider).loadSoundFont(path);
      } catch (_) {
        // Even the default failed (e.g. no audio); the choice still persisted
        // and will apply once audio is available.
      }
    }
  }

  PianoEntry _entry(String id) => ref
      .read(pianoCatalogProvider)
      .firstWhere((e) => e.id == id, orElse: () => defaultPiano);

  Future<void> _persist(String id) async {
    try {
      await ref.read(preferencesServiceProvider).setString(prefsKey, id);
    } catch (_) {
      // Best-effort: the in-memory selection still applies this session.
    }
  }
}
