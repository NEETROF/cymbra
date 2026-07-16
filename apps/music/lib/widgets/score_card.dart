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

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../l10n/gen/app_localizations.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';

/// The difficulty accent colour (green / teal / pink) shared by the badge + cover.
Color scoreLevelColor(PracticeLevel level) => switch (level) {
  PracticeLevel.beginner => CymbraColors.tertiary,
  PracticeLevel.intermediate => CymbraColors.secondary,
  PracticeLevel.advanced => CymbraColors.error,
};

/// A score tile used across the Score Hub and the library: a generated cover, a
/// difficulty badge, the title/composer, and an optional top-right [action]
/// (save / remove / delete — the caller decides, since each surface differs).
class ScoreCard extends StatelessWidget {
  const ScoreCard({
    super.key,
    required this.entry,
    required this.onTap,
    this.action,
  });

  final CatalogEntry entry;
  final VoidCallback onTap;

  /// Optional top-right overlay (e.g. a heart or delete button). `null` = none.
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Material(
      color: CymbraColors.surfaceContainerLow,
      borderRadius: BorderRadius.circular(18),
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            AspectRatio(
              aspectRatio: 16 / 11,
              child: Stack(
                fit: StackFit.expand,
                children: [
                  _CoverArt(entry: entry),
                  Positioned(
                    top: 10,
                    left: 10,
                    child: _DifficultyBadge(level: entry.level, l10n: l10n),
                  ),
                  if (action != null)
                    Positioned(top: 4, right: 4, child: action!),
                ],
              ),
            ),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      entry.title,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        color: CymbraColors.onSurface,
                        fontSize: 16,
                        fontWeight: FontWeight.w700,
                        height: 1.15,
                      ),
                    ),
                    if (entry.composer.isNotEmpty) ...[
                      const SizedBox(height: 4),
                      Text(
                        entry.composer,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          color: CymbraColors.onSurfaceVariant,
                          fontSize: 13,
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _DifficultyBadge extends StatelessWidget {
  const _DifficultyBadge({required this.level, required this.l10n});

  final PracticeLevel level;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    final color = scoreLevelColor(level);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.22),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.6)),
      ),
      child: Text(
        level.localizedLabel(l10n).toUpperCase(),
        style: TextStyle(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.w800,
          letterSpacing: 0.5,
        ),
      ),
    );
  }
}

/// A deterministic generated cover for a score (no artwork exists): a gradient +
/// soft bokeh glows + a subtle sound-wave, all seeded from the score id so each
/// card is stable and distinct, with the title's initial as a monogram.
class _CoverArt extends StatelessWidget {
  const _CoverArt({required this.entry});

  final CatalogEntry entry;

  static const List<List<Color>> _gradients = [
    [Color(0xFF3B1E6E), Color(0xFF7C3AED)], // purple
    [Color(0xFF0B3D4A), Color(0xFF03C6B2)], // teal
    [Color(0xFF10233F), Color(0xFF5B9DFF)], // blue
    [Color(0xFF4A1D3F), Color(0xFFE05299)], // magenta
    [Color(0xFF123A2C), Color(0xFF4EDEA3)], // green
    [Color(0xFF3A2A12), Color(0xFFFFB454)], // amber
    [Color(0xFF241645), Color(0xFF44E2CD)], // indigo→teal
    [Color(0xFF3A1220), Color(0xFFFF6B6B)], // rose
  ];

  String get _monogram {
    final t = entry.title.trim();
    return t.isEmpty ? '♪' : t.characters.first.toUpperCase();
  }

  @override
  Widget build(BuildContext context) {
    final seed = entry.id.hashCode;
    final colors = _gradients[seed.abs() % _gradients.length];
    return CustomPaint(
      painter: _CoverPainter(seed: seed, colors: colors),
      child: Center(
        child: Text(
          _monogram,
          style: TextStyle(
            fontSize: 64,
            fontWeight: FontWeight.w900,
            color: Colors.white.withValues(alpha: 0.16),
            height: 1,
          ),
        ),
      ),
    );
  }
}

class _CoverPainter extends CustomPainter {
  _CoverPainter({required this.seed, required this.colors});

  final int seed;
  final List<Color> colors;

  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    canvas.drawRect(
      rect,
      Paint()
        ..shader = LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: colors,
        ).createShader(rect),
    );
    final rng = math.Random(seed);
    for (var i = 0; i < 4; i++) {
      final c = Offset(
        rng.nextDouble() * size.width,
        rng.nextDouble() * size.height,
      );
      final r = size.shortestSide * (0.18 + rng.nextDouble() * 0.22);
      canvas.drawCircle(
        c,
        r,
        Paint()
          ..color = Colors.white.withValues(
            alpha: 0.05 + rng.nextDouble() * 0.06,
          )
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 18),
      );
    }
    final path = Path();
    final baseY = size.height * (0.6 + rng.nextDouble() * 0.15);
    final amp = size.height * 0.08;
    final phase = rng.nextDouble() * math.pi * 2;
    path.moveTo(0, baseY);
    for (double x = 0; x <= size.width; x += 6) {
      final y = baseY + math.sin(x / size.width * math.pi * 3 + phase) * amp;
      path.lineTo(x, y);
    }
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = Colors.white.withValues(alpha: 0.14),
    );
  }

  @override
  bool shouldRepaint(_CoverPainter old) =>
      old.seed != seed || old.colors != colors;
}
