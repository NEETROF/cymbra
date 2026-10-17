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

/// Draws the learning path's curve through its node centres (change:
/// add-music-motion-and-celebrations): the segment from node `i` to node
/// `i + 1` is filled over `fills[i]` of its length in the completion colour
/// and dashed beyond, so the path visibly "fills up" as lessons are done.
/// Pure geometry — one cubic per segment, no state.
class PathCurvePainter extends CustomPainter {
  const PathCurvePainter({
    required this.centers,
    required this.fills,
    this.dashLength = 6,
    this.dashGap = 6,
  });

  /// The node centres, top to bottom.
  final List<Offset> centers;

  /// Per segment, 0 → 1: how much of it is filled. Shorter than the segment
  /// count reads as unfilled for the rest.
  final List<double> fills;

  final double dashLength;
  final double dashGap;

  /// The cubic from [a] down to [b]: vertical tangents at both ends, so a
  /// meander reads as one continuous ribbon.
  static Path segment(Offset a, Offset b) {
    final mid = (a.dy + b.dy) / 2;
    return Path()
      ..moveTo(a.dx, a.dy)
      ..cubicTo(a.dx, mid, b.dx, mid, b.dx, b.dy);
  }

  @override
  void paint(Canvas canvas, Size size) {
    final dashed = Paint()
      ..color = CymbraColors.onSurfaceVariant.withValues(alpha: 0.35)
      ..strokeWidth = 3
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;
    final filled = Paint()
      ..color = CymbraColors.tertiary
      ..strokeWidth = 4
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;
    for (var i = 0; i + 1 < centers.length; i++) {
      final fill = (i < fills.length ? fills[i] : 0.0).clamp(0.0, 1.0);
      for (final metric in segment(
        centers[i],
        centers[i + 1],
      ).computeMetrics()) {
        final cut = metric.length * fill;
        if (cut > 0) canvas.drawPath(metric.extractPath(0, cut), filled);
        var d = cut;
        while (d < metric.length) {
          final end = (d + dashLength).clamp(0.0, metric.length);
          canvas.drawPath(metric.extractPath(d, end), dashed);
          d += dashLength + dashGap;
        }
      }
    }
  }

  @override
  bool shouldRepaint(PathCurvePainter old) =>
      !_sameOffsets(old.centers, centers) || !_sameDoubles(old.fills, fills);

  static bool _sameOffsets(List<Offset> a, List<Offset> b) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }

  static bool _sameDoubles(List<double> a, List<double> b) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }
}
