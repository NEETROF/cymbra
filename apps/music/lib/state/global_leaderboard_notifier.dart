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

import '../services/global_leaderboard_service.dart';
import 'global_leaderboard.dart';
import 'leaderboard.dart';
import 'play_sync_notifier.dart';

part 'global_leaderboard_notifier.g.dart';

/// One global board `(mode, season)` — the Community screen's data source (change:
/// add-global-leaderboard). Reads through the injectable
/// [GlobalLeaderboardService] seam, so the screen is driven by state a test can
/// override with a fake board (no native library, no live backend). Keyed by mode
/// and season, so the tempo/reaction toggle and the season selector simply watch a
/// different instance.
///
/// [seasonId] is the empty string for the CURRENT season (a Riverpod family key
/// must be a stable value, and `null` would collide with "not chosen yet" at the
/// call site).
@riverpod
Future<GlobalLeaderboard> globalLeaderboard(
  Ref ref,
  LeaderboardMode mode,
  String seasonId,
) {
  // Refresh once the play-session outbox delivers: a scored run is captured
  // locally and delivered to the server asynchronously, so the global standings
  // only reflect it after the session lands. Re-fetch when the pending count
  // DROPS (an entry was acked). Self-invalidation only — the provider reacts to
  // the sync source, it does not poke a sibling (see the Riverpod reactivity
  // rules); same pattern as the per-piece board.
  ref.listen(playSyncNotifierProvider, (previous, next) {
    if (previous != null && next < previous) ref.invalidateSelf();
  });
  return ref
      .watch(globalLeaderboardServiceProvider)
      .getGlobalLeaderboard(
        mode: mode,
        seasonId: seasonId.isEmpty ? null : seasonId,
      );
}

/// The seasons the Community screen's selector offers (current + snapshotted
/// past ones). Read once per screen open — the set only changes at a rollover.
@riverpod
Future<GlobalSeasons> globalSeasons(Ref ref) =>
    ref.watch(globalLeaderboardServiceProvider).listSeasons();
