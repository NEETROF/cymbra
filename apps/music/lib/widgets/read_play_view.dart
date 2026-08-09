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

import '../courses/course_manifest.dart';
import '../courses/lesson_pitch.dart';
import '../courses/lesson_rhythm.dart';
import '../courses/lesson_sounder.dart';
import '../services/audio_service.dart';
import '../services/midi_service.dart';
import '../src/rust/api/midi.dart' show MidiEvent, MidiEventKind;
import '../state/note_label.dart';
import '../theme/cymbra_theme.dart';
import 'lesson_keyboard.dart';
import 'lesson_staff.dart';
import 'reading_aid.dart' show namingConventionOf;

/// A course `readPlay` step (change: add-notation-courses, schema v2): read one
/// or more notes on a [LessonStaff] and play them on the on-screen keyboard or
/// a connected MIDI instrument — the core staff→key exercise, in the three
/// shapes of [ReadPlayMode] (a one-at-a-time drill, an in-order melody, an
/// any-order chord set).
///
/// A wrong key is never punished with lost progress: it flashes, is *heard*
/// (screen taps sound through [LessonSounder]; a real instrument already sounds
/// acoustically, so MIDI input is deliberately not echoed), and only costs the
/// `flawless` flag reported to [onCompleted]. Note-name help on the awaited
/// keys follows [ReadPlayBlock.labels] — shown from the start, revealed after
/// two misses on the same target, or never.
class ReadPlayView extends ConsumerStatefulWidget {
  const ReadPlayView({
    super.key,
    required this.block,
    required this.onCompleted,
  });

  final ReadPlayBlock block;

  /// Called exactly once, when every note has been played; [flawless] is false
  /// as soon as a single wrong key was pressed at any point.
  final void Function({required bool flawless}) onCompleted;

  @override
  ConsumerState<ReadPlayView> createState() => _ReadPlayViewState();
}

class _ReadPlayViewState extends ConsumerState<ReadPlayView> {
  /// Non-empty by construction: the manifest parser declines a `readPlay`
  /// block without notes.
  late final List<LessonPitch> _notes = widget.block.notes;

  late final LessonSounder _sounder;
  StreamSubscription<MidiEvent>? _sub;
  final Set<Timer> _timers = {};

  /// Next note to play (drill and melody); notes before it are done.
  int _index = 0;

  /// Notes validated in `set` mode — indexes, not pitches, so a duplicated
  /// pitch in a chord still needs each of its entries.
  final Set<int> _doneIndexes = {};

  /// Momentary press flash on the keyboard (a right or free press).
  final Set<int> _held = {};

  /// Momentary coral flash on a judged-wrong key — a mistake must never wear
  /// a neutral or success colour.
  final Set<int> _wrongFlash = {};

  /// Wrong keys pressed on the *current* target — two of them reveal the
  /// note-name help in [LessonLabelMode.afterMiss].
  int _missStreak = 0;
  bool _flawed = false;

  /// Drill only: the short tertiary flash between a correct key and the queue
  /// advancing, during which input sounds but is not judged.
  bool _advancing = false;
  bool _done = false;

