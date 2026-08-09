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

import '../courses/lesson_sounder.dart';
import '../painters/piano_keyboard_painter.dart';
import '../painters/piano_layout.dart';
import '../services/audio_service.dart';
import '../services/midi_service.dart';
import '../src/rust/api/midi.dart' show MidiEvent, MidiEventKind;
import '../theme/cymbra_theme.dart';

/// A course `playKey` step (change: add-notation-courses): asks the user to play
/// the target [notes] and validates the input from **the on-screen keyboard or a
/// connected MIDI instrument** (the same seams the game uses). When every target
/// note has been played it calls [onSatisfied] (the lesson advances) — but the
/// player's Next is always a non-blocking skip, so a stuck user is never trapped.
class PlayKeyView extends ConsumerStatefulWidget {
  const PlayKeyView({
    super.key,
    required this.notes,
    required this.prompt,
    required this.onSatisfied,
  });

  final List<int> notes;
  final String prompt;
  final VoidCallback onSatisfied;

  @override
  ConsumerState<PlayKeyView> createState() => _PlayKeyViewState();
}

class _PlayKeyViewState extends ConsumerState<PlayKeyView> {
  late final Set<int> _targets = widget.notes.toSet();
  final Set<int> _hit = {};
  final Set<int> _held = {}; // brief press flash
  StreamSubscription<MidiEvent>? _sub;
  final Set<Timer> _timers = {};
  late final LessonSounder _sounder;
  bool _done = false;

  @override
  void initState() {
    super.initState();
    // On-screen taps must sound — the app is an instrument even mid-lesson.
    // (A MIDI instrument is heard acoustically, so its input is not re-voiced.)
    _sounder = LessonSounder(ref.read(audioServiceProvider));
    // Listen to a connected MIDI instrument too, not just on-screen taps.
    _sub = ref.read(midiServiceProvider).events().listen((e) {
      if (e.kind == MidiEventKind.noteOn && e.velocity > 0) _play(e.pitch);
    });
  }

  @override
  void dispose() {
    _sub?.cancel();
    for (final t in _timers) {
      t.cancel();
    }
    _sounder.dispose();
    super.dispose();
  }

  /// A cancellable delay that never leaks a timer past [dispose].
  void _after(Duration d, VoidCallback fn) {
    late final Timer t;
    t = Timer(d, () {
      _timers.remove(t);
      if (mounted) fn();
    });
    _timers.add(t);
  }

  void _play(int pitch) {
    if (_done) return;
    setState(() => _held.add(pitch));
    _after(const Duration(milliseconds: 260), () {
      setState(() => _held.remove(pitch));
    });
    if (!_targets.contains(pitch)) return;
    setState(() => _hit.add(pitch));
    if (_hit.containsAll(_targets) && !_done) {
      _done = true;
      _after(const Duration(milliseconds: 450), widget.onSatisfied);
    }
  }

  ({int low, int high}) _range() {
    final min = _targets.reduce((a, b) => a < b ? a : b);
    final max = _targets.reduce((a, b) => a > b ? a : b);
    var low = min - 2;
    var high = max + 2;
    while (PianoLayout.isBlack(low)) {
      low--;
    }
    while (PianoLayout.isBlack(high)) {
      high++;
    }
    low = low.clamp(21, 103);
    high = high.clamp(low + 7, 108);
    return (low: low, high: high);
  }

  @override
  Widget build(BuildContext context) {
    final r = _range();
    const height = 132.0;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                widget.prompt,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 16,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            if (_done)
              const Icon(Icons.check_circle, color: CymbraColors.tertiary),
          ],
        ),
        const SizedBox(height: 12),
        LayoutBuilder(
          builder: (context, constraints) {
            final layout = PianoLayout(
              lowPitch: r.low,
              highPitch: r.high,
              width: constraints.maxWidth,
            );
            return Listener(
              onPointerDown: (e) {
                final pitch = layout.pitchAt(e.localPosition, height);
                if (pitch != null) {
                  _sounder.tap(pitch);
                  _play(pitch);
                }
              },
              child: CustomPaint(
                size: Size(constraints.maxWidth, height),
                painter: PianoKeyboardPainter(
                  layout: layout,
                  activeNotes: _held,
                  // Still-to-play targets stay highlighted (the standing hint).
                  requiredNotes: _targets.difference(_hit),
                ),
              ),
            );
          },
        ),
      ],
    );
  }
}
