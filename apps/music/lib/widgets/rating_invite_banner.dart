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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../layout/device_class.dart';
import '../screens/rating_deck_screen.dart';
import '../state/rating_activity_notifier.dart';
import 'notice_callout.dart';

/// The library's "rate some scores" nudge: shows a [NoticeCallout] when the user
/// hasn't rated in a few days (change: add-app-score-rating). Tapping the link
/// opens the rating deck; closing it snoozes the invite. Renders nothing when the
/// invite isn't due, so it can sit unconditionally at the top of the library.
class RatingInviteBanner extends ConsumerWidget {
  const RatingInviteBanner({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Async: false / loading / error all hide the banner (no flash, no nudge
    // when there's nothing to rate).
    final visible = ref.watch(ratingInviteVisibleProvider).valueOrNull ?? false;
    if (!visible) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
      child: NoticeCallout(
        // A phone stays compact even in landscape (wide but small screen).
        dense: context.isPhoneLayout,
        icon: Icons.reviews_outlined,
        title: l10n.ratingInviteTitle,
        message: l10n.ratingInviteBody,
        actionLabel: l10n.ratingInviteCta,
        onAction: () {
          Navigator.of(context).push(
            MaterialPageRoute<void>(builder: (_) => const RatingDeckScreen()),
          );
        },
        onClose: () =>
            unawaited(ref.read(ratingActivityProvider.notifier).snooze()),
      ),
    );
  }
}
