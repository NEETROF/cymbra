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
/// font's note-head anchors. Geometry only — no playback or interaction.
class PartitionPainter extends CustomPainter {
  final ScoreDocument document;
  final List<System> systems;

  PartitionPainter({required this.document, required this.systems});

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
    var y = _systemGap;
    for (var i = 0; i < systems.length; i++) {
      _paintSystem(canvas, systems[i], size.width, y, divPerMeasure, i == 0);
      y += _systemHeight + _systemGap;
    }
  }

  void _paintSystem(
    Canvas canvas,
    System system,
    double width,
    double yTop,
    int divPerMeasure,
    bool isFirst,
  ) {
    final trebleBottom = yTop + _topPad + _staffHeight;
    final bassBottom = trebleBottom + _interStaff + _staffHeight;
    final systemBottom = _twoStaff ? bassBottom : trebleBottom;

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

    // --- Header: clef, key signature, then time signature (first system). ---
    Smufl.draw(
      canvas,
      Smufl.clef(_clefSign(1)),
      _s * 0.4,
      trebleBottom - _s,
      _s,
      _ink,
    );
    if (_twoStaff) {
      Smufl.draw(
        canvas,
        Smufl.clef(_clefSign(2)),
        _s * 0.4,
        bassBottom - 3 * _s,
        _s,
        _ink,
      );
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
    for (final idx in indices) {
      final measure = document.measures[idx];
      final mWidth = measure.minWidth * scale;
      canvas.drawLine(
        Offset(x + mWidth, systemTop),
        Offset(x + mWidth, systemBottom),
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
  ) {
    double xForPosition(int position) {
      final frac = divPerMeasure > 0
          ? (position / divPerMeasure).clamp(0.0, 1.0)
          : 0.0;
      return measureX + _s + frac * (measureWidth - 2.4 * _s);
    }

    for (final dir in measure.directions) {
      final x = xForPosition(dir.positionDivisions);
      switch (dir.kind) {
        case DirectionKind_Words(:final field0):
          _text(
            canvas,
            field0,
            x,
            trebleBottom - _staffHeight - _s * 2.2,
            italic: true,
            color: CymbraColors.onSurfaceVariant,
          );
        case DirectionKind_Dynamics(:final field0):
          Smufl.draw(
            canvas,
            Smufl.dynamics(field0),
            x,
            trebleBottom + _s * 2.4,
            _s,
            CymbraColors.secondary,
          );
        case DirectionKind_Wedge():
        case DirectionKind_Metronome():
          break;
      }
    }

    final beamGroups = <String, List<_Note>>{};

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

      final y = _yForPitch(pitch, staffBottom, isBass);
      _drawLedgerLines(canvas, x, y, staffBottom);

      // Note head, centred on x.
      final headLeft = x - Smufl.noteheadWidth * _s / 2;
      Smufl.draw(canvas, _headGlyph(note), headLeft, y, _s, _ink);

      if (note.accidental != null) {
        final glyph = Smufl.accidental(note.accidental!);
        if (glyph != null) {
          Smufl.draw(canvas, glyph, headLeft - _s * 1.5, y, _s, _ink);
        }
      }
      _drawDots(canvas, x, y, note.dots);

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
  }

  /// Y of a pitch's note head; bottom staff line is E4 (treble) / G2 (bass).
  double _yForPitch(Pitch pitch, double bottomLineY, bool isBass) {
    final diatonic = pitch.octave * 7 + (_stepOrder[pitch.step] ?? 0);
    final ref = isBass ? (2 * 7 + 4) : (4 * 7 + 2);
    return bottomLineY - (diatonic - ref) * (_s / 2);
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
    final ext = Smufl.legerLineExtension * _s;
    final half = Smufl.noteheadWidth * _s / 2 + ext;
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

  String _clefSign(int staff) {
    final clef = document.attributes.clefs
        .where((c) => c.staff == staff)
        .firstOrNull;
    return clef?.sign ?? (staff >= 2 ? 'F' : 'G');
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

  @override
  bool shouldRepaint(PartitionPainter old) =>
      old.document != document || old.systems != systems;
}

/// A note's drawn geometry (head centre + stem direction), used for beaming.
class _Note {
  final double x;
  final double y;
  final bool up;
  final NoteEvent note;
  const _Note(this.x, this.y, this.up, this.note);
}
