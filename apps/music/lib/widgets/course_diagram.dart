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

import '../painters/smufl.dart';
import '../theme/cymbra_theme.dart';

/// The closed set of built-in notation diagrams a course `diagram` block may
/// reference (change: add-notation-courses). A manifest can only pick from these
/// ids — never an arbitrary asset — so third-party content stays safe.
const Set<String> kCourseDiagramIds = {
  'staff-lines',
  'treble-clef',
  'bass-clef',
  'note-whole',
  'note-half',
  'note-quarter',
  'note-eighth',
};

/// Renders a built-in notation [id] as a small staff drawing, reusing the SMuFL
/// glyphs. An unknown id shows a neutral placeholder rather than failing, so a
/// newer manifest never breaks an older app.
class CourseDiagram extends StatelessWidget {
  const CourseDiagram({super.key, required this.id, this.height = 96});

  final String id;
  final double height;

  @override
  Widget build(BuildContext context) {
    if (!kCourseDiagramIds.contains(id)) {
      return SizedBox(
        height: height,
        child: Center(
          child: Icon(
            Icons.music_note_outlined,
            color: CymbraColors.onSurfaceVariant.withValues(alpha: 0.5),
          ),
        ),
      );
    }
    return SizedBox(
      height: height,
      width: double.infinity,
      child: CustomPaint(painter: _DiagramPainter(id)),
    );
  }
}

class _DiagramPainter extends CustomPainter {
  _DiagramPainter(this.id);

  final String id;

  @override
  void paint(Canvas canvas, Size size) {
    final s = size.height / 8; // staff space
    final bottom = size.height * 0.72;
    final left = size.width * 0.12;
    final right = size.width * 0.88;
    const ink = CymbraColors.onSurface;
    final linePaint = Paint()
      ..color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.7)
      ..strokeWidth = Smufl.staffLineThickness * s;

    // Five staff lines.
    for (var i = 0; i < 5; i++) {
      final y = bottom - i * s;
      canvas.drawLine(Offset(left, y), Offset(right, y), linePaint);
    }
    final cx = (left + right) / 2;

    switch (id) {
      case 'treble-clef':
        Smufl.draw(canvas, Smufl.gClef, left + s, bottom - s, s, ink);
      case 'bass-clef':
        Smufl.draw(canvas, Smufl.fClef, left + s, bottom - 3 * s, s, ink);
      case 'note-whole':
        Smufl.draw(
          canvas,
          Smufl.noteheadWhole,
          cx,
          bottom - 2 * s,
          s,
          ink,
          centerX: true,
        );
      case 'note-half':
        _headWithStem(canvas, cx, bottom - 2 * s, s, ink, Smufl.noteheadHalf);
      case 'note-quarter':
        _headWithStem(canvas, cx, bottom - 2 * s, s, ink, Smufl.noteheadBlack);
      case 'note-eighth':
        _headWithStem(
          canvas,
          cx,
          bottom - 2 * s,
          s,
          ink,
          Smufl.noteheadBlack,
          flag: Smufl.flag8thUp,
        );
      case 'staff-lines':
        break; // the staff alone
    }
  }

  void _headWithStem(
    Canvas canvas,
    double cx,
    double y,
    double s,
    Color ink,
    String head, {
    String? flag,
  }) {
    Smufl.draw(canvas, head, cx - Smufl.noteheadWidth * s / 2, y, s, ink);
    final stemX = cx - Smufl.noteheadWidth * s / 2 + Smufl.stemUpAnchorX * s;
    final tipY = y - Smufl.stemUpAnchorY * s - 3.2 * s;
    canvas.drawLine(
      Offset(stemX, y - Smufl.stemUpAnchorY * s),
      Offset(stemX, tipY),
      Paint()
        ..color = ink
        ..strokeWidth = Smufl.stemThickness * s
        ..strokeCap = StrokeCap.round,
    );
    if (flag != null) Smufl.draw(canvas, flag, stemX, tipY, s, ink);
  }

  @override
  bool shouldRepaint(_DiagramPainter old) => old.id != id;
}
