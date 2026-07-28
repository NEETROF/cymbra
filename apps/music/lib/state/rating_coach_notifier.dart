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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';

part 'rating_coach_notifier.g.dart';

/// Whether the one-time swipe-rating coach mark has been seen (change:
/// add-app-score-rating). Persisted via [preferencesServiceProvider] so it shows
/// **once** and never again — on this device — after the user opens the deck.
///
/// The state is tri-state: `null` while the stored value is still loading (so the
/// deck shows nothing yet and never flashes the hint at a returning user),
/// `false` once we know it was never seen (→ show the coach mark), `true` after
/// it has been seen (→ never show again).
@Riverpod(keepAlive: true)
class RatingCoachMark extends _$RatingCoachMark {
  /// Preferences key under which the "seen" flag lives.
  static const String prefsKey = 'rating_coach_seen';

  @override
  bool? build() {
    _restore();
    return null; // unknown until storage answers
  }

  Future<void> _restore() async {
    String? raw;
    try {
      raw = await ref.read(preferencesServiceProvider).getString(prefsKey);
    } catch (_) {
      // Storage unavailable → treat as already seen so we never nag on a device
      // whose preferences can't persist the dismissal.
      state = true;
      return;
    }
    state = raw == 'true'; // absent (never written) → false → show once
  }

  /// Record that the coach mark has been shown, so it never appears again.
  /// Best-effort persistence: the in-memory flag still applies this session.
  Future<void> markSeen() async {
    if (state == true) return;
    state = true;
    try {
      await ref.read(preferencesServiceProvider).setString(prefsKey, 'true');
    } catch (_) {
      // Best-effort: the in-memory value still suppresses the hint this session.
    }
  }
}
