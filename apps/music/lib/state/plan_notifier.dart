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

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/app_platform.dart';
import '../services/auth_service.dart';
import '../services/legal_links.dart';
import '../services/plan_service.dart';
import '../services/store_client.dart';
import 'rating_activity_notifier.dart' show nowFnProvider;
import 'session_notifier.dart';

part 'plan_notifier.freezed.dart';
part 'plan_notifier.g.dart';

/// Whether the plan system is live (`plans.enabled`). Plain `false` by default so
/// tests never build the flag client; `main.dart` overrides it with the remote
/// flag. While off, every plan-aware seam behaves as before the plan system.
@Riverpod(keepAlive: true)
bool plansEnabled(Ref ref) => false;

/// Days a lapsed plan is still honoured locally while offline (mirrors the
/// server's default `plans.grace_days`) — the local belt of design D13.
const kLocalPlanGraceDays = 3;

/// The caller's plan snapshot (change: add-premium-subscription, design D10):
/// ONE server-decided read every gate, the paywall and the account status
/// render from. `PlanSnapshotView.free` when signed out or unreachable at first
/// read; a failed refresh keeps the last-known snapshot (never degrades to free
/// because of a network error).
///
/// Refreshed by: the initial read (identity-scoped — rebuilds on auth change),
/// a purchase / restore report (adopted from the response), and [refresh] (app
/// resume, explicit request, after a code redemption).
@Riverpod(keepAlive: true)
class Plan extends _$Plan {
  @override
  Future<PlanSnapshotView> build() async {
    if (!ref.watch(canUseOnlineServicesProvider)) {
      return PlanSnapshotView.free;
    }
    final platform = ref.watch(appPlatformProvider);
    try {
      return await ref.read(planServiceProvider).getMyPlan(platform);
    } on AuthException catch (e) {
      debugPrint('plan read failed: ${e.error}');
      return PlanSnapshotView.free;
    } catch (e) {
      debugPrint('plan read failed: $e');
      return PlanSnapshotView.free;
    }
  }

  /// Re-read from the server; a failure keeps the last-known snapshot.
  Future<void> refresh() async {
    if (!ref.read(canUseOnlineServicesProvider)) {
      state = const AsyncData(PlanSnapshotView.free);
      return;
    }
    try {
      final platform = ref.read(appPlatformProvider);
      state = AsyncData(
        await ref.read(planServiceProvider).getMyPlan(platform),
      );
    } catch (e) {
      debugPrint('plan refresh failed: $e');
    }
  }

  /// Adopt the snapshot a purchase report just returned (the server's truth).
  void adopt(PlanSnapshotView snapshot) {
    state = AsyncData(snapshot);
  }
}

/// The plan as the gates should honour it NOW: the last-known snapshot, minus
/// the local belt — once `endsAt + grace` has passed (device offline, no
/// refresh possible) plan-only content stops opening until a reconnect. While
/// `plans.enabled` is off the answer is always "as before" ([PlanSnapshotView.free]
/// with every consumer treating it as ungated).
@riverpod
PlanSnapshotView effectivePlan(Ref ref) {
  final snap = ref.watch(planProvider).valueOrNull ?? PlanSnapshotView.free;
  final ends = snap.endsAt;
  if (ends == null) return snap;
  final now = ref.watch(nowFnProvider)().toUtc();
  final lapsed = now.isAfter(
    ends.add(const Duration(days: kLocalPlanGraceDays)),
  );
  return lapsed ? PlanSnapshotView.free : snap;
}

/// Whether catalog scores may be cached for offline play (spec
/// `offline-score-cache`, "Offline caching of catalog scores is a premium
/// unlock"): everyone while the plan system is off (pre-plan behaviour), else
/// only a plan granting `offline.cache`. Own uploads are never gated by this.
@riverpod
bool catalogOfflineCacheAllowed(Ref ref) {
  if (!ref.watch(plansEnabledProvider)) return true;
  return ref.watch(effectivePlanProvider).grants('offline.cache');
}

/// Outcome of the last purchase-flow attempt (a state, never an exception to
/// the UI); a dedicated listener widget reacts to it.
enum PurchaseOutcome {
  none,
  purchased,
  restored,
  cancelled,
  pending,
  checkoutOpened,
  nothingToRestore,
  failed,
}

@freezed
abstract class PurchaseFlowState with _$PurchaseFlowState {
  const factory PurchaseFlowState({
    @Default(false) bool busy,

    /// Increments once per completed attempt so two identical outcomes in a
    /// row both fire.
    @Default(0) int seq,
    @Default(PurchaseOutcome.none) PurchaseOutcome outcome,
  }) = _PurchaseFlowState;
}

