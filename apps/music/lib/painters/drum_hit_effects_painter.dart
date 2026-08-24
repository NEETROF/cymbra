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

import '../state/drum_kit.dart';
import '../state/performance_scoring.dart';
import '../state/performance_scoring_core.dart';
import '../theme/cymbra_theme.dart';
import '../theme/scoring_style.dart';

/// The percussion counterpart of `HitEffectsPainter` (change:
/// add-drum-scoring): a transient verdict-coloured spark at the cascade's hit
/// line for each judged stroke.
///
/// The anchor follows the cascade's own geometry rather than a keyboard
/// mapping: a **hand** stroke sparks on its lane, a **kick** across the
/// **full-width bar** — the bar is a note in a different shape, and a kick
/// landed cleanly deserves the same spark as a snare.
///
/// The lane is resolved at the piece grain ([pieceLaneIndexOf]), the same
/// resolution the scorer bound the stroke with, so the spark can never land on
/// a lane the judgment did not mean. A stroke of a piece the score does not
/// present has no lane and simply draws nothing.
class DrumHitEffectsPainter extends CustomPainter {
  /// The lanes in presentation order (`PlayerData.presentedDrumLanes`) — the
  /// list the cascade laid the notes out with, so the sparks land on the same
  /// columns.
  final List<DrumLane> lanes;

  final List<HitEffect> hits;

  /// Current wall-clock ms (the clock the hits were stamped with).
  final int nowMs;
  final int fadeMs;

  const DrumHitEffectsPainter({
    required this.lanes,
    required this.hits,
    required this.nowMs,
    this.fadeMs = 600,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final hitLineY = size.height - 2;
    for (final hit in hits) {
      final age = nowMs - hit.atMs;
      if (age < 0 || age >= fadeMs) continue;
      final progress = age / fadeMs; // 0 → 1
      final alpha = (1 - progress).clamp(0.0, 1.0);
      final color = hit.wrong
          ? CymbraColors.error
          : verdictColor(hit.verdict, fallback: CymbraColors.secondary);
      final solid = !hit.wrong && hit.verdict != TimingVerdict.missed;

      if (kKickGmNumbers.contains(hit.pitch)) {
        // The foot's spark spans the bar: a swelling full-width band, so a
        // clean kick reads in peripheral vision exactly where the note was.
        final height = 6 + progress * 14;
        canvas.drawRRect(
          RRect.fromRectAndRadius(
            Rect.fromLTWH(0, hitLineY - height, size.width, height),
            const Radius.circular(3),
          ),
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 2
            ..color = color.withValues(alpha: alpha),
        );
        if (solid) {
          canvas.drawRect(
            Rect.fromLTWH(0, hitLineY - height * 0.5, size.width, height * 0.5),
            Paint()..color = color.withValues(alpha: alpha * 0.28),
          );
        }
        continue;
      }

      if (lanes.isEmpty) continue;
      final lane = pieceLaneIndexOf(lanes, hit.pitch);
      if (lane == null) continue;
      final laneWidth = size.width / lanes.length;
      final x = laneWidth * (lane + 0.5);
      final radius = 8 + progress * 16;
      canvas.drawCircle(
        Offset(x, hitLineY),
        radius,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..color = color.withValues(alpha: alpha),
      );
      if (solid) {
        canvas.drawCircle(
          Offset(x, hitLineY),
          radius * 0.5,
          Paint()..color = color.withValues(alpha: alpha * 0.35),
        );
      }
    }
  }

  @override
  bool shouldRepaint(DrumHitEffectsPainter old) =>
      old.nowMs != nowMs || old.hits != hits || old.lanes != lanes;
}
