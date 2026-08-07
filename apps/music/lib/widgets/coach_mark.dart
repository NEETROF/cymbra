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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../l10n/gen/app_localizations.dart';
import '../state/coaching_notifier.dart';
import '../theme/cymbra_theme.dart';
import 'coach_copy.dart';

part 'coach_mark.g.dart';

/// A coachable control the spotlight can point at. Each anchor is registered by
/// a [CoachTarget] wrapped around the real control, so the overlay reads the
/// control's on-screen rect instead of hard-coding coordinates.
enum CoachAnchor { pianoSound, midiDevice, hands }

/// Registry mapping each [CoachAnchor] to the `GlobalKey` of the widget that
/// currently renders it (D9, "target discovery").
///
/// Rects only exist after layout, so callers resolve them from a post-frame
/// callback; [rectFor] returns `null` while the control is not mounted (for
/// example before the settings surface is opened), and the overlay simply falls
/// back to an untargeted, centered bubble.
class CoachTargetRegistry {
  final Map<CoachAnchor, GlobalKey> _keys = {};

  /// The stable key for [anchor], created on first use.
  GlobalKey keyFor(CoachAnchor anchor) =>
      _keys.putIfAbsent(anchor, () => GlobalKey(debugLabel: anchor.name));

  /// The element currently rendering [anchor]'s control, or `null` when it is
  /// not mounted. Used to scroll the control into view before spotlighting it.
  BuildContext? contextFor(CoachAnchor anchor) => _keys[anchor]?.currentContext;

  /// The global rect of [anchor]'s control, or `null` when it is not laid out.
  Rect? rectFor(CoachAnchor anchor) {
    final context = _keys[anchor]?.currentContext;
    if (context == null) return null;
    final box = context.findRenderObject();
    if (box is! RenderBox || !box.attached || !box.hasSize) return null;
    return box.localToGlobal(Offset.zero) & box.size;
  }
}

/// The app-wide target registry. `keepAlive` so keys survive a control being
/// rebuilt between two steps of a guided sequence.
@Riverpod(keepAlive: true)
CoachTargetRegistry coachTargetRegistry(Ref ref) => CoachTargetRegistry();

/// Registers [child] as the control behind [anchor] so a coach mark can spotlight
/// it in place. Purely additive: it does not change the child's layout.
class CoachTarget extends ConsumerWidget {
  const CoachTarget({required this.anchor, required this.child, super.key});

  final CoachAnchor anchor;
  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) => KeyedSubtree(
    key: ref.read(coachTargetRegistryProvider).keyFor(anchor),
    child: child,
  );
}

/// Shared coach-mark / spotlight overlay (D9).
///
/// Renders a dimming scrim with an optional cut-out over [hole] plus a bubble
/// placed next to it (below, above, or — on a short landscape viewport — beside
/// it), carrying the copy and the Next/Skip actions.
///
/// Two interaction modes:
/// - [passThrough] `false` (passive hint): the whole surface absorbs input and a
///   tap anywhere dismisses, so the hint never traps the user.
/// - [passThrough] `true` (guided "do it now" step): only the scrim *around* the
///   hole absorbs input — the real control stays tappable through the cut-out.
///
/// Meant to be stacked over the screen (e.g. via `MaterialApp.builder`), so it
/// can point at controls that live inside a dialog.
class CoachMarkOverlay extends StatelessWidget {
  const CoachMarkOverlay({
    required this.title,
    required this.body,
    required this.nextLabel,
    this.hole,
    this.onNext,
    this.onSkip,
    this.skipLabel,
    this.stepLabel,
    this.passThrough = false,
    super.key,
  });

  /// The control to spotlight, in global coordinates; `null` centers the bubble
  /// with no cut-out (an untargeted hint).
  final Rect? hole;

  final String title;
  final String body;

  /// Label of the primary action ("Next" / "Got it").
  final String nextLabel;

  /// Label of the secondary action; omit to hide it.
  final String? skipLabel;

  /// Optional progress caption, e.g. "2 / 3".
  final String? stepLabel;

  final VoidCallback? onNext;
  final VoidCallback? onSkip;

  /// Whether taps inside [hole] reach the real control underneath.
  final bool passThrough;

  /// Padding grown around the target so the highlight breathes.
  static const double _holePadding = 8;

  /// Gap between the hole and the bubble, and the viewport margin.
  static const double _gap = 12;
  static const double _margin = 16;

  /// Below this, a vertical band is too short to hold a readable bubble and the
  /// bubble is placed beside the target instead (landscape edge-avoidance).
  static const double _minBandHeight = 132;

  static const double _maxBubbleWidth = 360;

  /// How much of the target must fall inside the viewport for the cut-out to be
  /// worth drawing. A control that is mostly off screen — below the fold of a
  /// scrollable surface, say — would otherwise be "highlighted" as a sliver at
  /// the edge, pointing at nothing; the bubble is shown untargeted instead.
  static const double _minVisibleFraction = 0.6;

