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

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../notation/notation_help_content.dart';
import '../painters/staff_hit_index.dart';
import '../state/notation_help_notifier.dart';
import 'notation_help_bubble.dart';

/// Wraps a staff renderer (the scrolling player staff or the engraved partition)
/// to make its symbols long-pressable for help (change: add-notation-help, D3).
///
/// It owns one [StaffHitIndex] for the lifetime of the area — handed to the
/// painter through [builder] so the painter refills it every frame — and adds a
/// [LongPressGestureRecognizer] over the canvas. On a long-press it resolves the
/// symbol under the finger and opens a help bubble (via
/// [notationHelpBubbleControllerProvider]); it never touches playback and does not claim
/// taps or drags, so playing/scrubbing is unaffected. The gesture is only wired
/// when [enabled] (off for the in-card preview, which is not interactive).
class NotationHelpArea extends ConsumerStatefulWidget {
  const NotationHelpArea({
    super.key,
    required this.builder,
    this.enabled = true,
  });

  /// Builds the canvas child, receiving the hit index to give the painter.
  final Widget Function(BuildContext context, StaffHitIndex hitIndex) builder;

  /// Whether the long-press help is active. When false the child is returned as
  /// is (no gesture, no bubble).
  final bool enabled;

  @override
  ConsumerState<NotationHelpArea> createState() => _NotationHelpAreaState();
}

class _NotationHelpAreaState extends ConsumerState<NotationHelpArea> {
  final StaffHitIndex _hits = StaffHitIndex();

  // No dispose cleanup needed: the bubble provider is autoDispose and this area
  // is its only watcher, so leaving the screen resets it to "no bubble".

  Size get _areaSize {
    final box = context.findRenderObject();
    return box is RenderBox && box.hasSize ? box.size : Size.zero;
  }

  void _onLongPressStart(LongPressStartDetails details) {
    final controller = ref.read(notationHelpBubbleControllerProvider.notifier);
    final descriptor = _hits.hitTest(details.localPosition);
    if (descriptor == null) {
      controller.dismiss();
      return;
    }
    controller.show(descriptor, details.localPosition, _areaSize);
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.enabled) return widget.builder(context, _hits);

    final bubble = ref.watch(notationHelpBubbleControllerProvider);
    final child = GestureDetector(
      // Long-press only: taps and drags fall through to the canvas / scroll view,
      // so playing and scrubbing keep working while help is available.
      onLongPressStart: _onLongPressStart,
      child: widget.builder(context, _hits),
    );

    return Stack(
      clipBehavior: Clip.none,
      children: [
        child,
        if (bubble != null) ...[
          // Tap anywhere outside the bubble to dismiss it.
          Positioned.fill(
            child: GestureDetector(
              behavior: HitTestBehavior.translucent,
              onTap: () =>
                  ref.read(notationHelpBubbleControllerProvider.notifier).dismiss(),
            ),
          ),
          _positioned(context, bubble),
        ],
      ],
    );
  }

  Widget _positioned(BuildContext context, NotationHelpBubbleState bubble) {
    final l10n = AppLocalizations.of(context);
    final style = notationNameStyle(Localizations.localeOf(context));
    final help = notationHelpFor(
      l10n,
      bubble.descriptor,
      solfege: style.solfege,
      frenchRe: style.frenchRe,
    );

    final area = bubble.areaSize;
    final maxWidth = math.min(300.0, math.max(160.0, area.width - 16));
    final left = (bubble.anchor.dx - maxWidth / 2)
        .clamp(8.0, math.max(8.0, area.width - maxWidth - 8))
        .toDouble();
    // Place above the finger when the press is in the lower half of the area,
    // below it otherwise, so the bubble doesn't run off the top or bottom edge.
    final placeAbove = bubble.anchor.dy > area.height / 2;

    final card = ConstrainedBox(
      constraints: BoxConstraints(maxWidth: maxWidth),
      child: NotationHelpBubble(
        title: help.title,
        body: help.body,
        onClose: () =>
            ref.read(notationHelpBubbleControllerProvider.notifier).dismiss(),
      ),
    );

    return Positioned(
      left: left,
      top: placeAbove ? null : bubble.anchor.dy + 18,
      bottom: placeAbove ? math.max(8.0, area.height - bubble.anchor.dy + 14) : null,
      child: card,
    );
  }
}
