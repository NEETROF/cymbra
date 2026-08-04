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

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/leaderboard_service.dart';
import 'leaderboard.dart';
import 'play_sync_notifier.dart';

part 'my_standings_notifier.g.dart';

/// The caller's own leaderboard standings, keyed by piece id — the source for the
/// compact score-card badges (change: add-play-leaderboards). A shared, keep-alive
/// map: each catalog card asks for its own piece via [MyStandings.request], and
/// the requests are **coalesced** into a single batch RPC per frame, so a whole
/// page of cards costs one call. Only pieces the caller is ranked on land in the
/// map (a missing id ⇒ show a bare trophy).
///
/// Refreshes after a played session is delivered (the outbox count drops), so a
/// card's rank updates once the just-played result lands — without poking a
/// sibling provider (it reacts to the sync source and rebuilds itself).
@Riverpod(keepAlive: true)
class MyStandings extends _$MyStandings {
  /// Piece ids already requested (loaded or in-flight), to avoid re-fetching.
  final Set<String> _requested = {};

  /// Ids waiting for the next coalesced flush.
  final Set<String> _pending = {};
  bool _flushScheduled = false;

  @override
  Map<String, LeaderboardStanding> build() {
    // On delivery (pending outbox count drops), forget what we loaded so the next
    // `request` re-fetches — the just-played run may have changed the rank.
    ref.listen(playSyncNotifierProvider, (previous, next) {
      if (previous != null && next < previous) {
        _requested.clear();
        state = const {};
      }
    });
    return const {};
  }

  /// Ask for `scoreId`'s standing. Cheap + idempotent: already-known ids are
  /// skipped, and new ids are batched into one RPC on the next microtask, so many
  /// cards in the same frame produce a single call. Safe to call from `build`.
  void request(String scoreId) {
    if (_requested.contains(scoreId) || _pending.contains(scoreId)) return;
    _pending.add(scoreId);
    if (_flushScheduled) return;
    _flushScheduled = true;
    Future.microtask(_flush);
  }

  Future<void> _flush() async {
    _flushScheduled = false;
    final batch = _pending.toList(growable: false);
    _pending.clear();
    if (batch.isEmpty) return;
    _requested.addAll(batch);
    try {
      final fetched = await ref
          .read(leaderboardServiceProvider)
          .getMyStandings(batch);
      state = {...state, ...fetched};
    } catch (_) {
      // A failed batch just leaves those cards without a rank (bare trophy); drop
      // them from `_requested` so a later frame can retry.
      _requested.removeAll(batch);
    }
  }
}
