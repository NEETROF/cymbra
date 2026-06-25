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
import 'smufl.dart';

/// Draws engraved notation ("Partition" mode) from a parsed [ScoreDocument] and
/// its laid-out [System]s, using SMuFL/Bravura glyphs for note heads, clefs,
/// flags, accidentals, rests and dynamics. Stems, beams, staff and ledger lines
/// are stroked at Bravura's engraving thicknesses; stem attachment uses the
/// font's note-head anchors. During playback it also draws a playhead cursor and
/// highlights the notes at the playhead (see [elapsedMs]/[measureStartMs]).
class PartitionPainter extends CustomPainter {
  final ScoreDocument document;
  final List<System> systems;

  /// Playback playhead (ms) and per-measure start times, so the cursor and note
  /// highlighting track the current position. [activeNotes] are the held MIDI
  /// pitches (a highlighted note reads "correct" when held, else "expected").
  final double elapsedMs;
  final List<int> measureStartMs;
  final double songEndMs;
  final Set<int> activeNotes;

  PartitionPainter({
    required this.document,
    required this.systems,
    this.elapsedMs = 0,
    this.measureStartMs = const [],
    this.songEndMs = 0,
    this.activeNotes = const {},
  });

  static const Map<String, int> _semitoneOfStep = {
    'C': 0,
    'D': 2,
    'E': 4,
    'F': 5,
    'G': 7,
    'A': 9,
    'B': 11,
  };

  int _midiOf(Pitch p) =>
      (p.octave + 1) * 12 + (_semitoneOfStep[p.step] ?? 0) + p.alter;

  /// The measure index containing the playhead and the fraction within it, or
  /// null when there is no timing (demo score) or the playhead is out of range.
  ({int index, double fraction})? get _cursor {
    final starts = measureStartMs;
    if (starts.isEmpty || elapsedMs < starts.first) return null;
    for (var i = 0; i < starts.length; i++) {
      final start = starts[i];
      final end = (i + 1 < starts.length ? starts[i + 1] : songEndMs)
          .toDouble();
      if (elapsedMs >= start && elapsedMs < end) {
        final span = end - start;
        final frac = span > 0
            ? ((elapsedMs - start) / span).clamp(0.0, 1.0)
            : 0.0;
        return (index: i, fraction: frac);
      }
    }
    return null;
  }

  /// Staff space (distance between two staff lines), in pixels. Everything else
  /// is derived from it so the engraving scales as one unit.
  static const double _s = 12;
  static const double _staffHeight = 4 * _s;
  static const double _interStaff = 8 * _s; // treble bottom → bass top
  static const double _topPad = 5 * _s; // words/dynamics above
  static const double _bottomPad = 4.5 * _s; // lyrics below
  static const double _systemGap = 3 * _s;
  static const double _stemLen = 3.5; // staff spaces

  static const Map<String, int> _stepOrder = {
    'C': 0,
    'D': 1,
    'E': 2,
    'F': 3,
    'G': 4,
    'A': 5,
    'B': 6,
  };

  Color get _ink => CymbraColors.onSurface;

  bool get _twoStaff => document.staves >= 2;

  double get _systemHeight =>
      _topPad +
      _staffHeight +
      (_twoStaff ? _interStaff + _staffHeight : 0) +
      _bottomPad;

  double heightFor(double width) =>
      systems.length * (_systemHeight + _systemGap) + _systemGap;

  /// Vertical distance between consecutive system tops (matches `paint`).
  double get systemStride => _systemHeight + _systemGap;

  /// Y of the top of system [index] in the scrollable content (matches `paint`).
  double systemTopY(int index) => _systemGap + index * systemStride;

  int _divisionsPerMeasure() {
    final a = document.attributes;
    final beatType = a.time.beatType == 0 ? 4 : a.time.beatType;
    final perMeasure = a.divisions * a.time.beats * 4 ~/ beatType;
    return perMeasure > 0 ? perMeasure : a.divisions * 4;
  }

  @override
  void paint(Canvas canvas, Size size) {
    if (systems.isEmpty) return;
    final divPerMeasure = _divisionsPerMeasure();
    final clefAt = _computeClefAt();
    final cursor = _cursor;
    var y = _systemGap;
    for (var i = 0; i < systems.length; i++) {
      _paintSystem(
        canvas,
        systems[i],
        size.width,
        y,
        divPerMeasure,
        i == 0,
        clefAt,
        cursor,
      );
      y += _systemHeight + _systemGap;
    }
  }

