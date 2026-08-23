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

import '../services/preferences_service.dart';
import '../services/streak_service.dart';
import 'curator_profile_notifier.dart';
import 'rating_activity_notifier.dart';
import 'session_notifier.dart';

part 'streak_notifier.g.dart';

/// A monotonic bump signal for "a play was delivered to the server". The play
/// outbox bumps it **after** the server acks; the streak provider `ref.listen`s
/// it and refreshes itself, so the chip catches up the moment today's session
/// lands — without the outbox knowing the streak exists.
@Riverpod(keepAlive: true)
class StreakRevision extends _$StreakRevision {
  @override
  int build() => 0;

  void bump() => state = state + 1;
}

/// The signed-in player's practice streak (change: add-practice-streak).
///
/// An AsyncNotifier over the injectable [streakServiceProvider]. The value is
/// whatever the **server** says: the app never advances a streak locally, so a
/// second device (or a session synced from the outbox) is reflected on the next
/// read rather than drifting.
@riverpod
class Streak extends _$Streak {
  @override
  Future<StreakView> build() {
    // A play delivered from the outbox is what advances the streak server-side,
    // so re-read when one lands. Reactive: the outbox bumps a revision after it
    // persists and this provider invalidates ITSELF — no provider reaches in and
    // invalidates a sibling (architecture rule 2).
    ref.listen(streakRevisionProvider, (_, _) => ref.invalidateSelf());
    // A streak is server-owned account data: a signed-out (or offline-only)
    // session has none, and must not reach for the network to learn that. The
    // chip then renders its muted zero state, and signing in re-reads.
    if (!ref.watch(canUseOnlineServicesProvider)) {
      return Future.value(StreakView.none);
    }
    return ref.read(streakServiceProvider).getStreak();
  }

  /// Reload the standing (a session was just recorded, or the app resumed). A
  /// failure lands in the state, never thrown.
  Future<void> refresh() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(
      () => ref.read(streakServiceProvider).getStreak(),
    );
  }

  /// Spend points to restore the broken streak — called ONLY after the user
  /// confirms the offer (design D2: never a silent debit).
  ///
  /// Reports through `state`: the UI listens for the fresh [StreakView] (or the
  /// error) rather than awaiting a return value and branching on it. A refusal
  /// (grace elapsed, balance moved) surfaces as `AsyncError` with the standing
  /// left as the server last reported it.
  Future<void> recover() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(() async {
      final result = await ref.read(streakServiceProvider).recover();
      // The debit changed the points balance: nudge everything reading rewards.
      ref.read(rewardRevisionProvider.notifier).bump();
      return result.streak;
    });
  }
}

/// The local calendar day of [now] as `yyyy-mm-dd` — the key a declined offer is
/// filed under. Local, not UTC, because that is the day the *server* decided the
/// offer on (the app sends its UTC offset with every streak read).
String localDayKey(DateTime now) =>
    '${now.year.toString().padLeft(4, '0')}-'
    '${now.month.toString().padLeft(2, '0')}-'
    '${now.day.toString().padLeft(2, '0')}';

/// The local day on which the user last said "not this time" to a recovery offer
/// (`null` = never), persisted through the injectable preferences seam.
///
/// A declined offer used to be remembered only in the listener widget's `State`,
/// so it came back on the next cold start — and even on a plain navigation
/// between the two screens that mount the listener. Since the grace window is one
/// local day, "declined today" means declined for the entire life of *this*
/// offer: a new break tomorrow is a new question, and gets asked.
@Riverpod(keepAlive: true)
class StreakRecoveryDecline extends _$StreakRecoveryDecline {
  /// Preferences key for the last declined day.
  static const String prefsKey = 'streak_recovery_declined_day';

  @override
  Future<String?> build() async {
    try {
      return await ref.read(preferencesServiceProvider).getString(prefsKey);
    } catch (_) {
      // Storage unavailable: no recorded decline. The offer still stands, which
      // is the harmless direction — nothing is ever debited without a yes.
      return null;
    }
  }

  /// Record that today's offer was declined. The state moves first, so the offer
  /// is gone the instant the dialog closes rather than one disk round-trip later.
  Future<void> declineToday() async {
    final day = localDayKey(ref.read(nowFnProvider)());
    state = AsyncData(day);
    try {
      await ref.read(preferencesServiceProvider).setString(prefsKey, day);
    } catch (_) {
      // The in-memory decline still holds for this session; only the next cold
      // start would ask again.
    }
  }
}

/// Whether a recovery offer should be surfaced right now — a broken streak that
/// is inside the grace window, affordable, AND not already declined today.
///
/// Deliberately matches `AsyncData` only: while loading, and after a failure
/// (which keeps the previous value around), this is false — so a transient error
/// or a just-refused spend can never re-pop a confirmation to spend points. The
/// decline is read the same way: until it has loaded there is no offer, because
/// re-asking someone who already said no is the failure this guards against.
@riverpod
bool streakRecoveryOffered(Ref ref) {
  if (ref.watch(streakProvider) case AsyncData(
    :final value,
  ) when value.recoverable) {
    return switch (ref.watch(streakRecoveryDeclineProvider)) {
      AsyncData(value: final declined) =>
        declined != localDayKey(ref.read(nowFnProvider)()),
      _ => false,
    };
  }
  return false;
}

/// Whether the at-risk nudge has already been shown in this app session. Held in
/// a provider rather than the listener's `State` so the two screens that mount
/// `StreakListener` share one answer — a cue that reappears on every navigation
/// is nagging, not helpful.
@Riverpod(keepAlive: true)
class StreakNudgeShown extends _$StreakNudgeShown {
  @override
  bool build() => false;

  void mark() => state = true;
}
