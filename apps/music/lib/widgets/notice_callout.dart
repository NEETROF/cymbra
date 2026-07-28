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

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: CymbraColors.primary.withValues(alpha: 0.35)),
      ),
      padding: const EdgeInsets.fromLTRB(16, 14, 8, 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 34,
            height: 34,
            alignment: Alignment.center,
            decoration: const BoxDecoration(
              color: CymbraColors.primaryContainer,
              shape: BoxShape.circle,
            ),
            child: Icon(icon, color: Colors.white, size: 19),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              // Size to content: the callout is often placed in an unbounded-
              // height parent (e.g. a Column above an Expanded list).
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Expanded(
                      child: Padding(
                        padding: const EdgeInsets.only(top: 6),
                        child: Text(
                          title,
                          style: const TextStyle(
                            color: CymbraColors.onSurface,
                            fontSize: 16,
                            fontWeight: FontWeight.w800,
                            height: 1.2,
                          ),
                        ),
                      ),
                    ),
                    if (onClose != null)
                      IconButton(
                        icon: const Icon(Icons.close, size: 18),
                        color: CymbraColors.onSurfaceVariant,
                        visualDensity: VisualDensity.compact,
                        tooltip: MaterialLocalizations.of(
                          context,
                        ).closeButtonTooltip,
                        onPressed: onClose,
                      )
                    else
                      const SizedBox(width: 8),
                  ],
                ),
                const SizedBox(height: 2),
                Padding(
                  padding: const EdgeInsets.only(right: 8),
                  child: Text(
                    message,
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                      fontSize: 13.5,
                      height: 1.3,
                    ),
                  ),
                ),
                const SizedBox(height: 12),
                InkWell(
                  onTap: onAction,
                  borderRadius: BorderRadius.circular(6),
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 2,
                      vertical: 4,
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(
                          actionLabel,
                          style: const TextStyle(
                            color: CymbraColors.primary,
                            fontSize: 14,
                            fontWeight: FontWeight.w800,
                            decoration: TextDecoration.underline,
                            decorationColor: CymbraColors.primary,
                          ),
                        ),
                        const SizedBox(width: 6),
                        const Icon(
                          Icons.arrow_forward,
                          color: CymbraColors.primary,
                          size: 18,
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
