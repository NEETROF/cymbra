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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/curator_rewards_service.dart';
import '../services/preferences_service.dart';
import 'play_reward_cue.dart';

part 'curator_profile_notifier.g.dart';

/// A monotonic bump signal for reward changes made *outside* the profile provider
/// (a redeem in the reward shop). The redeem action bumps it **after** it
/// persists; the profile `ref.listen`s it and refreshes itself — so no provider
/// invalidates a sibling (architecture rule 2).
@riverpod
class RewardRevision extends _$RewardRevision {
  @override
  int build() => 0;

  void bump() => state = state + 1;
}

/// The signed-in curator's rewards snapshot (change: add-curation-rewards):
/// standing, curator stats, the full badge grid, and recent activity. An
/// AsyncNotifier over the injectable [curatorRewardsServiceProvider]; testable
/// without a live backend via a provider override.
@riverpod
class CuratorProfile extends _$CuratorProfile {
  @override
  Future<CuratorRewardsView> build() {
    // Refresh when a redeem happened elsewhere (the shop bumps the revision after
    // it persists) — reactive, never invalidated by a sibling.
    ref.listen(rewardRevisionProvider, (_, _) => ref.invalidateSelf());
    // Same shape for points earned by PLAYING (change: add-play-rewards): the
    // outbox publishes the award as the ack lands, and the standing (lifetime,
    // level, balance) that this snapshot carries has just moved.
    ref.listen(
      playRewardCueProvider.select((s) => s.seq),
      (_, _) => ref.invalidateSelf(),
    );
    return ref.read(curatorRewardsServiceProvider).getRewards();
  }

  /// Reload the snapshot (pull-to-refresh / retry after an error). A failure lands
  /// in the state (`AsyncValue.guard`), never thrown.
  Future<void> refresh() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(
      () => ref.read(curatorRewardsServiceProvider).getRewards(),
    );
  }
}

/// The activity kinds that represent a *deferred* award the user should be
/// notified about (honesty settlements + moderator adjustments) — as opposed to
/// the immediate coverage award they already saw as a "+N" cue, or their own
/// redeem.
const Set<String> _deferredAwardKinds = {'honesty', 'adjustment'};

/// Persisted "last seen curator activity" timestamp, so the chip/profile entry
/// can show a notification dot when a deferred honesty/adjustment award has
/// landed since the user last opened the profile.
@Riverpod(keepAlive: true)
class CuratorActivitySeen extends _$CuratorActivitySeen {
  /// Preferences key holding the epoch-millis of the newest activity the user has
  /// seen (by opening the profile).
  static const String prefsKey = 'curator_activity_seen';

  @override
  Future<DateTime?> build() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      final ms = int.tryParse(raw ?? '');
      return ms == null ? null : DateTime.fromMillisecondsSinceEpoch(ms);
    } catch (_) {
      return null; // storage unavailable → treat as "never seen"
    }
  }

  /// Mark activity up to [at] as seen (best-effort persisted).
  Future<void> markSeen(DateTime at) async {
    state = AsyncData(at);
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, at.millisecondsSinceEpoch.toString());
    } catch (_) {
      // Best-effort: the in-memory value still applies this session.
    }
  }
}

/// Whether the curator has unseen deferred awards (honesty/adjustment) landed
/// since they last opened the profile — drives the notification dot on the chip.
/// False while either source is still loading, or when there is no such award.
@riverpod
bool curatorHasUnseenAwards(Ref ref) {
  final rewards = ref.watch(curatorProfileProvider).valueOrNull;
  if (rewards == null) return false;
  final newestDeferred = rewards.recent
      .where((a) => _deferredAwardKinds.contains(a.kind))
      .map((a) => a.createdAt)
      .fold<DateTime?>(
        null,
        (best, at) => best == null || at.isAfter(best) ? at : best,
      );
  if (newestDeferred == null) return false;
  final seen = ref.watch(curatorActivitySeenProvider).valueOrNull;
  return seen == null || newestDeferred.isAfter(seen);
}
