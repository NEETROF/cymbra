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

  bool contains(int pitch) => pitch >= lowPitch && pitch <= highPitch;
}
