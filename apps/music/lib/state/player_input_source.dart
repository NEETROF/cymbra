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

import 'package:flutter_riverpod/flutter_riverpod.dart' show Ref;
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import 'acoustic_input_access.dart';

part 'player_input_source.g.dart';

/// What feeds the player's note events (change: add-acoustic-piano-input):
/// the MIDI path (a connected instrument, with its auto-connect behavior), or
/// acoustic detection over the microphone.
enum PlayerInputSource { midi, microphone }

/// The user's stored input-source choice. Per-installation, like the play
/// preferences. Selecting the microphone never touches the remembered MIDI
/// port — the two live under different keys by construction (spec: Input
/// Source Selection).
@Riverpod(keepAlive: true)
class StoredInputSource extends _$StoredInputSource {
  static const String prefsKey = 'player_input_source';

  @override
  PlayerInputSource build() {
    _restore();
    return PlayerInputSource.midi;
  }

  Future<void> _restore() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      final restored = PlayerInputSource.values.asNameMap()[raw];
      if (restored != null) state = restored;
    } catch (_) {
      // Storage unavailable → the MIDI default stands.
    }
  }

  /// The user's explicit choice — the only writer.
  void select(PlayerInputSource source) {
    if (source == state) return;
    state = source;
    _persist(source);
  }

  Future<void> _persist(PlayerInputSource source) async {
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, source.name);
    } catch (_) {
      // Best-effort: the in-memory value still applies this session.
    }
  }
}

/// The input source the player PRESENTS AND USES: the stored choice, falling
/// back to MIDI while the flag is off for this caller (spec: Feature Flag
/// Audience — flag off means no trace). Presentational fallback only: the
/// stored value is never rewritten, so the microphone re-applies the moment
/// the flag returns.
@riverpod
PlayerInputSource effectivePlayerInputSource(Ref ref) {
  final stored = ref.watch(storedInputSourceProvider);
  final enabled = ref.watch(acousticInputEnabledProvider);
  return enabled ? stored : PlayerInputSource.midi;
}
