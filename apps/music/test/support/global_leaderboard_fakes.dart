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

import 'package:music/services/global_leaderboard_service.dart';
import 'package:music/state/global_leaderboard.dart';
import 'package:music/state/leaderboard.dart';

/// A hand-written [GlobalLeaderboardService] fake (change: add-global-
/// leaderboard). A hand fake rather than the default mockito mock because the
/// screen tests drive it as a tiny **stateful stub**: the board returned depends
/// on the requested `(mode, season)` and the calls are counted, which is exactly
/// the "records interactions / behaves like a fixture" case the testing skill
/// keeps for fakes.
class FakeGlobalLeaderboardService implements GlobalLeaderboardService {
  FakeGlobalLeaderboardService({
    Map<String, GlobalLeaderboard>? boards,
    GlobalSeasons seasons = GlobalSeasons.empty,
    this.fail = false,
  }) : _boards = boards ?? const {},
       _seasons = seasons;

  /// Boards keyed by `"<mode>|<seasonId>"` (season id empty = current season).
  final Map<String, GlobalLeaderboard> _boards;
  final GlobalSeasons _seasons;

  /// When true every read throws — drives the error state.
  final bool fail;

  /// The `(mode, seasonId)` pairs asked for, in order.
  final List<(LeaderboardMode, String)> requests = [];

  static String key(LeaderboardMode mode, String seasonId) =>
      '${mode.wire}|$seasonId';

  @override
  Future<GlobalLeaderboard> getGlobalLeaderboard({
    required LeaderboardMode mode,
    String? seasonId,
    int offset = 0,
    int limit = 50,
  }) async {
    final season = seasonId ?? '';
    requests.add((mode, season));
    if (fail) throw Exception('boom');
    return _boards[key(mode, season)] ?? GlobalLeaderboard.empty;
  }

  @override
  Future<GlobalSeasons> listSeasons() async {
    if (fail) throw Exception('boom');
    return _seasons;
  }
}

/// A ranked entry with sensible defaults, so a test states only what it asserts.
GlobalLeaderboardEntry globalEntry({
  required int rank,
  required String userId,
  String? handle,
  double score = 10,
  int contributingPieces = 3,
  int reachedAtMs = 1,
}) => GlobalLeaderboardEntry(
  rank: rank,
  userId: userId,
  handle: handle ?? '@$userId',
  displayName: null,
  score: score,
  contributingPieces: contributingPieces,
  reachedAtMs: reachedAtMs,
);