  /// Clef in effect per staff for each measure index, honouring mid-piece clef
  /// changes (a measure's `clefs` override the running clef from its start).
  List<Map<int, Clef>> _computeClefAt() {
    final running = <int, Clef>{};
    for (final c in document.attributes.clefs) {
      running[c.staff] = c;
    }
    final out = <Map<int, Clef>>[];
    for (final m in document.measures) {
      for (final c in m.clefs) {
        running[c.staff] = c;
      }
      out.add(Map<int, Clef>.from(running));
    }
    return out;
  }

  Clef _clefFor(Map<int, Clef> clefs, int staff) =>
      clefs[staff] ??
      (staff >= 2
          ? const Clef(staff: 2, sign: 'F', line: 4)
          : const Clef(staff: 1, sign: 'G', line: 2));

  void _paintSystem(
    Canvas canvas,
    System system,
    double width,
    double yTop,
    int divPerMeasure,
    bool isFirst,
    List<Map<int, Clef>> clefAt,
    ({int index, double fraction})? cursor,
  ) {
    final trebleBottom = yTop + _topPad + _staffHeight;
    final bassBottom = trebleBottom + _interStaff + _staffHeight;
    final systemBottom = _twoStaff ? bassBottom : trebleBottom;
    final headerClefs = clefAt[system.measures.first];
    final words = _TextLanes(_s * 1.3);
    final arcs = _Arcs();

    final linePaint = Paint()
      ..color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.7)
      ..strokeWidth = Smufl.staffLineThickness * _s;
    final barPaint = Paint()
      ..color = _ink.withValues(alpha: 0.7)
      ..strokeWidth = Smufl.thinBarlineThickness * _s;

    _drawStaffLines(canvas, trebleBottom, width, linePaint);
    if (_twoStaff) _drawStaffLines(canvas, bassBottom, width, linePaint);

    final systemTop = trebleBottom - _staffHeight;
    // Left system bracket connecting the grand staff (a single brace glyph does
    // not stretch cleanly, so a thick rounded bar is used instead).
    canvas.drawLine(
      Offset(1.2, systemTop),
      Offset(1.2, systemBottom),
      Paint()
        ..color = _ink.withValues(alpha: 0.8)
        ..strokeWidth = _s * 0.35
        ..strokeCap = StrokeCap.round,
    );

    // --- Header: clef (in effect here), key signature, time signature. ---
    _drawClef(canvas, _clefFor(headerClefs, 1), _s * 0.4, trebleBottom, _s);
    if (_twoStaff) {
      _drawClef(canvas, _clefFor(headerClefs, 2), _s * 0.4, bassBottom, _s);
    }
    var hx = _s * 3.0; // after the clef

    final fifths = document.attributes.keyFifths;
    final keyWidth = Smufl.drawKeySignature(
      canvas,
      hx,
      trebleBottom,
      _s,
      fifths,
      false,
      _ink,
    );
    if (_twoStaff) {
      Smufl.drawKeySignature(canvas, hx, bassBottom, _s, fifths, true, _ink);
    }
    hx += keyWidth;

    if (isFirst) {
      final time = document.attributes.time;
      final timeWidth = Smufl.drawTimeSignature(
        canvas,
        hx,
        trebleBottom,
        _s,
        time.beats,
        time.beatType,
        _ink,
      );
      if (_twoStaff) {
        Smufl.drawTimeSignature(
          canvas,
          hx,
          bassBottom,
          _s,
          time.beats,
          time.beatType,
          _ink,
        );
      }
      hx += timeWidth;
    }
    final headerX = hx + _s * 0.6;

    // Measures justified to fill the line width after the header.
    final indices = system.measures;
    var totalMin = 0.0;
    for (final idx in indices) {
      totalMin += document.measures[idx].minWidth;
    }
    final usable = width - headerX;
    final scale = totalMin > 0 ? usable / totalMin : 1.0;