  @override
  void initState() {
    super.initState();
    // Capture the service now: dispose() must not touch ref.
    _sounder = LessonSounder(ref.read(audioServiceProvider));
    // Listen to a connected MIDI instrument too, not just on-screen taps.
    _sub = ref.read(midiServiceProvider).events().listen((e) {
      if (e.kind == MidiEventKind.noteOn && e.velocity > 0) {
        _play(e.pitch, fromScreen: false);
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

  /// A cancellable delay that never leaks a timer past [dispose].
  void _after(Duration d, VoidCallback fn) {
    late final Timer t;
    t = Timer(d, () {
      _timers.remove(t);
      if (mounted) fn();
    });
    _timers.add(t);
  }

  void _play(int pitch, {required bool fromScreen}) {
    if (_done) return;
    // Screen taps must be heard; a MIDI instrument already sounds acoustically,
    // so echoing it would double every note.
    if (fromScreen) _sounder.tap(pitch);
    // Judge before flashing: a wrong key flashes coral, never a colour that
    // could read as accepted. Free presses during the validation beat stay
    // neutral.
    final setIdx = widget.block.mode == ReadPlayMode.set
        ? _matchInSet(pitch)
        : null;
    final wrong =
        !_advancing &&
        (widget.block.mode == ReadPlayMode.set
            ? setIdx == null
            : pitch != _notes[_index.clamp(0, _notes.length - 1)].midi);
    final flash = wrong ? _wrongFlash : _held;
    setState(() => flash.add(pitch));
    _after(const Duration(milliseconds: 260), () {
      setState(() => flash.remove(pitch));
    });
    if (_advancing) return;

    if (widget.block.mode == ReadPlayMode.set) {
      if (setIdx == null) {
        _miss();
        return;
      }
      setState(() => _doneIndexes.add(setIdx));
      if (_doneIndexes.length == _notes.length) _complete();
      return;
    }

    if (wrong) {
      _miss();
      return;
    }
    if (widget.block.mode == ReadPlayMode.drill) {
      // Let the learner *see* the note validate on the staff before it is
      // replaced by the next one.
      setState(() => _advancing = true);
      _after(const Duration(milliseconds: 350), _advance);
    } else {
      _advance();
    }
  }

  void _advance() {
    setState(() {
      _advancing = false;
      _index++;
      _missStreak = 0;
    });
    if (_index >= _notes.length) _complete();
  }

  void _miss() {
    setState(() {
      _flawed = true;
      _missStreak++;
    });
  }

  /// The first not-yet-validated `set` entry sounding as [pitch], or null.
  int? _matchInSet(int pitch) {
    for (var i = 0; i < _notes.length; i++) {
      if (!_doneIndexes.contains(i) && _notes[i].midi == pitch) return i;
    }
    return null;
  }

  void _complete() {
    if (_done) return;
    setState(() => _done = true);
    _sounder.chime();
    _after(const Duration(milliseconds: 450), () {
      widget.onCompleted(flawless: !_flawed);
    });
  }

  /// The note the staff centres on in drill mode (the last one once done).
  LessonPitch get _current => _notes[_index.clamp(0, _notes.length - 1)];

  LessonClef get _clef =>
      widget.block.clef ??
      (widget.block.mode == ReadPlayMode.drill ? _current : _notes.first)
          .nearestClef;

  List<LessonStaffElement> _staffElements() {
    const fig = RhythmFigure(NoteFigure.quarter);
    if (widget.block.mode == ReadPlayMode.drill) {
      return [LessonStaffElement(pitch: _current, fig: fig)];
    }
    return [for (final p in _notes) LessonStaffElement(pitch: p, fig: fig)];
  }

  Map<int, Color> _staffColors() => switch (widget.block.mode) {
    ReadPlayMode.drill =>
      _advancing || _done ? const {0: CymbraColors.tertiary} : const {},
    ReadPlayMode.melody => {
      for (var i = 0; i < _index; i++) i: CymbraColors.tertiary,
      if (_index < _notes.length) _index: CymbraColors.primary,
    },
    ReadPlayMode.set => {
      for (final i in _doneIndexes) i: CymbraColors.tertiary,
    },
  };

  /// The keys awaited right now — the keyboard's standing hint.
  Set<int> _required() {
    if (_done || _advancing || _index >= _notes.length) return {};
    if (widget.block.mode == ReadPlayMode.set) {
      return {
        for (var i = 0; i < _notes.length; i++)
          if (!_doneIndexes.contains(i)) _notes[i].midi,
      };
    }
    return {_notes[_index].midi};
  }

  /// Note names on the awaited keys, per [ReadPlayBlock.labels].
  Map<int, String> _labels(BuildContext context, Set<int> required) {
    final mode = widget.block.labels;
    if (mode == LessonLabelMode.never) return const {};
    if (mode == LessonLabelMode.afterMiss && _missStreak < 2) return const {};
    final naming = namingConventionOf(context);
    return {
      for (final p in _notes)
        if (required.contains(p.midi))
          p.midi: p.name.label(
            solfege: naming.solfege,
            frenchRe: naming.frenchRe,
          ),
    };
  }

  bool _dotDone(int i) => i < _index || (i == _index && (_advancing || _done));

  @override
  Widget build(BuildContext context) {
    final block = widget.block;
    final lang = Localizations.localeOf(context).languageCode;
    final naming = namingConventionOf(context);
    final required = _required();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                resolveInline(block.prompt, lang),
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
        if (block.mode == ReadPlayMode.drill) ...[
          Row(
            children: [
              for (var i = 0; i < _notes.length; i++)
                Padding(
                  padding: EdgeInsets.only(left: i == 0 ? 0 : 6),
                  child: Container(
                    key: Key('readplay-dot-$i'),
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: _dotDone(i)
                          ? CymbraColors.tertiary
                          : CymbraColors.outline,
                    ),
                  ),
                ),
            ],
          ),
          const SizedBox(height: 8),
        ],
        LessonStaff(
          clef: _clef,
          keyFifths: block.keyFifths,
          elements: _staffElements(),
          elementColors: _staffColors(),
          stacked: block.mode == ReadPlayMode.set,
        ),
        const SizedBox(height: 12),
        LessonKeyboard(
          paintKey: const Key('readplay-keyboard'),
          rangeTargets: [for (final p in _notes) p.midi],
          activeNotes: _held,
          requiredNotes: required,
          wrongNotes: _wrongFlash,
          noteLabels: _labels(context, required),
          solfege: naming.solfege,
          frenchRe: naming.frenchRe,
          onKeyDown: (pitch) => _play(pitch, fromScreen: true),
        ),
      ],
    );
  }
}
