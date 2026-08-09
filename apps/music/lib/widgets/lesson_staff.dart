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

import 'package:flutter/foundation.dart' show listEquals, mapEquals;
import 'package:flutter/material.dart';

import '../courses/course_manifest.dart' show LessonStaffElement, LessonTimeSig;
import '../courses/lesson_pitch.dart';
import '../courses/lesson_rhythm.dart';
import '../painters/smufl.dart';
import '../state/note_label.dart';
import '../theme/cymbra_theme.dart';
import 'reading_aid.dart' show namingConventionOf;

/// The teaching staff every interactive solfège block draws on (change:
/// add-notation-courses, schema v2): five lines, a clef, an optional key/time
/// signature and hand-authored [elements] (notes and rests), with per-element
/// colour overrides for exercise feedback, an optional [ghost] preview note,
/// localized note-name [labels], and — when [onTapStep] is set — tap-to-step
/// hit-testing (the `placeNote` interaction).
///
/// Purpose-built for lessons rather than reusing the engraving painters: an
/// exercise needs arbitrary highlights, ghost previews and step hit-testing,
/// none of which the score-driven painters expose — while all the glyph work
/// comes from the shared [Smufl] toolbox, so it stays visually consistent.
class LessonStaff extends StatelessWidget {
  const LessonStaff({
    super.key,
    required this.clef,
    this.keyFifths = 0,
    this.time,
    this.elements = const [],
    this.elementColors = const {},
    this.labels = false,
    this.stacked = false,
    this.ghost,
    this.ghostColor,
    this.onTapStep,
    this.height = 128,
  });

  final LessonClef clef;
  final int keyFifths;
  final LessonTimeSig? time;
  final List<LessonStaffElement> elements;

  /// Per-element ink override (element index → colour) for exercise feedback —
  /// e.g. the awaited note in primary, a validated one in tertiary.
  final Map<int, Color> elementColors;

  /// Draw the localized note name under each pitched element.
  final bool labels;

  /// Draw every element at the same x (a chord) instead of left-to-right.
  final bool stacked;

  /// A translucent preview note (the `placeNote` finger-follow affordance).
  final LessonPitch? ghost;
  final Color? ghostColor;

  /// When set, a tap reports the nearest staff step (half staff-spaces above
  /// the bottom line, ledger territory included) and the staff is tappable.
  final ValueChanged<int>? onTapStep;

  final double height;

  /// Staff-space and bottom-line y for a paint area of [h] — one shared
  /// geometry for painting and hit-testing.
  static ({double s, double bottom}) geometryFor(double h) =>
      (s: h / 9, bottom: h * 0.66);

  /// The staff step nearest to a tap at [dy] in a paint area of [h], clamped to
  /// the teachable range (3 ledger positions out on each side).
  static int stepAt(double dy, double h) {
    final g = geometryFor(h);
    return ((g.bottom - dy) / (g.s / 2)).round().clamp(-6, 14);
  }

  @override
  Widget build(BuildContext context) {
    final naming = namingConventionOf(context);
    final painter = _LessonStaffPainter(
      clef: clef,
      keyFifths: keyFifths,
      time: time,
      elements: elements,
      elementColors: elementColors,
      labels: labels,
      stacked: stacked,
      ghost: ghost,
      ghostColor: ghostColor ?? CymbraColors.primary.withValues(alpha: 0.45),
      solfege: naming.solfege,
      frenchRe: naming.frenchRe,
    );
    final paint = CustomPaint(
      size: Size(double.infinity, height),
      painter: painter,
    );
    if (onTapStep == null) return SizedBox(height: height, child: paint);
    return SizedBox(
      height: height,
      child: GestureDetector(
        key: const Key('lesson-staff-tap'),
        behavior: HitTestBehavior.opaque,
        onTapDown: (d) => onTapStep!(stepAt(d.localPosition.dy, height)),
        child: paint,
      ),
    );
  }
}

class _LessonStaffPainter extends CustomPainter {
  _LessonStaffPainter({
    required this.clef,
    required this.keyFifths,
    required this.time,
    required this.elements,
    required this.elementColors,
    required this.labels,
    required this.stacked,
    required this.ghost,
    required this.ghostColor,
    required this.solfege,
    required this.frenchRe,
  });

