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

import '../l10n/gen/app_localizations.dart';
import '../state/curator_profile_notifier.dart';
import 'reward_celebration.dart';

/// Dedicated listener widget for the player subtree (architecture rule 4:
/// `ref.listen` side effects live in one place, not scattered through build
/// methods). It renders [child] and only wires listeners.
///
/// One job: celebrate a level crossed **by playing** (change: add-play-rewards),
/// through the same [showRewardCelebration] path a redeem already uses — a level
/// is a level, whichever activity earned the points.
///
/// It `watch`es the curator standing rather than only listening, so a baseline
/// level exists *before* the award lands; without one, the first level-up of the
/// session would have nothing to compare against and pass unnoticed. The standing
/// refreshes itself when the play-reward cue fires (see [CuratorProfile]), so this
/// widget never pokes a sibling provider.
class PlayRewardListeners extends ConsumerWidget {
  const PlayRewardListeners({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    ref.listen(curatorProfileProvider, (previous, next) {
      final before = previous?.valueOrNull?.level;
      final after = next.valueOrNull?.level;
      // Only a genuine RISE between two loaded snapshots celebrates: a first
      // load, a reload after an error, and a guest's failed read all have no
      // "before" to have crossed.
      if (before == null || after == null || after <= before) return;
      showRewardCelebration(
        context,
        title: l10n.rewardCelebrationLevelUpTitle,
        message: l10n.rewardCelebrationLevelUpMessage(after),
      );
    });
    // Keeps the standing loaded while the player is on screen, so the listener
    // above has something to compare the post-award value against.
    ref.watch(curatorProfileProvider);
    return child;
  }
}
