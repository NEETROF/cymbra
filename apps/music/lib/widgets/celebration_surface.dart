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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../painters/confetti_painter.dart';
import '../services/haptics_service.dart';
import '../theme/cymbra_theme.dart';
import '../theme/motion.dart';
import 'animated_counter.dart';

/// The one animated celebration every reward moment goes through (change:
/// add-music-motion-and-celebrations): the icon scales in, a burst of drawn
/// confetti fires once, the title settles, a counter counts up when points are
/// involved, and the actions fade in last — interactive only once the sequence
/// is over, so nothing can be tapped mid-flight. Under reduced motion the whole
/// thing paints its final frame at once.
///
/// It is a dialog body: callers wrap it in [Dialog] and keep their own keys,
/// strings and actions. The accent is the reward accent, never success green.
class CelebrationSurface extends ConsumerStatefulWidget {
  const CelebrationSurface({
    super.key,
    required this.icon,
    required this.title,
    required this.actions,
    this.subtitle,
    this.extra,
    this.counterTo,
    this.counterFrom,
    this.counterFormat,
    this.accent = CymbraColors.primary,
    this.haptic = true,
  });

  final IconData icon;
  final String title;

  /// A line under the title (the lesson's name, the badge's label…).
  final String? subtitle;

  /// Anything else the moment carries (a first-try stat, a stars row).
  final Widget? extra;

  /// When set, a number counting from [counterFrom] (0 by default) to here.
  final int? counterTo;
  final int? counterFrom;
  final String Function(int value)? counterFormat;

  /// Actions, in order; enabled only once the sequence is complete.
  final List<Widget> actions;
  final Color accent;

  /// One light haptic as the sequence starts (a pass, a completion).
  final bool haptic;

  @override
  ConsumerState<CelebrationSurface> createState() => _CelebrationSurfaceState();
}

class _CelebrationSurfaceState extends ConsumerState<CelebrationSurface>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(
    vsync: this,
    duration: Motion.celebrate,
  );
  final ConfettiField _field = ConfettiField();
  bool _started = false;

  late final Animation<double> _iconScale = CurvedAnimation(
    parent: _c,
    curve: const Interval(0.0, 0.45, curve: Motion.pop),
  );
  late final Animation<double> _confetti = CurvedAnimation(
    parent: _c,
    curve: const Interval(0.1, 1.0),
  );
  late final Animation<double> _title = CurvedAnimation(
    parent: _c,
    curve: const Interval(0.3, 0.6, curve: Motion.enter),
  );
  late final Animation<double> _actions = CurvedAnimation(
    parent: _c,
    curve: const Interval(0.7, 1.0, curve: Motion.enter),
  );

  @override
  void initState() {
    super.initState();
    _c.addStatusListener((_) => setState(() {}));
    if (widget.haptic) {
      unawaited(
        ref.read(hapticsServiceProvider).light().catchError((Object _) {}),
      );
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (_started) return;
    _started = true;
    // Reduced motion: the final frame, no animated frames at all.
    if (Motion.reduced(context)) {
      _c.value = 1;
    } else {
      _c.forward();
    }
  }

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final accent = widget.accent;
    final complete = _c.isCompleted;
    final tints = [
      accent,
      CymbraColors.primaryContainer,
      CymbraColors.secondary,
      const Color(0xFFF5D0FE),
    ];
    return ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: 380),
      child: Stack(
        children: [
          // The burst, over the dialog only — never over a play surface.
          Positioned.fill(
            child: IgnorePointer(
              child: RepaintBoundary(
                child: AnimatedBuilder(
                  animation: _confetti,
                  builder: (context, _) => CustomPaint(
                    painter: ConfettiPainter(
                      field: _field,
                      progress: _confetti.value,
                      colors: tints,
                      origin: const Offset(190, 70),
                    ),
                  ),
                ),
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(24, 28, 24, 20),
            // Scrollable so a short landscape-phone viewport can never make the
            // celebration overflow — it scrolls instead.
            child: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  ScaleTransition(
                    scale: _iconScale,
                    child: Container(
                      width: 72,
                      height: 72,
                      decoration: BoxDecoration(
                        color: accent.withValues(alpha: 0.15),
                        shape: BoxShape.circle,
                      ),
                      child: Icon(widget.icon, size: 40, color: accent),
                    ),
                  ),
                  const SizedBox(height: 16),
                  FadeTransition(
                    opacity: _title,
                    child: SlideTransition(
                      position: Tween<Offset>(
                        begin: const Offset(0, 0.25),
                        end: Offset.zero,
                      ).animate(_title),
                      child: Column(
                        children: [
                          Text(
                            widget.title,
                            textAlign: TextAlign.center,
                            style: TextStyle(
                              color: accent,
                              fontSize: 22,
                              fontWeight: FontWeight.w800,
                            ),
                          ),
                          if (widget.subtitle case final s?) ...[
                            const SizedBox(height: 6),
                            Text(
                              s,
                              textAlign: TextAlign.center,
                              style: const TextStyle(
                                color: CymbraColors.onSurface,
                                fontSize: 15,
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                          ],
                          if (widget.extra case final e?) ...[
                            const SizedBox(height: 10),
                            e,
                          ],
                          if (widget.counterTo case final to?) ...[
                            const SizedBox(height: 12),
                            AnimatedCounter(
                              key: const Key('celebration-counter'),
                              value: to,
                              initialFrom: widget.counterFrom ?? 0,
                              format: widget.counterFormat,
                              textAlign: TextAlign.center,
                              style: const TextStyle(
                                color: CymbraColors.onSurface,
                                fontSize: 28,
                                fontWeight: FontWeight.w800,
                              ),
                            ),
                          ],
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 20),
                  // Tappable only once the sequence is over.
                  IgnorePointer(
                    ignoring: !complete,
                    child: FadeTransition(
                      opacity: _actions,
                      child: Column(
                        key: const Key('celebration-actions'),
                        mainAxisSize: MainAxisSize.min,
                        children: widget.actions,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
