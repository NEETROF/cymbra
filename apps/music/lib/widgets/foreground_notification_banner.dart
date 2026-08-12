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

import '../state/foreground_notification.dart';
import '../theme/cymbra_theme.dart';

/// Widest the banner gets; beyond this it reads as a sheet, not a notification.
const double _maxBannerWidth = 420;

/// The in-app surface for a foreground notification (change:
/// add-foreground-notifications, design D2): a dismissible, tappable card
/// pinned top-center, rendered in the app's own idiom — never an OS alert.
///
/// Mounted in the `MaterialApp.builder` stack **above the navigator** (like the
/// coach layer) so it paints over whatever screen is open. It only renders when
/// the notifier holds a banner; empty state is a zero-sized box that installs
/// no input barrier. Tap and dismiss go to the notifier — the tap's navigation
/// side effect is the `ForegroundNotificationListener`'s job, since this layer
/// lives outside the navigator.
class ForegroundNotificationLayer extends ConsumerWidget {
  const ForegroundNotificationLayer({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final banner = ref.watch(
      foregroundNotificationProvider.select((s) => s.banner),
    );
    if (banner == null) return const SizedBox.shrink();
    final notifier = ref.read(foregroundNotificationProvider.notifier);
    return SafeArea(
      child: Align(
        alignment: Alignment.topCenter,
        child: Padding(
          padding: const EdgeInsets.only(top: 8, left: 16, right: 16),
          child: Material(
            key: const Key('foreground-banner'),
            color: CymbraColors.surfaceContainerHigh,
            elevation: 6,
            borderRadius: BorderRadius.circular(12),
            child: InkWell(
              borderRadius: BorderRadius.circular(12),
              onTap: notifier.tap,
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: _maxBannerWidth),
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(16, 10, 8, 10),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Flexible(
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            // The copy arrives already localized from the
                            // feature that owns the category — rendered as-is.
                            if (banner.title.isNotEmpty)
                              Text(
                                banner.title,
                                style: const TextStyle(
                                  color: CymbraColors.onSurface,
                                  fontSize: 14,
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                            if (banner.body.isNotEmpty)
                              Text(
                                banner.body,
                                style: const TextStyle(
                                  color: CymbraColors.onSurfaceVariant,
                                  fontSize: 14,
                                ),
                              ),
                          ],
                        ),
                      ),
                      const SizedBox(width: 8),
                      IconButton(
                        key: const Key('foreground-banner-dismiss'),
                        // A semantic label, not a `tooltip`: this layer sits
                        // above the navigator, so there is no Overlay for a
                        // Tooltip to mount into (it asserts in debug).
                        icon: Icon(
                          Icons.close,
                          size: 18,
                          semanticLabel: MaterialLocalizations.of(
                            context,
                          ).closeButtonTooltip,
                        ),
                        color: CymbraColors.onSurfaceVariant,
                        visualDensity: VisualDensity.compact,
                        onPressed: notifier.dismiss,
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