  final LessonClef clef;
  final int keyFifths;
  final LessonTimeSig? time;
  final List<LessonStaffElement> elements;
  final Map<int, Color> elementColors;
  final bool labels;
  final bool stacked;
  final LessonPitch? ghost;
  final Color ghostColor;
  final bool solfege;
  final bool frenchRe;

  static const Color _ink = CymbraColors.onSurface;

  @override
  void paint(Canvas canvas, Size size) {
    final g = LessonStaff.geometryFor(size.height);
    final s = g.s;
    final bottom = g.bottom;
    const left = 8.0;
    final right = size.width - 8.0;

    final linePaint = Paint()
      ..color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.7)
      ..strokeWidth = Smufl.staffLineThickness * s;
    for (var i = 0; i < 5; i++) {
      final y = bottom - i * s;
      canvas.drawLine(Offset(left, y), Offset(right, y), linePaint);
    }

    // Clef: treble sits on line 2, bass on line 4.
    var x = left + s * 0.6;
    if (clef == LessonClef.treble) {
      Smufl.draw(canvas, Smufl.gClef, x, bottom - s, s, _ink);
    } else {
      Smufl.draw(canvas, Smufl.fClef, x, bottom - 3 * s, s, _ink);
    }
    x += 3.2 * s;
    x += Smufl.drawKeySignature(
      canvas,
      x,
      bottom,
      s,
      keyFifths,
      clef == LessonClef.bass,
      _ink,
    );
    final t = time;
    if (t != null) {
      x += Smufl.drawTimeSignature(
        canvas,
        x,
        bottom,
        s,
        t.beats,
        t.beatType,
        _ink,
      );
    }
    x += s * 0.8;

    // Elements, spread over the remaining width (or stacked as a chord).
    if (elements.isNotEmpty) {
      final span = right - s - x;
      final gap = stacked || elements.length == 1
          ? 0.0
          : span / (elements.length - 1);
      for (var i = 0; i < elements.length; i++) {
        final cx = stacked ? x + span / 2 : x + gap * i;
        _drawElement(canvas, i, elements[i], cx, bottom, s);
      }
    }

