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
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import 'drum_input_mapping.dart';
import 'midi_status_notifier.dart';

part 'drum_input_mapping_notifier.g.dart';

/// Every calibrated device's input mapping, persisted locally (change:
/// add-drum-input-calibration; spec: `local-preferences`).
///
/// Its **own** preferences key rather than a field of `PlayerPrefs`: that record
/// is one setting per player, this is one table per piece of hardware, and
/// folding a growing per-device map into a record every setter rewrites whole
/// would make an unrelated tempo change carry the mapping with it.
///
/// Seeded empty synchronously so the first frame never blocks, then reconciled
/// against storage — the same shape as `PlayerPreferences`. Nothing here is a
/// credential, so it lives in plain preferences, and nothing is ever sent to a
/// server: a mapping describes a kit standing in one room.
@Riverpod(keepAlive: true)
class DrumInputMappingStore extends _$DrumInputMappingStore {
  /// Preferences key holding every device's table, JSON-encoded.
  static const String prefsKey = 'drum_input_mappings';

  // A completion signal, not state — the same exemption `PlayerPreferences`
  // takes for its own `restored`.
  // ignore: avoid_public_notifier_properties
  /// Completes when storage has been read (or found absent/corrupt, in which
  /// case no device is calibrated). A caller that must not act on an
  /// uncalibrated-looking store at startup awaits this first.
  Future<void> get restored => _restored;
  Future<void> _restored = Future.value();

  @override
  DrumInputMappings build() {
    _restored = _restore();
    return const {};
  }

  Future<void> _restore() async {
    String? raw;
    try {
      raw = await ref.read(preferencesServiceProvider).getString(prefsKey);
    } catch (_) {
      return; // storage unavailable → every device uncalibrated
    }
    final restored = decodeDrumInputMappings(raw);
    if (restored.isNotEmpty) state = restored;
  }

  /// The table learned for [port], or the empty (identity) mapping — which is
  /// also the answer for a null port, an unknown device, and a device whose
  /// stored entry could not be read.
  DrumInputMapping forPort(String? port) => port == null
      ? DrumInputMapping.empty
      : state[port] ?? DrumInputMapping.empty;

  /// Stores [mapping] against [port], replacing whatever that device had.
  /// An empty mapping clears the device instead of storing a hollow entry.
  void save(String port, DrumInputMapping mapping) {
    if (mapping.isEmpty) {
      clear(port);
      return;
    }
    _update({...state, port: mapping});
  }

  /// Records one piece for [port] without disturbing its other entries — the
  /// edit path, which must not require re-running the whole pass.
  void setPiece(String port, String pieceId, int number) =>
      save(port, forPort(port).withPiece(pieceId, number));

  /// Clears one piece for [port]. Clearing the last one returns the device to
  /// uncalibrated behaviour, which is [clear].
  void clearPiece(String port, String pieceId) =>
      save(port, forPort(port).withoutPiece(pieceId));

  /// Returns [port] to uncalibrated behaviour: every number interpreted as it
  /// arrives, exactly as before it was ever calibrated.
  void clear(String port) {
    if (!state.containsKey(port)) return;
    _update({...state}..remove(port));
  }

  void _update(DrumInputMappings next) {
    state = next;
    _persist(next);
  }

  Future<void> _persist(DrumInputMappings mappings) async {
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, encodeDrumInputMappings(mappings));
    } catch (_) {
      // Best-effort: the in-memory table still applies this session.
    }
  }
}

/// The mapping in force **right now**: the connected device's, or the identity
/// when nothing is connected or the device was never calibrated.
///
/// One derived answer rather than a lookup repeated at each consumer, so the
/// monitor, the review table and the calibration pass can never disagree about
/// which device's table applies.
///
/// Keyed off [midiStatusProvider] rather than the player: the surfaces that
/// need this run with **no score loaded** (calibration is reached from the
/// settings), and reading the player here would also make the player's own
/// read of the store — the translation seam in [Player] — a dependency cycle.
@riverpod
DrumInputMapping activeDrumMapping(Ref ref) {
  final port = ref.watch(midiStatusProvider.select((s) => s.connected));
  final mappings = ref.watch(drumInputMappingStoreProvider);
  return port == null
      ? DrumInputMapping.empty
      : mappings[port] ?? DrumInputMapping.empty;
}
