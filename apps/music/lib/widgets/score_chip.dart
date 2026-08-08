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

/// The live score chip: a compact horizontal pill in the player top bar showing
/// the sync % filled left-to-right in the current tier's colour, plus the combo
/// counter. Living in the top bar — outside the play surface — it can never
/// occlude notes, in any render mode. The chip shakes and plays a small
/// firework burst each time the player crosses **up** into a higher 20% tier
/// (`gamified-feedback`), and hides entirely when no scored run is active.
class ScoreChip extends ConsumerStatefulWidget {
  const ScoreChip({super.key, this.compact = false});

  /// Phone top bars are short on width: the compact variant drops the combo
  /// text and keeps just the tier dot + sync %.
  final bool compact;

  @override
  ConsumerState<ScoreChip> createState() => _ScoreChipState();
}

class _ScoreChipState extends ConsumerState<ScoreChip>
    with TickerProviderStateMixin {
  // Created eagerly in initState: the chip stays mounted (hidden) in the top
  // bar while no run is active, and a lazily-created controller would first be
  // touched by dispose() — during unmount, when ticker lookup is unsafe.
  late final AnimationController _shake;

  // A little firework burst inside the chip on each 20% tier-up.
  late final AnimationController _firework;

  @override
  void initState() {
    super.initState();
    _shake = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 450),
    );
    _firework = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 700),
    );
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
        final dx = t == 0 ? 0.0 : math.sin(t * math.pi * 4) * 4 * (1 - t);
        final angle = t == 0 ? 0.0 : math.sin(t * math.pi * 4) * 0.04 * (1 - t);
        return Transform.rotate(
          angle: angle,
          child: Transform.translate(offset: Offset(dx, 0), child: child),
        );
      },
      child: Semantics(
        label: l10n.scoringGaugeLabel(pct.round()),
        child: SizedBox(
          height: 32,
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: CymbraColors.surfaceContainerLow,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: color, width: 1.5),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(15),
              // Centre the (non-positioned) dot+%+combo row vertically in the
              // pill — a Stack aligns loose children top-left by default.
              child: Stack(
                alignment: Alignment.center,
                children: [
                  // Left-anchored fill proportional to the sync %.
                  Positioned.fill(
                    child: _AnimatedHorizontalFill(
                      fraction: (pct / 100).clamp(0.0, 1.0),
                      color: color.withValues(alpha: 0.26),
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 10),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        DecoratedBox(
                          decoration: BoxDecoration(
                            color: color,
                            shape: BoxShape.circle,
                          ),
                          child: const SizedBox(width: 8, height: 8),
                        ),
                        const SizedBox(width: 6),
                        Text(
                          '${pct.round()}%',
                          style: const TextStyle(
                            fontSize: 14,
                            fontWeight: FontWeight.w600,
                            color: CymbraColors.onSurface,
                            height: 1,
                          ),
                        ),
                        if (!widget.compact) ...[
                          const SizedBox(width: 8),
                          Text(
                            l10n.scoringCombo(combo),
                            style: TextStyle(fontSize: 12, color: color),
                          ),
                        ],
                      ],
                    ),
                  ),
                  // Firework burst on a tier-up (clipped to the chip).
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

/// A left-anchored fractional-width fill whose fraction animates, used for the
/// chip's sync-level so it glides rather than jumping.
class _AnimatedHorizontalFill extends ImplicitlyAnimatedWidget {
  const _AnimatedHorizontalFill({required this.fraction, required this.color})
    : super(duration: const Duration(milliseconds: 250));

  final double fraction;
  final Color color;

  @override
  AnimatedWidgetBaseState<_AnimatedHorizontalFill> createState() =>
      _AnimatedHorizontalFillState();
}

class _AnimatedHorizontalFillState
    extends AnimatedWidgetBaseState<_AnimatedHorizontalFill> {
  Tween<double>? _fraction;

  @override
  void forEachTween(TweenVisitor<dynamic> visitor) {
    _fraction =
        visitor(
              _fraction,
              widget.fraction,
              (value) => Tween<double>(begin: value as double),
            )
            as Tween<double>?;
  }

  @override
  Widget build(BuildContext context) => FractionallySizedBox(
    alignment: Alignment.centerLeft,
    widthFactor: (_fraction?.evaluate(animation) ?? widget.fraction).clamp(
      0.0,
      1.0,
    ),
    heightFactor: 1,
    child: ColoredBox(color: widget.color),
  );
}

/// A small firework burst drawn inside the chip on a tier-up: [count] particles
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
