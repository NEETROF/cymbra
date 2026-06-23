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

import '../painters/keyboard_range.dart';
import '../src/rust/api/musicxml.dart' show BeamState;
import '../src/rust/api/score.dart';

export '../painters/keyboard_range.dart'
    show KeyboardRangeMode, KeyboardRangeModeLabel;

part 'player_data.freezed.dart';

/// The score rendering modes: scrolling staff, Synthesia waterfall, and the
/// engraved Partition (sheet-music) view of a loaded MusicXML score.
enum RenderMode { staff, synthesia, partition }

/// A score note with its time bounds in milliseconds (int), more convenient to
/// handle on the Dart side than the bridge's `BigInt`.
class TimedNote {
  final int pitch;
  final int startMs;
  final int durationMs;

  /// Staff the note belongs to (1 = treble/right hand, 2 = bass/left hand).
  /// Lets the Staff painter lay out a real grand staff.
  final int staff;

  /// Beam states carried from the parsed notation (begin/continue/end), so the
  /// Staff painter can beam eighth/sixteenth runs instead of drawing flags.
  final List<BeamState> beams;

  const TimedNote({
    required this.pitch,
    required this.startMs,
    required this.durationMs,
    this.staff = 1,
    this.beams = const [],
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

    /// The loaded demo score (null when a MusicXML partition is loaded instead).
    Score? score,

    /// Title of the piece currently loaded (null → the built-in demo).
    String? title,

    /// Tempo in BPM used to place staff bar-lines and for the tempo readout.
    @Default(80) int bpm,

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

    /// On-screen keyboard range mode. Defaults to the full 88-key piano; the
    /// user can switch to auto-fit or a smaller preset from the chooser.
    @Default(KeyboardRangeMode.keys88) KeyboardRangeMode keyboardRange,
  }) = _PlayerData;

  bool get midiConnected => connectedDevice != null;

  /// Inclusive (low, high) MIDI pitches the on-screen keyboard should show for
  /// the current [keyboardRange] and loaded [notes]. Feeds the shared
  /// `PianoLayout` so the keyboard and waterfall stay aligned.
  ({int low, int high}) get keyboardBounds =>
      computeKeyboardRange(keyboardRange, [for (final n in notes) n.pitch]);

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
