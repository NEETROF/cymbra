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
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../analytics/usage_actions.dart';
import '../services/auth_service.dart';
import '../services/catalog_service.dart';
import 'curator_profile_notifier.dart';
import 'score_catalog.dart';
import 'session_notifier.dart';
import 'usage_tracking_notifier.dart';

part 'catalog_daily_access_notifier.freezed.dart';
part 'catalog_daily_access_notifier.g.dart';

/// The caller's freemium daily-access state (change: add-score-daily-access-
/// rewards, design D8): ONE read the hub/library chip, the opened-today marks
/// and the unlock sheet all render from. `null` = no gate (signed out, or the
/// backend has none) — nothing to show.
///
/// Refreshed by: the initial read (identity-scoped — rebuilds on auth change),
/// every served/locked open (the bytes fetch carries the state, reported by the
/// notation notifier), a successful unlock, and [refresh] (app resume).
@Riverpod(keepAlive: true)
class CatalogDailyAccess extends _$CatalogDailyAccess {
  @override
  Future<CatalogAccessState?> build() async {
    // Identity-scoped: a sign-in/sign-out re-reads (or clears) the state.
    if (!ref.watch(canUseOnlineServicesProvider)) return null;
    try {
      return await ref.read(catalogServiceProvider).dailyAccess();
    } on AuthException catch (e) {
      // A backend without the RPC / a transient failure: no chip, no error UI.
      debugPrint('daily access read failed: ${e.error}');
      return null;
    }
  }

  /// Adopt the state a bytes fetch or an unlock just returned (the server's
  /// truth after that operation) — cheaper and fresher than a re-read.
  void report(CatalogAccessState? access) {
    if (access == null) return;
    // The `locked` bit belongs to the open that produced it, not the standing.
    state = AsyncData(access.copyWith(locked: false));
  }

  /// Re-read from the server (app resume, pull-to-refresh).
  Future<void> refresh() async {
    if (!ref.read(canUseOnlineServicesProvider)) {
      state = const AsyncData(null);
      return;
    }
    try {
      state = AsyncData(await ref.read(catalogServiceProvider).dailyAccess());
    } on AuthException catch (e) {
      debugPrint('daily access refresh failed: ${e.error}');
    }
  }
}

/// Immutable state of the confirmed day-slot unlock (change: add-score-daily-
/// access-rewards, design D4/D8): which piece was just unlocked (so the listener
/// re-opens it), a monotonic sequence so two unlocks in a row both fire, and a
/// typed failure — a state, never a thrown exception to the UI.
@freezed
abstract class CatalogUnlockState with _$CatalogUnlockState {
  const factory CatalogUnlockState({
    /// The entry whose day-slot was just bought (the listener re-opens it).
    CatalogEntry? unlocked,

    /// Increments once per completed attempt (success or failure).
    @Default(0) int seq,

    /// The last attempt failed for lack of points (FAILED_PRECONDITION).
    @Default(false) bool insufficient,

    /// The last attempt failed for any other reason.
    @Default(false) bool error,

    /// An attempt is in flight (the sheet disables its button).
    @Default(false) bool busy,
  }) = _CatalogUnlockState;
}

/// Drives the confirmed points day-slot: `unlock(entry)` is fire-and-observe —
/// the UI never awaits it; a dedicated listener reacts to [CatalogUnlockState].
/// On success the returned access state is reported to [catalogDailyAccessProvider]
/// and [rewardRevisionProvider] is bumped so the balance/profile refresh
/// reactively (no sibling invalidation).
@Riverpod(keepAlive: true)
class CatalogUnlock extends _$CatalogUnlock {
  int _claimed = 0;

  @override
  CatalogUnlockState build() => const CatalogUnlockState();

  /// Claim outcome [seq] for surfacing: `true` for the FIRST caller only. The
  /// listener widget is mounted on stacked screens (the hub is pushed over the
  /// library, both alive), so without this the same unlock would be surfaced —
  /// and the piece re-opened — twice.
  bool claim(int seq) {
    if (seq <= _claimed) return false;
    _claimed = seq;
    return true;
  }

  Future<void> unlock(CatalogEntry entry) async {
    final catalogId = entry.catalogId;
    if (catalogId == null || state.busy) return;
    state = state.copyWith(busy: true, insufficient: false, error: false);
    try {
      final access = await ref
          .read(catalogServiceProvider)
          .unlockForToday(catalogId);
      ref.read(catalogDailyAccessProvider.notifier).report(access);
      ref.read(rewardRevisionProvider.notifier).bump();
      unawaited(
        ref
            .read(usageTrackingNotifierProvider.notifier)
            .record(UsageActions.catalogDaySlotUnlock, subjectId: catalogId),
      );
      state = CatalogUnlockState(unlocked: entry, seq: state.seq + 1);
    } on AuthException catch (e) {
      debugPrint('day-slot unlock failed for $catalogId: ${e.error}');
      state = CatalogUnlockState(
        seq: state.seq + 1,
        insufficient: e.error == AuthError.failedPrecondition,
        error: e.error != AuthError.failedPrecondition,
      );
    } catch (e) {
      debugPrint('day-slot unlock failed for $catalogId: $e');
      state = CatalogUnlockState(seq: state.seq + 1, error: true);
    }
  }
}
