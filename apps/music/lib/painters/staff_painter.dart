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

import '../src/rust/api/musicxml.dart' show BeamState;
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

  /// Key signature (fifths) and time signature of the loaded piece, drawn as the
  /// armature + meter at the head of the system.
  final int keyFifths;
  final int beats;
  final int beatType;

  const StaffPainter({
    required this.notes,
    required this.elapsedMs,
    required this.activeNotes,
    required this.bpm,
    required this.songEndMs,
    this.keyFifths = 0,
    this.beats = 4,
    this.beatType = 4,
  });

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

    // Clefs (SMuFL glyphs) at the head of the system — the clef in effect at
    // the playhead, so a mid-piece clef change is reflected as you scroll.
    final trebleClef = _clefAtPlayhead(1);
    Smufl.draw(
      canvas,
      Smufl.clef(trebleClef.$1),
      6,
      trebleBottom - (trebleClef.$2 - 1) * lineGap,
      lineGap,
      CymbraColors.onSurfaceVariant,
    );
    if (bassBottom != null) {
      final bassClef = _clefAtPlayhead(2);
      Smufl.draw(
        canvas,
        Smufl.clef(bassClef.$1),
        6,
        bassBottom - (bassClef.$2 - 1) * lineGap,
        lineGap,
        CymbraColors.onSurfaceVariant,
      );
    }

    // Key signature (armature) + time signature at the head of the system.
    const headColor = CymbraColors.onSurfaceVariant;
    var hx = 6 + lineGap * 2.8;
    final keyW = Smufl.drawKeySignature(
      canvas,
      hx,
      trebleBottom,
      lineGap,
      keyFifths,
      false,
      headColor,
    );
    if (bassBottom != null) {
      Smufl.drawKeySignature(
        canvas,
        hx,
        bassBottom,
        lineGap,
        keyFifths,
        true,
        headColor,
      );
    }
    hx += keyW;
    Smufl.drawTimeSignature(
      canvas,
      hx,
      trebleBottom,
      lineGap,
      beats,
      beatType,
      headColor,
    );
    if (bassBottom != null) {
      Smufl.drawTimeSignature(
        canvas,
        hx,
        bassBottom,
        lineGap,
        beats,
        beatType,
        headColor,
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

    // Vertical position of a note on its staff.
    double noteY(TimedNote n) {
      final isBass = bassBottom != null && n.staff >= 2;
      final base = isBass ? bassBottom : trebleBottom;
      // Position by the clef in effect for this note (not its staff index).
      final bottom = _clefBottomDiatonic(n.clefSign, n.clefLine);
      return base - (_diatonic(n.pitch) - bottom) * stepGap;
    }

    final quarterMs = bpm > 0 ? 60000.0 / bpm : 500.0;
    int flagsOf(TimedNote n) {
      final ratio = n.durationMs / quarterMs;
      return ratio <= 0.32 ? 2 : (ratio <= 0.62 ? 1 : 0);
    }

    // Beam groups carried from the notation (per staff). Members get a beam
    // instead of individual flags.
    final beamGroups = <List<TimedNote>>[];
    final openGroups = <int, List<TimedNote>>{};
    for (final n in notes) {
      if (n.beams.isEmpty) continue;
      final g = openGroups.putIfAbsent(n.staff, () => <TimedNote>[]);
      g.add(n);
      if (n.beams.contains(BeamState.end)) {
        beamGroups.add(g);
        openGroups.remove(n.staff);
      }
    }
    beamGroups.addAll(openGroups.values);
    final beamed = beamGroups.expand((g) => g).toSet();

    bool visible(double x) =>
        x >= margin - lineGap && x <= size.width - margin + lineGap;

    Color colorFor(TimedNote n) {
      final atPlayhead =
          n.startMs <= elapsedMs && elapsedMs < n.startMs + n.durationMs;
      if (atPlayhead && activeNotes.contains(n.pitch)) {
        return CymbraColors.tertiary; // well played
      } else if (atPlayhead) {
        return CymbraColors.secondary; // expected
      }
      return CymbraColors.onSurfaceVariant.withValues(alpha: 0.55);
    }

    // 4) Scrolling notes, routed to their staff.
    for (final n in notes) {
      final x = xForTime(n.startMs.toDouble());
      if (!visible(x)) continue;
      final y = noteY(n);
      final atPlayhead =
          n.startMs <= elapsedMs && elapsedMs < n.startMs + n.durationMs;
      final color = colorFor(n);

      _drawHead(canvas, Offset(x, y), lineGap, atPlayhead, color);
      // Beamed notes get their stems/beam from the group pass; others stem now.
      if (!beamed.contains(n)) {
        _drawStemFlag(canvas, Offset(x, y), lineGap, color, flagsOf(n));
      }
      final isBass = bassBottom != null && n.staff >= 2;
      final base = isBass ? bassBottom : trebleBottom;
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

    // 5) Beams over their groups (stems up, one straight beam each).
    for (final group in beamGroups) {
      if (group.length < 2) {
        if (group.length == 1 &&
            visible(xForTime(group.first.startMs.toDouble()))) {
          final n = group.first;
          _drawStemFlag(
            canvas,
            Offset(xForTime(n.startMs.toDouble()), noteY(n)),
            lineGap,
            colorFor(n),
            flagsOf(n),
          );
        }
        continue;
      }
      final pts = group
          .map((n) => Offset(xForTime(n.startMs.toDouble()), noteY(n)))
          .toList();
      if (pts.every((p) => !visible(p.dx))) continue;
      _drawBeam(canvas, pts, group, lineGap);
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

  /// Diatonic value of the bottom staff line for a clef (sign on its `line`).
  /// Uses MIDI reference pitches so it matches [_diatonic] (which keys on MIDI
  /// numbers, not the musical octave).
  int _clefBottomDiatonic(String sign, int line) {
    final refMidi = switch (sign) {
      'F' => 53, // F3
      'C' => 60, // C4
      _ => 67, // G4
    };
    return _diatonic(refMidi) - (line - 1) * 2;
  }

  /// The clef (sign, line) in effect on [staff] at the current playhead — the
  /// latest note at/before [elapsedMs], else the first note on that staff.
  (String, int) _clefAtPlayhead(int staff) {
    TimedNote? chosen;
    for (final n in notes) {
      if (n.staff != staff) continue;
      chosen ??= n; // fallback: first note on the staff
      if (n.startMs <= elapsedMs) chosen = n; // latest before the playhead
    }
    if (chosen == null) return (staff >= 2 ? 'F' : 'G', staff >= 2 ? 4 : 2);
    return (chosen.clefSign, chosen.clefLine);
  }

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

  /// SMuFL note head (plus a soft glow when under the playhead).
  void _drawHead(
    Canvas canvas,
    Offset center,
    double lineGap,
    bool emphasized,
    Color color,
  ) {
    if (emphasized) {
      canvas.drawCircle(
        center,
        lineGap * 1.1,
        Paint()
          ..color = color.withValues(alpha: 0.45)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
      );
    }
    final headLeft = center.dx - Smufl.noteheadWidth * lineGap / 2;
    Smufl.draw(
      canvas,
      Smufl.noteheadBlack,
      headLeft,
      center.dy,
      lineGap,
      color,
    );
  }

  /// Up-stem and (for unbeamed notes) flags, attached at the head anchor.
  void _drawStemFlag(
    Canvas canvas,
    Offset center,
    double lineGap,
    Color color,
    int flags,
  ) {
    final stemX = _stemX(center, lineGap);
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

  double _stemX(Offset center, double lineGap) =>
      center.dx -
      Smufl.noteheadWidth * lineGap / 2 +
      Smufl.stemUpAnchorX * lineGap;

  /// One straight beam over a group of note heads, with stems of varying length
  /// reaching it (matching the Partition engraving), plus a secondary beam for
  /// consecutive sixteenths.
  void _drawBeam(
    Canvas canvas,
    List<Offset> pts,
    List<TimedNote> group,
    double lineGap,
  ) {
    final color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.75);
    final quarterMs = bpm > 0 ? 60000.0 / bpm : 500.0;
    final stemPaint = Paint()
      ..color = color
      ..strokeWidth = Smufl.stemThickness * lineGap
      ..strokeCap = StrokeCap.round;

    var beamY = double.infinity;
    final stemBottoms = <double>[];
    final stemXs = <double>[];
    for (final p in pts) {
      final sb = p.dy - Smufl.stemUpAnchorY * lineGap;
      stemBottoms.add(sb);
      stemXs.add(_stemX(p, lineGap));
      final top = sb - lineGap * 3.2;
      if (top < beamY) beamY = top;
    }
    for (var i = 0; i < pts.length; i++) {
      canvas.drawLine(
        Offset(stemXs[i], stemBottoms[i]),
        Offset(stemXs[i], beamY),
        stemPaint,
      );
    }
    canvas.drawLine(
      Offset(stemXs.first, beamY),
      Offset(stemXs.last, beamY),
      Paint()
        ..color = color
        ..strokeWidth = Smufl.beamThickness * lineGap,
    );
    // Secondary beam between consecutive sixteenths (duration < a third beat).
    bool isSixteenth(TimedNote n) => n.durationMs / quarterMs <= 0.32;
    final off = (Smufl.beamThickness + 0.2) * lineGap;
    for (var i = 0; i < group.length - 1; i++) {
      if (isSixteenth(group[i]) && isSixteenth(group[i + 1])) {
        canvas.drawLine(
          Offset(stemXs[i], beamY + off),
          Offset(stemXs[i + 1], beamY + off),
          Paint()
            ..color = color
            ..strokeWidth = Smufl.beamThickness * lineGap,
        );
      }
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
