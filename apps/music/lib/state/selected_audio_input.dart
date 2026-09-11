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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/audio_capture_service.dart';
import '../services/preferences_service.dart';

part 'selected_audio_input.freezed.dart';
part 'selected_audio_input.g.dart';

/// The desktop input-device choice (spec: Desktop Capture Device Selection):
/// what the user pinned (null = follow the system default) and the devices on
/// offer. Empty and inert wherever the seam reports no selection support.
@freezed
abstract class SelectedAudioInputState with _$SelectedAudioInputState {
  const factory SelectedAudioInputState({
    /// The pinned device name, or null for the system default. Presentation
    /// falls back to the default when the pinned device is absent — the
    /// engine applies the same rule at open time.
    String? selected,

    /// The inputs the user may choose from (empty where unsupported).
    @Default(<CaptureRoute>[]) List<CaptureRoute> inputs,

    /// Whether this platform offers a device choice at all.
    @Default(false) bool supportsSelection,

    /// Runtime-only: whether the persisted choice has been read back.
    @Default(false) bool hydrated,
  }) = _SelectedAudioInputState;
}

/// Owns the persisted input-device choice: restores it at launch, applies it
/// through the seam (the engine resolves absences to the system default), and
/// re-applies a running capture on change. Warmed from `main.dart` like the
/// output selection, so the choice governs the first capture of the session —
/// not only once the settings section has been opened.
@Riverpod(keepAlive: true)
class SelectedAudioInput extends _$SelectedAudioInput {
  static const String prefsKey = 'selected_audio_input';

  @override
  SelectedAudioInputState build() {
    Future<void>.microtask(_restore);
    return const SelectedAudioInputState();
  }

  Future<void> _restore() async {
    final service = ref.read(audioCaptureServiceProvider);
    if (!service.supportsDeviceSelection) {
      state = state.copyWith(hydrated: true);
      return;
    }
    String? selected;
    try {
      selected = await ref.read(preferencesServiceProvider).getString(prefsKey);
    } catch (_) {
      // Storage unavailable → the system default stands.
    }
    List<CaptureRoute> inputs;
    try {
      inputs = await service.listInputs();
    } catch (_) {
      inputs = const [];
    }
    await service.selectInput(selected);
    state = state.copyWith(
      selected: selected,
      inputs: inputs,
      supportsSelection: true,
      hydrated: true,
    );
  }

  /// The user's explicit choice — the only writer. Null returns to the
  /// system default.
  Future<void> select(String? name) async {
    state = state.copyWith(selected: name);
    await ref.read(audioCaptureServiceProvider).selectInput(name);
    try {
      final prefs = ref.read(preferencesServiceProvider);
      if (name == null) {
        await prefs.remove(prefsKey);
      } else {
        await prefs.setString(prefsKey, name);
      }
    } catch (_) {
      // Best-effort: the in-memory value still applies this session.
    }
  }

  /// Re-enumerates the devices (e.g. when the section opens).
  Future<void> refreshInputs() async {
    final service = ref.read(audioCaptureServiceProvider);
    if (!service.supportsDeviceSelection) return;
    try {
      state = state.copyWith(inputs: await service.listInputs());
    } catch (_) {}
  }
}
