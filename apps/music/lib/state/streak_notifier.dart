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

/// The run a recovery in flight would restore, or 0 when none is running
/// (change: make-streak-recovery-reachable).
///
/// Exists because the spend and the report of its outcome are no longer in the
/// same widget: the sheet starts it, the listener says how it went. Armed by
/// [Streak.recover] itself so no call site can forget, and cleared by whoever
/// reports it.
@Riverpod(keepAlive: true)
class StreakRecoveryPending extends _$StreakRecoveryPending {
  @override
  int build() => 0;

  void arm(int run) => state = run;
  void clear() => state = 0;
}

/// The recovery offer the user last said "not this time" to (`null` = never),
/// persisted through the injectable preferences seam.
///
/// **Keyed to the break, not to the calendar day** (change:
/// add-drum-input-mapping — beta fix). It used to be filed under the local day,
/// on the reasoning that the grace window is one day wide and so a second
/// question could only be a second break. That reasoning infers a constant from
/// a **back-office value**: `streak.grace_days` is a flag, and a window wider
/// than a day re-raises the identical question every morning.
///
/// Production reads `1` (checked 2026-08-30), so that widening never happened
/// and this key fixed nothing anyone had met — the beta report was the refusal
/// being written after a `mounted` check and lost when the app was killed with
/// the dialog open. It is kept as what it is: protection against the flag being
/// widened, and a record whose grain matches the thing it answers.
///
/// A recovery offer only exists while the user has NOT played since the break —
/// once they play, the run is restarted and the offer is gone for good. So the
/// run a recovery would restore ([StreakView.recoverableStreak]) identifies the
/// offer for its whole life, and refusing it silences *that* offer until the
/// standing genuinely changes. A later break presents a different run and is
/// asked, once.
@Riverpod(keepAlive: true)
class StreakRecoveryCue extends _$StreakRecoveryCue {
  /// Preferences key for the declined offer. Distinct from the day-keyed key it
  /// replaces, so a stored day string is never read back as a streak count — an
  /// upgrading device simply has no recorded decline, and is asked once.
  static const String prefsKey = 'streak_recovery_declined_run';

  @override
  Future<int?> build() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      return raw == null ? null : int.tryParse(raw);
    } catch (_) {
      // Storage unavailable: no recorded decline. The offer still stands, which
      // is the harmless direction — nothing is ever debited without a yes.
      return null;
    }
  }

  /// Record that the unprompted cue for the offer to restore [run] days has
  /// been raised, so it is not raised again for this break — not on the other
  /// screen that mounts the listener, and not on the next launch.
  ///
  /// **Silences the cue, never the offer** (change:
  /// make-streak-recovery-reachable). The recovery stays reachable from the
  /// streak chip for as long as the server allows it. It was called `decline`
  /// when the cue *was* the offer and answering "not this time" ended both;
  /// keeping that name now would say the player forfeited something they did
  /// not.
  Future<void> silence(int run) async {
    state = AsyncData(run);
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, run.toString());
    } catch (_) {
      // The in-memory decline still holds for this session; only the next cold
      // start would ask again.
    }
  }
}

/// Whether the **unprompted cue** should be raised right now — a broken streak
/// inside the grace window, affordable, and not one already cued.
///
/// Not "whether a recovery is available": that question is answered by
/// `StreakView.recoverable`, and the chip offers it regardless of this
/// (change: make-streak-recovery-reachable). This gates the interruption only.
///
/// Deliberately matches `AsyncData` only: while loading, and after a failure
/// (which keeps the previous value around), this is false — so a transient error
/// or a just-refused spend can never re-pop a confirmation to spend points. The
/// record is read the same way: until it has loaded no cue is raised, because
/// re-raising it at someone who has already seen it is the failure this guards
/// against.
@riverpod
bool streakRecoveryCueDue(Ref ref) {
  if (ref.watch(streakProvider) case AsyncData(
    :final value,
  ) when value.recoverable) {
    return switch (ref.watch(streakRecoveryCueProvider)) {
      AsyncData(value: final cued) => cued != value.recoverableStreak,
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
