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

/// A course `placeNote` step (change: add-notation-courses, schema v2): the
/// learner is named a note and taps its position on the staff. Targets are
/// **naturals by construction** (the parser declines altered ones), because
/// placement is a position-only concept — so the tapped step maps back to a
/// pitch via [LessonPitch.forStaffStep] and is *sounded*, meaning a wrong spot
/// is heard as the wrong pitch, not just flagged.
///
/// The queue walks [PlaceNoteBlock.targets] left to right; each correct
/// placement accumulates on the staff in the success colour. A miss shows the
/// tapped natural as an error-tinted ghost (never a success colour) and marks
/// the run non-flawless. [onCompleted] fires exactly once, after the
/// completion convention (check mark, chime, short pause).
class PlaceNoteView extends ConsumerStatefulWidget {
  const PlaceNoteView({
    super.key,
    required this.block,
    required this.onCompleted,
  });

  final PlaceNoteBlock block;

  /// Called exactly once, when every target has been placed; [flawless] is
  /// false when any wrong position was tapped along the way.
  final void Function({required bool flawless}) onCompleted;

  @override
  ConsumerState<PlaceNoteView> createState() => _PlaceNoteViewState();
}

class _PlaceNoteViewState extends ConsumerState<PlaceNoteView> {
  /// Captured in [initState] (audition-widget pattern) so [dispose] can
  /// silence and cancel everything without touching `ref`.
  late final LessonSounder _sounder;

  /// Correctly placed targets, accumulating left to right on the staff.
  final List<LessonStaffElement> _placed = [];
  final Set<Timer> _timers = {};

  /// Index of the awaited target — advances ~350 ms after a correct tap so the
  /// learner sees the note land before the chip changes.
  int _index = 0;

  /// The wrongly tapped natural, previewed in the error tint for ~600 ms.
  LessonPitch? _ghost;
  int _ghostToken = 0;

  bool _missed = false;
  bool _advancing = false;
  bool _done = false;

  List<LessonPitch> get _targets => widget.block.targets;

  @override
  void initState() {
    super.initState();
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

  void _onTapStep(int step) {
    if (_done || _advancing) return;
    final clef = widget.block.clef;
    final tapped = LessonPitch.forStaffStep(step, clef);
    // Every tap is audible: a wrong spot is *heard* as the wrong pitch.
    _sounder.tap(tapped.midi);
    final target = _targets[_index];
    if (step == target.staffStep(clef)) {
      setState(() {
        _ghost = null;
        _placed.add(
          LessonStaffElement(
            pitch: target,
            fig: const RhythmFigure(NoteFigure.quarter),
          ),
        );
        _advancing = true;
      });
      _after(const Duration(milliseconds: 350), _advance);
    } else {
      final token = ++_ghostToken;
      setState(() {
        _ghost = tapped;
        _missed = true;
      });
      _after(const Duration(milliseconds: 600), () {
        // Clear only the ghost this miss put up, not a fresher one.
        if (_ghostToken == token) setState(() => _ghost = null);
      });
    }
  }

  void _advance() {
    if (_placed.length >= _targets.length) {
      setState(() {
        _done = true;
        _advancing = false;
      });
      _sounder.chime();
      _after(
        const Duration(milliseconds: 450),
        () => widget.onCompleted(flawless: !_missed),
      );
    } else {
      setState(() {
        _index = _placed.length;
        _advancing = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final naming = namingConventionOf(context);
    final prompt = resolveInline(widget.block.prompt, lang);
    final target = _targets[_index];
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
        Row(
          children: [
            Container(
              key: const Key('placenote-target'),
              padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 8),
              decoration: BoxDecoration(
                color: CymbraColors.surfaceContainerHigh,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Text(
                target.name.label(
                  solfege: naming.solfege,
                  frenchRe: naming.frenchRe,
                ),
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 24,
                  fontWeight: FontWeight.w700,
                ),
              ),
            ),
            IconButton(
              key: const Key('placenote-listen'),
              tooltip: l10n.lessonListen,
              icon: const Icon(Icons.volume_up, color: CymbraColors.primary),
              onPressed: () => _sounder.tap(target.midi),
            ),
            const Spacer(),
            for (var i = 0; i < _targets.length; i++)
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 3),
                child: Container(
                  key: Key('placenote-dot-$i'),
                  width: 10,
                  height: 10,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: i < _placed.length
                        ? CymbraColors.tertiary
                        : i == _placed.length
                        ? CymbraColors.primary
                        : CymbraColors.surfaceContainerHighest,
                  ),
                ),
              ),
          ],
        ),
        const SizedBox(height: 8),
        LessonStaff(
          clef: widget.block.clef,
          elements: List.of(_placed),
          elementColors: {
            for (var i = 0; i < _placed.length; i++) i: CymbraColors.tertiary,
          },
          ghost: _ghost,
          ghostColor: CymbraColors.error.withValues(alpha: 0.7),
          onTapStep: _onTapStep,
        ),
      ],
    );
  }
}