  /// The cut-out to draw for [hole] in a viewport of [size], or `null` when too
  /// little of the target is actually visible.
  Rect? _visibleSpot(Size size) {
    final target = hole?.inflate(_holePadding);
    if (target == null || target.isEmpty) return null;
    final shown = target.intersect(Offset.zero & size);
    if (shown.isEmpty) return null;
    final visible =
        (shown.width * shown.height) / (target.width * target.height);
    return visible >= _minVisibleFraction ? shown : null;
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(constraints.maxWidth, constraints.maxHeight);
        final spot = _visibleSpot(size);
        return Stack(
          children: [
            // Visual only — the barriers below own hit-testing, so the cut-out
            // can stay live for a "do it now" step.
            Positioned.fill(
              child: IgnorePointer(
                child: CustomPaint(painter: SpotlightPainter(hole: spot)),
              ),
            ),
            ..._barriers(spot, size),
            _bubble(spot, size),
          ],
        );
      },
    );
  }

  /// Input barriers: the full surface for a passive hint (tap anywhere to
  /// dismiss), or the four bands around the hole when the control must stay
  /// tappable.
  List<Widget> _barriers(Rect? spot, Size size) {
    if (spot == null || !passThrough) {
      return [
        Positioned.fill(
          child: GestureDetector(
            key: const Key('coach-mark-scrim'),
            behavior: HitTestBehavior.opaque,
            onTap: onNext,
          ),
        ),
      ];
    }
    Widget band(Rect rect) => Positioned.fromRect(
      rect: rect,
      child: const AbsorbPointer(child: SizedBox.expand()),
    );
    return [
      band(Rect.fromLTRB(0, 0, size.width, spot.top)),
      band(Rect.fromLTRB(0, spot.bottom, size.width, size.height)),
      band(Rect.fromLTRB(0, spot.top, spot.left, spot.bottom)),
      band(Rect.fromLTRB(spot.right, spot.top, size.width, spot.bottom)),
    ];
  }

  /// Places the bubble in the roomiest band around the target: below or above
  /// it, or — when both vertical bands are too short (typical in the app's
  /// landscape-locked player) — beside it.
  Widget _bubble(Rect? spot, Size size) {
    final width = (size.width - 2 * _margin).clamp(0.0, _maxBubbleWidth);
    final card = _BubbleCard(
      title: title,
      body: body,
      nextLabel: nextLabel,
      skipLabel: skipLabel,
      stepLabel: stepLabel,
      onNext: onNext,
      onSkip: onSkip,
    );

    if (spot == null) {
      return Positioned.fill(
        child: Center(
          child: ConstrainedBox(
            constraints: BoxConstraints(
              maxWidth: width,
              maxHeight: size.height - 2 * _margin,
            ),
            child: card,
          ),
        ),
      );
    }

    final below = size.height - spot.bottom - _gap - _margin;
    final above = spot.top - _gap - _margin;
    final left = (spot.center.dx - width / 2).clamp(
      _margin,
      (size.width - width - _margin).clamp(_margin, double.infinity),
    );

    if (below >= above && below >= _minBandHeight) {
      return Positioned(
        left: left,
        top: spot.bottom + _gap,
        width: width,
        child: ConstrainedBox(
          constraints: BoxConstraints(maxHeight: below),
          child: card,
        ),
      );
    }
    if (above > below && above >= _minBandHeight) {
      return Positioned(
        left: left,
        bottom: size.height - spot.top + _gap,
        width: width,
        child: ConstrainedBox(
          constraints: BoxConstraints(maxHeight: above),
          child: card,
        ),
      );
    }
    // Both vertical bands are too short: sit beside the target, on the side with
    // the most room, vertically centred on it.
    final rightBand = size.width - spot.right - _gap - _margin;
    final leftBand = spot.left - _gap - _margin;
    final sideWidth = (rightBand >= leftBand ? rightBand : leftBand).clamp(
      0.0,
      _maxBubbleWidth,
    );
    final maxHeight = size.height - 2 * _margin;
    final top = (spot.center.dy - maxHeight / 2).clamp(
      _margin,
      (size.height - _margin).clamp(_margin, double.infinity),
    );
    return Positioned(
      left: rightBand >= leftBand ? spot.right + _gap : null,
      right: rightBand >= leftBand ? null : size.width - spot.left + _gap,
      top: top,
      width: sideWidth,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: maxHeight),
        child: card,
      ),
    );
  }
}

/// The coach-mark bubble: copy plus the Skip/Next actions.
///
/// Only the **copy** scrolls when the band it was given is short (the app's
/// landscape-locked phone viewport is): the action row stays pinned at the
/// bottom, so Next/Skip are always reachable without scrolling — the hint must
/// never be something the user has to fight to leave.
class _BubbleCard extends StatelessWidget {
  const _BubbleCard({
    required this.title,
    required this.body,
    required this.nextLabel,
    required this.skipLabel,
    required this.stepLabel,
    required this.onNext,
    required this.onSkip,
  });

