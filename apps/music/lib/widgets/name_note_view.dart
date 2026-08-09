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
import '../l10n/gen/app_localizations.dart';
import '../services/audio_service.dart';
import '../state/note_label.dart' show NoteFigure;
import '../theme/cymbra_theme.dart';
import 'lesson_staff.dart';
import 'reading_aid.dart' show namingConventionOf;

/// A course `nameNote` drill (change: add-notation-courses, schema v2): one
/// note at a time on a [LessonStaff], named among localized chips.
///
/// The choices are **deterministic** — the correct name plus the adjacent
/// *degrees* as naturals (degree −1, +1, +2, octave-adjusted to stay on the
/// piano), rotated by the queue position so the correct chip moves around
/// without any randomness. Tapping **any** chip sounds its pitch (browsing the
/// answers is ear training), through the [LessonSounder] audition seam
/// captured in `initState` — the sanctioned audition-widget pattern — so no
/// timer or voice can outlive the widget. A wrong attempt flashes the error
/// fill (never a success colour) and forfeits `flawless`; [onCompleted] fires
/// exactly once, after the check + chime completion beat.
class NameNoteView extends ConsumerStatefulWidget {
  const NameNoteView({
    super.key,
    required this.block,
    required this.onCompleted,
  });

  final NameNoteBlock block;

  /// Called exactly once, when every note has been named. `flawless` is false
  /// if any wrong chip was tapped along the way.
  final void Function({required bool flawless}) onCompleted;

  @override
  ConsumerState<NameNoteView> createState() => _NameNoteViewState();
}

class _NameNoteViewState extends ConsumerState<NameNoteView> {
  late final LessonSounder _sounder;
  final Set<Timer> _timers = {};
  int _queueIndex = 0;
  bool _advancing = false; // correct chip picked; brief tertiary beat
  int? _wrongIndex; // chip currently flashing the error fill
  bool _missed = false;
  bool _done = false;
  bool _completed = false;

  @override
  void initState() {
    super.initState();
    // Capture the service here (sanctioned audition-widget pattern): dispose
    // can then silence everything without touching ref.
    _sounder = LessonSounder(ref.read(audioServiceProvider));
  }

  @override
  void dispose() {
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

  LessonPitch get _current =>
      widget.block.notes[_queueIndex.clamp(0, widget.block.notes.length - 1)];

  /// The natural sitting [offset] diatonic degrees away from [target],
  /// octave-shifted if needed so its sounding pitch stays on the piano.
  static LessonPitch _neighborOf(LessonPitch target, int offset) {
    var diatonic = target.diatonic + offset;
    var p = _natural(diatonic);
    while (p.midi < 21) {
      diatonic += 7;
      p = _natural(diatonic);
    }
    while (p.midi > 108) {
      diatonic -= 7;
      p = _natural(diatonic);
    }
    return p;
  }

  static LessonPitch _natural(int diatonic) {
    final octave = (diatonic / 7).floor();
    return LessonPitch(diatonic - octave * 7, 0, octave);
  }

  /// The chips for the current note: the target (keeping its spelling) plus
  /// its neighbor degrees as naturals, rotated by the queue position so the
  /// correct chip moves around — all deterministic, no randomness.
  List<({LessonPitch pitch, bool correct})> _choices() {
    final target = _current;
    final count = widget.block.choiceCount.clamp(2, 4);
    final list = <LessonPitch>[target];
    for (final offset in const [-1, 1, 2]) {
      if (list.length >= count) break;
      list.add(_neighborOf(target, offset));
    }
    final rot = _queueIndex % list.length;
    final rotated = [...list.sublist(rot), ...list.sublist(0, rot)];
    return [for (final p in rotated) (pitch: p, correct: p == target)];
  }

  void _tapChip(int index, ({LessonPitch pitch, bool correct}) choice) {
    // Every chip sounds — browsing the answers is ear training.
    _sounder.tap(choice.pitch.midi);
    if (_done || _advancing) return;
    if (choice.correct) {
      setState(() => _advancing = true);
      _after(const Duration(milliseconds: 500), _advance);
      return;
    }
    setState(() {
      _missed = true;
      _wrongIndex = index;
    });
    _after(const Duration(milliseconds: 350), () {
      setState(() => _wrongIndex = null);
    });
  }

  void _advance() {
    if (_queueIndex + 1 < widget.block.notes.length) {
      setState(() {
        _queueIndex++;
        _advancing = false;
        _wrongIndex = null;
      });
      return;
    }
    setState(() => _done = true);
    _sounder.chime();
    _after(const Duration(milliseconds: 450), () {
      if (_completed) return;
      _completed = true;
      widget.onCompleted(flawless: !_missed);
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final naming = namingConventionOf(context);
    final b = widget.block;
    final note = _current;
    final choices = _choices();
    final prompt = resolveInline(b.prompt, lang);

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
        const SizedBox(height: 12),
        LessonStaff(
          clef: b.clef ?? note.nearestClef,
          keyFifths: b.keyFifths,
          elements: [
            LessonStaffElement(
              pitch: note,
              fig: const RhythmFigure(NoteFigure.quarter),
            ),
          ],
          elementColors: {if (_advancing || _done) 0: CymbraColors.tertiary},
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (var i = 0; i < choices.length; i++)
              OutlinedButton(
                key: Key('namenote-chip-$i'),
                onPressed: () => _tapChip(i, choices[i]),
                style: OutlinedButton.styleFrom(
                  backgroundColor: (_advancing || _done) && choices[i].correct
                      ? CymbraColors.tertiary.withValues(alpha: 0.15)
                      : (_wrongIndex == i
                            ? CymbraColors.error.withValues(alpha: 0.12)
                            : null),
                ),
                child: Text(
                  choices[i].pitch.name.label(
                    solfege: naming.solfege,
                    frenchRe: naming.frenchRe,
                  ),
                  style: const TextStyle(color: CymbraColors.onSurface),
                ),
              ),
          ],
        ),
        const SizedBox(height: 12),
        Row(
          children: [
            for (var i = 0; i < b.notes.length; i++)
              Container(
                key: Key('namenote-dot-$i'),
                width: 8,
                height: 8,
                margin: const EdgeInsets.only(right: 6),
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: _done || i < _queueIndex
                      ? CymbraColors.tertiary
                      : (i == _queueIndex
                            ? CymbraColors.primary
                            : CymbraColors.surfaceContainerHighest),
                ),
              ),
          ],
        ),
      ],
    );
  }
}
