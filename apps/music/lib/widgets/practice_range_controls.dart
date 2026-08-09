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

import 'package:flutter/material.dart';

import '../l10n/gen/app_localizations.dart';
import '../theme/cymbra_theme.dart';
import 'practice_score_strip.dart';

/// The measure-range controls of a practice (selective) run — the score ribbon
/// and the from/to steppers (change: add-measure-range-practice, D5). A practice
/// run always loops, endlessly, so there is nothing else to set.
///
/// Purely presentational: it holds no state, so both entry points — the pre-play
/// setup sheet and the picker opened from the end-of-run summary — drive the
/// *same* controls from their own drafts and stay in sync by construction.
/// Measure indices are 0-based here and displayed 1-based (bar numbers).
class PracticeRangeControls extends StatelessWidget {
  const PracticeRangeControls({
    super.key,
    required this.lastMeasure,
    required this.fromMeasure,
    required this.toMeasure,
    required this.onFromChanged,
    required this.onToChanged,
    this.scoreHeight = 190,
  });

  /// Highest valid measure index of the piece (0-based).
  final int lastMeasure;

  final int fromMeasure;
  final int toMeasure;

  final ValueChanged<int> onFromChanged;
  final ValueChanged<int> onToChanged;

  /// Preferred height of the engraved score picker. The passage STEP hands it a
  /// large value (it owns the modal); an inline use keeps the compact default.
  final double scoreHeight;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        // The PRIMARY picker: the engraved score itself. A musician recognises
        // the passage by how it looks, not by its bar numbers — so the range is
        // chosen by tapping the first and last bar. Renders nothing when the
        // piece has no engraved layout, leaving the steppers as the whole
        // control.
        PracticeScoreStrip(
          height: scoreHeight,
          fromMeasure: fromMeasure,
          toMeasure: toMeasure,
          onRangeChanged: (start, end) {
            onFromChanged(start);
            onToChanged(end);
          },
        ),
        const SizedBox(height: 8),
        // Fine-tuning (and the accessible, mode-independent fallback): the same
        // range, one bar at a time.
        _stepper(
          stepperKey: const Key('practice-from'),
          label: l10n.practiceFromBar,
          value: fromMeasure,
          max: lastMeasure,
          onChanged: onFromChanged,
        ),
        _stepper(
          stepperKey: const Key('practice-to'),
          label: l10n.practiceToBar,
          value: toMeasure,
          max: lastMeasure,
          onChanged: onToChanged,
        ),
        Text(
          l10n.practiceUnscoredNote,
          style: const TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: 11,
          ),
        ),
      ],
    );
  }

  /// One measure bound: a −/+ stepper around the 1-based bar number, clamped to
  /// `[0, max]` (0-based internally).
  Widget _stepper({
    required Key stepperKey,
    required String label,
    required int value,
    required int max,
    required ValueChanged<int> onChanged,
  }) => Padding(
    key: stepperKey,
    padding: const EdgeInsets.symmetric(vertical: 2),
    child: Row(
      children: [
        Expanded(
          child: Text(
            label,
            style: const TextStyle(color: CymbraColors.onSurfaceVariant),
          ),
        ),
        IconButton(
          visualDensity: VisualDensity.compact,
          onPressed: value > 0 ? () => onChanged(value - 1) : null,
          icon: const Icon(Icons.remove, color: CymbraColors.onSurfaceVariant),
        ),
        SizedBox(
          width: 36,
          child: Text(
            // Bars read 1-based for the musician; the state is 0-based.
            '${value + 1}',
            textAlign: TextAlign.center,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontWeight: FontWeight.w700,
            ),
          ),
        ),
        IconButton(
          visualDensity: VisualDensity.compact,
          onPressed: value < max ? () => onChanged(value + 1) : null,
          icon: const Icon(Icons.add, color: CymbraColors.onSurfaceVariant),
        ),
      ],
    ),
  );
}
