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

/// The small help card shown when a staff symbol is long-pressed: a bold title
/// and a one- or two-sentence explanation, plus a close affordance (change:
/// add-notation-help). Purely presentational — the staff area positions it and
/// supplies the localized [title]/[body]; dismissal is routed through [onClose].
///
/// Accessible: the whole card is one semantics node labelled with the title and
/// body, it carries an explicit close button (so it is dismissible without
/// relying on the long-press or an outside tap), and its colours use the theme's
/// container/onSurface pair for contrast.
class NotationHelpBubble extends StatelessWidget {
  const NotationHelpBubble({
    super.key,
    required this.title,
    required this.body,
    required this.onClose,
  });

  final String title;
  final String body;
  final VoidCallback onClose;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Semantics(
      container: true,
      label: '$title. $body',
      child: Material(
        color: Colors.transparent,
        child: Container(
          decoration: BoxDecoration(
            color: CymbraColors.surfaceContainerHigh,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: CymbraColors.onSurfaceVariant.withValues(alpha: 0.2),
            ),
            boxShadow: const [
              BoxShadow(
                color: Color(0x33000000),
                blurRadius: 12,
                offset: Offset(0, 4),
              ),
            ],
          ),
          padding: const EdgeInsets.fromLTRB(14, 12, 8, 12),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Flexible(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      title,
                      style: const TextStyle(
                        color: CymbraColors.onSurface,
                        fontSize: 15,
                        fontWeight: FontWeight.w800,
                      ),
                    ),
                    const SizedBox(height: 3),
                    Text(
                      body,
                      style: const TextStyle(
                        color: CymbraColors.onSurfaceVariant,
                        fontSize: 13,
                        height: 1.35,
                      ),
                    ),
                  ],
                ),
              ),
              // Explicit, screen-reader-friendly dismissal.
              IconButton(
                key: const Key('notation-help-close'),
                onPressed: onClose,
                visualDensity: VisualDensity.compact,
                iconSize: 18,
                tooltip: l10n.notationHelpClose,
                icon: const Icon(
                  Icons.close,
                  color: CymbraColors.onSurfaceVariant,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
