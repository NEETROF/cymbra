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

/// The on-screen buttons under the rating deck (change: add-app-score-rating,
/// task 3.3): dislike / like / love mirror the swipes exactly (accessibility
/// parity — the deck is fully usable without swiping), plus a Skip that advances
/// without recording a rating and a Stars control that opens the 1–5 star sheet.
class RatingDeckControls extends StatelessWidget {
  const RatingDeckControls({
    super.key,
    required this.onDislike,
    required this.onLike,
    required this.onLove,
    required this.onSkip,
    required this.onStars,
    this.locked = false,
  });

  final VoidCallback onDislike;
  final VoidCallback onLike;
  final VoidCallback onLove;
  final VoidCallback onSkip;

  /// Opens the star rating for the current card; null disables the control (e.g.
  /// no top card, or locked).
  final VoidCallback? onStars;

  /// While true the verdict/star controls are disabled (the card's preview has
  /// not been heard enough yet); Skip stays available and a hint is shown.
  final bool locked;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 20),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              _RoundAction(
                key: const Key('rating-dislike'),
                icon: Icons.thumb_down,
                color: CymbraColors.error,
                tooltip: l10n.ratingDislike,
                onPressed: locked ? null : onDislike,
              ),
              _RoundAction(
                key: const Key('rating-skip'),
                icon: Icons.skip_next,
                color: CymbraColors.onSurfaceVariant,
                tooltip: l10n.ratingSkip,
                onPressed: onSkip, // Skip is always available
                small: true,
              ),
              _RoundAction(
                key: const Key('rating-like'),
                icon: Icons.thumb_up,
                color: CymbraColors.secondary,
                tooltip: l10n.ratingLike,
                onPressed: locked ? null : onLike,
              ),
              _RoundAction(
                key: const Key('rating-love'),
                icon: Icons.favorite,
                color: CymbraColors.primary,
                tooltip: l10n.ratingLove,
                onPressed: locked ? null : onLove,
              ),
            ],
          ),
          const SizedBox(height: 10),
          // Locked → a hint to keep listening; unlocked → the star-rating entry.
          locked
              ? Row(
                  key: const Key('rating-locked-hint'),
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Icon(
                      Icons.headphones,
                      size: 16,
                      color: CymbraColors.onSurfaceVariant,
                    ),
                    const SizedBox(width: 6),
                    Text(
                      l10n.ratingLockedHint,
                      style: const TextStyle(
                        color: CymbraColors.onSurfaceVariant,
                        fontSize: 13,
                      ),
                    ),
                  ],
                )
              : TextButton.icon(
                  key: const Key('rating-stars'),
                  onPressed: onStars,
                  icon: const Icon(Icons.star, color: CymbraColors.primary),
                  label: Text(l10n.ratingStarsButton),
                ),
        ],
      ),
    );
  }
}

/// A circular action button used by the deck controls. A null [onPressed] renders
/// it disabled (dimmed, not tappable).
class _RoundAction extends StatelessWidget {
  const _RoundAction({
    super.key,
    required this.icon,
    required this.color,
    required this.tooltip,
    required this.onPressed,
    this.small = false,
  });

  final IconData icon;
  final Color color;
  final String tooltip;
  final VoidCallback? onPressed;
  final bool small;

  @override
  Widget build(BuildContext context) {
    final size = small ? 48.0 : 62.0;
    final enabled = onPressed != null;
    final tint = enabled ? color : CymbraColors.outlineVariant;
    return Tooltip(
      message: tooltip,
      child: Material(
        color: CymbraColors.surfaceContainerHigh,
        shape: CircleBorder(
          side: BorderSide(color: tint.withValues(alpha: 0.6), width: 1.5),
        ),
        clipBehavior: Clip.antiAlias,
        child: InkWell(
          onTap: onPressed,
          child: SizedBox(
            width: size,
            height: size,
            child: Icon(icon, color: tint, size: small ? 22 : 28),
          ),
        ),
      ),
    );
  }
}
