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
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/curator_rewards_service.dart';
import 'curator_profile_notifier.dart';

part 'reward_shop_notifier.freezed.dart';
part 'reward_shop_notifier.g.dart';

/// Immutable state of the reward shop (change: add-curation-rewards): the loaded
/// items, plus a transient "just redeemed" cue so a dedicated listener can show a
/// celebration exactly once per redeem.
@freezed
abstract class RewardShopState with _$RewardShopState {
  const factory RewardShopState({
    required List<RewardShopItemView> items,

    /// The label of the reward just redeemed (for the celebration), or null.
    String? lastRedeemedLabel,

    /// Increments once per completed redeem so a listener fires even for two
    /// consecutive redeems (or a redeem after a prior one with the same label).
    @Default(0) int redeemSeq,
    @Default(false) bool redeemError,
  }) = _RewardShopState;
}

/// Drives the reward shop: lists items through the injectable
/// [curatorRewardsServiceProvider] and owns the redeem mutation. On a successful
/// redeem it reloads its own items and bumps [rewardRevisionProvider] so the
/// curator profile (balance) refreshes reactively — the shop never invalidates a
/// sibling (architecture rule 2).
@riverpod
class RewardShop extends _$RewardShop {
  @override
  Future<RewardShopState> build() async {
    final items = await ref.read(curatorRewardsServiceProvider).listShop();
    return RewardShopState(items: items);
  }

  /// Redeem the reward keyed by [rewardKey]. Fire-and-observe: the UI reacts to
  /// the resulting state (loaded items refresh; a `lastRedeemedLabel`/`redeemSeq`
  /// cue drives the celebration; `redeemError` drives the error snackbar). A
  /// failure lands in the state, never thrown.
  Future<void> redeem(String rewardKey) async {
    final prev = state.valueOrNull;
    final label = prev?.items
        .where((i) => i.key == rewardKey)
        .map((i) => i.label)
        .firstOrNull;
    state = await AsyncValue.guard(() async {
      try {
        await ref.read(curatorRewardsServiceProvider).redeem(rewardKey);
      } catch (_) {
        // Reload so the shop reflects the (unchanged) server truth, and surface
        // the failure through the state (never a raw exception to the UI).
        final items = await ref.read(curatorRewardsServiceProvider).listShop();
        return RewardShopState(
          items: items,
          redeemError: true,
          redeemSeq: (prev?.redeemSeq ?? 0) + 1,
        );
      }
      // Persisted: signal the profile (balance) to refresh reactively.
      ref.read(rewardRevisionProvider.notifier).bump();
      final items = await ref.read(curatorRewardsServiceProvider).listShop();
      return RewardShopState(
        items: items,
        lastRedeemedLabel: label,
        redeemSeq: (prev?.redeemSeq ?? 0) + 1,
      );
    });
  }
}

/// The reward-shop items keyed by their `key`, for cross-referencing SoundFont
/// catalog entries (a piano `id` == a shop item `key`). Empty while loading / on
/// error, so callers just treat an absent key as "not a costed reward".
@riverpod
Map<String, RewardShopItemView> rewardShopItemsByKey(Ref ref) {
  final items = ref.watch(rewardShopProvider).valueOrNull?.items ?? const [];
  return {for (final i in items) i.key: i};
}
