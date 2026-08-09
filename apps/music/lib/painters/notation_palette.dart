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

import 'dart:ui';

import '../theme/cymbra_theme.dart';

/// User-selectable rendering theme of the notation views (Portée + Partition):
/// the app's dark surface, or a paper-like light background — engraved music
/// is sharpest dark-on-light (no halation on small dense glyphs), which is why
/// print scores are black on white.
enum NotationTheme { dark, paper }

/// The colour set a notation painter draws with. Each theme keeps the same
/// roles — per-hand note heads, correct green, accent cursor/wash — tuned for
/// contrast against its own background (the dark hand colours would drop to
/// 1.6–2.5:1 on paper, so paper carries darkened equivalents at ≥4.5:1).
class NotationPalette {
  final Color background;

  /// Glyph ink: clefs, stems, beams, signatures, flags.
  final Color ink;

  /// Staff/ledger lines, rests, lyrics and words — the quieter lane.
  final Color staffLine;

  /// Note heads by hand, and the satisfied ("correct") colour.
  final Color handRight;
  final Color handLeft;
  final Color correct;

  /// Playhead cursor and current-measure wash.
  final Color accent;

  /// Lerp target that *emphasises* a note head at the playhead: towards white
  /// on a dark background, towards black on paper (towards white would fade
  /// into the page).
  final Color emphasisTint;

  const NotationPalette({
    required this.background,
    required this.ink,
    required this.staffLine,
    required this.handRight,
    required this.handLeft,
    required this.correct,
    required this.accent,
    required this.emphasisTint,
  });

  static const dark = NotationPalette(
    background: CymbraColors.surfaceContainerLow,
    ink: CymbraColors.onSurface,
    staffLine: CymbraColors.onSurfaceVariant,
    handRight: CymbraColors.handRight,
    handLeft: CymbraColors.handLeft,
    correct: CymbraColors.tertiary,
    accent: CymbraColors.secondary,
    emphasisTint: Color(0xFFFFFFFF),
  );

  static const paper = NotationPalette(
    background: Color(0xFFF7F4EC), // ivory
    ink: Color(0xFF1A1A1A), // 15.8:1 on ivory
    staffLine: Color(0xFF3A3A3A),
    handRight: Color(0xFF1D5FD6), // 5.2:1
    handLeft: Color(0xFFA85E00), // 4.5:1
    correct: Color(0xFF0E7D52), // 4.7:1
    accent: Color(0xFF0B8577), // darkened teal for cursor/wash
    emphasisTint: Color(0xFF000000),
  );

  static NotationPalette of(NotationTheme theme) => switch (theme) {
    NotationTheme.dark => dark,
    NotationTheme.paper => paper,
  };
}