    final gp = ghost;
    if (gp != null) {
      final cx = x + (right - s - x) / 2;
      _drawNote(
        canvas,
        gp,
        const RhythmFigure(NoteFigure.quarter),
        cx,
        bottom,
        s,
        ghostColor,
      );
    }
  }

  void _drawElement(
    Canvas canvas,
    int index,
    LessonStaffElement e,
    double cx,
    double bottom,
    double s,
  ) {
    final color = elementColors[index] ?? _ink;
    if (e.fig.rest) {
      _drawRest(canvas, e.fig, cx, bottom, s, color);
      return;
    }
    final pitch = e.pitch!;
    _drawNote(canvas, pitch, e.fig, cx, bottom, s, color);
    if (labels) {
      final name = pitch.name.label(solfege: solfege, frenchRe: frenchRe);
      final tp = TextPainter(
        text: TextSpan(
          text: name,
          style: TextStyle(
            color: color == _ink ? CymbraColors.onSurfaceVariant : color,
            fontSize: (s * 1.15).clamp(9.0, 14.0),
            fontWeight: FontWeight.w600,
            fontFamilyFallback: const [Smufl.fontFamily],
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(canvas, Offset(cx - tp.width / 2, bottom + 2.2 * s));
    }
  }

  void _drawRest(
    Canvas canvas,
    RhythmFigure fig,
    double cx,
    double bottom,
    double s,
    Color color,
  ) {
    // SMuFL rest origins: the whole rest hangs from line 4, the others centre
    // on the middle line.
    final (glyph, y) = switch (fig.figure) {
      NoteFigure.whole => (Smufl.restWhole, bottom - 3 * s),
      NoteFigure.half => (Smufl.restHalf, bottom - 2 * s),
      NoteFigure.eighth => (Smufl.rest8th, bottom - 2 * s),
      NoteFigure.sixteenth => (Smufl.rest16th, bottom - 2 * s),
      _ => (Smufl.restQuarter, bottom - 2 * s),
    };
    Smufl.draw(canvas, glyph, cx, y, s, color, centerX: true);
    _drawDots(canvas, fig.dots, cx + s, bottom - 2.5 * s, s, color);
  }

  void _drawNote(
    Canvas canvas,
    LessonPitch pitch,
    RhythmFigure fig,
    double cx,
    double bottom,
    double s,
    Color color,
  ) {
    final step = pitch.staffStep(clef);
    final y = bottom - step * (s / 2);
    final headLeft = cx - Smufl.noteheadWidth * s / 2;

    // Ledger lines toward the note, above and below the staff.
    final ledgerPaint = Paint()
      ..color = color
      ..strokeWidth = Smufl.legerLineThickness * s;
    final ext = Smufl.legerLineExtension * s;
    for (var l = -2; l >= step; l -= 2) {
      final ly = bottom - l * (s / 2);
      canvas.drawLine(
        Offset(headLeft - ext, ly),
        Offset(headLeft + Smufl.noteheadWidth * s + ext, ly),
        ledgerPaint,
      );
    }
    for (var l = 10; l <= step; l += 2) {
      final ly = bottom - l * (s / 2);
      canvas.drawLine(
        Offset(headLeft - ext, ly),
        Offset(headLeft + Smufl.noteheadWidth * s + ext, ly),
        ledgerPaint,
      );
    }

    // Engrave the accidental only when the key signature does not already
    // carry it — the armure's whole point, kept visually honest in lessons.
    if (pitch.alter != keySignatureAlter(keyFifths, pitch.step)) {
      final glyph = switch (pitch.alter) {
        -2 => Smufl.accidentalDoubleFlat,
        -1 => Smufl.accidentalFlat,
        0 => Smufl.accidentalNatural,
        2 => Smufl.accidentalDoubleSharp,
        _ => Smufl.accidentalSharp,
      };
      Smufl.draw(canvas, glyph, headLeft - 1.3 * s, y, s, color);
    }

    final head = switch (fig.figure) {
      NoteFigure.whole => Smufl.noteheadWhole,
      NoteFigure.half => Smufl.noteheadHalf,
      _ => Smufl.noteheadBlack,
    };
    Smufl.draw(canvas, head, headLeft, y, s, color);
    _drawDots(
      canvas,
      fig.dots,
      headLeft + Smufl.noteheadWidth * s,
      // A dot sits in the space above a line-note, beside a space-note.
      step.isEven ? y - s / 2 : y,
      s,
      color,
    );

    if (fig.figure == NoteFigure.whole) return;

    // Stem: up below the middle line, down at and above it.
    final stemUp = step < 4;
    final stemPaint = Paint()
      ..color = color
      ..strokeWidth = Smufl.stemThickness * s
      ..strokeCap = StrokeCap.round;
    final double stemX;
    final double tipY;
    if (stemUp) {
      stemX = headLeft + Smufl.stemUpAnchorX * s;
      final baseY = y - Smufl.stemUpAnchorY * s;
      tipY = baseY - 3.2 * s;
      canvas.drawLine(Offset(stemX, baseY), Offset(stemX, tipY), stemPaint);
    } else {
      stemX = headLeft + Smufl.stemDownAnchorX * s;
      final baseY = y - Smufl.stemDownAnchorY * s;
      tipY = baseY + 3.2 * s;
      canvas.drawLine(Offset(stemX, baseY), Offset(stemX, tipY), stemPaint);
    }

    final flag = switch (fig.figure) {
      NoteFigure.eighth => stemUp ? Smufl.flag8thUp : Smufl.flag8thDown,
      NoteFigure.sixteenth => stemUp ? Smufl.flag16thUp : Smufl.flag16thDown,
      _ => null,
    };
    if (flag != null) Smufl.draw(canvas, flag, stemX, tipY, s, color);
  }

  void _drawDots(
    Canvas canvas,
    int dots,
    double x,
    double y,
    double s,
    Color color,
  ) {
    for (var d = 0; d < dots; d++) {
      Smufl.draw(
        canvas,
        Smufl.augmentationDot,
        x + 0.35 * s + d * 0.6 * s,
        y,
        s,
        color,
      );
    }
  }

  @override
  bool shouldRepaint(_LessonStaffPainter old) =>
      old.clef != clef ||
      old.keyFifths != keyFifths ||
      old.time != time ||
      !listEquals(old.elements, elements) ||
      !mapEquals(old.elementColors, elementColors) ||
      old.labels != labels ||
      old.stacked != stacked ||
      old.ghost != ghost ||
      old.ghostColor != ghostColor ||
      old.solfege != solfege ||
      old.frenchRe != frenchRe;
}