/// Drives purchases and restores (change: add-premium-subscription, design D7–D10):
/// store builds go through the [StoreClient] (StoreKit 2 / Play Billing) and
/// report every receipt to the server for verification; desktop / web open the
/// hosted merchant-of-record checkout in the browser. Fire-and-observe: the UI
/// never awaits [buy] / [restore]; it reacts to [PurchaseFlowState].
@Riverpod(keepAlive: true)
class PurchaseFlow extends _$PurchaseFlow {
  StreamSubscription<StoreEvent>? _sub;
  int _claimed = 0;

  @override
  PurchaseFlowState build() {
    _sub?.cancel();
    _sub = ref.watch(storeClientProvider).events.listen(_onStoreEvent);
    ref.onDispose(() => _sub?.cancel());
    return const PurchaseFlowState();
  }

  /// Claim outcome [seq] for surfacing: `true` for the FIRST caller only (the
  /// listener widget may be mounted on stacked screens).
  bool claim(int seq) {
    if (seq <= _claimed) return false;
    _claimed = seq;
    return true;
  }

  void _finish(PurchaseOutcome outcome) {
    state = state.copyWith(busy: false, seq: state.seq + 1, outcome: outcome);
  }

  /// Start a purchase of [productId] through this platform's channel.
  Future<void> buy(String productId) async {
    if (state.busy) return;
    final plan = ref.read(planProvider).valueOrNull;
    if (plan == null || !plan.canPurchaseHere) return;
    state = state.copyWith(busy: true);
    final platform = ref.read(appPlatformProvider);
    if (platform.usesWebCheckout) {
      try {
        final url = await ref
            .read(planServiceProvider)
            .createWebCheckout(productId);
        await ref.read(legalLinkLauncherProvider).open(url);
        _finish(PurchaseOutcome.checkoutOpened);
      } catch (e) {
        debugPrint('web checkout failed: $e');
        _finish(PurchaseOutcome.failed);
      }
      return;
    }
    final userId = ref.read(currentUserIdProvider);
    if (userId == null) {
      _finish(PurchaseOutcome.failed);
      return;
    }
    try {
      await ref.read(storeClientProvider).buy(productId, accountToken: userId);
      // The outcome arrives on the store event stream.
    } catch (e) {
      debugPrint('store purchase failed: $e');
      _finish(PurchaseOutcome.failed);
    }
  }

  /// Re-assert the store transactions of this account ("restore purchases").
  Future<void> restore() async {
    if (state.busy) return;
    final userId = ref.read(currentUserIdProvider);
    if (userId == null) return;
    state = state.copyWith(busy: true);
    _restoring = true;
    _restoredAny = false;
    try {
      await ref.read(storeClientProvider).restore(accountToken: userId);
      // Give the store a moment to replay; then settle if nothing came back.
      await Future<void>.delayed(const Duration(seconds: 3));
      if (_restoring && !_restoredAny) {
        _restoring = false;
        _finish(PurchaseOutcome.nothingToRestore);
      }
    } catch (e) {
      debugPrint('restore failed: $e');
      _restoring = false;
      _finish(PurchaseOutcome.failed);
    }
  }

  bool _restoring = false;
  bool _restoredAny = false;

  Future<void> _onStoreEvent(StoreEvent event) async {
    switch (event) {
      case StoreEventCancelled():
        _finish(PurchaseOutcome.cancelled);
      case StoreEventPending():
        _finish(PurchaseOutcome.pending);
      case StoreEventError(:final message):
        debugPrint('store error: $message');
        _finish(PurchaseOutcome.failed);
      case StoreEventReceipt(:final receipt):
        await _report(receipt);
    }
  }

  Future<void> _report(StoreReceipt receipt) async {
    final platform = ref.read(appPlatformProvider);
    final channel = switch (platform) {
      AppPlatform.android => PlanChannel.google,
      _ => PlanChannel.apple,
    };
    try {
      final view = await ref
          .read(planServiceProvider)
          .reportStorePurchase(
            channel: channel,
            payload: receipt.payload,
            productId: receipt.productId,
          );
      await ref.read(storeClientProvider).complete(receipt);
      ref.read(planProvider.notifier).adopt(view.plan);
      if (receipt.restored) {
        _restoredAny = true;
        _restoring = false;
        _finish(PurchaseOutcome.restored);
      } else {
        _finish(PurchaseOutcome.purchased);
      }
    } catch (e) {
      // The store keeps the transaction pending; a later restore re-reports it.
      debugPrint('purchase report failed: $e');
      _restoring = false;
      _finish(PurchaseOutcome.failed);
    }
  }
}
