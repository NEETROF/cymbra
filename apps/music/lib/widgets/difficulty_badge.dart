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
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';

/// The difficulty accent colour (green / teal / pink) shared by the badge + cover.
Color scoreLevelColor(PracticeLevel level) => switch (level) {
  PracticeLevel.beginner => CymbraColors.tertiary,
  PracticeLevel.intermediate => CymbraColors.secondary,
  PracticeLevel.advanced => CymbraColors.error,
};

/// A small pill showing a score's practice level, coloured by difficulty. Shared
/// by the score card and the pre-play setup modal.
class DifficultyBadge extends StatelessWidget {
  const DifficultyBadge({required this.level, required this.l10n, super.key});

  final PracticeLevel level;
  final AppLocalizations l10n;

  @override
  Widget build(BuildContext context) {
    final color = scoreLevelColor(level);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.22),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.6)),
      ),
      child: Text(
        level.localizedLabel(l10n).toUpperCase(),
        style: TextStyle(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.w800,
          letterSpacing: 0.5,
        ),
      ),
    );
  }
}
