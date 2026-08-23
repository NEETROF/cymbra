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

import '../services/audio_service.dart';
import '../services/soundfont_catalog_service.dart';
import '../services/soundfont_source.dart';
import 'imported_soundfonts.dart';
import 'piano_catalog.dart';
import 'selected_kit.dart';
import 'selected_piano.dart';

part 'score_font.g.dart';

/// Whether the drum-kit font is installed and sounding — the percussion
/// readiness gate (change: add-drum-audio-channel). The player sounds a
/// percussion score's notes only in [ready]; every other state is visual-only,
/// so a drum part is never sounded through a keyboard font.
enum KitFontStatus {
  /// No percussion score is active — the keyboard font rules.
  inactive,

  /// A percussion score is active and the kit swap is in flight (the outgoing
  /// keyboard font is still installed).
  loading,

  /// The kit font's awaited install resolved true: percussion may sound.
  ready,

  /// No kit font could be resolved or installed (no bundled asset, a broken
  /// import, audio unavailable): playback stays honestly visual-only — the
  /// same degradation the kit view shipped before this change.
  unavailable,
}

/// Font-follows-score controller (change: add-drum-audio-channel): keys the
/// synthesizer's loaded SoundFont on the **loaded score's** instrument family.
/// A dedicated listener widget in the player subtree reports the family
/// ([setScoreFamily]); this notifier is the single place that resolves and
/// applies fonts for it — the UI never calls the audio service, and no sibling
/// provider is imperatively invalidated (the kit-selection change is consumed
/// via `ref.listen`).
///
/// Entering a percussion score installs the remembered kit through the awaited
/// swap and publishes the readiness the player's audio routing gates on;
/// leaving it restores the remembered piano through [SelectedPiano]'s own
/// resolve+load path. The home instrument context never participates — the
/// score carries its own instrument.
@Riverpod(keepAlive: true)
class ScoreFont extends _$ScoreFont {
  /// Monotonic guard: a stale kit install (the score was left, or a newer swap
  /// started) must not publish its outcome over the current state.
  int _generation = 0;

  @override
  KitFontStatus build() {
    // Re-apply when the user picks a different kit while a percussion score is
    // active (the picker only writes the selection; this controller reacts).
    ref.listen(selectedKitProvider, (previous, next) {
      if (previous != next && state != KitFontStatus.inactive) {
        unawaited(_installKit());
      }
    });
    return KitFontStatus.inactive;
  }

  /// Reports the loaded score's family. Percussion (re)installs the remembered
  /// kit; keyboard restores the remembered piano — but only when a kit swap
  /// actually happened, so opening keyboard scores never re-parses the piano.
  /// Non-throwing: a container tearing down mid-flight is swallowed (the app
  /// is leaving; there is nothing left to keep consistent).
  Future<void> setScoreFamily({required bool percussion}) async {
    try {
      if (percussion) {
        await _installKit();
        return;
      }
      if (state == KitFontStatus.inactive) return; // piano already active
      _generation++; // cancel any in-flight kit install
      state = KitFontStatus.inactive;
      // Restore the keyboard memory through its own resolve+load+fallback
      // path; the selection itself is untouched.
      await ref.read(selectedPianoProvider.notifier).reapply();
    } catch (_) {
      // Disposed mid-flight (leaving the app/screen); nothing to restore.
    }
  }

  /// Swaps the remembered kit in through the awaited load and publishes the
  /// outcome. Percussion is ready to sound only once the completion resolves
  /// true — before that (and on failure) the player stays visual-only.
  Future<void> _installKit() async {
    final generation = ++_generation;
    state = KitFontStatus.loading;
    final ok = await _loadRememberedKit();
    if (generation != _generation) return; // superseded — outcome is stale
    state = ok ? KitFontStatus.ready : KitFontStatus.unavailable;
  }

  /// Resolves and installs the remembered kit, falling back to the bundled kit
  /// when the chosen one cannot be resolved or loaded — mirroring the piano
  /// fallback chain. False when no kit could be installed at all (e.g. the
  /// bundled kit asset has not landed yet and nothing else is available).
  Future<bool> _loadRememberedKit() async {
    // Wait for the async catalog sources first (the server download list and
    // the imported registry), so a kit held there is not missed when a
    // percussion score opens right at cold start — the same startup-window
    // guard the selection notifiers apply.
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
    final id = ref.read(selectedKitProvider);
    final catalog = ref.read(pianoCatalogProvider);
    PianoEntry? entryOf(String id) {
      for (final e in catalog) {
        if (e.id == id && e.family == SoundFamily.percussion) return e;
      }
      return null;
    }

    final chosen = entryOf(id);
    if (chosen != null && await _tryLoad(chosen)) return true;
    // Fallback: the bundled kit (when present — the asset lands separately).
    if (id != defaultKitId) {
      final bundled = entryOf(defaultKitId);
      if (bundled != null && await _tryLoad(bundled)) return true;
    }
    return false;
  }

  /// Resolves [entry]'s bytes and awaits the swap completion. False on any
  /// failure (unresolvable bytes, invalid font, audio unavailable).
  Future<bool> _tryLoad(PianoEntry entry) async {
    try {
      final path = await ref.read(soundFontSourceProvider).resolve(entry);
      return await ref.read(audioServiceProvider).loadSoundFontAwaited(path);
    } catch (_) {
      return false;
    }
  }
}
