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

import 'package:flutter/foundation.dart' show setEquals;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../courses/course_manifest.dart';
import '../courses/lesson_sounder.dart';
import '../l10n/gen/app_localizations.dart';
import '../painters/piano_keyboard_painter.dart';
import '../painters/piano_layout.dart';
import '../services/audio_service.dart';
import '../services/midi_service.dart';
import '../src/rust/api/midi.dart' show MidiEvent, MidiEventKind;
import '../theme/cymbra_theme.dart';
import 'reading_aid.dart' show namingConventionOf;

/// A course `buildChord` step (change: add-notation-courses, schema v2): the
/// learner *toggles* keys on the keyboard (on-screen taps or a MIDI
/// instrument) to assemble [BuildChordBlock.notes]. The selection is
/// persistent — teal via the painter's `selectedNotes` — unlike the momentary
/// press flash.
///
/// Once as many keys are selected as the chord has notes, the attempt
/// auto-validates and is always **strummed**: a right chord plays itself as a
/// reward, a wrong one plays what the learner actually built so the error is
/// heard, not just marked. A wrong set stays on the keys for the learner to
/// adjust (never cleared, never shown in a success colour). [onCompleted]
/// fires exactly once, after the completion convention (check mark, chime,
/// short pause).
class BuildChordView extends ConsumerStatefulWidget {
  const BuildChordView({
    super.key,
    required this.block,
    required this.onCompleted,
  });

  final BuildChordBlock block;

  /// Called exactly once, when the exact chord has been built; [flawless] is
  /// false when any wrong full set was validated along the way.
  final void Function({required bool flawless}) onCompleted;

  @override
  ConsumerState<BuildChordView> createState() => _BuildChordViewState();
}

class _BuildChordViewState extends ConsumerState<BuildChordView> {
  /// Captured in [initState] (audition-widget pattern) so [dispose] can
  /// silence and cancel everything without touching `ref`.
  late final LessonSounder _sounder;

  late final Set<int> _targets = {for (final p in widget.block.notes) p.midi};

  // The painter compares sets by identity, so every change assigns a FRESH
  // Set instance rather than mutating in place.
  Set<int> _selection = const {};
  Set<int> _active = const {};
  Set<int> _required = const {};

  StreamSubscription<MidiEvent>? _sub;
  final Set<Timer> _timers = {};
  bool _missed = false;

  /// True while a wrong attempt strums/flashes — input is briefly ignored so
  /// overlapping validations can't stack.
  bool _validating = false;
  bool _done = false;

  @override
  void initState() {
    super.initState();
    _sounder = LessonSounder(ref.read(audioServiceProvider));
    // A connected MIDI instrument toggles too; it makes its own sound, so no
    // app-synth tap is layered on top.
    _sub = ref.read(midiServiceProvider).events().listen((e) {
      if (e.kind == MidiEventKind.noteOn && e.velocity > 0) {
        _toggle(e.pitch, sound: false);
      }
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

  /// A cancellable delay that never fires past [dispose].
  void _after(Duration d, VoidCallback fn) {
    late final Timer t;
    t = Timer(d, () {
      _timers.remove(t);
      if (mounted) fn();
    });
    _timers.add(t);
  }

  void _toggle(int pitch, {required bool sound}) {
    if (_done || _validating) return;
    if (sound) _sounder.tap(pitch);
    setState(() {
      _selection = _selection.contains(pitch)
          ? ({..._selection}..remove(pitch))
          : {..._selection, pitch};
    });
    if (_selection.length == _targets.length) _validate();
  }

  void _validate() {
    if (setEquals(_selection, _targets)) {
      setState(() {
        _done = true;
        // Targets both required and active → the painter's "correct" state.
        _required = {..._targets};
        _active = {..._targets};
      });
      final chord = _targets.toList()..sort();
      _sounder.playSequence(
        chord,
        gapMs: 90,
        noteMs: 500,
        onDone: () {
          _sounder.chime();
          _after(
            const Duration(milliseconds: 450),
            () => widget.onCompleted(flawless: !_missed),
          );
        },
      );
    } else {
      _missed = true;
      _validating = true;
      // Flash only the keys that don't belong — never a success colour.
      setState(() => _active = {..._selection.difference(_targets)});
      // Strum what was actually built, so the learner hears *why* it's wrong.
      _sounder.playSequence(_selection.toList()..sort(), gapMs: 90);
      _after(const Duration(milliseconds: 500), () {
        setState(() {
          _active = const {};
          _validating = false;
        });
      });
    }
  }

  /// Keyboard range: the chord ±2 semitones, widened to white keys so the
  /// keyboard never starts or ends on a black one.
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
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final naming = namingConventionOf(context);
    final prompt = resolveInline(widget.block.prompt, lang);
    final names = [
      for (final p in widget.block.notes)
        p.name.label(solfege: naming.solfege, frenchRe: naming.frenchRe),
    ].join(' – ');
    final r = _range();
    const height = 132.0;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                prompt.isEmpty ? l10n.lessonYourTurn : prompt,
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
        const SizedBox(height: 4),
        Text(
          names,
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 14,
            fontWeight: FontWeight.w600,
          ),
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
                if (pitch != null) _toggle(pitch, sound: true);
              },
              child: CustomPaint(
                key: const Key('buildchord-keyboard'),
                size: Size(constraints.maxWidth, height),
                painter: PianoKeyboardPainter(
                  layout: layout,
                  activeNotes: _active,
                  requiredNotes: _required,
                  selectedNotes: _selection,
                ),
              ),
            );
          },
        ),
      ],
    );
  }
}
