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

/// A small celebratory dialog for a gamified reward moment (change: add-curation-
/// rewards): a level-up, a badge earned, or a reward redeemed. Mirrors the
/// [showSessionSummary] modal convention (a centered [Dialog]).
///
/// Honors the gamified-feedback colour discipline: the accent is the flawless
/// feedback-tier colour (`4.tierColor`, lilac), NOT a plain success green.
Future<void> showRewardCelebration(
  BuildContext context, {
  required String title,
  required String message,
  IconData icon = Icons.celebration,
}) => showDialog<void>(
  context: context,
  builder: (context) =>
      _CelebrationDialog(title: title, message: message, icon: icon),
);

class _CelebrationDialog extends StatelessWidget {
  const _CelebrationDialog({
    required this.title,
    required this.message,
    required this.icon,
  });

  final String title;
  final String message;
  final IconData icon;

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
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 360),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(24, 24, 24, 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: accent.withValues(alpha: 0.15),
                  shape: BoxShape.circle,
                ),
                child: Icon(icon, size: 44, color: accent),
              ),
              const SizedBox(height: 16),
              Text(
                title,
                textAlign: TextAlign.center,
                style: TextStyle(
                  color: accent,
                  fontSize: 20,
                  fontWeight: FontWeight.w800,
                ),
              ),
              const SizedBox(height: 8),
              Text(
                message,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 14.5,
                  height: 1.35,
                ),
              ),
              const SizedBox(height: 20),
              SizedBox(
                width: double.infinity,
                child: FilledButton(
                  style: FilledButton.styleFrom(backgroundColor: accent),
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text(l10n.rewardCelebrationClose),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
