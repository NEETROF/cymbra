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

/// Draws the piano keyboard at the bottom of the screen and highlights the
/// currently pressed keys ([activeNotes]) with a purple halo.
class PianoKeyboardPainter extends CustomPainter {
  final PianoLayout layout;
  final Set<int> activeNotes;

  const PianoKeyboardPainter({
    required this.layout,
    required this.activeNotes,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final whiteH = size.height;
    final blackH = size.height * 0.62;

    // 1) White keys (background).
    for (var p = layout.lowPitch; p <= layout.highPitch; p++) {
      if (PianoLayout.isBlack(p)) continue;
      final r = layout.keyRect(p);
      final rect = Rect.fromLTWH(r.left, 0, r.width, whiteH);
      final active = activeNotes.contains(p);
      _drawKey(canvas, rect, active, isBlack: false);
    }

    // 2) Black keys (on top).
    for (var p = layout.lowPitch; p <= layout.highPitch; p++) {
      if (!PianoLayout.isBlack(p)) continue;
      final r = layout.keyRect(p);
      final rect = Rect.fromLTWH(r.left, 0, r.width, blackH);
      final active = activeNotes.contains(p);
      _drawKey(canvas, rect, active, isBlack: true);
    }
  }

  void _drawKey(Canvas canvas, Rect rect, bool active, {required bool isBlack}) {
    // Rounded bottom corners (4px) to mimic a physical key.
    final rrect = RRect.fromRectAndCorners(
      rect,
      bottomLeft: const Radius.circular(4),
      bottomRight: const Radius.circular(4),
    );

    final Color fill;
    if (active) {
      fill = CymbraColors.primaryContainer;
    } else {
      fill = isBlack ? CymbraColors.pianoBlack : CymbraColors.pianoWhite;
    }

    // Purple halo under an active key.
    if (active) {
      canvas.drawRRect(
        rrect,
        Paint()
          ..color = CymbraColors.primaryContainer
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
    }
  }

  @override
  bool shouldRepaint(PianoKeyboardPainter old) =>
      old.activeNotes != activeNotes || old.layout.width != layout.width;
}
