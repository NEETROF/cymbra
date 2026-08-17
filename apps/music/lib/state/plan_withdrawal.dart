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

import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/curator_rewards_service.dart';
import '../services/offline_score_cache.dart';
import '../services/soundfont_storage.dart';
import 'saved_catalog_scores.dart';

part 'plan_withdrawal.g.dart';

/// Pure decision: which downloaded catalog font ids are plan-only leftovers —
/// costed items the caller no longer owns (neither free, nor points-redeemed).
/// Own imports and bundled pianos are not catalog fonts and never appear here.
List<String> lockedFontIds(Iterable<RewardShopItemView> shop) => [
  for (final i in shop)
    if (i.pointCost > 0 && !i.owned) i.key,
];

/// The app side of withdrawal on lapse (change: add-premium-subscription,
/// design D13; spec `music-premium-paywall` "Plan-only downloads are purged at
/// the next connection after the plan lapses"): delete the local copies of
/// premium SoundFonts no longer owned and the offline cache of CATALOG scores;
/// keep own uploads' cache, imported fonts, points-redeemed pianos. Driven by the
/// server's plan answer (the listener), never by the device clock alone.
@Riverpod(keepAlive: true)
class PlanWithdrawal extends _$PlanWithdrawal {
  @override
  int build() => 0; // number of withdrawals performed this session

  /// Perform the withdrawal once; returns `true` when anything was removed or
  /// the shop confirmed locked items. Best-effort: a failure is logged, never
  /// surfaced raw.
  Future<bool> withdraw() async {
    var didSomething = false;
    try {
      // 1. Premium SoundFont files not owned any more (the shop, re-read now that
      //    the plan is free, says which items are locked).
      final shop = await ref.read(curatorRewardsServiceProvider).listShop();
      final locked = lockedFontIds(shop);
      final dir = await ref.read(soundFontStorageDirProvider.future);
      for (final id in locked) {
        final f = File('${dir.path}/$id.sf2');
        if (await f.exists()) {
          await f.delete();
          didSomething = true;
        }
      }
      // 2. The offline cache of catalog scores (own uploads' cache is kept).
      final saved = await ref.read(savedCatalogScoresProvider.future);
      final cache = ref.read(offlineScoreCacheProvider);
      for (final entry in saved) {
        final id = entry.catalogId;
        if (id == null) continue;
        if (await cache.has('catalog:$id')) {
          await cache.evict('catalog:$id');
          didSomething = true;
        }
      }
      // The shop re-reads itself on the plan change (it watches the plan), so
      // the picker's lock mirror follows without a sibling invalidation.
    } catch (e) {
      debugPrint('plan withdrawal failed: $e');
    }
    if (didSomething) state = state + 1;
    return didSomething;
  }
}
