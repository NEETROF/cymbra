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

import 'package:flutter/material.dart';

import '../theme/cymbra_theme.dart';
import 'piano_layout.dart';

/// Visual state of a key, by precedence.
enum _KeyState {
  /// Not pressed and not expected.
  idle,

  /// Required at the playhead but not held — "press this key".
  expected,

  /// Required and held — correctly played.
  correct,

  /// Held but not required.
  pressed,
}

/// Draws the piano keyboard at the bottom of the screen with three-state
/// feedback: keys the player must press now ([requiredNotes]) glow teal, turn
/// green once correctly held, while any other pressed key ([activeNotes]) glows
/// purple.
class PianoKeyboardPainter extends CustomPainter {
  final PianoLayout layout;
  final Set<int> activeNotes;

  /// Notes expected at the current playhead (the Wait Mode gate).
  final Set<int> requiredNotes;

  const PianoKeyboardPainter({
    required this.layout,
    required this.activeNotes,
    this.requiredNotes = const {},
  });

  _KeyState _stateOf(int pitch) {
    final required = requiredNotes.contains(pitch);
    final active = activeNotes.contains(pitch);
    if (required && active) return _KeyState.correct;
    if (required) return _KeyState.expected;
    if (active) return _KeyState.pressed;
    return _KeyState.idle;
  }

  Color _fillFor(_KeyState state, {required bool isBlack}) => switch (state) {
    _KeyState.correct => CymbraColors.tertiary,
    _KeyState.expected => CymbraColors.secondaryContainer,
    _KeyState.pressed => CymbraColors.primaryContainer,
    _KeyState.idle =>
      isBlack ? CymbraColors.pianoBlack : CymbraColors.pianoWhite,
  };

  @override
  void paint(Canvas canvas, Size size) {
    final whiteH = size.height;
    final blackH = size.height * 0.62;

    // 1) White keys (background).
    for (var p = layout.lowPitch; p <= layout.highPitch; p++) {
      if (PianoLayout.isBlack(p)) continue;
      final r = layout.keyRect(p);
      final rect = Rect.fromLTWH(r.left, 0, r.width, whiteH);
      _drawKey(canvas, rect, _stateOf(p), isBlack: false);
    }

    // 2) Black keys (on top).
    for (var p = layout.lowPitch; p <= layout.highPitch; p++) {
      if (!PianoLayout.isBlack(p)) continue;
      final r = layout.keyRect(p);
      final rect = Rect.fromLTWH(r.left, 0, r.width, blackH);
      _drawKey(canvas, rect, _stateOf(p), isBlack: true);
    }
  }

  void _drawKey(
    Canvas canvas,
    Rect rect,
    _KeyState state, {
    required bool isBlack,
  }) {
    // Rounded bottom corners (4px) to mimic a physical key.
    final rrect = RRect.fromRectAndCorners(
      rect,
      bottomLeft: const Radius.circular(4),
      bottomRight: const Radius.circular(4),
    );

    final highlighted = state != _KeyState.idle;
    final fill = _fillFor(state, isBlack: isBlack);

    // Colored halo under any highlighted key.
    if (highlighted) {
      canvas.drawRRect(
        rrect,
        Paint()
          ..color = fill
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 12),
      );
    }

    canvas.drawRRect(rrect, Paint()..color = fill);

    // Border to separate white keys.
    if (!isBlack) {
      canvas.drawRRect(
        rrect,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1
          ..color = const Color(0xFFE2E8F0),
      );
    } else if (highlighted) {
      // A highlighted black key is narrow; outline it so the state still reads.
      canvas.drawRRect(
        rrect,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..color = CymbraColors.onSurface,
      );
    }
  }

  @override
  bool shouldRepaint(PianoKeyboardPainter old) =>
      old.activeNotes != activeNotes ||
      old.requiredNotes != requiredNotes ||
      old.layout.width != layout.width ||
      old.layout.lowPitch != layout.lowPitch ||
      old.layout.highPitch != layout.highPitch;
}
