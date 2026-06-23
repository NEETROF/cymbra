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
import 'smufl.dart';

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

  // Bottom-line reference pitches: E4 (MIDI 64) for the treble staff,
  // G2 (MIDI 43) for the bass staff.
  static const int _trebleBottomPitch = 64;
  static const int _bassBottomPitch = 43;

  // Visible time window to the right of the playhead.
  static const double _lookAheadMs = 4000;

  @override
  void paint(Canvas canvas, Size size) {
    if (notes.isEmpty) return;

    const margin = 48.0;
    // A grand staff is drawn whenever any note belongs to the bass staff.
    final twoStaff = notes.any((n) => n.staff >= 2);
    final lineGap = (size.height * (twoStaff ? 0.055 : 0.10)).clamp(8.0, 18.0);
    final stepGap = lineGap / 2;

    // Playhead fixed at the left quarter; time advances toward the left.
    final playLineX = size.width * 0.25;
    final pxPerMs = (size.width - playLineX - margin) / _lookAheadMs;
    double xForTime(double tMs) => playLineX + (tMs - elapsedMs) * pxPerMs;

    // Vertical placement of the staff/staves.
    final double trebleBottom;
    final double? bassBottom;
    if (twoStaff) {
      final between = lineGap * 6; // gap between the two staves
      final blockHeight = 8 * lineGap + between;
      final top = (size.height - blockHeight) / 2 + 4 * lineGap;
      trebleBottom = top;
      bassBottom = trebleBottom + between + 4 * lineGap;
    } else {
      trebleBottom = size.height / 2 + 2 * lineGap;
      bassBottom = null;
    }

    final linePaint = Paint()
      ..color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.45)
      ..strokeWidth = 1.2;
    final barPaint = Paint()
      ..color = CymbraColors.onSurface.withValues(alpha: 0.5)
      ..strokeWidth = 1.4;

    // 1) Staff lines (treble, and bass for a grand staff).
    _drawStaffLines(
      canvas,
      trebleBottom,
      margin,
      size.width,
      lineGap,
      linePaint,
    );
    if (bassBottom != null) {
      _drawStaffLines(
        canvas,
        bassBottom,
        margin,
        size.width,
        lineGap,
        linePaint,
      );
    }

    final systemTop = trebleBottom - 4 * lineGap;
    final systemBottom = bassBottom ?? trebleBottom;

    // Clefs (SMuFL glyphs) at the head of the system.
    Smufl.draw(
      canvas,
      Smufl.gClef,
      6,
      trebleBottom - lineGap,
      lineGap,
      CymbraColors.onSurfaceVariant,
    );
    if (bassBottom != null) {
      Smufl.draw(
        canvas,
        Smufl.fClef,
        6,
        bassBottom - 3 * lineGap,
        lineGap,
        CymbraColors.onSurfaceVariant,
      );
    }

    // 2) Scrolling measure bars (span the whole system).
    final measureMs = (60000.0 / bpm) * 4; // 4 beats per measure
    if (measureMs > 0) {
      for (var t = 0.0; t <= songEndMs + measureMs; t += measureMs) {
        final x = xForTime(t);
        if (x < margin || x > size.width - margin) continue;
        canvas.drawLine(
          Offset(x, systemTop),
          Offset(x, systemBottom),
          barPaint,
        );
      }
    }

    // 3) Playhead (playback line) across the whole system.
    final playPaint = Paint()
      ..color = CymbraColors.secondary.withValues(alpha: 0.9)
      ..strokeWidth = 2;
    canvas.drawLine(
      Offset(playLineX, systemTop - lineGap * 1.5),
      Offset(playLineX, systemBottom + lineGap * 1.5),
      playPaint,
    );

    // 4) Scrolling notes, routed to their staff.
    for (final n in notes) {
      final x = xForTime(n.startMs.toDouble());
      if (x < margin - lineGap || x > size.width - margin + lineGap) continue;

      final isBass = bassBottom != null && n.staff >= 2;
      final base = isBass ? bassBottom : trebleBottom;
      final ref = isBass ? _bassBottomPitch : _trebleBottomPitch;
      final y = base - _staffSteps(n.pitch, ref) * stepGap;

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

      // Approximate the note value from its duration to choose flags (the
      // scrolling timeline carries no note-type), so eighths/sixteenths read.
      final quarterMs = bpm > 0 ? 60000.0 / bpm : 500.0;
      final ratio = n.durationMs / quarterMs;
      final flags = ratio <= 0.32 ? 2 : (ratio <= 0.62 ? 1 : 0);
      _drawNoteHead(canvas, Offset(x, y), lineGap, atPlayhead, color, flags);
      _drawLedgerLines(
        canvas,
        x,
        y,
        base,
        base - 4 * lineGap,
        lineGap,
        linePaint,
      );
    }
  }

  void _drawStaffLines(
    Canvas canvas,
    double bottomLineY,
    double margin,
    double width,
    double lineGap,
    Paint linePaint,
  ) {
    for (var i = 0; i < 5; i++) {
      final y = bottomLineY - i * lineGap;
      canvas.drawLine(Offset(margin, y), Offset(width - margin, y), linePaint);
    }
  }

  int _staffSteps(int pitch, int refPitch) =>
      _diatonic(pitch) - _diatonic(refPitch);

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
    int flags,
  ) {
    // Soft glow behind the note under the playhead.
    if (emphasized) {
      canvas.drawCircle(
        center,
        lineGap * 1.1,
        Paint()
          ..color = color.withValues(alpha: 0.45)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
      );
    }
    // SMuFL note head (stems always up on the scrolling staff).
    final headLeft = center.dx - Smufl.noteheadWidth * lineGap / 2;
    Smufl.draw(
      canvas,
      Smufl.noteheadBlack,
      headLeft,
      center.dy,
      lineGap,
      color,
    );

    final stemX =
        headLeft + Smufl.stemUpAnchorX * lineGap; // right side of the head
    final stemBottom = center.dy - Smufl.stemUpAnchorY * lineGap;
    final stemTop = stemBottom - lineGap * 3.2;
    canvas.drawLine(
      Offset(stemX, stemBottom),
      Offset(stemX, stemTop),
      Paint()
        ..color = color
        ..strokeWidth = Smufl.stemThickness * lineGap
        ..strokeCap = StrokeCap.round,
    );

    if (flags > 0) {
      final glyph = flags >= 2 ? Smufl.flag16thUp : Smufl.flag8thUp;
      Smufl.draw(canvas, glyph, stemX, stemTop, lineGap, color);
    }
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
