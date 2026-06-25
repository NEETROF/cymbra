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

/// "Sonic Luminescence" design system palette (see DESIGN.md).
///
/// Centralizes all the POC's colors to avoid hex values scattered across the
/// painters and widgets.
class CymbraColors {
  CymbraColors._();

  // Backgrounds (Midnight Navy), from deepest to lightest.
  static const background = Color(0xFF0B1326);
  static const surfaceContainerLowest = Color(0xFF060E20);
  static const surfaceContainerLow = Color(0xFF131B2E);
  static const surfaceContainer = Color(0xFF171F33);
  static const surfaceContainerHigh = Color(0xFF222A3D);
  static const surfaceContainerHighest = Color(0xFF2D3449);

  // Text / icons on dark background.
  static const onSurface = Color(0xFFDAE2FD);
  static const onSurfaceVariant = Color(0xFFCCC3D8);
  static const outline = Color(0xFF958DA1);
  static const outlineVariant = Color(0xFF4A4455);

  // Primary — Electric Purple (main actions, active states, branding).
  static const primary = Color(0xFFD2BBFF);
  static const primaryContainer = Color(0xFF7C3AED);

  // Secondary — Bright Teal ("play" elements, metronome).
  static const secondary = Color(0xFF44E2CD);
  static const secondaryContainer = Color(0xFF03C6B2);

  // Tertiary — Success Green (correct note, success).
  static const tertiary = Color(0xFF4EDEA3);

  // Error — Pink (missed note).
  static const error = Color(0xFFFFB4AB);

  // Hand colours — used to tell the right and left hands apart on the keyboard
  // (expected keys) and on the partition (note heads). Cool blue vs warm amber
  // for an at-a-glance contrast, distinct from the green "correct" state.
  static const handRight = Color(0xFF5B9DFF); // right hand (treble / staff 1)
  static const handLeft = Color(0xFFFFB454); // left hand (bass / staff 2+)

  // Physical piano keys.
  static const pianoWhite = Color(0xFFFFFFFF);
  static const pianoBlack = Color(0xFF1E293B);
}

/// Global Material theme of the app.
ThemeData buildCymbraTheme() {
  final base = ThemeData.dark(useMaterial3: true);
  return base.copyWith(
    scaffoldBackgroundColor: CymbraColors.background,
    colorScheme: const ColorScheme.dark(
      surface: CymbraColors.background,
      primary: CymbraColors.primaryContainer,
      onPrimary: Color(0xFFEDE0FF),
      secondary: CymbraColors.secondaryContainer,
      tertiary: CymbraColors.tertiary,
      error: CymbraColors.error,
      onSurface: CymbraColors.onSurface,
    ),
    textTheme: base.textTheme.apply(
      bodyColor: CymbraColors.onSurface,
      displayColor: CymbraColors.onSurface,
    ),
  );
}
