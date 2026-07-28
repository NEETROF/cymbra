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
import '../layout/device_class.dart';
import '../theme/cymbra_theme.dart';

/// The on-screen buttons for the rating deck (change: add-app-score-rating,
/// task 3.3): dislike / like / love mirror the swipes exactly (accessibility
/// parity — the deck is fully usable without swiping), plus a Skip that advances
/// without recording a rating and a Stars control that opens the 1–5 star sheet.
///
/// Adapts to the viewport: buttons shrink on a phone, and [axis] switches between
/// the default bottom bar ([Axis.horizontal]) and a compact side rail
/// ([Axis.vertical]) used in phone-landscape so the card keeps the full height.
class RatingDeckControls extends StatelessWidget {
  const RatingDeckControls({
    super.key,
    required this.onDislike,
    required this.onLike,
    required this.onLove,
    required this.onSkip,
    required this.onStars,
    this.locked = false,
    this.axis = Axis.horizontal,
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

  /// Layout direction: a bottom bar (horizontal) or a side rail (vertical).
  final Axis axis;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    // Buttons shrink on a phone and in the (inherently cramped) side rail — they
    // were oversized in landscape.
    final compact = context.isPhoneLayout || axis == Axis.vertical;
    final big = compact ? 50.0 : 62.0;
    final small = compact ? 42.0 : 48.0;
    final gap = compact ? 8.0 : 12.0;

    final dislike = _RoundAction(
      key: const Key('rating-dislike'),
      icon: Icons.thumb_down,
      color: CymbraColors.error,
      tooltip: l10n.ratingDislike,
      onPressed: locked ? null : onDislike,
      size: big,
    );
    final skip = _RoundAction(
      key: const Key('rating-skip'),
      icon: Icons.skip_next,
      color: CymbraColors.onSurfaceVariant,
      tooltip: l10n.ratingSkip,
      onPressed: onSkip, // Skip is always available
      size: small,
    );
    final like = _RoundAction(
      key: const Key('rating-like'),
      icon: Icons.thumb_up,
      color: CymbraColors.secondary,
      tooltip: l10n.ratingLike,
      onPressed: locked ? null : onLike,
      size: big,
    );
    final love = _RoundAction(
      key: const Key('rating-love'),
      icon: Icons.favorite,
      color: CymbraColors.primary,
      tooltip: l10n.ratingLove,
      onPressed: locked ? null : onLove,
      size: big,
    );

    // Unlocked → the star-rating entry (compact icon in the side rail, labelled
    // button in the bottom bar). While locked it's omitted — the persistent
    // "listen before rating" caption above the controls explains the gate.
    final Widget starsControl = locked
        ? const SizedBox.shrink()
        : (axis == Axis.vertical
              ? _RoundAction(
                  key: const Key('rating-stars'),
                  icon: Icons.star,
                  color: CymbraColors.primary,
                  tooltip: l10n.ratingStarsButton,
                  onPressed: onStars,
                  size: small,
                )
              : TextButton.icon(
                  key: const Key('rating-stars'),
                  onPressed: onStars,
                  icon: const Icon(Icons.star, color: CymbraColors.primary),
                  label: Text(l10n.ratingStarsButton),
                ));

    if (axis == Axis.vertical) {
      // Side rail (phone-landscape): a narrow vertical column of controls.
      return Padding(
        padding: const EdgeInsets.fromLTRB(4, 8, 12, 8),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          mainAxisSize: MainAxisSize.min,
          children: [
            love,
            SizedBox(height: gap),
            like,
            SizedBox(height: gap),
            dislike,
            SizedBox(height: gap),
            skip,
            SizedBox(height: gap + 2),
            starsControl,
          ],
        ),
      );
    }

    // Default bottom bar.
    return Padding(
      padding: EdgeInsets.fromLTRB(16, compact ? 4 : 8, 16, compact ? 10 : 20),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [dislike, skip, like, love],
          ),
          SizedBox(height: compact ? 4 : 10),
          starsControl,
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
    required this.size,
  });

  final IconData icon;
  final Color color;
  final String tooltip;
  final VoidCallback? onPressed;
  final double size;

  @override
  Widget build(BuildContext context) {
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
            child: Icon(icon, color: tint, size: size * 0.45),
          ),
        ),
      ),
    );
  }
}