    var x = headerX;
    double? cursorX; // set when the active measure is in this system
    for (var k = 0; k < indices.length; k++) {
      final idx = indices[k];
      final measure = document.measures[idx];
      final mWidth = measure.minWidth * scale;
      canvas.drawLine(
        Offset(x + mWidth, systemTop),
        Offset(x + mWidth, systemBottom),
        barPaint,
      );
      final isCursorMeasure = cursor != null && cursor.index == idx;
      if (isCursorMeasure) cursorX = x + cursor.fraction * mWidth;
      _paintMeasure(
        canvas,
        measure,
        x,
        mWidth,
        divPerMeasure,
        trebleBottom,
        bassBottom,
        clefAt[idx],
        k == 0, // first measure of the system → clef already in the header
        words,
        arcs,
        isCursorMeasure ? cursor.fraction * divPerMeasure : null,
      );
      x += mWidth;
    }

    // Playhead cursor, drawn over the system's staves.
    if (cursorX != null) {
      canvas.drawLine(
        Offset(cursorX, systemTop),
        Offset(cursorX, systemBottom),
        Paint()
          ..color = CymbraColors.secondary
          ..strokeWidth = _s * 0.18
          ..strokeCap = StrokeCap.round,
      );
    }
  }

  /// Draws a clef glyph for [clef] on the staff whose bottom line is at
  /// [staffBottom] (the clef sign sits on its `line`).
  void _drawClef(
    Canvas canvas,
    Clef clef,
    double x,
    double staffBottom,
    double size,
  ) {
    final baselineY = staffBottom - (clef.line - 1) * size;
    Smufl.draw(canvas, Smufl.clef(clef.sign), x, baselineY, size, _ink);
  }

  void _drawStaffLines(
    Canvas canvas,
    double bottom,
    double width,
    Paint paint,
  ) {
    for (var i = 0; i < 5; i++) {
      final y = bottom - i * _s;
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
    Map<int, Clef> clefs,
    bool isSystemFirst,
    _TextLanes words,
    _Arcs arcs,
    double? cursorDiv,
  ) {
    // A mid-system clef change is drawn at the measure start and reserves space.
    final showClefChange = !isSystemFirst && measure.clefs.isNotEmpty;
    final clefLead = showClefChange ? _s * 2.6 : 0.0;
    if (showClefChange) {
      for (final c in measure.clefs) {
        final sb = c.staff >= 2 && _twoStaff ? bassBottom : trebleBottom;
        _drawClef(canvas, c, measureX + _s * 0.3, sb, _s * 0.9);
      }
    }

    double xForPosition(int position) {
      final frac = divPerMeasure > 0
          ? (position / divPerMeasure).clamp(0.0, 1.0)
          : 0.0;
      final left = measureX + _s + clefLead;
      return left + frac * (measureWidth - clefLead - 2.4 * _s);
    }

    for (final dir in measure.directions) {
      final x = xForPosition(dir.positionDivisions);
      switch (dir.kind) {
        case DirectionKind_Words(:final field0):
          // Stack overlapping words onto separate rows above the staff.
          final w = _textWidth(field0, _s * 1.05, italic: true);
          final baseY = trebleBottom - _staffHeight - _s * 1.8;
          final y = words.yFor(x, w + _s * 0.6, baseY);
          _text(
            canvas,
            field0,
            x,
            y,
            italic: true,
            color: CymbraColors.onSurfaceVariant,
          );
        case DirectionKind_Dynamics(:final field0):
          // Dynamics sit a little below note-head size (≈ 0.78 staff spaces).
          Smufl.draw(
            canvas,
            Smufl.dynamics(field0),
            x,
            trebleBottom + _s * 2.2,
            _s * 0.78,
            CymbraColors.secondary,
          );
        case DirectionKind_Wedge():
        case DirectionKind_Metronome():
          break;
      }
    }

    final beamGroups = <String, List<_Note>>{};
    final openTuplets = <String, _TupletAcc>{};

    for (final note in measure.notes) {
      final isBass = note.staff >= 2 && _twoStaff;
      final staffBottom = isBass ? bassBottom : trebleBottom;
      final x = xForPosition(note.positionDivisions);

      if (note.isRest) {
        Smufl.draw(
          canvas,
          _restGlyph(note),
          x,
          staffBottom - 2 * _s,
          _s,
          _ink,
          centerX: true,
        );
        continue;
      }
      final pitch = note.pitch;
      if (pitch == null) continue;

      final y = _yForPitch(pitch, staffBottom, _clefFor(clefs, note.staff));
      _drawLedgerLines(canvas, x, y, staffBottom);

      // Highlight the note when the playhead is within its time window: it reads
      // "correct" if its pitch is held, otherwise "expected".
      final isAtPlayhead =
          cursorDiv != null &&
          note.positionDivisions <= cursorDiv &&
          cursorDiv < note.positionDivisions + note.durationDivisions;
      final headColor = isAtPlayhead
          ? (activeNotes.contains(_midiOf(pitch))
                ? CymbraColors.tertiary
                : CymbraColors.secondaryContainer)
          : _ink;

      // Note head, centred on x.
      final headLeft = x - Smufl.noteheadWidth * _s / 2;
      Smufl.draw(canvas, _headGlyph(note), headLeft, y, _s, headColor);

      if (note.accidental != null) {
        final glyph = Smufl.accidental(note.accidental!);
        if (glyph != null) {
          Smufl.draw(canvas, glyph, headLeft - _s * 1.5, y, _s, _ink);
        }
      }
      _drawDots(canvas, x, y, note.dots);

      // Ties (same-pitch) and slurs (phrase), connecting to a stored start.
      const headR = Smufl.noteheadWidth * _s / 2;
      final tieKey =
          '${note.staff}_${note.voice}_${pitch.step}'
          '${pitch.octave}_${pitch.alter}';
      if (note.tieStop) {
        final start = arcs.takeTie(tieKey);
        if (start != null) _drawTie(canvas, start, Offset(x - headR, y));
      }
      if (note.tieStart) arcs.putTie(tieKey, Offset(x + headR, y));

      final slurKey = '${note.staff}_${note.voice}';
      arcs.observeSlur(slurKey, y); // track the phrase's highest note
      if (note.slurStop) {
        final s = arcs.popSlur(slurKey);
        if (s != null) _drawSlur(canvas, s.start, Offset(x, y), s.minY);
      }
      if (note.slurStart) arcs.pushSlur(slurKey, Offset(x, y));

      // Stem + beam grouping (chord members share the principal note's stem).
      if (_headGlyph(note) != Smufl.noteheadWhole && !note.isChord) {
        final midY = staffBottom - 2 * _s;
        final up = note.stem != null ? note.stem == StemDir.up : y >= midY;
        final n = _Note(x, y, up, note);
        if (note.beams.isEmpty) {
          _drawStemAndFlag(canvas, n);
        } else {
          final key = '${note.staff}_${note.voice}';
          beamGroups.putIfAbsent(key, () => <_Note>[]).add(n);
          if (note.beams.contains(BeamState.end)) {
            _drawBeamGroup(canvas, beamGroups.remove(key)!);
          }
        }

        // Tuplet grouping: collect consecutive same-voice notes that carry a
        // tuplet ratio; draw the number once the group is complete.
        final t = note.tuplet;
        final key = '${note.staff}_${note.voice}';
        if (t != null) {
          final acc = openTuplets.putIfAbsent(
            key,
            () => _TupletAcc(t.actual, up),
          );
          acc.add(x, y, note.beams.isNotEmpty);
          if (acc.count >= t.actual) {
            _drawTuplet(canvas, openTuplets.remove(key)!);
          }
        } else if (openTuplets.containsKey(key)) {
          _drawTuplet(canvas, openTuplets.remove(key)!);
        }
      }

      final lyric = note.lyric;
      if (lyric != null) {
        _text(
          canvas,
          lyric.text,
          x - _s,
          staffBottom + _s * 1.6,
          size: _s * 1.05,
        );
      }
    }
    for (final group in beamGroups.values) {
      _drawBeamGroup(canvas, group);
    }
    for (final acc in openTuplets.values) {
      _drawTuplet(canvas, acc);
    }
  }

  /// Draws a tuplet's number (e.g. "3") centred over/under its note group, with
  /// a bracket when the group is not beamed (the beam already implies grouping).
  void _drawTuplet(Canvas canvas, _TupletAcc acc) {
    if (acc.xs.isEmpty) return;
    final cx = (acc.xs.first + acc.xs.last) / 2;
    final double y;
    if (acc.up) {
      var top = acc.ys.first;
      for (final v in acc.ys) {
        if (v < top) top = v;
      }
      y = top - (_stemLen + 1.6) * _s;
    } else {
      var bot = acc.ys.first;
      for (final v in acc.ys) {
        if (v > bot) bot = v;
      }
      y = bot + (_stemLen + 0.8) * _s;
    }
    // Tuplet numbers are drawn smaller than note heads (≈ 0.6 staff spaces).
    Smufl.draw(
      canvas,
      Smufl.tupletNumber(acc.actual),
      cx,
      y,
      _s * 0.6,
      _ink,
      centerX: true,
    );

    if (!acc.allBeamed && acc.xs.length >= 2) {
      final x0 = acc.xs.first;
      final x1 = acc.xs.last;
      const gap = _s * 0.9;
      final hook = acc.up ? _s * 0.5 : -_s * 0.5;
      final paint = Paint()
        ..color = _ink
        ..strokeWidth = Smufl.stemThickness * _s;
      canvas.drawLine(Offset(x0, y), Offset(cx - gap, y), paint);
      canvas.drawLine(Offset(cx + gap, y), Offset(x1, y), paint);
      canvas.drawLine(Offset(x0, y), Offset(x0, y + hook), paint);
      canvas.drawLine(Offset(x1, y), Offset(x1, y + hook), paint);
    }
  }

  Paint get _arcPaint => Paint()
    ..color = _ink
    ..style = PaintingStyle.stroke
    ..strokeWidth = _s * 0.13
    ..strokeCap = StrokeCap.round;

  /// A short tie between two same-pitch heads, curving below (belly down).
  void _drawTie(Canvas canvas, Offset a, Offset b) {
    final ctrl = Offset((a.dx + b.dx) / 2, (a.dy + b.dy) / 2 + _s * 1.0);
    canvas.drawPath(
      Path()
        ..moveTo(a.dx, a.dy)
        ..quadraticBezierTo(ctrl.dx, ctrl.dy, b.dx, b.dy),
      _arcPaint,
    );
  }

  /// A phrase slur from the first to the last note, arcing above the whole
  /// phrase: the control point is placed so the curve clears the highest note
  /// ([minY]) seen while the slur was open.
  void _drawSlur(Canvas canvas, Offset a, Offset b, double minY) {
    final cx = (a.dx + b.dx) / 2;
    // Longer phrases bulge a little more so the arc stays clear of the notes.
    final clearance = _s * 1.4 + (b.dx - a.dx).abs() * 0.05;
    // Quadratic midpoint y = 0.25*(a.y+b.y) + 0.5*ctrlY; solve so it sits at
    // minY - clearance (above the highest note head).
    final ctrlY = 2 * (minY - clearance) - 0.5 * (a.dy + b.dy);
    canvas.drawPath(
      Path()
        ..moveTo(a.dx, a.dy)
        ..quadraticBezierTo(cx, ctrlY, b.dx, b.dy),
      _arcPaint,
    );
  }

  /// Y of a pitch's note head for the clef in effect on its staff.
  double _yForPitch(Pitch pitch, double bottomLineY, Clef clef) {
    final diatonic = pitch.octave * 7 + (_stepOrder[pitch.step] ?? 0);
    return bottomLineY - (diatonic - _clefBottomDiatonic(clef)) * (_s / 2);
  }

  /// Diatonic value of a clef's bottom staff line. The clef sign sits on its
  /// `line` (G→G4, F→F3, C→C4); each staff line is two diatonic steps apart.
  int _clefBottomDiatonic(Clef clef) {
    final refOnLine = switch (clef.sign) {
      'F' => 3 * 7 + 3, // F3
      'C' => 4 * 7 + 0, // C4
      _ => 4 * 7 + 4, // G4
    };
    return refOnLine - (clef.line - 1) * 2;
  }

  Paint get _stemPaint => Paint()
    ..color = _ink
    ..strokeWidth = Smufl.stemThickness * _s
    ..strokeCap = StrokeCap.round;

  /// Stem-attachment point on the note head for the given direction.
  Offset _stemAnchor(_Note n) => n.up
      ? Offset(
          n.x - Smufl.noteheadWidth * _s / 2 + Smufl.stemUpAnchorX * _s,
          n.y - Smufl.stemUpAnchorY * _s,
        )
      : Offset(
          n.x - Smufl.noteheadWidth * _s / 2 + Smufl.stemDownAnchorX * _s,
          n.y - Smufl.stemDownAnchorY * _s,
        );

  void _drawStemAndFlag(Canvas canvas, _Note n) {
    final anchor = _stemAnchor(n);
    final tipY = n.up ? anchor.dy - _stemLen * _s : anchor.dy + _stemLen * _s;
    canvas.drawLine(anchor, Offset(anchor.dx, tipY), _stemPaint);
    final flag = _flagGlyph(n.note, n.up);
    if (flag != null) {
      Smufl.draw(canvas, flag, anchor.dx, tipY, _s, _ink);
    }
  }

  /// Draws a beamed group: one straight beam, stems of varying length reaching
  /// it, plus secondary beams for 16th-or-shorter runs.
  void _drawBeamGroup(Canvas canvas, List<_Note> group) {
    if (group.isEmpty) return;
    if (group.length == 1) {
      _drawStemAndFlag(canvas, group.first);
      return;
    }
    final up = group.first.up;
    final anchors = group.map(_stemAnchor).toList();
    // Flat beam clearing the extreme note by the stem length.
    double beamY;
    if (up) {
      beamY =
          anchors.map((a) => a.dy).reduce((a, b) => a < b ? a : b) -
          _stemLen * _s;
    } else {
      beamY =
          anchors.map((a) => a.dy).reduce((a, b) => a > b ? a : b) +
          _stemLen * _s;
    }
    for (final a in anchors) {
      canvas.drawLine(a, Offset(a.dx, beamY), _stemPaint);
    }
    final beamPaint = Paint()
      ..color = _ink
      ..strokeWidth = Smufl.beamThickness * _s
      ..strokeCap = StrokeCap.butt;
    canvas.drawLine(
      Offset(anchors.first.dx, beamY),
      Offset(anchors.last.dx, beamY),
      beamPaint,
    );
    // Secondary beam for consecutive 16th-or-shorter notes.
    final dir = up ? 1.0 : -1.0;
    final off = dir * (Smufl.beamThickness + 0.2) * _s;
    final thin = Paint()
      ..color = _ink
      ..strokeWidth = Smufl.beamThickness * _s;
    for (var i = 0; i < group.length - 1; i++) {
      if (_flagCount(group[i].note) >= 2 &&
          _flagCount(group[i + 1].note) >= 2) {
        canvas.drawLine(
          Offset(anchors[i].dx, beamY + off),
          Offset(anchors[i + 1].dx, beamY + off),
          thin,
        );
      }
    }
  }

  void _drawDots(Canvas canvas, double x, double y, int dots) {
    for (var i = 0; i < dots; i++) {
      Smufl.draw(
        canvas,
        Smufl.augmentationDot,
        x + Smufl.noteheadWidth * _s / 2 + _s * (0.3 + i * 0.5),
        y,
        _s,
        _ink,
      );
    }
  }

  void _drawLedgerLines(Canvas canvas, double x, double y, double bottomLineY) {
    final topLineY = bottomLineY - _staffHeight;
    const ext = Smufl.legerLineExtension * _s;
    const half = Smufl.noteheadWidth * _s / 2 + ext;
    final paint = Paint()
      ..color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.8)
      ..strokeWidth = Smufl.legerLineThickness * _s;
    for (var ly = bottomLineY + _s; ly <= y + 0.5; ly += _s) {
      canvas.drawLine(Offset(x - half, ly), Offset(x + half, ly), paint);
    }
    for (var ly = topLineY - _s; ly >= y - 0.5; ly -= _s) {
      canvas.drawLine(Offset(x - half, ly), Offset(x + half, ly), paint);
    }
  }

  String _headGlyph(NoteEvent note) {
    final div = document.attributes.divisions;
    switch (note.noteType) {
      case 'whole':
        return Smufl.noteheadWhole;
      case 'half':
        return Smufl.noteheadHalf;
      case null:
        if (note.durationDivisions >= 4 * div) return Smufl.noteheadWhole;
        if (note.durationDivisions >= 2 * div) return Smufl.noteheadHalf;
        return Smufl.noteheadBlack;
      default:
        return Smufl.noteheadBlack;
    }
  }

  String? _flagGlyph(NoteEvent note, bool up) => switch (note.noteType) {
    'eighth' => up ? Smufl.flag8thUp : Smufl.flag8thDown,
    '16th' => up ? Smufl.flag16thUp : Smufl.flag16thDown,
    '32nd' => up ? Smufl.flag32ndUp : Smufl.flag32ndDown,
    _ => null,
  };

  int _flagCount(NoteEvent note) => switch (note.noteType) {
    'eighth' => 1,
    '16th' => 2,
    '32nd' => 3,
    '64th' => 4,
    _ => 0,
  };

  String _restGlyph(NoteEvent note) => switch (note.noteType) {
    'whole' => Smufl.restWhole,
    'half' => Smufl.restHalf,
    'eighth' => Smufl.rest8th,
    '16th' => Smufl.rest16th,
    _ => Smufl.restQuarter,
  };

  void _text(
    Canvas canvas,
    String text,
    double x,
    double y, {
    double size = 13,
    Color color = CymbraColors.onSurface,
    bool italic = false,
  }) {
    if (text.isEmpty) return;
    TextPainter(
        text: TextSpan(
          text: text,
          style: TextStyle(
            color: color,
            fontSize: size,
            fontStyle: italic ? FontStyle.italic : FontStyle.normal,
          ),
        ),
        textDirection: TextDirection.ltr,
      )
      ..layout()
      ..paint(canvas, Offset(x, y));
  }

  double _textWidth(String text, double size, {bool italic = false}) {
    final tp = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          fontSize: size,
          fontStyle: italic ? FontStyle.italic : FontStyle.normal,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    return tp.width;
  }

  @override
  bool shouldRepaint(PartitionPainter old) =>
      old.document != document ||
      old.systems != systems ||
      old.elapsedMs != elapsedMs ||
      old.activeNotes != activeNotes;
}

