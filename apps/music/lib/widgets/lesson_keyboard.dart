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

import 'package:flutter/foundation.dart' show setEquals;
import 'package:flutter/material.dart';

import '../painters/piano_keyboard_painter.dart';
import '../painters/piano_layout.dart';

/// The one on-screen keyboard every course exercise embeds (change:
/// add-notation-courses, schema v2).
///
/// Owns the presentation rules a lesson keyboard shares: the range always
/// shows a real stretch of piano ([PianoLayout.lessonRange], ≥ 66 keys, so
/// keys keep believable proportions on any screen), octave anchors are hidden
/// (a "Do4" puck under the awaited key reads as clutter, not guidance), and
/// the awaited keys **breathe** — a three-cycle glow burst on every new target
/// that says "press here" without a word of explanation.
class LessonKeyboard extends StatefulWidget {
  const LessonKeyboard({
    super.key,
    required this.rangeTargets,
    this.activeNotes = const {},
    this.requiredNotes = const {},
    this.selectedNotes = const {},
    this.noteLabels = const {},
    this.solfege = false,
    this.frenchRe = false,
    this.onKeyDown,
    this.height = 132,
    this.paintKey,
  });

  /// Every pitch the exercise may ask for — determines the drawn range, which
  /// stays FIXED for the exercise's whole life (a jumping keyboard disorients).
  final Iterable<int> rangeTargets;

  final Set<int> activeNotes;
  final Set<int> requiredNotes;
  final Set<int> selectedNotes;
  final Map<int, String> noteLabels;
  final bool solfege;
  final bool frenchRe;

  /// Called with the tapped pitch (hit-tested through the shared layout).
  final ValueChanged<int>? onKeyDown;

  final double height;

  /// Key put on the CustomPaint, so each exercise keeps its test finder.
  final Key? paintKey;

  @override
  State<LessonKeyboard> createState() => _LessonKeyboardState();
}

class _LessonKeyboardState extends State<LessonKeyboard>
    with SingleTickerProviderStateMixin {
  late final AnimationController _pulse = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 500),
  );

  /// Invalidates a running burst when a newer one starts (or we dispose).
  int _burstGen = 0;

  late final ({int low, int high}) _range = PianoLayout.lessonRange(
    widget.rangeTargets,
  );

  @override
  void initState() {
    super.initState();
    if (widget.requiredNotes.isNotEmpty) _runBurst();
  }

  @override
  void didUpdateWidget(LessonKeyboard old) {
    super.didUpdateWidget(old);
    // A new target (or target set) gets its own "press here" burst.
    if (!setEquals(old.requiredNotes, widget.requiredNotes) &&
        widget.requiredNotes.isNotEmpty) {
      _runBurst();
    }
  }

  @override
  void dispose() {
    _burstGen++;
    _pulse.dispose();
    super.dispose();
  }

  /// Three breathing cycles, then rest — finite by design, so the cue draws
  /// the eye without turning into a permanent blink (and tests can settle).
  Future<void> _runBurst() async {
    final gen = ++_burstGen;
    _pulse.value = 0;
    for (var i = 0; i < 3; i++) {
      if (!mounted || gen != _burstGen) return;
      await _pulse.animateTo(1, curve: Curves.easeInOut);
      if (!mounted || gen != _burstGen) return;
      await _pulse.animateBack(0, curve: Curves.easeInOut);
    }
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final layout = PianoLayout(
          lowPitch: _range.low,
          highPitch: _range.high,
          width: constraints.maxWidth,
        );
        return Listener(
          onPointerDown: (e) {
            final pitch = layout.pitchAt(e.localPosition, widget.height);
            if (pitch != null) widget.onKeyDown?.call(pitch);
          },
          child: AnimatedBuilder(
            animation: _pulse,
            builder: (context, _) => CustomPaint(
              key: widget.paintKey,
              size: Size(constraints.maxWidth, widget.height),
              painter: PianoKeyboardPainter(
                layout: layout,
                // Fresh sets on every build: the painter compares by identity
                // to decide on repainting.
                activeNotes: {...widget.activeNotes},
                requiredNotes: {...widget.requiredNotes},
                selectedNotes: {...widget.selectedNotes},
                noteLabels: widget.noteLabels,
                solfege: widget.solfege,
                frenchRe: widget.frenchRe,
                waitPulse: _pulse.value,
                showOctaveMarkers: false,
              ),
            ),
          ),
        );
      },
    );
  }
}
