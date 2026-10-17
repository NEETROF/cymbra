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
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import 'instrument_context.dart';
import 'learn_door_access.dart';

part 'home_door.freezed.dart';
part 'home_door.g.dart';

/// The two doors of the home (change: add-music-learn-play-home): **Play** is
/// the library as it always was; **Learn** is the piano learning path.
enum HomeDoor { play, learn }

/// The persisted door record. Per-installation, like the instrument context:
/// the choice survives a relaunch and a sign-out/sign-in cycle.
@freezed
abstract class HomeDoorState with _$HomeDoorState {
  const factory HomeDoorState({
    /// The stored choice — written only by the player's tap, never by a
    /// visibility change (the fallback to Play under drums is presentational).
    @Default(HomeDoor.play) HomeDoor door,

    /// Runtime-only: whether the stored record has been read back. The body
    /// waits for it before animating a door change, so a cold start lands on
    /// the remembered door without a visible swap.
    @Default(false) bool hydrated,
  }) = _HomeDoorState;
}

/// Device-persisted home door. Seeded synchronously with the default (Play) so
/// the first frame never blocks, then reconciled against storage — the same
/// pattern as the instrument context and the play preferences.
@Riverpod(keepAlive: true)
class HomeDoorChoice extends _$HomeDoorChoice {
  static const String prefsKey = 'home_door';

  @override
  HomeDoorState build() {
    _restore();
    return const HomeDoorState();
  }

  Future<void> _restore() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      final door = HomeDoor.values.asNameMap()[raw];
      if (door != null) state = state.copyWith(door: door);
    } catch (_) {
      // Storage unavailable / corrupt value → Play stands.
    }
    state = state.copyWith(hydrated: true);
  }

  /// The player's explicit choice — the ONLY thing that writes the door.
  void select(HomeDoor door) {
    if (door == state.door) return;
    state = state.copyWith(door: door);
    _persist(door);
  }

  Future<void> _persist(HomeDoor door) async {
    try {
      await ref.read(preferencesServiceProvider).setString(prefsKey, door.name);
    } catch (_) {
      // Best-effort: the in-memory value still applies this session.
    }
  }
}

/// The door the home PRESENTS: the stored choice, falling back to Play while
/// the Learn door is unavailable — the two-door home switched off by its flag,
/// or the drums instrument context in force (the Learn door is keyboard-only).
/// Presentational only: the stored value is never rewritten, so Learn reapplies
/// the moment the keyboard context or the flag returns.
@riverpod
HomeDoor effectiveHomeDoor(Ref ref) {
  final stored = ref.watch(homeDoorChoiceProvider.select((s) => s.door));
  if (!ref.watch(learnDoorEnabledProvider)) return HomeDoor.play;
  final drums =
      ref.watch(effectiveInstrumentContextProvider) == AppInstrument.drums;
  return drums ? HomeDoor.play : stored;
}

/// Whether the Learn door can be offered right now: the flag is on and the
/// instrument context is the keyboard.
@riverpod
bool learnDoorAvailable(Ref ref) =>
    ref.watch(learnDoorEnabledProvider) &&
    ref.watch(effectiveInstrumentContextProvider) == AppInstrument.keyboard;
