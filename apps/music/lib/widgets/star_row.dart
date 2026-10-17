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

import '../l10n/gen/app_localizations.dart';
import '../theme/cymbra_theme.dart';
import '../theme/motion.dart';

/// One to three stars, filled or outlined (change: add-music-piece-stars):
/// compact on cards and nodes, large in the summary. Stars gained since
/// [animateFrom] pop in (change: add-music-motion-and-celebrations); under
/// reduced motion they are simply there. Announced as "n of 3 stars".
class StarRow extends StatelessWidget {
  const StarRow({
    super.key,
    required this.stars,
    this.compact = true,
    this.animateFrom,
    this.color = const Color(0xFFF4C542),
  });

  /// 0 to 3.
  final int stars;
  final bool compact;

  /// The stars already held before this moment; those above it pop in.
  final int? animateFrom;
  final Color color;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final size = compact ? 12.0 : 30.0;
    final from = animateFrom ?? stars;
    return Semantics(
      label: l10n.starsA11y(stars),
      excludeSemantics: true,
      child: Row(
        key: Key('star-row-$stars'),
        mainAxisSize: MainAxisSize.min,
        children: [
          for (var i = 0; i < 3; i++)
            _Star(
              filled: i < stars,
              pops: i >= from && i < stars,
              size: size,
              color: color,
              delay: Duration(milliseconds: 120 * (i - from).clamp(0, 2)),
            ),
        ],
      ),
    );
  }
}

class _Star extends StatelessWidget {
  const _Star({
    required this.filled,
    required this.pops,
    required this.size,
    required this.color,
    required this.delay,
  });

  final bool filled;
  final bool pops;
  final double size;
  final Color color;
  final Duration delay;

  @override
  Widget build(BuildContext context) {
    final icon = Icon(
      filled ? Icons.star_rounded : Icons.star_outline_rounded,
      size: size,
      color: filled
          ? color
          : CymbraColors.onSurfaceVariant.withValues(alpha: 0.5),
    );
    if (!pops) return icon;
    // A new star: scales in with the pop curve, one after the other.
    return TweenAnimationBuilder<double>(
      tween: Tween(begin: 0, end: 1),
      duration: Motion.of(context, Motion.celebrate ~/ 2 + delay),
      curve: Interval(
        (delay.inMilliseconds /
                (Motion.celebrate.inMilliseconds / 2 + delay.inMilliseconds))
            .clamp(0.0, 0.9),
        1,
        curve: Motion.pop,
      ),
      builder: (context, v, child) => Transform.scale(scale: v, child: child),
      child: icon,
    );
  }
}
