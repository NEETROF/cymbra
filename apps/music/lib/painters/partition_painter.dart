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

import '../src/rust/api/musicxml.dart';
import '../theme/cymbra_theme.dart';

/// Draws engraved notation ("Partition" mode) from a parsed [ScoreDocument] and
/// its laid-out [System]s.
///
/// Each system is a grand staff (treble + bass for a 2-staff piano part); within
/// a measure, notes are placed by their `position_divisions` so the two staves
/// stay vertically aligned. Renders note heads (with dots and accidentals),
/// stems, rests, lyrics, and word/dynamics directions. Geometry only — no
/// playback or interaction.
class PartitionPainter extends CustomPainter {
  final ScoreDocument document;
  final List<System> systems;

  PartitionPainter({required this.document, required this.systems});

  static const double _gap = 9; // staff line spacing
  static const double _staffHeight = 4 * _gap;
  static const double _interStaff = 7 * _gap; // treble bottom → bass top
  static const double _topPad = 4 * _gap; // room for words/dynamics above
  static const double _bottomPad = 4 * _gap; // room for lyrics below
  static const double _systemGap = 3 * _gap; // between systems
  static const double _notePad = 18; // inset for notes inside a measure

  /// Diatonic step order within an octave (C=0 … B=6).
  static const Map<String, int> _stepOrder = {
    'C': 0,
    'D': 1,
    'E': 2,
    'F': 3,
    'G': 4,
    'A': 5,
    'B': 6,
  };

  /// Total height needed to draw all systems at the given [width]; used by the
  /// screen to size the scrollable canvas.
  double heightFor(double width) {
    final systemHeight =
        _topPad +
        _staffHeight +
        (document.staves >= 2 ? _interStaff + _staffHeight : 0) +
        _bottomPad;
    return systems.length * (systemHeight + _systemGap) + _systemGap;
  }

  bool get _twoStaff => document.staves >= 2;

  int _divisionsPerMeasure() {
    final a = document.attributes;
    final beatType = a.time.beatType == 0 ? 4 : a.time.beatType;
    final perMeasure = a.divisions * a.time.beats * 4 ~/ beatType;
    return perMeasure > 0 ? perMeasure : a.divisions * 4;
  }

  @override
  void paint(Canvas canvas, Size size) {
    if (systems.isEmpty) return;
    final width = size.width;
    final divPerMeasure = _divisionsPerMeasure();

    final systemHeight =
        _topPad +
        _staffHeight +
        (_twoStaff ? _interStaff + _staffHeight : 0) +
        _bottomPad;

    var y = _systemGap;
    for (final system in systems) {
      _paintSystem(canvas, system, width, y, divPerMeasure);
      y += systemHeight + _systemGap;
    }
  }

  void _paintSystem(
    Canvas canvas,
    System system,
    double width,
    double yTop,
    int divPerMeasure,
  ) {
    final trebleTop = yTop + _topPad;
    final trebleBottom = trebleTop + _staffHeight;
    final bassTop = trebleBottom + _interStaff;
    final bassBottom = bassTop + _staffHeight;

    final linePaint = Paint()
      ..color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.5)
      ..strokeWidth = 1.1;
    final barPaint = Paint()
      ..color = CymbraColors.onSurface.withValues(alpha: 0.6)
      ..strokeWidth = 1.3;

    // Staff lines spanning the whole system width.
    _drawStaffLines(canvas, trebleTop, width, linePaint);
    if (_twoStaff) _drawStaffLines(canvas, bassTop, width, linePaint);

    // Left and right system barlines (connect the grand staff).
    final connectBottom = _twoStaff ? bassBottom : trebleBottom;
    canvas.drawLine(
      Offset(0.5, trebleTop),
      Offset(0.5, connectBottom),
      barPaint,
    );

    // Clef hint at the head of each system.
    _drawText(canvas, _clefGlyph(1), Offset(3, trebleTop - 2), size: 20);
    if (_twoStaff) {
      _drawText(canvas, _clefGlyph(2), Offset(3, bassTop - 2), size: 20);
    }

    // Measures, scaled so the system fills the available width (justified).
    final indices = system.measures;
    var totalMin = 0.0;
    for (final idx in indices) {
      totalMin += document.measures[idx].minWidth;
    }
    final scale = totalMin > 0 ? width / totalMin : 1.0;

