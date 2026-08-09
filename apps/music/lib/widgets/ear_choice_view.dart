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
import '../courses/lesson_rhythm.dart';
import '../courses/lesson_sounder.dart';
import '../l10n/gen/app_localizations.dart';
import '../services/audio_service.dart';
import '../state/note_label.dart' show NoteFigure;
import '../theme/cymbra_theme.dart';
import 'lesson_staff.dart';

/// A course `earChoice` exercise (change: add-notation-courses, schema v2):
/// listen to a short sequence, then answer a choice question about it.
///
/// The sequence auto-plays once shortly after mount and the Listen button
/// replays it **free and unlimited** — the ear is the whole exercise, so
/// re-listening is never penalized. A wrong chip flashes the error fill (never
/// a success colour), forfeits `flawless` and replays the sequence once so the
/// learner hears again what they just misjudged. The correct chip fills
/// tertiary and — when the block asks to `reveal` — shows the notes on a
/// [LessonStaff] so the eye confirms what the ear heard. All sound goes
/// through the [LessonSounder] audition seam captured in `initState` (the
/// sanctioned audition-widget pattern), so no timer or voice outlives the
/// widget; [onCompleted] fires exactly once, after the check + chime beat.
class EarChoiceView extends ConsumerStatefulWidget {
  const EarChoiceView({
    super.key,
    required this.block,
    required this.onCompleted,
  });

  final EarChoiceBlock block;

  /// Called exactly once, when the right choice has been picked. `flawless` is
  /// false if any wrong chip was tapped along the way.
  final void Function({required bool flawless}) onCompleted;

  @override
  ConsumerState<EarChoiceView> createState() => _EarChoiceViewState();
}

class _EarChoiceViewState extends ConsumerState<EarChoiceView> {
  late final LessonSounder _sounder;
  final Set<Timer> _timers = {};
  String? _wrongId; // chip currently flashing the error fill
  bool _missed = false;
  bool _done = false;
  bool _completed = false;

  @override
  void initState() {
    super.initState();
    // Capture the service here (sanctioned audition-widget pattern): dispose
    // can then silence everything without touching ref.
    _sounder = LessonSounder(ref.read(audioServiceProvider));
    // Auto-play once shortly after mount — the learner should not have to find
    // the button before the exercise makes sense.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _after(const Duration(milliseconds: 400), _play);
    });
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

  void _play() {
    _sounder.playSequence(
      [for (final n in widget.block.notes) n.midi],
      gapMs: widget.block.gapMs,
      harmonic: widget.block.harmonic,
    );
  }

  void _choose(EarOption option) {
    if (_done) return;
    if (option.id == widget.block.answerId) {
      setState(() => _done = true);
      _sounder.chime();
      _after(const Duration(milliseconds: 450), () {
        if (_completed) return;
        _completed = true;
        widget.onCompleted(flawless: !_missed);
      });
      return;
    }
    setState(() {
      _missed = true;
      _wrongId = option.id;
    });
    _after(const Duration(milliseconds: 350), () {
      setState(() => _wrongId = null);
    });
    // Replay once so the learner hears again what they just misjudged.
    _after(const Duration(milliseconds: 400), _play);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final b = widget.block;
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
        Center(
          child: FilledButton.icon(
            key: const Key('earchoice-listen'),
            onPressed: _play,
            icon: const Icon(Icons.volume_up),
            label: Text(l10n.lessonListen),
          ),
        ),
        const SizedBox(height: 16),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final option in b.choices)
              OutlinedButton(
                key: Key('earchoice-chip-${option.id}'),
                onPressed: () => _choose(option),
                style: OutlinedButton.styleFrom(
                  backgroundColor: _done && option.id == b.answerId
                      ? CymbraColors.tertiary.withValues(alpha: 0.15)
                      : (_wrongId == option.id
                            ? CymbraColors.error.withValues(alpha: 0.12)
                            : null),
                ),
                child: Text(
                  resolveInline(option.label, lang),
                  style: const TextStyle(color: CymbraColors.onSurface),
                ),
              ),
          ],
        ),
        if (_done && b.reveal) ...[
          const SizedBox(height: 16),
          // The eye confirms what the ear heard.
          LessonStaff(
            clef: b.notes.first.nearestClef,
            elements: [
              for (final n in b.notes)
                LessonStaffElement(
                  pitch: n,
                  fig: const RhythmFigure(NoteFigure.quarter),
                ),
            ],
            elementColors: {
              for (var i = 0; i < b.notes.length; i++) i: CymbraColors.tertiary,
            },
          ),
        ],
        if (_done || _missed) ...[
          const SizedBox(height: 8),
          Text(
            _done ? l10n.lessonWellDone : l10n.lessonAlmost,
            style: TextStyle(
              color: _done ? CymbraColors.tertiary : CymbraColors.error,
              fontWeight: FontWeight.w700,
            ),
          ),
        ],
      ],
    );
  }
}
