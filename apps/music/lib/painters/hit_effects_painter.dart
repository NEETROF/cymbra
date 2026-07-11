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

import '../state/performance_scoring.dart';
import '../state/performance_scoring_core.dart';
import '../theme/cymbra_theme.dart';
import '../theme/scoring_style.dart';
import 'piano_layout.dart';

/// Transient Guitar-Hero–style hit sparks at the note-hit line. Each recent hit
/// is a verdict-coloured ring that expands and fades over [fadeMs]; nothing
/// persists. Drawn above the falling notes but along the bottom hit line so it
/// never occludes upcoming notes, and a missed/wrong hit uses the error colour —
/// never a success colour (`gamified-feedback` learning-safe rules).
class HitEffectsPainter extends CustomPainter {
  final PianoLayout layout;
  final List<HitEffect> hits;

  /// Current wall-clock ms (same clock the hits were stamped with).
  final int nowMs;
  final int fadeMs;

  const HitEffectsPainter({
    required this.layout,
    required this.hits,
    required this.nowMs,
    this.fadeMs = 600,
  });

  @override
  void paint(Canvas canvas, Size size) {
    for (final hit in hits) {
      final age = nowMs - hit.atMs;
      if (age < 0 || age >= fadeMs) continue;
      if (!layout.contains(hit.pitch)) continue;
      final progress = age / fadeMs; // 0 → 1
      final alpha = (1 - progress).clamp(0.0, 1.0);
      final x = layout.centerX(hit.pitch);
      final y = size.height - 2;
      final radius = 8 + progress * 16;
      final color = hit.wrong
          ? CymbraColors.error
          : verdictColor(hit.verdict, fallback: CymbraColors.secondary);

      canvas.drawCircle(
        Offset(x, y),
        radius,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..color = color.withValues(alpha: alpha),
      );
      if (!hit.wrong && hit.verdict != TimingVerdict.missed) {
        canvas.drawCircle(
          Offset(x, y),
          radius * 0.5,
          Paint()..color = color.withValues(alpha: alpha * 0.35),
        );
      }
    }
  }

  @override
  bool shouldRepaint(HitEffectsPainter old) =>
      old.nowMs != nowMs || old.hits != hits;
}