  final String title;
  final String body;
  final String nextLabel;
  final String? skipLabel;
  final String? stepLabel;
  final VoidCallback? onNext;
  final VoidCallback? onSkip;

  @override
  Widget build(BuildContext context) {
    // One semantics container announced as a whole, so a screen reader reads the
    // hint on appearance; the actions below stay individually focusable.
    return Semantics(
      container: true,
      liveRegion: true,
      child: Material(
        key: const Key('coach-mark-bubble'),
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(14),
        elevation: 8,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(16, 14, 16, 10),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Flexible(
                child: SingleChildScrollView(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      if (stepLabel != null)
                        Padding(
                          padding: const EdgeInsets.only(bottom: 4),
                          child: Text(
                            stepLabel!,
                            style: const TextStyle(
                              color: CymbraColors.tertiary,
                              fontSize: 11,
                              fontWeight: FontWeight.w700,
                              letterSpacing: 0.6,
                            ),
                          ),
                        ),
                      Text(
                        title,
                        style: const TextStyle(
                          color: CymbraColors.onSurface,
                          fontSize: 16,
                          fontWeight: FontWeight.w800,
                        ),
                      ),
                      const SizedBox(height: 6),
                      Text(
                        body,
                        style: const TextStyle(
                          color: CymbraColors.onSurfaceVariant,
                          fontSize: 13.5,
                          height: 1.35,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 6),
              // Pinned outside the scroll view: always reachable.
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  if (skipLabel != null)
                    TextButton(
                      key: const Key('coach-mark-skip'),
                      onPressed: onSkip,
                      child: Text(skipLabel!),
                    ),
                  const SizedBox(width: 4),
                  FilledButton(
                    key: const Key('coach-mark-next'),
                    onPressed: onNext,
                    child: Text(nextLabel),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// A one-time hint rendered **inline**, next to the feature it introduces, for
/// surfaces where a scrim would be heavy-handed (a section of a scrollable
/// screen). Same shared mechanism as the spotlight — same [CoachHint] "seen"
/// store and copy — different presentation: it sits in the flow, never blocks
/// the action, and disappears for good once dismissed.
class CoachHintCallout extends ConsumerWidget {
  const CoachHintCallout({
    required this.hint,
    this.icon = Icons.lightbulb_outline,
    super.key,
  });

  final CoachHint hint;
  final IconData icon;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final coaching = ref.watch(coachingProvider);
    if (!coaching.shouldShow(hint)) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    final copy = coachHintCopy(l10n, hint);
    return Padding(
      key: Key('coach-hint-${hint.name}'),
      padding: const EdgeInsets.only(bottom: 12),
      child: Semantics(
        container: true,
        liveRegion: true,
        child: Container(
          padding: const EdgeInsets.fromLTRB(14, 12, 8, 8),
          decoration: BoxDecoration(
            color: CymbraColors.surfaceContainerHigh,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: CymbraColors.tertiary.withValues(alpha: 0.5),
            ),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(icon, size: 20, color: CymbraColors.tertiary),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      copy.title,
                      style: const TextStyle(
                        color: CymbraColors.onSurface,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      copy.body,
                      style: const TextStyle(
                        color: CymbraColors.onSurfaceVariant,
                        fontSize: 13.5,
                        height: 1.3,
                      ),
                    ),
                    Align(
                      alignment: Alignment.centerRight,
                      child: TextButton(
                        key: Key('coach-hint-dismiss-${hint.name}'),
                        onPressed: () =>
                            ref.read(coachingProvider.notifier).markSeen(hint),
                        child: Text(l10n.coachGotIt),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Paints the dimming scrim with a rounded cut-out over the spotlighted control
/// (`Path.combine(difference, …)`) and a thin ring around it. A `null` hole
/// paints a plain scrim.
class SpotlightPainter extends CustomPainter {
  const SpotlightPainter({this.hole, this.radius = 12});

  final Rect? hole;
  final double radius;

  static const Color scrimColor = Color(0xB3000000); // black @ 70%

  @override
  void paint(Canvas canvas, Size size) {
    final full = Path()..addRect(Offset.zero & size);
    final spot = hole;
    final scrim = Paint()..color = scrimColor;
    if (spot == null || spot.isEmpty) {
      canvas.drawPath(full, scrim);
      return;
    }
    final cut = Path()
      ..addRRect(RRect.fromRectAndRadius(spot, Radius.circular(radius)));
    canvas.drawPath(Path.combine(PathOperation.difference, full, cut), scrim);
    canvas.drawRRect(
      RRect.fromRectAndRadius(spot, Radius.circular(radius)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = CymbraColors.tertiary,
    );
  }

  @override
  bool shouldRepaint(SpotlightPainter oldDelegate) =>
      oldDelegate.hole != hole || oldDelegate.radius != radius;
}
