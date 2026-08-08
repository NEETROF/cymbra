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

import 'dart:math' as math;

/// On-screen keyboard range modes: [auto] fits the loaded piece; the rest are
/// fixed key-count presets.
enum KeyboardRangeMode { auto, keys25, keys49, keys61, keys88 }

/// Lowest / highest MIDI pitch of an 88-key piano (A0 .. C8).
const int kPianoLowest = 21;
const int kPianoHighest = 108;

/// Default range used when no score is loaded (C4 .. C6, the historical POC
/// range and a 2-octave span).
const int _defaultLow = 60;
const int _defaultHigh = 84;

/// Minimum span (in semitones) for the auto range, so a sparse piece still gets
/// a usable keyboard. 24 = two octaves, matching the default C4..C6.
const int _minAutoSpan = 24;

extension KeyboardRangeModeLabel on KeyboardRangeMode {
  /// Short label for the chooser UI (and tests).
  String get label => switch (this) {
    KeyboardRangeMode.auto => 'Auto',
    KeyboardRangeMode.keys25 => '25',
    KeyboardRangeMode.keys49 => '49',
    KeyboardRangeMode.keys61 => '61',
    KeyboardRangeMode.keys88 => '88',
  };
}

/// Physical key count for a preset mode, or null for [KeyboardRangeMode.auto].
int? presetKeyCount(KeyboardRangeMode mode) => switch (mode) {
  KeyboardRangeMode.auto => null,
  KeyboardRangeMode.keys25 => 25,
  KeyboardRangeMode.keys49 => 49,
  KeyboardRangeMode.keys61 => 61,
  KeyboardRangeMode.keys88 => 88,
};

/// Standard low anchor (MIDI pitch) for each preset window.
int _presetAnchorLow(KeyboardRangeMode mode) => switch (mode) {
  KeyboardRangeMode.keys25 => 48, // C3 .. C5
  KeyboardRangeMode.keys49 => 36, // C2 .. C6
  KeyboardRangeMode.keys61 => 36, // C2 .. C7
  KeyboardRangeMode.keys88 => kPianoLowest, // A0 .. C8
  KeyboardRangeMode.auto => _defaultLow,
};

/// Inclusive (low, high) MIDI pitch range the on-screen keyboard should display
/// for [mode], given the score's note [pitches]. The result is always clamped to
/// the 88-key bounds and always covers every pitch in [pitches] (so the aligned
/// waterfall never drops a note).
({int low, int high}) computeKeyboardRange(
  KeyboardRangeMode mode,
  List<int> pitches,
) {
  if (mode == KeyboardRangeMode.auto) {
    return _autoRange(pitches);
  }
  final window = _presetWindow(mode, pitches);
  if (pitches.isEmpty) return window;
  final minP = pitches.reduce(math.min);
  final maxP = pitches.reduce(math.max);
  // Widen the fixed window rather than clip, so the aligned waterfall never
  // drops a note. The chosen size then acts as a minimum; the extra keys are
  // marked on the keyboard (see [chosenSizeWindow]).
  var low = window.low;
  var high = window.high;
  if (minP < low) low = minP;
  if (maxP > high) high = maxP;
  return _clampWithCoverage(low, high, minP, maxP);
}

/// The fixed-size window (exactly `presetKeyCount` keys) the preset [mode]
/// represents, slid to sit over the music but **before** any widening to cover
/// out-of-window notes. `null` for [KeyboardRangeMode.auto] (no fixed size).
///
/// This is where the *chosen* keyboard size sits. Keys in the drawn range
/// ([computeKeyboardRange]) that fall outside it are the extra keys added to
/// avoid clipping notes, so the on-screen keyboard can mark that boundary.
({int low, int high})? chosenSizeWindow(
  KeyboardRangeMode mode,
  List<int> pitches,
) => mode == KeyboardRangeMode.auto ? null : _presetWindow(mode, pitches);

/// Slides the fixed-size window for a preset [mode] over the music (preserving
/// its key count), clamped to the piano bounds. Shared by [computeKeyboardRange]
/// (which then widens it) and [chosenSizeWindow] (which does not).
({int low, int high}) _presetWindow(
  KeyboardRangeMode mode,
  List<int> pitches,
) {
  final span = presetKeyCount(mode)! - 1; // contiguous semitones
  var low = _presetAnchorLow(mode);
  var high = low + span;
  if (pitches.isNotEmpty) {
    final minP = pitches.reduce(math.min);
    final maxP = pitches.reduce(math.max);
    // Slide the fixed-size window to sit over the music.
    if (minP < low) {
      low = minP;
      high = low + span;
    }
    if (maxP > high) {
      high = maxP;
      low = high - span;
    }
  }
  return (
    low: low.clamp(kPianoLowest, kPianoHighest),
    high: high.clamp(kPianoLowest, kPianoHighest),
  );
}

({int low, int high}) _autoRange(List<int> pitches) {
  if (pitches.isEmpty) {
    return (low: _defaultLow, high: _defaultHigh);
  }
  final minP = pitches.reduce(math.min);
  final maxP = pitches.reduce(math.max);

  // Snap low down to a C and high up to a C (octave boundaries).
  var low = minP - (minP % 12);
  var high = maxP + ((12 - maxP % 12) % 12);

  // Enforce a minimum span; grow upward first, then downward.
  while (high - low < _minAutoSpan) {
    if (high + 12 <= kPianoHighest) {
      high += 12;
    } else if (low - 12 >= kPianoLowest) {
      low -= 12;
    } else {
      break;
    }
  }

  return _clampWithCoverage(low, high, minP, maxP);
}

/// Clamps [low]/[high] to the piano bounds, then re-asserts that the range still
/// covers [minP]..[maxP] (clamping can otherwise crop a boundary note).
({int low, int high}) _clampWithCoverage(
  int low,
  int high,
  int minP,
  int maxP,
) {
  var lo = low.clamp(kPianoLowest, kPianoHighest);
  var hi = high.clamp(kPianoLowest, kPianoHighest);
  if (lo > minP) lo = minP.clamp(kPianoLowest, kPianoHighest);
  if (hi < maxP) hi = maxP.clamp(kPianoLowest, kPianoHighest);
  return (low: lo, high: hi);
}
