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
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../painters/hit_effects_painter.dart';
import '../painters/piano_layout.dart';
import '../services/clock_service.dart';
import '../state/performance_scoring.dart';
import '../state/player_notifier.dart';
import 'scoring_gauge.dart';

/// The gamified-feedback layer stacked over a scored render area: the transient
/// hit sparks along the note-hit line plus the [ScoringGauge] in the top-right
/// corner. Renders nothing when no scored run is active, and is wrapped in an
/// [IgnorePointer] so it never intercepts keyboard/gesture input — the play
/// surface underneath stays fully interactive and legible.
class ScoringOverlay extends ConsumerWidget {
  const ScoringOverlay({
    super.key,
    required this.layout,
    this.showEffects = true,
  });

  final PianoLayout layout;

  /// Whether to draw the hit sparks. They anchor to the keyboard/note-hit line
  /// at the bottom, so they are suppressed when the keyboard is hidden — the
  /// gauge still shows.
  final bool showEffects;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final active = ref.watch(performanceScorerProvider.select((s) => s.active));
    if (!active) return const SizedBox.shrink();

    return IgnorePointer(
      child: Stack(
        children: [
          if (showEffects) Positioned.fill(child: _HitEffects(layout: layout)),
          const Positioned(top: 8, right: 8, child: ScoringGauge()),
        ],
      ),
    );
  }
}

/// The transient hit-spark layer. Split out so watching the playhead each frame
/// (for the fade) only happens when the sparks are actually drawn.
class _HitEffects extends ConsumerWidget {
  const _HitEffects({required this.layout});

  final PianoLayout layout;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Rebuild each frame while the playhead advances so the sparks fade
    // smoothly; the scorer's own state drives the hit list.
    ref.watch(playerProvider.select((d) => d.elapsedMs));
    final hits = ref.watch(
      performanceScorerProvider.select((s) => s.recentHits),
    );
    final nowMs = ref.read(clockProvider).nowMs();
    return CustomPaint(
      painter: HitEffectsPainter(layout: layout, hits: hits, nowMs: nowMs),
    );
  }
}