    var x = 0.0;
    for (final idx in indices) {
      final measure = document.measures[idx];
      final mWidth = measure.minWidth * scale;
      // Trailing barline.
      canvas.drawLine(
        Offset(x + mWidth, trebleTop),
        Offset(x + mWidth, connectBottom),
        barPaint,
      );
      _paintMeasure(
        canvas,
        measure,
        x,
        mWidth,
        divPerMeasure,
        trebleBottom,
        bassBottom,
      );
      x += mWidth;
    }
  }

  void _drawStaffLines(Canvas canvas, double top, double width, Paint paint) {
    for (var i = 0; i < 5; i++) {
      final y = top + i * _gap;
      canvas.drawLine(Offset(0, y), Offset(width, y), paint);
    }
  }

  void _paintMeasure(
    Canvas canvas,
    NotationMeasure measure,
    double measureX,
    double measureWidth,
    int divPerMeasure,
    double trebleBottom,
    double bassBottom,
  ) {
    double xForPosition(int position) {
      final frac = divPerMeasure > 0
          ? (position / divPerMeasure).clamp(0.0, 1.0)
          : 0.0;
      return measureX + _notePad + frac * (measureWidth - 2 * _notePad);
    }

    // Directions: words above the treble staff, dynamics just below it.
    for (final dir in measure.directions) {
      final x = xForPosition(dir.positionDivisions);
      switch (dir.kind) {
        case DirectionKind_Words(:final field0):
          _drawText(
            canvas,
            field0,
            Offset(x, trebleBottom - _staffHeight - _gap * 2.4),
            color: CymbraColors.onSurfaceVariant,
            italic: true,
          );
        case DirectionKind_Dynamics(:final field0):
          _drawText(
            canvas,
            field0,
            Offset(x, trebleBottom + _gap * 0.4),
            color: CymbraColors.secondary,
            italic: true,
            size: 14,
          );
        case DirectionKind_Wedge():
        case DirectionKind_Metronome():
          break;
      }
    }

    for (final note in measure.notes) {
      final isBass = note.staff >= 2 && _twoStaff;
      final bottom = isBass ? bassBottom : trebleBottom;
      final x = xForPosition(note.positionDivisions);

      if (note.isRest) {
        _drawRest(canvas, x, bottom - _staffHeight / 2);
        continue;
      }
      final pitch = note.pitch;
      if (pitch == null) continue;

      final y = _yForPitch(pitch, bottom, isBass);
      _drawLedgerLines(canvas, x, y, bottom);
      _drawNoteHead(canvas, x, y, note);
      _drawStem(canvas, x, y, note);
      _drawDots(canvas, x, y, note.dots);
      if (note.accidental != null) {
        _drawText(
          canvas,
          _accidentalGlyph(note.accidental!),
          Offset(x - _gap * 2.1, y - _gap),
          size: 15,
        );
      }
      final lyric = note.lyric;
      if (lyric != null) {
        _drawText(
          canvas,
          lyric.text,
          Offset(x - _gap, bottom + _gap * 1.4),
          size: 12,
          color: CymbraColors.onSurface,
        );
      }
    }
  }

  /// Bottom staff line reference pitch: E4 (treble) / G2 (bass).
  double _yForPitch(Pitch pitch, double bottomLineY, bool isBass) {
    final diatonic = pitch.octave * 7 + (_stepOrder[pitch.step] ?? 0);
    final refDiatonic = isBass ? (2 * 7 + 4) : (4 * 7 + 2);
    return bottomLineY - (diatonic - refDiatonic) * (_gap / 2);
  }

  void _drawNoteHead(Canvas canvas, double x, double y, NoteEvent note) {
    final rx = _gap * 0.62;
    final ry = _gap * 0.46;
    final hollow =
        note.noteType == 'whole' ||
        note.noteType == 'half' ||
        note.noteType == null && note.durationDivisions >= 8;
    final rect = Rect.fromCenter(
      center: Offset(x, y),
      width: rx * 2,
      height: ry * 2,
    );
    final paint = Paint()..color = CymbraColors.onSurface;
    if (hollow) {
      paint
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.6;
    }
    canvas.drawOval(rect, paint);
  }

  void _drawStem(Canvas canvas, double x, double y, NoteEvent note) {
    if (note.noteType == 'whole') return;
    final up = note.stem != StemDir.down;
    final rx = _gap * 0.62;
    final dx = up ? x + rx : x - rx;
    final dy = up ? y - _gap * 3 : y + _gap * 3;
    canvas.drawLine(
      Offset(dx, y),
      Offset(dx, dy),
      Paint()
        ..color = CymbraColors.onSurface
        ..strokeWidth = 1.4,
    );
  }

  void _drawDots(Canvas canvas, double x, double y, int dots) {
    final paint = Paint()..color = CymbraColors.onSurface;
    for (var i = 0; i < dots; i++) {
      canvas.drawCircle(
        Offset(x + _gap * (1.3 + i * 0.6), y),
        _gap * 0.16,
        paint,
      );
    }
  }

  void _drawRest(Canvas canvas, double x, double y) {
    canvas.drawRect(
      Rect.fromCenter(center: Offset(x, y), width: _gap * 1.1, height: _gap),
      Paint()..color = CymbraColors.onSurfaceVariant,
    );
  }

  /// Ledger lines for note heads above/below their staff.
  void _drawLedgerLines(Canvas canvas, double x, double y, double bottomLineY) {
    final topLineY = bottomLineY - _staffHeight;
    final paint = Paint()
      ..color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.6)
      ..strokeWidth = 1.1;
    for (var ly = bottomLineY + _gap; ly <= y + 0.5; ly += _gap) {
      canvas.drawLine(Offset(x - _gap, ly), Offset(x + _gap, ly), paint);
    }
    for (var ly = topLineY - _gap; ly >= y - 0.5; ly -= _gap) {
      canvas.drawLine(Offset(x - _gap, ly), Offset(x + _gap, ly), paint);
    }
  }

  String _clefGlyph(int staff) {
    final clef = document.attributes.clefs
        .where((c) => c.staff == staff)
        .firstOrNull;
    final sign = clef?.sign ?? (staff >= 2 ? 'F' : 'G');
    return switch (sign) {
      'F' => '𝄢',
      'C' => '𝄡',
      _ => '𝄞',
    };
  }

  String _accidentalGlyph(String accidental) => switch (accidental) {
    'flat' => '♭',
    'sharp' => '♯',
    'natural' => '♮',
    'double-sharp' => '𝄪',
    'flat-flat' => '𝄫',
    _ => '',
  };

  void _drawText(
    Canvas canvas,
    String text,
    Offset offset, {
    double size = 13,
    Color color = CymbraColors.onSurface,
    bool italic = false,
  }) {
    if (text.isEmpty) return;
    final tp = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          color: color,
          fontSize: size,
          fontStyle: italic ? FontStyle.italic : FontStyle.normal,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(canvas, offset);
  }

  @override
  bool shouldRepaint(PartitionPainter old) =>
      old.document != document || old.systems != systems;
}
