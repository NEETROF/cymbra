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
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../state/performance_scoring.dart';
import '../theme/cymbra_theme.dart';
import '../theme/scoring_style.dart';

/// The synchronization gauge: a compact box whose fill rises from the bottom in
/// the current tier's colour, showing the live sync %, tier name, and combo. The
/// box shakes each time the player crosses **up** into a higher 20% tier
/// (`gamified-feedback`). Hidden entirely when no scored run is active, and sized
/// to sit in a corner so it never occludes the falling notes.
class ScoringGauge extends ConsumerStatefulWidget {
  const ScoringGauge({super.key});

  @override
  ConsumerState<ScoringGauge> createState() => _ScoringGaugeState();
}

class _ScoringGaugeState extends ConsumerState<ScoringGauge>
    with TickerProviderStateMixin {
  late final AnimationController _shake = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 450),
  );

  // A little firework burst inside the box on each 20% tier-up.
  late final AnimationController _firework = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 700),
  );

  @override
  void initState() {
    super.initState();
    // Shake + firework on a rising tier crossing only — a drop de-escalates
    // quietly.
    ref.listenManual(
      performanceScorerProvider.select((s) => s.active ? s.tier : 0),
      (prev, next) {
        if (next > (prev ?? 0)) {
          _shake.forward(from: 0);
          _firework.forward(from: 0);
        }
      },
    );
  }

  @override
  void dispose() {
    _shake.dispose();
    _firework.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final active = ref.watch(performanceScorerProvider.select((s) => s.active));
    if (!active) return const SizedBox.shrink();

    final pct = ref.watch(
      performanceScorerProvider.select((s) => s.syncPercent),
    );
    final tier = ref.watch(performanceScorerProvider.select((s) => s.tier));
    final combo = ref.watch(performanceScorerProvider.select((s) => s.combo));
    final l10n = AppLocalizations.of(context);
    final color = tier.tierColor;

    return AnimatedBuilder(
      animation: _shake,
      builder: (context, child) {
        // Damped sinusoidal wobble, zero at rest.
        final t = _shake.value;
        final dx = t == 0 ? 0.0 : math.sin(t * math.pi * 4) * 5 * (1 - t);
        final angle = t == 0 ? 0.0 : math.sin(t * math.pi * 4) * 0.05 * (1 - t);
        return Transform.rotate(
          angle: angle,
          child: Transform.translate(offset: Offset(dx, 0), child: child),
        );
      },
      child: Semantics(
        label: l10n.scoringGaugeLabel(pct.round()),
        child: SizedBox(
          width: 88,
          height: 120,
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: CymbraColors.surfaceContainerLow,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: color, width: 1.5),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(11),
              child: Stack(
                children: [
                  // Bottom-anchored fill proportional to the sync %.
                  Positioned.fill(
                    child: AnimatedFractionallySizedBox(
                      duration: const Duration(milliseconds: 250),
                      heightFactor: (pct / 100).clamp(0.0, 1.0),
                      widthFactor: 1,
                      alignment: Alignment.bottomCenter,
                      child: ColoredBox(color: color.withValues(alpha: 0.26)),
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.symmetric(
                      vertical: 8,
                      horizontal: 6,
                    ),
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      crossAxisAlignment: CrossAxisAlignment.center,
                      children: [
                        Text(
                          tier.tierName(l10n),
                          style: TextStyle(
                            fontSize: 11,
                            fontWeight: FontWeight.w500,
                            color: color,
                          ),
                        ),
                        Text(
                          '${pct.round()}%',
                          style: const TextStyle(
                            fontSize: 26,
                            fontWeight: FontWeight.w500,
                            color: CymbraColors.onSurface,
                            height: 1,
                          ),
                        ),
                        Text(
                          l10n.scoringCombo(combo),
                          style: TextStyle(fontSize: 12, color: color),
                        ),
                      ],
                    ),
                  ),
                  // Firework burst on a tier-up (clipped to the box).
                  Positioned.fill(
                    child: IgnorePointer(
                      child: AnimatedBuilder(
                        animation: _firework,
                        builder: (context, _) => CustomPaint(
                          painter: GaugeFireworkPainter(
                            progress: _firework.value,
                            color: color,
                          ),
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// A small firework burst drawn inside the gauge on a tier-up: [count] particles
/// shoot out from the centre and fade over [progress] (0 → 1). Deterministic
/// (fixed angles) so it is host-testable. Idle (paints nothing) at the ends.
class GaugeFireworkPainter extends CustomPainter {
  final double progress;
  final Color color;

  const GaugeFireworkPainter({required this.progress, required this.color});

  static const int count = 12;

  @override
  void paint(Canvas canvas, Size size) {
    if (progress <= 0 || progress >= 1) return;
    final center = Offset(size.width / 2, size.height * 0.55);
    final maxR = math.min(size.width, size.height) * 0.5;
    final reach = (1 - math.pow(1 - progress, 2).toDouble()) * maxR; // easeOut
    final alpha = (1 - progress).clamp(0.0, 1.0);
    final paint = Paint()..color = color.withValues(alpha: alpha);
    for (var i = 0; i < count; i++) {
      final angle = 2 * math.pi * i / count + i * 0.3;
      final p =
          center + Offset(math.cos(angle) * reach, math.sin(angle) * reach);
      canvas.drawCircle(p, (1 - progress) * 2.5 + 0.8, paint);
    }
  }

  @override
  bool shouldRepaint(GaugeFireworkPainter old) =>
      old.progress != progress || old.color != color;
}

/// A [FractionallySizedBox] whose [heightFactor] animates — used for the gauge
/// fill so the level glides rather than jumping.
class AnimatedFractionallySizedBox extends ImplicitlyAnimatedWidget {
  const AnimatedFractionallySizedBox({
    super.key,
    required this.heightFactor,
    required this.widthFactor,
    required this.alignment,
    required this.child,
    required super.duration,
    super.curve,
  });

  final double heightFactor;
  final double widthFactor;
  final AlignmentGeometry alignment;
  final Widget child;

  @override
  AnimatedWidgetBaseState<AnimatedFractionallySizedBox> createState() =>
      _AnimatedFractionallySizedBoxState();
}

class _AnimatedFractionallySizedBoxState
    extends AnimatedWidgetBaseState<AnimatedFractionallySizedBox> {
  Tween<double>? _height;

  @override
  void forEachTween(TweenVisitor<dynamic> visitor) {
    _height =
        visitor(
              _height,
              widget.heightFactor,
              (value) => Tween<double>(begin: value as double),
            )
            as Tween<double>?;
  }

  @override
  Widget build(BuildContext context) => FractionallySizedBox(
    heightFactor: (_height?.evaluate(animation) ?? widget.heightFactor).clamp(
      0.0,
      1.0,
    ),
    widthFactor: widget.widthFactor,
    alignment: widget.alignment,
    child: widget.child,
  );
}
