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
  });

  final VoidCallback onDislike;
  final VoidCallback onLike;
  final VoidCallback onLove;
  final VoidCallback onSkip;

  /// Opens the star rating for the current card; null disables the control (e.g.
  /// no top card).
  final VoidCallback? onStars;

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
                onPressed: onDislike,
              ),
              _RoundAction(
                key: const Key('rating-skip'),
                icon: Icons.skip_next,
                color: CymbraColors.onSurfaceVariant,
                tooltip: l10n.ratingSkip,
                onPressed: onSkip,
                small: true,
              ),
              _RoundAction(
                key: const Key('rating-like'),
                icon: Icons.thumb_up,
                color: CymbraColors.secondary,
                tooltip: l10n.ratingLike,
                onPressed: onLike,
              ),
              _RoundAction(
                key: const Key('rating-love'),
                icon: Icons.favorite,
                color: CymbraColors.primary,
                tooltip: l10n.ratingLove,
                onPressed: onLove,
              ),
            ],
          ),
          const SizedBox(height: 10),
          TextButton.icon(
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

/// A circular action button used by the deck controls.
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
  final VoidCallback onPressed;
  final bool small;

  @override
  Widget build(BuildContext context) {
    final size = small ? 48.0 : 62.0;
    return Tooltip(
      message: tooltip,
      child: Material(
        color: CymbraColors.surfaceContainerHigh,
        shape: CircleBorder(
          side: BorderSide(color: color.withValues(alpha: 0.6), width: 1.5),
        ),
        clipBehavior: Clip.antiAlias,
        child: InkWell(
          onTap: onPressed,
          child: SizedBox(
            width: size,
            height: size,
            child: Icon(icon, color: color, size: small ? 22 : 28),
          ),
        ),
      ),
    );
  }
}