/// A note's drawn geometry (head centre + stem direction), used for beaming.
class _Note {
  final double x;
  final double y;
  final bool up;
  final NoteEvent note;
  const _Note(this.x, this.y, this.up, this.note);
}

/// Tracks open tie/slur starts within a system so the arc can be drawn when the
/// matching stop note is reached. Ties key on pitch (same note resumed); slurs
/// key on staff+voice and nest as a stack.
class _Arcs {
  final Map<String, Offset> _ties = {};
  final Map<String, List<_SlurOpen>> _slurs = {};

  void putTie(String key, Offset start) => _ties[key] = start;
  Offset? takeTie(String key) => _ties.remove(key);

  void pushSlur(String key, Offset start) =>
      _slurs.putIfAbsent(key, () => <_SlurOpen>[]).add(_SlurOpen(start));

  /// Updates the open slur's highest note (smallest y) as the phrase is drawn.
  void observeSlur(String key, double y) {
    final stack = _slurs[key];
    if (stack != null && stack.isNotEmpty && y < stack.last.minY) {
      stack.last.minY = y;
    }
  }

  _SlurOpen? popSlur(String key) {
    final stack = _slurs[key];
    if (stack == null || stack.isEmpty) return null;
    return stack.removeLast();
  }
}

/// An open slur: its start point and the highest note (smallest y) seen so far.
class _SlurOpen {
  final Offset start;
  double minY;
  _SlurOpen(this.start) : minY = start.dy;
}

/// Accumulates a tuplet's notes so its number/bracket can be drawn once the
/// group (`actual` notes) is complete.
class _TupletAcc {
  final int actual;
  final bool up;
  final List<double> xs = [];
  final List<double> ys = [];
  bool allBeamed = true;
  _TupletAcc(this.actual, this.up);

  void add(double x, double y, bool beamed) {
    xs.add(x);
    ys.add(y);
    if (!beamed) allBeamed = false;
  }

  int get count => xs.length;
}

/// Assigns text (e.g. expression words) to stacked rows so overlapping items at
/// nearby x-positions don't collide: each item takes the lowest row whose last
/// occupied x has cleared, else a new row above.
class _TextLanes {
  final double rowGap;
  final List<double> _rowEndX = [];
  _TextLanes(this.rowGap);

  /// Baseline Y for an item of [width] starting at [x]; rows stack upward.
  double yFor(double x, double width, double baseY) {
    for (var r = 0; r < _rowEndX.length; r++) {
      if (_rowEndX[r] <= x) {
        _rowEndX[r] = x + width;
        return baseY - r * rowGap;
      }
    }
    _rowEndX.add(x + width);
    return baseY - (_rowEndX.length - 1) * rowGap;
  }
}
