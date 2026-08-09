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

import 'dart:ui' show Offset;

/// Shared geometry of the piano keyboard.
///
/// Serves as a common X-axis reference for [PianoKeyboardPainter] and
/// [SynthesiaPainter]: a note of a given pitch falls exactly above its key.
class PianoLayout {
  /// First and last MIDI pitch displayed (default C4..C6, 2 octaves).
  final int lowPitch;
  final int highPitch;

  /// Total available width in pixels.
  final double width;

  const PianoLayout({
    this.lowPitch = 60,
    this.highPitch = 84,
    required this.width,
  });

  // "White" semitones within an octave: C D E F G A B.
  static const Set<int> _whiteSemitones = {0, 2, 4, 5, 7, 9, 11};

  static bool isBlack(int pitch) => !_whiteSemitones.contains(pitch % 12);

  /// The keyboard span a course exercise draws for [targets]: the notes plus
  /// two keys of context, widened symmetrically to at least [minKeys] keys —
  /// a lesson keyboard must read as a real stretch of piano, never a handful
  /// of giant keys — then snapped outward to white keys and clamped to the
  /// 88-key range.
  static ({int low, int high}) lessonRange(
    Iterable<int> targets, {
    int minKeys = 20,
  }) {
    var low = 60;
    var high = 60;
    var first = true;
    for (final p in targets) {
      if (first || p < low) low = p;
      if (first || p > high) high = p;
      first = false;
    }
    low = (low - 2).clamp(21, 108);
    high = (high + 2).clamp(21, 108);
    while (high - low + 1 < minKeys && (low > 21 || high < 108)) {
      if (low > 21) low--;
      if (high - low + 1 < minKeys && high < 108) high++;
    }
    while (isBlack(low) && low > 21) {
      low--;
    }
    while (isBlack(high) && high < 108) {
      high++;
    }
    return (low: low, high: high);
  }

  int get _whiteCount {
    var count = 0;
    for (var p = lowPitch; p <= highPitch; p++) {
      if (!isBlack(p)) count++;
    }
    return count;
  }

  double get whiteWidth => width / _whiteCount;
  double get blackWidth => whiteWidth * 0.62;

  /// Number of white keys strictly before [pitch].
  int _whiteIndex(int pitch) {
    var count = 0;
    for (var p = lowPitch; p < pitch; p++) {
      if (!isBlack(p)) count++;
    }
    return count;
  }

  /// Left edge and width of the [pitch] key (in pixels).
  ({double left, double width}) keyRect(int pitch) {
    if (!isBlack(pitch)) {
      return (left: _whiteIndex(pitch) * whiteWidth, width: whiteWidth);
    }
    // A black key is centered on the boundary between two white keys.
    final boundary = _whiteIndex(pitch) * whiteWidth;
    return (left: boundary - blackWidth / 2, width: blackWidth);
  }

  /// Horizontal center of the [pitch] key (used by the cascade).
  double centerX(int pitch) {
    final r = keyRect(pitch);
    return r.left + r.width / 2;
  }

  /// X of the left edge of [pitch]'s slot — where a vertical boundary drawn just
  /// before [pitch] sits. Equals the cumulative white-key width before [pitch].
  double leftEdgeX(int pitch) => _whiteIndex(pitch) * whiteWidth;

  bool contains(int pitch) => pitch >= lowPitch && pitch <= highPitch;

  /// MIDI pitch of the key drawn under [p] on a keyboard of [height] pixels, or
  /// null when no key is there (outside the keyboard or past the last white
  /// key). Black keys take priority within their upper band, matching how
  /// [PianoKeyboardPainter] stacks them on top of the white keys (the top 62%
  /// of the height).
  int? pitchAt(Offset p, double height) {
    final x = p.dx;
    final y = p.dy;
    if (x < 0 || y < 0 || y > height) return null;

    // Black keys occupy only the upper band and are painted over the whites, so
    // a hit there wins over the white key beneath it.
    final blackH = height * 0.62;
    if (y < blackH) {
      for (var pitch = lowPitch; pitch <= highPitch; pitch++) {
        if (!isBlack(pitch)) continue;
        final r = keyRect(pitch);
        if (x >= r.left && x < r.left + r.width) return pitch;
      }
    }

    // White keys span the full height and tile the width edge-to-edge.
    for (var pitch = lowPitch; pitch <= highPitch; pitch++) {
      if (isBlack(pitch)) continue;
      final r = keyRect(pitch);
      if (x >= r.left && x < r.left + r.width) return pitch;
    }
    return null;
  }
}
