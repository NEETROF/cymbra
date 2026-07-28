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

import '../theme/cymbra_theme.dart';

/// A generic, content-agnostic informational callout: a leading icon, a bold
/// [title], a [message], and a stand-alone action link (label + arrow) that fires
/// [onAction]. When [onClose] is provided it also shows a close (×) button that
/// fires it. Reusable anywhere a dismissible "notice with a call to action" is
/// needed — it knows nothing about what it announces. Styled from the app theme.
class NoticeCallout extends StatelessWidget {
  const NoticeCallout({
    super.key,
    required this.title,
    required this.message,
    required this.actionLabel,
    required this.onAction,
    this.onClose,
    this.icon = Icons.info_outline,
    this.dense,
  });

  /// Concise headline (kept short; put details in [message]).
  final String title;

  /// Supporting body text.
  final String message;

  /// Label of the stand-alone action link (shown with a trailing arrow).
  final String actionLabel;

  /// Fired when the action link is tapped.
  final VoidCallback onAction;

  /// Fired when the close (×) button is tapped. `null` = not closable (no button).
  final VoidCallback? onClose;

  /// Leading icon shown in the accent circle.
  final IconData icon;

  /// Force the compact (smaller) sizing. `null` = auto by width (compact on a
  /// narrow callout). Set `true` on a phone, where the callout can be wide
  /// (landscape) yet should still read small.
  final bool? dense;

  @override
  Widget build(BuildContext context) {
    // Compact on a narrow (phone) callout so it doesn't dominate the top of the
    // screen; roomier on tablet/desktop.
    return LayoutBuilder(
      builder: (context, c) {
        final compact = dense ?? (c.maxWidth < 520);
        final m = compact ? _CalloutMetrics.dense : _CalloutMetrics.regular;
        return Container(
          decoration: BoxDecoration(
            color: CymbraColors.surfaceContainerHigh,
            borderRadius: BorderRadius.circular(16),
            border: Border.all(
              color: CymbraColors.primary.withValues(alpha: 0.35),
            ),
          ),
          padding: m.padding,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _iconBadge(m),
              SizedBox(width: m.gapAfterIcon),
              Expanded(child: _content(context, m)),
            ],
          ),
        );
      },
    );
  }

  Widget _iconBadge(_CalloutMetrics m) => Container(
    width: m.circle,
    height: m.circle,
    alignment: Alignment.center,
    decoration: const BoxDecoration(
      color: CymbraColors.primaryContainer,
      shape: BoxShape.circle,
    ),
    child: Icon(icon, color: Colors.white, size: m.iconSize),
  );

  Widget _content(BuildContext context, _CalloutMetrics m) => Column(
    // Size to content: the callout is often placed in an unbounded-height parent
    // (e.g. a Column above an Expanded list).
    mainAxisSize: MainAxisSize.min,
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _header(context, m),
      SizedBox(height: m.gapTitleToMessage),
      Padding(
        padding: const EdgeInsets.only(right: 8),
        child: Text(
          message,
          style: TextStyle(
            color: CymbraColors.onSurfaceVariant,
            fontSize: m.messageSize,
            height: 1.3,
          ),
        ),
      ),
      SizedBox(height: m.gapBeforeLink),
      _link(m),
    ],
  );

  Widget _header(BuildContext context, _CalloutMetrics m) => Row(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Expanded(
        child: Padding(
          padding: EdgeInsets.only(top: m.titleTop),
          child: Text(
            title,
            style: TextStyle(
              color: CymbraColors.onSurface,
              fontSize: m.titleSize,
              fontWeight: FontWeight.w800,
              height: 1.2,
            ),
          ),
        ),
      ),
      _closeButton(context, m),
    ],
  );

  Widget _closeButton(BuildContext context, _CalloutMetrics m) {
    final onClose = this.onClose;
    if (onClose == null) return const SizedBox(width: 8);
    return IconButton(
      icon: Icon(Icons.close, size: m.closeSize),
      color: CymbraColors.onSurfaceVariant,
      visualDensity: VisualDensity.compact,
      tooltip: MaterialLocalizations.of(context).closeButtonTooltip,
      onPressed: onClose,
    );
  }

  Widget _link(_CalloutMetrics m) => InkWell(
    onTap: onAction,
    borderRadius: BorderRadius.circular(6),
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 2, vertical: 4),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            actionLabel,
            style: TextStyle(
              color: CymbraColors.primary,
              fontSize: m.linkSize,
              fontWeight: FontWeight.w800,
              decoration: TextDecoration.underline,
              decorationColor: CymbraColors.primary,
            ),
          ),
          const SizedBox(width: 6),
          Icon(
            Icons.arrow_forward,
            color: CymbraColors.primary,
            size: m.arrowSize,
          ),
        ],
      ),
    ),
  );
}

/// The size/spacing constants for a [NoticeCallout], in a [regular] and a [dense]
/// preset — selected once so the widget tree carries no scattered ternaries.
class _CalloutMetrics {
  const _CalloutMetrics({
    required this.padding,
    required this.circle,
    required this.iconSize,
    required this.gapAfterIcon,
    required this.titleTop,
    required this.titleSize,
    required this.closeSize,
    required this.gapTitleToMessage,
    required this.messageSize,
    required this.gapBeforeLink,
    required this.linkSize,
    required this.arrowSize,
  });

  final EdgeInsets padding;
  final double circle;
  final double iconSize;
  final double gapAfterIcon;
  final double titleTop;
  final double titleSize;
  final double closeSize;
  final double gapTitleToMessage;
  final double messageSize;
  final double gapBeforeLink;
  final double linkSize;
  final double arrowSize;

  static const regular = _CalloutMetrics(
    padding: EdgeInsets.fromLTRB(16, 14, 8, 16),
    circle: 34,
    iconSize: 19,
    gapAfterIcon: 12,
    titleTop: 6,
    titleSize: 16,
    closeSize: 18,
    gapTitleToMessage: 2,
    messageSize: 13.5,
    gapBeforeLink: 12,
    linkSize: 14,
    arrowSize: 18,
  );

  static const dense = _CalloutMetrics(
    padding: EdgeInsets.fromLTRB(12, 10, 4, 12),
    circle: 26,
    iconSize: 15,
    gapAfterIcon: 10,
    titleTop: 3,
    titleSize: 13.5,
    closeSize: 16,
    gapTitleToMessage: 1,
    messageSize: 12,
    gapBeforeLink: 8,
    linkSize: 12.5,
    arrowSize: 15,
  );
}
