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

import '../state/player_state.dart';
import '../theme/cymbra_theme.dart';

/// "Standard staff" rendering synchronized to time, like the waterfall.
///
/// The staff (5 lines) stays fixed; the notes and measure bars scroll
/// horizontally from right to left based on [elapsedMs]. A vertical playhead
/// line marks the current instant: only the note aligned on it is highlighted —
/// green if it's correctly played, teal if it's expected.
class StaffPainter extends CustomPainter {
  final List<TimedNote> notes;
  final double elapsedMs;
  final Set<int> activeNotes;

  /// Tempo, to place the measure bars.
  final int bpm;

  /// End of the song (ms), to bound the measure bars.
  final double songEndMs;

  const StaffPainter({
    required this.notes,
    required this.elapsedMs,
    required this.activeNotes,
    required this.bpm,
    required this.songEndMs,
  });

  // Reference: E4 (MIDI 64) = bottom line of the treble clef staff.
  static const int _bottomLinePitch = 64;

  // Visible time window to the right of the playhead.
  static const double _lookAheadMs = 4000;

  @override
  void paint(Canvas canvas, Size size) {
    if (notes.isEmpty) return;

    const margin = 48.0;
    final lineGap = (size.height * 0.10).clamp(10.0, 20.0);
    final stepGap = lineGap / 2;
    final bottomLineY = size.height / 2 + 2 * lineGap;
    final topLineY = bottomLineY - 4 * lineGap;

    // Playhead fixed at the left quarter; time advances toward the left.
    final playLineX = size.width * 0.25;
    final pxPerMs = (size.width - playLineX - margin) / _lookAheadMs;

    double xForTime(double tMs) => playLineX + (tMs - elapsedMs) * pxPerMs;

    final linePaint = Paint()
      ..color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.45)
      ..strokeWidth = 1.2;
    final barPaint = Paint()
      ..color = CymbraColors.onSurface.withValues(alpha: 0.5)
      ..strokeWidth = 1.4;

    // 1) The 5 staff lines (fixed).
    for (var i = 0; i < 5; i++) {
      final y = bottomLineY - i * lineGap;
      canvas.drawLine(
        Offset(margin, y),
        Offset(size.width - margin, y),
        linePaint,
      );
    }

    // 2) Scrolling measure bars.
    final measureMs = (60000.0 / bpm) * 4; // 4 beats per measure
    if (measureMs > 0) {
      for (var t = 0.0; t <= songEndMs + measureMs; t += measureMs) {
        final x = xForTime(t);
        if (x < margin || x > size.width - margin) continue;
        canvas.drawLine(Offset(x, topLineY), Offset(x, bottomLineY), barPaint);
      }
    }

    // 3) Playhead (playback line).
    final playPaint = Paint()
      ..color = CymbraColors.secondary.withValues(alpha: 0.9)
      ..strokeWidth = 2;
    canvas.drawLine(
      Offset(playLineX, topLineY - lineGap * 1.5),
      Offset(playLineX, bottomLineY + lineGap * 1.5),
      playPaint,
    );

    // 4) Scrolling notes.
    for (final n in notes) {
      final x = xForTime(n.startMs.toDouble());
      if (x < margin - lineGap || x > size.width - margin + lineGap) continue;

      final y = bottomLineY - _staffSteps(n.pitch) * stepGap;

      // Only the note under the playhead (expected now) is emphasized.
      final atPlayhead =
          n.startMs <= elapsedMs && elapsedMs < n.startMs + n.durationMs;
      final Color color;
      if (atPlayhead && activeNotes.contains(n.pitch)) {
        color = CymbraColors.tertiary; // well played
      } else if (atPlayhead) {
        color = CymbraColors.secondary; // expected
      } else {
        color = CymbraColors.onSurfaceVariant.withValues(
          alpha: 0.55,
        ); // upcoming / past
      }

      _drawNoteHead(canvas, Offset(x, y), lineGap, atPlayhead, color);
      _drawLedgerLines(canvas, x, y, bottomLineY, topLineY, lineGap, linePaint);
    }
  }

  int _staffSteps(int pitch) => _diatonic(pitch) - _diatonic(_bottomLinePitch);

  int _diatonic(int pitch) {
    const whiteInOctave = {
      0: 0,
      1: 0,
      2: 1,
      3: 1,
      4: 2,
      5: 3,
      6: 3,
      7: 4,
      8: 4,
      9: 5,
      10: 5,
      11: 6,
    };
    final octave = pitch ~/ 12;
    final semitone = pitch % 12;
    return octave * 7 + (whiteInOctave[semitone] ?? 0);
  }

  void _drawNoteHead(
    Canvas canvas,
    Offset center,
    double lineGap,
    bool emphasized,
    Color color,
  ) {
    final rx = lineGap * 0.6;
    final ry = lineGap * 0.45;
    if (emphasized) {
      canvas.drawOval(
        Rect.fromCenter(center: center, width: rx * 3, height: ry * 3),
        Paint()
          ..color = color.withValues(alpha: 0.5)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
      );
    }
    canvas.drawOval(
      Rect.fromCenter(center: center, width: rx * 2, height: ry * 2),
      Paint()..color = color,
    );
    // Stem.
    canvas.drawLine(
      Offset(center.dx + rx, center.dy),
      Offset(center.dx + rx, center.dy - lineGap * 2.6),
      Paint()
        ..color = color
        ..strokeWidth = 1.4,
    );
  }

  void _drawLedgerLines(
    Canvas canvas,
    double x,
    double y,
    double bottomLineY,
    double topLineY,
    double lineGap,
    Paint linePaint,
  ) {
    for (var ly = bottomLineY + lineGap; ly <= y + 0.5; ly += lineGap) {
      canvas.drawLine(
        Offset(x - lineGap, ly),
        Offset(x + lineGap, ly),
        linePaint,
      );
    }
    for (var ly = topLineY - lineGap; ly >= y - 0.5; ly -= lineGap) {
      canvas.drawLine(
        Offset(x - lineGap, ly),
        Offset(x + lineGap, ly),
        linePaint,
      );
    }
  }

  @override
  bool shouldRepaint(StaffPainter old) =>
      old.elapsedMs != elapsedMs ||
      old.activeNotes != activeNotes ||
      old.notes != notes;
}
