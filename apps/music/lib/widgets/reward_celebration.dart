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
import '../theme/scoring_style.dart';
import 'celebration_surface.dart';

/// A small celebratory dialog for a gamified reward moment (change: add-curation-
/// rewards): a level-up, a badge earned, a reward redeemed, a piece passed, a
/// streak milestone. Mirrors the [showSessionSummary] modal convention (a
/// centered [Dialog]); animated through the shared [CelebrationSurface]
/// (change: add-music-motion-and-celebrations).
///
/// Honors the gamified-feedback colour discipline: the accent is the flawless
/// feedback-tier colour (`4.tierColor`, lilac), NOT a plain success green.
/// [counterTo] / [counterFrom] show a number counting up (points) under the
/// message; [counterFormat] localizes it.
Future<void> showRewardCelebration(
  BuildContext context, {
  required String title,
  required String message,
  IconData icon = Icons.celebration,
  int? counterTo,
  int? counterFrom,
  String Function(int value)? counterFormat,
}) => showDialog<void>(
  context: context,
  builder: (context) => _CelebrationDialog(
    title: title,
    message: message,
    icon: icon,
    counterTo: counterTo,
    counterFrom: counterFrom,
    counterFormat: counterFormat,
  ),
);

class _CelebrationDialog extends StatelessWidget {
  const _CelebrationDialog({
    required this.title,
    required this.message,
    required this.icon,
    this.counterTo,
    this.counterFrom,
    this.counterFormat,
  });

  final String title;
  final String message;
  final IconData icon;
  final int? counterTo;
  final int? counterFrom;
  final String Function(int value)? counterFormat;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    // Gamified accent: the flawless (top) feedback-tier colour, not a success hue.
    final accent = 4.tierColor;
    return Dialog(
      key: const Key('reward-celebration'),
      backgroundColor: CymbraColors.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      insetPadding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      child: CelebrationSurface(
        icon: icon,
        accent: accent,
        title: title,
        subtitle: message,
        counterTo: counterTo,
        counterFrom: counterFrom,
        counterFormat: counterFormat,
        actions: [
          SizedBox(
            width: double.infinity,
            child: FilledButton(
              // Dark text on the light lilac: white on it was barely legible
              // (found while dogfooding).
              style: FilledButton.styleFrom(
                backgroundColor: accent,
                foregroundColor: CymbraColors.background,
              ),
              onPressed: () => Navigator.of(context).pop(),
              child: Text(l10n.rewardCelebrationClose),
            ),
          ),
        ],
      ),
    );
  }
}
