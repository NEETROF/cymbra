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

  /// Clef in effect for this note's staff (sign + line), so the Staff painter
  /// positions it correctly through mid-piece clef changes (e.g. a left hand
  /// that starts in treble and moves to bass).
  final String clefSign;
  final int clefLine;

  const TimedNote({
    required this.pitch,
    required this.startMs,
    required this.durationMs,
    this.staff = 1,
    this.beams = const [],
    this.clefSign = 'G',
    this.clefLine = 2,
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

    /// Key signature (fifths) of the loaded piece, for the staff armature.
    @Default(0) int keyFifths,

    /// Time signature of the loaded piece (beats / beat-type).
    @Default(4) int beats,
    @Default(4) int beatType,

    /// Score notes flattened and sorted by start.
    @Default(<TimedNote>[]) List<TimedNote> notes,

    /// End of the song (ms).
    @Default(0.0) double songEndMs,

    /// Start time (ms) of each measure, in order (Partition cursor placement).
    /// Empty for the demo score; populated from a parsed MusicXML document.
    @Default(<int>[]) List<int> measureStartMs,

    @Default(RenderMode.synthesia) RenderMode mode,
    @Default(true) bool waitMode,
    @Default(false) bool isPlaying,

    /// Playback position (playhead), in milliseconds.
    @Default(0.0) double elapsedMs,

    /// Speed multiplier (1.0 = 100%).
    @Default(1.0) double speed,

    /// True when Wait Mode is currently blocking progression.
    @Default(false) bool blocked,

    /// Pitches already pressed for the onset the playhead is currently waiting
    /// at (Wait Mode). Latched on key-down so a note counts even once released —
    /// validation is by attack, not sustained hold. Reset when the gate advances.
    @Default(<int>{}) Set<int> gateSatisfied,

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

  /// Pitches of notes whose onset is at instant [t] (their start coincides with
  /// the playhead, within a 1ms tolerance). This is the Wait Mode gate set: the
  /// notes that must be *attacked* here, regardless of their duration.
  Set<int> onsetPitchesAt(double t) {
    final result = <int>{};
    for (final n in notes) {
      if ((n.startMs - t).abs() <= 1.0) result.add(n.pitch);
    }
    return result;
  }

  /// The next note onset strictly after [t] (ms), or null if there are none.
  double? nextOnsetAfter(double t) {
    double? best;
    for (final n in notes) {
      if (n.startMs > t + 1 && (best == null || n.startMs < best)) {
        best = n.startMs.toDouble();
      }
    }
    return best;
  }

  /// Keys to highlight as "expected" on the keyboard. In Wait Mode this is the
  /// onset gate the playhead sits on, or — while travelling between onsets — the
  /// upcoming onset, so the preview shows the next note to play. Outside Wait
  /// Mode it is the notes sounding under the playhead.
  Set<int> get expectedKeys {
    if (!waitMode) return requiredNotesAt(elapsedMs);
    final here = onsetPitchesAt(elapsedMs);
    if (here.isNotEmpty) return here;
    final ns = nextOnsetAfter(elapsedMs);
    return ns == null ? const {} : onsetPitchesAt(ns);
  }

  /// The subset of [expectedKeys] belonging to one hand (staff 1 = right, staff
  /// 2+ = left), so the keyboard can colour expected keys per hand.
  Set<int> expectedKeysForHand({required bool rightHand}) {
    // The time the expected set refers to: the onset under the playhead (or the
    // upcoming one while travelling) in Wait Mode, else the playhead itself.
    final double t;
    if (waitMode) {
      if (onsetPitchesAt(elapsedMs).isEmpty) {
        final ns = nextOnsetAfter(elapsedMs);
        if (ns == null) return const {};
        t = ns;
      } else {
        t = elapsedMs;
      }
    } else {
      t = elapsedMs;
    }
    final result = <int>{};
    for (final n in notes) {
      final isRight = n.staff == 1;
      if (rightHand != isRight) continue;
      final hit = waitMode
          ? (n.startMs - t).abs() <= 1.0
          : (n.startMs <= t + 1 && t < n.startMs + n.durationMs);
      if (hit) result.add(n.pitch);
    }
    return result;
  }

  /// The measure containing playhead [t] and the fraction (0..1) elapsed within
  /// it, or null when no timing is known (e.g. the demo score) or [t] is outside
  /// the piece. Drives the Partition playhead cursor.
  ({int index, double fraction})? measureAt(double t) {
    final starts = measureStartMs;
    if (starts.isEmpty || t < starts.first) return null;
    for (var i = 0; i < starts.length; i++) {
      final start = starts[i];
      final end = (i + 1 < starts.length ? starts[i + 1] : songEndMs)
          .toDouble();
      if (t >= start && t < end) {
        final span = end - start;
        final frac = span > 0 ? ((t - start) / span).clamp(0.0, 1.0) : 0.0;
        return (index: i, fraction: frac);
      }
    }
    return null;
  }

  /// Expected notes at instant [t] for one hand: staff 1 is the right hand,
  /// staff 2+ the left hand. Same window as [requiredNotesAt], split by staff,
  /// so the assist keys play exactly the hand's due notes.
  Set<int> expectedNotesForHand(double t, {required bool rightHand}) {
    final result = <int>{};
    for (final n in notes) {
      final isRight = n.staff == 1;
      if (rightHand != isRight) continue;
      if (n.startMs <= t + 1 && t < n.startMs + n.durationMs) {
        result.add(n.pitch);
      }
    }
    return result;
  }
}

/// A pitch near [expected] (within ±[spread] semitones) that is **not** in
/// [avoid] and lies within `[lowBound, highBound]` — a deliberate near-miss that
/// never matches an expected note, so it cannot satisfy the Wait Mode gate.
///
/// [nextRandom] is called with an exclusive upper bound to choose among the
/// candidates; injecting it keeps the pick deterministic in tests. When no
/// candidate exists within [spread], the nearest in-range non-avoided pitch is
/// returned; if even that is impossible, [expected] is returned unchanged.
int nearMissPitch(
  int expected, {
  required int lowBound,
  required int highBound,
  required Set<int> avoid,
  required int Function(int) nextRandom,
  int spread = 3,
}) {
  final candidates = <int>[];
  for (var d = 1; d <= spread; d++) {
    for (final p in [expected - d, expected + d]) {
      if (p >= lowBound && p <= highBound && !avoid.contains(p)) {
        candidates.add(p);
      }
    }
  }
  if (candidates.isNotEmpty) {
    return candidates[nextRandom(candidates.length)];
  }
  // Fallback: nearest in-range pitch that is not avoided.
  for (var d = 1; d <= highBound - lowBound; d++) {
    for (final p in [expected - d, expected + d]) {
      if (p >= lowBound && p <= highBound && !avoid.contains(p)) return p;
    }
  }
  return expected;
}
