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

/// The direction a card was flung off the deck.
enum SwipeDirection { left, right, up }

/// A draggable card that reports a directional swipe (left / right / up) once the
/// user drags past a threshold or flings it, then animates off-screen (change:
/// add-app-score-rating). A vetted card-swiper package was evaluated (design D6);
/// with the app carrying zero swipe dependencies, this bounded custom stack
/// (drag + fling + snap-back) is the deliberate fallback. Swiping is a shortcut
/// only — the same actions are available as buttons, so this never needs to be
/// the sole path.
class SwipeCard extends StatefulWidget {
  const SwipeCard({
    super.key,
    required this.child,
    required this.onDislike,
    required this.onLike,
    required this.onLove,
  });

  final Widget child;

  /// Left swipe.
  final VoidCallback onDislike;

  /// Right swipe.
  final VoidCallback onLike;

  /// Up swipe.
  final VoidCallback onLove;

  @override
  State<SwipeCard> createState() => _SwipeCardState();
}

class _SwipeCardState extends State<SwipeCard>
    with SingleTickerProviderStateMixin {
  /// Drag distance past which a release commits the swipe (logical px).
  static const double _commitDistance = 110;

  /// Fling velocity past which a release commits regardless of distance (px/s).
  static const double _commitVelocity = 800;

  late final AnimationController _controller;
  Offset _drag = Offset.zero;

  /// While animating a committed swipe off-screen, the direction being committed
  /// (drives the exit target); `null` while idle or snapping back.
  SwipeDirection? _exiting;
  Offset _exitStart = Offset.zero;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 220),
    )..addListener(_onAnimate);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _onAnimate() {
    final t = Curves.easeOut.transform(_controller.value);
    setState(() {
      if (_exiting != null) {
        _drag = Offset.lerp(_exitStart, _exitTarget(_exiting!), t)!;
      } else {
        // Snap-back to centre.
        _drag = Offset.lerp(_exitStart, Offset.zero, t)!;
      }
    });
  }

  Offset _exitTarget(SwipeDirection dir) {
    final size = MediaQuery.of(context).size;
    return switch (dir) {
      SwipeDirection.left => Offset(-size.width * 1.5, _drag.dy),
      SwipeDirection.right => Offset(size.width * 1.5, _drag.dy),
      SwipeDirection.up => Offset(_drag.dx, -size.height * 1.4),
    };
  }

  void _onPanUpdate(DragUpdateDetails d) {
    if (_controller.isAnimating) return;
    setState(() => _drag += d.delta);
  }

  void _onPanEnd(DragEndDetails d) {
    if (_controller.isAnimating) return;
    final v = d.velocity.pixelsPerSecond;
    final dir = _decideDirection(v);
    if (dir == null) {
      _animateSnapBack();
    } else {
      _animateExit(dir);
    }
  }

  /// The committed direction for the current drag + release velocity, or `null`
  /// to snap back. Horizontal intent wins over vertical unless the up-fling is
  /// clearly dominant.
  SwipeDirection? _decideDirection(Offset velocity) {
    final dx = _drag.dx, dy = _drag.dy;
    final horizontal =
        dx.abs() > _commitDistance || velocity.dx.abs() > _commitVelocity;
    final upward =
        (dy < -_commitDistance || velocity.dy < -_commitVelocity) &&
        dy.abs() >= dx.abs();
    if (upward) return SwipeDirection.up;
    if (horizontal) return dx > 0 ? SwipeDirection.right : SwipeDirection.left;
    return null;
  }

  void _animateSnapBack() {
    _exiting = null;
    _exitStart = _drag;
    _controller.forward(from: 0);
  }

  void _animateExit(SwipeDirection dir) {
    _exiting = dir;
    _exitStart = _drag;
    _controller.forward(from: 0).whenComplete(() {
      switch (dir) {
        case SwipeDirection.left:
          widget.onDislike();
        case SwipeDirection.right:
          widget.onLike();
        case SwipeDirection.up:
          widget.onLove();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    // Rotate slightly with the horizontal drag for a natural card feel.
    final rotation = (_drag.dx / 1000).clamp(-0.2, 0.2);
    return GestureDetector(
      // Opaque so the whole card area is draggable regardless of what the child
      // paints (a translucent cover or gaps must not let a swipe fall through).
      behavior: HitTestBehavior.opaque,
      onPanUpdate: _onPanUpdate,
      onPanEnd: _onPanEnd,
      child: Transform.translate(
        offset: _drag,
        child: Transform.rotate(
          angle: rotation * math.pi / 4,
          child: widget.child,
        ),
      ),
    );
  }
}
