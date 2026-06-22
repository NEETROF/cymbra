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

import '../state/player_data.dart';
import '../theme/cymbra_theme.dart';
import 'piano_layout.dart';

/// "Synthesia" rendering (waterfall): colored rectangles fall from the top of
/// the screen toward the keyboard line. The X axis maps to the keys (via
/// [PianoLayout]), the Y axis to time.
class SynthesiaPainter extends CustomPainter {
  final PianoLayout layout;
  final List<TimedNote> notes;

  /// Playhead in milliseconds.
  final double elapsedMs;

  /// Currently pressed notes (for the "note hit" visual feedback).
  final Set<int> activeNotes;

  /// Visible time window (how many ms span the height).
  final double lookAheadMs;

  const SynthesiaPainter({
    required this.layout,
    required this.notes,
    required this.elapsedMs,
    required this.activeNotes,
    this.lookAheadMs = 3000,
  });

  @override
  void paint(Canvas canvas, Size size) {
    // Midnight Navy background.
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = CymbraColors.background,
    );

    final hitLineY = size.height; // bottom of the area = keyboard line
    final pxPerMs = size.height / lookAheadMs;

    // Hit line (subtle) at the bottom.
    canvas.drawLine(
      Offset(0, hitLineY - 1),
      Offset(size.width, hitLineY - 1),
      Paint()
        ..color = CymbraColors.outlineVariant.withValues(alpha: 0.6)
        ..strokeWidth = 1,
    );

    for (final n in notes) {
      if (!layout.contains(n.pitch)) continue;

      // The bottom (head) of the note reaches the hit line at t = startMs.
      final bottomY = hitLineY - (n.startMs - elapsedMs) * pxPerMs;
      final height = n.durationMs * pxPerMs;
      final topY = bottomY - height;

      // Off screen: skip.
      if (bottomY < 0 || topY > hitLineY) continue;

      final r = layout.keyRect(n.pitch);
      // Slight inset to give breathing room between columns.
      final inset = r.width * 0.12;
      final rect = Rect.fromLTRB(
        r.left + inset,
        topY,
        r.left + r.width - inset,
        bottomY,
      );
      final rrect = RRect.fromRectAndRadius(rect, const Radius.circular(4));

      // Note in the hit zone (should be played now) → teal,
      // otherwise purple. Key correctly held → success green.
      final inHitZone =
          n.startMs <= elapsedMs && elapsedMs < n.startMs + n.durationMs;
      final Color base;
      if (inHitZone && activeNotes.contains(n.pitch)) {
        base = CymbraColors.tertiary; // well played
      } else if (inHitZone) {
        base = CymbraColors.secondaryContainer; // to play now
      } else {
        base = CymbraColors.primaryContainer; // upcoming
      }

      // Halo.
      canvas.drawRRect(
        rrect,
        Paint()
          ..color = base.withValues(alpha: 0.55)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 8),
      );

      // Vertical gradient for volume.
      final gradient = LinearGradient(
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
        colors: [base.withValues(alpha: 0.85), base],
      ).createShader(rect);
      canvas.drawRRect(rrect, Paint()..shader = gradient);
    }
  }

  @override
  bool shouldRepaint(SynthesiaPainter old) =>
      old.elapsedMs != elapsedMs ||
      old.activeNotes != activeNotes ||
      old.notes != notes ||
      old.layout.width != layout.width;
}
