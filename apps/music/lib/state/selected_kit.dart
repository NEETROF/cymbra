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
import '../services/preferences_service.dart';
import '../services/soundfont_catalog_service.dart';
import 'imported_soundfonts.dart';
import 'piano_catalog.dart';
import 'usage_tracking_notifier.dart';

part 'selected_kit.g.dart';

/// The id of the drum kit the synthesizer uses for **percussion scores**,
/// persisted across launches (change: add-drum-audio-channel). The percussion
/// family's memory, kept separate from [SelectedPiano] so choosing a kit never
/// disturbs the chosen piano and vice versa — same restore/fallback shape.
///
/// Unlike the piano notifier this one never touches the synthesizer: the kit
/// only sounds while a percussion score is loaded, and the font-follows-score
/// controller ([ScoreFont]) is the single application point — it listens to
/// this selection and swaps the font when (and only when) a kit is active.
@Riverpod(keepAlive: true)
class SelectedKit extends _$SelectedKit {
  /// Preferences key under which the selected kit id lives — distinct from
  /// [SelectedPiano.prefsKey] by construction (per-family memory).
  static const String prefsKey = 'selected_kit';

  /// True once startup restore finished, so the catalog listener only treats a
  /// disappearing selection as a genuine removal (not the empty startup window
  /// before imports are restored).
  bool _ready = false;

  @override
  String build() {
    _restore();
    // React to catalog changes: if the selected kit is removed from the
    // catalog (an imported one deleted), fall back to the bundled kit id.
    // A sibling notifier never pokes this one imperatively — it listens.
    ref.listen(pianoCatalogProvider, (previous, next) {
      if (_ready && state != defaultKitId && !_inCatalog(next, state)) {
        state = defaultKitId;
        unawaited(_persist(defaultKitId));
      }
    });
    return defaultKitId;
  }

  /// Whether [id] names a selectable kit: a percussion-family entry of the
  /// catalog, or the bundled-kit id itself — which stays valid even while the
  /// bundled asset has not landed (the readiness gate then reports the honest
  /// "no kit available" outcome instead of this selection lying about it).
  bool _inCatalog(List<PianoEntry> catalog, String id) =>
      id == defaultKitId ||
      catalog.any((e) => e.id == id && e.family == SoundFamily.percussion);

  Future<void> _restore() async {
    // Wait for the async catalog sources (the server download list and the
    // imported registry) so a selected download/imported kit is not mistaken
    // for an unknown id during the empty startup window.
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
    if (stored != null && _inCatalog(catalog, stored)) {
      state = stored;
    } else if (stored != null) {
      // Self-heal: an unknown stored id resolves to the bundled kit;
      // re-persist so storage matches what is active.
      await _persist(defaultKitId);
    }
    _ready = true;
    // No synth load here: the kit is applied by the font-follows-score
    // controller when a percussion score opens, never at startup (a keyboard
    // surface is active then).
  }

  /// Selects [id] (must be a percussion-family font of the catalog, or the
  /// bundled kit), and persists it. A no-op for an unknown id. The active
  /// synth swap — when a percussion score is open — is the font-follows-score
  /// controller's reaction to this state, not an imperative call from here.
  ///
  /// No reward-shop gate here (unlike [SelectedPiano.select]): the shop holds
  /// no percussion font today, and a costed kit arriving later must add the
  /// same cost-keyed gate the piano selection carries.
  Future<void> select(String id) async {
    if (id == state) return;
    if (!_inCatalog(ref.read(pianoCatalogProvider), id)) return;
    state = id;
    await _persist(id);
    // Usage telemetry (category only — the chosen kit is never recorded).
    unawaited(
      ref
          .read(usageTrackingNotifierProvider.notifier)
          .record(
            UsageActions.settingsChange,
            variant: UsageVariants.pianoType,
          ),
    );
  }

  Future<void> _persist(String id) async {
    try {
      await ref.read(preferencesServiceProvider).setString(prefsKey, id);
    } catch (_) {
      // Best-effort: the in-memory selection still applies this session.
    }
  }
}
