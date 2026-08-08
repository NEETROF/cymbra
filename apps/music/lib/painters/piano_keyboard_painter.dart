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

import 'package:flutter/material.dart';

import '../theme/cymbra_theme.dart';
import 'piano_layout.dart';

/// Visual state of a key, by precedence.
enum _KeyState {
  /// Not pressed and not expected.
  idle,

  /// Required by the right hand but not held — "press this key" (blue).
  expectedRight,

  /// Required by the left hand but not held — "press this key" (amber).
  expectedLeft,

  /// Required and held — correctly played.
  correct,

  /// Held but not required.
  pressed,
}

/// Draws the piano keyboard at the bottom of the screen with feedback: keys the
/// player must press now ([requiredNotes]) glow in their hand's colour (right =
/// blue, left = amber — [leftHandNotes] marks the left-hand subset), turn green
/// once correctly held, while any other pressed key ([activeNotes]) glows purple.
class PianoKeyboardPainter extends CustomPainter {
  final PianoLayout layout;
  final Set<int> activeNotes;

  /// Notes expected at the current playhead (the Wait Mode gate).
  final Set<int> requiredNotes;

  /// The subset of [requiredNotes] belonging to the left hand (staff 2+); the
  /// rest are right-hand. Drives the per-hand expected colour.
  final Set<int> leftHandNotes;

  /// The fixed-size window the chosen keyboard size represents (exactly N keys),
  /// or null in auto mode. When the drawn range ([layout]) is wider than this,
  /// a dashed boundary marks where the chosen size ends and the extra keys begin.
  final ({int low, int high})? chosenWindow;

  const PianoKeyboardPainter({
    required this.layout,
    required this.activeNotes,
    this.requiredNotes = const {},
    this.leftHandNotes = const {},
    this.chosenWindow,
  });

  /// MIDI middle C (C4) — the anchor note the octave labels emphasise.
  static const int _middleC = 60;

  _KeyState _stateOf(int pitch) {
    final required = requiredNotes.contains(pitch);
    final active = activeNotes.contains(pitch);
    if (required && active) return _KeyState.correct;
    if (required) {
      return leftHandNotes.contains(pitch)
          ? _KeyState.expectedLeft
          : _KeyState.expectedRight;
    }
    if (active) return _KeyState.pressed;
    return _KeyState.idle;
  }

  Color _fillFor(_KeyState state, {required bool isBlack}) => switch (state) {
    _KeyState.correct => CymbraColors.tertiary,
    _KeyState.expectedRight => CymbraColors.handRight,
    _KeyState.expectedLeft => CymbraColors.handLeft,
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

    // 3) Octave anchors: label each C (C3, C4…) at the bottom of its white key so
    // the player can orient their hands; middle C (C4) is emphasised.
    _drawOctaveLabels(canvas, whiteH);

    // 4) Chosen-size boundary: when the drawn range is wider than the chosen
    // keyboard size, a dashed line marks where the chosen size ends.
    _drawChosenSizeBoundary(canvas, whiteH);
  }

  /// Draws a "C{octave}" label under every C white key. Scientific pitch
  /// notation (MIDI 60 = C4). The label is skipped when it would not fit the key
  /// width, but middle C always keeps a small dot so it stays findable.
  void _drawOctaveLabels(Canvas canvas, double height) {
    for (var p = layout.lowPitch; p <= layout.highPitch; p++) {
      if (p % 12 != 0) continue; // C keys only
      final isMiddle = p == _middleC;
      final r = layout.keyRect(p);
      final octave = p ~/ 12 - 1; // MIDI: C4 = 60
      final fontSize = (r.width * 0.32).clamp(6.0, 11.0);
      final tp = TextPainter(
        text: TextSpan(
          text: 'C$octave',
          style: TextStyle(
            fontSize: fontSize,
            height: 1.0,
            fontWeight: isMiddle ? FontWeight.w800 : FontWeight.w500,
            color: isMiddle
                ? CymbraColors.primaryContainer
                : const Color(0xFF64748B),
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();

      // Middle C keeps an always-visible dot even if its label is too wide.
      if (isMiddle) {
        canvas.drawCircle(
          Offset(r.left + r.width / 2, height - fontSize - 8),
          math.max(2.0, r.width * 0.06),
          Paint()..color = CymbraColors.primaryContainer,
        );
      }

      if (tp.width > r.width - 2) continue; // too narrow: skip the text
      tp.paint(
        canvas,
        Offset(r.left + (r.width - tp.width) / 2, height - tp.height - 3),
      );
    }
  }

  /// Draws a dashed vertical line where the chosen keyboard size ends, on each
  /// side the drawn range was widened past it (see [chosenWindow]).
  void _drawChosenSizeBoundary(Canvas canvas, double height) {
    final window = chosenWindow;
    if (window == null) return;
    final paint = Paint()
      ..color = CymbraColors.outline
      ..strokeWidth = 1.5
      ..strokeCap = StrokeCap.round;
    // Extra keys below the chosen window → boundary at its low edge.
    if (layout.lowPitch < window.low) {
      _dashedVLine(canvas, layout.leftEdgeX(window.low), height, paint);
    }
    // Extra keys above the chosen window → boundary at its high edge.
    if (layout.highPitch > window.high) {
      _dashedVLine(canvas, layout.leftEdgeX(window.high + 1), height, paint);
    }
  }

  void _dashedVLine(Canvas canvas, double x, double height, Paint paint) {
    const dash = 5.0;
    const gap = 4.0;
    for (var y = 0.0; y < height; y += dash + gap) {
      canvas.drawLine(Offset(x, y), Offset(x, math.min(y + dash, height)), paint);
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
      old.leftHandNotes != leftHandNotes ||
      old.chosenWindow != chosenWindow ||
      old.layout.width != layout.width ||
      old.layout.lowPitch != layout.lowPitch ||
      old.layout.highPitch != layout.highPitch;
}
