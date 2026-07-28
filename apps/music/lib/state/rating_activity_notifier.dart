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

/// Whether to show the "rate a few scores" invite given when the user last rated
/// (`null` = never) and the current time. Pure + host-testable. A user who has
/// never rated is invited; otherwise the invite returns once at least
/// [RatingActivity.inviteAfter] has elapsed since their last rating.
bool shouldInviteToRate(DateTime? lastRatedAt, DateTime now) {
  if (lastRatedAt == null) return true;
  return now.difference(lastRatedAt) >= RatingActivity.inviteAfter;
}

/// Tracks when the signed-in user last submitted a score rating, persisted across
/// restarts, so the library can nudge them to rate again after a lull. The deck
/// calls [markRatedNow] on every successful rating.
@Riverpod(keepAlive: true)
class RatingActivity extends _$RatingActivity {
  /// Preferences key holding the epoch-millis of the last rating.
  static const String prefsKey = 'rating_last_at';

  /// How long since the last rating before the invite reappears.
  static const Duration inviteAfter = Duration(days: 3);

  @override
  Future<DateTime?> build() => _read();

  Future<DateTime?> _read() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      final ms = raw == null ? null : int.tryParse(raw);
      return ms == null ? null : DateTime.fromMillisecondsSinceEpoch(ms);
    } catch (_) {
      return null; // storage unavailable → treat as "never rated"
    }
  }

  /// Record that the user just rated a score (persisted best-effort).
  Future<void> markRatedNow() => _stampNow();

  /// Dismiss the invite for now: snoozes it for [inviteAfter] (same effect as a
  /// rating — the timer resets), so closing the banner hides it for a few days.
  Future<void> snooze() => _stampNow();

  Future<void> _stampNow() async {
    final now = ref.read(nowFnProvider)();
    state = AsyncData(now);
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, now.millisecondsSinceEpoch.toString());
    } catch (_) {
      // Best-effort: the in-memory value still suppresses the invite this session.
    }
  }
}

/// Whether the library should show the rating invite right now: only for a
/// signed-in user, once their last-rated timestamp has loaded, and only when
/// [shouldInviteToRate] says enough time has passed.
@riverpod
bool ratingInviteVisible(Ref ref) {
  if (!ref.watch(canUseOnlineServicesProvider)) return false;
  final activity = ref.watch(ratingActivityProvider);
  final now = ref.watch(nowFnProvider)();
  return activity.maybeWhen(
    data: (lastRatedAt) => shouldInviteToRate(lastRatedAt, now),
    orElse: () => false, // still loading / error → don't flash the banner
  );
}
