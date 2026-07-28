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
import 'session_notifier.dart';

part 'rating_activity_notifier.g.dart';

/// Wall-clock "now" seam. Deliberately separate from the monotonic
/// [Clock] (which only measures elapsed differences): the rating invite needs
/// real calendar time to know how many days since the user last rated. Override
/// in tests with a fixed clock so the "a few days" logic is deterministic.
@Riverpod(keepAlive: true)
DateTime Function() nowFn(Ref ref) => DateTime.now;

/// Persisted rating-nudge state: when the user last engaged with rating, and how
/// many times they have explicitly dismissed the invite.
class RatingActivityData {
  const RatingActivityData({this.lastAt, this.dismissals = 0});

  /// When the user last rated/engaged with the deck (`null` = never).
  final DateTime? lastAt;

  /// How many times the user has explicitly closed the invite.
  final int dismissals;
}

/// Whether to show the "rate a few scores" invite given the persisted [data] and
/// the current time. Pure + host-testable. Stops **permanently** once the user
/// has dismissed it [RatingActivity.maxDismissals] times (an uninterested user is
/// not nagged forever); otherwise a user who never rated is invited, and after a
/// rating the invite returns once [RatingActivity.inviteAfter] has elapsed.
bool shouldInviteToRate(RatingActivityData data, DateTime now) {
  if (data.dismissals >= RatingActivity.maxDismissals) return false;
  final last = data.lastAt;
  if (last == null) return true;
  return now.difference(last) >= RatingActivity.inviteAfter;
}

/// Tracks the rating nudge's persisted state so the library can nudge inactive
/// users to rate — and stop once they've dismissed it enough. The deck calls
/// [markRatedNow] on engagement; the banner's close calls [snooze].
@Riverpod(keepAlive: true)
class RatingActivity extends _$RatingActivity {
  /// Preferences key holding the epoch-millis of the last rating/engagement.
  static const String prefsKey = 'rating_last_at';

  /// Preferences key holding the explicit-dismissal count.
  static const String dismissKey = 'rating_dismiss_count';

  /// How long since the last rating before the invite reappears.
  static const Duration inviteAfter = Duration(days: 3);

  /// After this many explicit dismissals the invite never shows again.
  static const int maxDismissals = 3;

  @override
  Future<RatingActivityData> build() => _read();

  Future<RatingActivityData> _read() async {
    try {
      final prefs = ref.read(preferencesServiceProvider);
      final ms = int.tryParse(await prefs.getString(prefsKey) ?? '');
      final dis = int.tryParse(await prefs.getString(dismissKey) ?? '');
      return RatingActivityData(
        lastAt: ms == null ? null : DateTime.fromMillisecondsSinceEpoch(ms),
        dismissals: dis ?? 0,
      );
    } catch (_) {
      return const RatingActivityData(); // storage unavailable → "never rated"
    }
  }

  /// Record that the user engaged with rating (persisted best-effort). Resets the
  /// snooze window; does not count as a dismissal.
  Future<void> markRatedNow() => _update(dismissed: false);

  /// Dismiss the invite: snoozes it for [inviteAfter] and counts toward the
  /// permanent stop ([maxDismissals]).
  Future<void> snooze() => _update(dismissed: true);

  Future<void> _update({required bool dismissed}) async {
    final now = ref.read(nowFnProvider)();
    final prev = state.valueOrNull ?? const RatingActivityData();
    final next = RatingActivityData(
      lastAt: now,
      dismissals: prev.dismissals + (dismissed ? 1 : 0),
    );
    state = AsyncData(next);
    try {
      final prefs = ref.read(preferencesServiceProvider);
      await prefs.setString(prefsKey, now.millisecondsSinceEpoch.toString());
      if (dismissed) {
        await prefs.setString(dismissKey, next.dismissals.toString());
      }
    } catch (_) {
      // Best-effort: the in-memory value still applies this session.
    }
  }
}

/// Whether the library should show the rating invite right now: only for a
/// signed-in user, once the persisted state has loaded, and only when
/// [shouldInviteToRate] says so (not snoozed, not permanently dismissed).
@riverpod
bool ratingInviteVisible(Ref ref) {
  if (!ref.watch(canUseOnlineServicesProvider)) return false;
  final activity = ref.watch(ratingActivityProvider);
  final now = ref.watch(nowFnProvider)();
  return activity.maybeWhen(
    data: (data) => shouldInviteToRate(data, now),
    orElse: () => false, // still loading / error → don't flash the banner
  );
}
