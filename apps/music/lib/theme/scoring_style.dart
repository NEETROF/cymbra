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
import '../state/performance_scoring_core.dart';
import 'cymbra_theme.dart';

/// Presentation mapping for the gamified feedback: one accent colour per 20%
/// feedback tier (0–4) and per timing verdict. Kept in one place so the gauge,
/// the hit-effect layer, and the summary/replay stay visually consistent.
extension ScoringTierStyle on int {
  /// Accent colour for this feedback tier (0 = lowest … 4 = flawless band).
  Color get tierColor => switch (clamp(0, 4)) {
    0 => CymbraColors.error, // 0–20  soft coral
    1 => CymbraColors.handLeft, // 20–40 amber
    2 => CymbraColors.secondary, // 40–60 teal
    3 => CymbraColors.tertiary, // 60–80 green
    _ => CymbraColors.primary, // 80–100 lilac
  };

  /// Localized short label for this feedback tier.
  String tierName(AppLocalizations l10n) =>
      l10n.scoringTier(clamp(0, 4).toString());
}

/// Colour a judged onset by verdict for the hit-effect flash and the replay
/// highlight. Non-miss, well-played onsets use the [fallback] (usually the
/// hand's own colour) so correct notes are never recoloured as "mistakes".
Color verdictColor(TimingVerdict verdict, {required Color fallback}) =>
    switch (verdict) {
      TimingVerdict.perfect => CymbraColors.tertiary,
      TimingVerdict.good => CymbraColors.secondary,
      TimingVerdict.early || TimingVerdict.late => CymbraColors.handLeft,
      TimingVerdict.missed => CymbraColors.error,
    };
