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
import 'dart:typed_data';

import 'package:flutter/material.dart';

/// How many pieces a burst has. One constant: lowering it is the whole
/// performance lever (change: add-music-motion-and-celebrations, D3).
const int kConfettiParticleCount = 40;

/// A burst's particles, computed once from a seed so a golden is stable and so
/// painting allocates nothing: per particle, the launch angle, speed, spin,
/// size and colour index live in flat typed arrays.
class ConfettiField {
  ConfettiField({int seed = 7, this.count = kConfettiParticleCount})
    : _angle = Float32List(count),
      _speed = Float32List(count),
      _spin = Float32List(count),
      _width = Float32List(count),
      _height = Float32List(count),
      _color = Uint8List(count) {
    final rng = math.Random(seed);
    for (var i = 0; i < count; i++) {
      // Mostly upward, fanning out: a burst, not a sprinkle.
      _angle[i] = -math.pi / 2 + (rng.nextDouble() - 0.5) * math.pi * 1.3;
      _speed[i] = 0.55 + rng.nextDouble() * 0.45;
      _spin[i] = (rng.nextDouble() - 0.5) * 6 * math.pi;
      _width[i] = 4 + rng.nextDouble() * 4;
      _height[i] = 6 + rng.nextDouble() * 6;
      _color[i] = rng.nextInt(4);
    }
  }

  final int count;
  final Float32List _angle;
  final Float32List _speed;
  final Float32List _spin;
  final Float32List _width;
  final Float32List _height;
  final Uint8List _color;
}

/// Draws a [ConfettiField] at [progress] (0 → 1): pieces fly out from
/// [origin], slow down, drop a little and fade. Drawn over a dialog only,
/// never over a play surface (the learning-safe rule).
class ConfettiPainter extends CustomPainter {
  const ConfettiPainter({
    required this.field,
    required this.progress,
    required this.colors,
    this.origin,
    this.spread = 150,
  });

  final ConfettiField field;
  final double progress;

  /// Four tints: the reward accent and its neighbours.
  final List<Color> colors;

  /// Where the burst starts; the canvas centre when null.
  final Offset? origin;
  final double spread;

  @override
  void paint(Canvas canvas, Size size) {
    if (progress <= 0 || progress >= 1) return;
    final o = origin ?? Offset(size.width / 2, size.height / 2);
    // Ease out: fast launch, gentle settle.
    final t = Curves.easeOutCubic.transform(progress);
    final alpha = (1 - progress).clamp(0.0, 1.0);
    final paint = Paint();
    for (var i = 0; i < field.count; i++) {
      final d = field._speed[i] * spread * t;
      final x = o.dx + math.cos(field._angle[i]) * d;
      final y =
          o.dy +
          math.sin(field._angle[i]) * d * 0.75 +
          60 * progress * progress;
      paint.color = colors[field._color[i] % colors.length].withValues(
        alpha: alpha,
      );
      canvas.save();
      canvas.translate(x, y);
      canvas.rotate(field._angle[i] + field._spin[i] * progress);
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromCenter(
            center: Offset.zero,
            width: field._width[i],
            height: field._height[i],
          ),
          const Radius.circular(1.5),
        ),
        paint,
      );
      canvas.restore();
    }
  }

  @override
  bool shouldRepaint(ConfettiPainter old) =>
      old.progress != progress || old.field != field || old.colors != colors;
}
