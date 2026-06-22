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

import '../src/rust/api/score.dart';

part 'player_data.freezed.dart';

/// The two score rendering modes.
enum RenderMode { staff, synthesia }

/// A score note with its time bounds in milliseconds (int), more convenient to
/// handle on the Dart side than the bridge's `BigInt`.
class TimedNote {
  final int pitch;
  final int startMs;
  final int durationMs;

  const TimedNote({
    required this.pitch,
    required this.startMs,
    required this.durationMs,
  });
}

/// Immutable player state (replaces the former `ChangeNotifier`).
///
/// Held by the `Player` Riverpod notifier; the UI watches it and mutates it via
/// `copyWith` only.
@freezed
abstract class PlayerData with _$PlayerData {
  const PlayerData._();

  const factory PlayerData({
    /// MIDI notes currently pressed (real MIDI keyboard + keyboard fallback).
    @Default(<int>{}) Set<int> activeNotes,

    /// Detected MIDI devices.
    @Default(<String>[]) List<String> midiPorts,

    /// Currently connected port (null if none).
    String? connectedDevice,

    /// The loaded score (null until [Player] finishes loading it).
    Score? score,

    /// Score notes flattened and sorted by start.
    @Default(<TimedNote>[]) List<TimedNote> notes,

    /// End of the song (ms).
    @Default(0.0) double songEndMs,

    @Default(RenderMode.synthesia) RenderMode mode,
    @Default(true) bool waitMode,
    @Default(false) bool isPlaying,

    /// Playback position (playhead), in milliseconds.
    @Default(0.0) double elapsedMs,

    /// Speed multiplier (1.0 = 100%).
    @Default(1.0) double speed,

    /// True when Wait Mode is currently blocking progression.
    @Default(false) bool blocked,
  }) = _PlayerData;

  bool get midiConnected => connectedDevice != null;

  /// Notes that should be held at instant [t] (playhead within the window
  /// [start, start+duration]). Acts as the "gate" for Wait Mode.
  Set<int> requiredNotesAt(double t) {
    final result = <int>{};
    for (final n in notes) {
      if (n.startMs <= t + 1 && t < n.startMs + n.durationMs) {
        result.add(n.pitch);
      }
    }
    return result;
  }
}
