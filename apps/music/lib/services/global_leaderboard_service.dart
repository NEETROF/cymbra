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
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/grpc/global_leaderboard.pbgrpc.dart' as gl;
import '../state/global_leaderboard.dart';
import '../state/leaderboard.dart';
import 'grpc_client.dart';

part 'global_leaderboard_service.g.dart';

/// Seam for reading the GLOBAL, seasonal leaderboards (change:
/// add-global-leaderboard). Behind a provider so the Community screen and the
/// profile standing are testable with a fake — no native library, no live backend.
abstract class GlobalLeaderboardService {
  /// Read one global board `(mode, season)`: the ranked public entries (page)
  /// plus the caller's own standing. A null [seasonId] means the current season.
  /// The public/eligible listing gate + own-rank are enforced server-side.
  Future<GlobalLeaderboard> getGlobalLeaderboard({
    required LeaderboardMode mode,
    String? seasonId,
    int offset = 0,
    int limit = 50,
  });

  /// The seasons a selector may offer: the live one plus the snapshotted past
  /// ones (most recent first).
  Future<GlobalSeasons> listSeasons();
}

/// Production [GlobalLeaderboardService] over the generated
/// `GlobalLeaderboardServiceClient`. Protected calls run through [authedCall] so a
/// stale access token is refreshed once and retried transparently (mirrors the
/// per-piece leaderboard adapter).
class GrpcGlobalLeaderboardService implements GlobalLeaderboardService {
  GrpcGlobalLeaderboardService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = gl.GlobalLeaderboardServiceClient(channel),
       _authed = authed;

  final gl.GlobalLeaderboardServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<GlobalLeaderboard> getGlobalLeaderboard({
    required LeaderboardMode mode,
    String? seasonId,
    int offset = 0,
    int limit = 50,
  }) => _authed((bearer) async {
    final resp = await _client.getGlobalLeaderboard(
      gl.GetGlobalLeaderboardRequest(
        mode: mode.wire,
        // An empty season id means "the current season" on the wire (proto3 has
        // no null); the server echoes back which season it read.
        seasonId: seasonId ?? '',
        offset: offset,
        limit: limit,
      ),
      options: bearerOptions(bearer),
    );
    return GlobalLeaderboard(
      seasonId: resp.seasonId,
      entries: resp.entries.map(_toEntry).toList(growable: false),
      total: resp.total,
      // `own` is an optional message: its presence is the "has own standing" flag.
      own: resp.hasOwn() ? _toEntry(resp.own) : null,
    );
  });

  @override
  Future<GlobalSeasons> listSeasons() => _authed((bearer) async {
    final resp = await _client.listGlobalSeasons(
      gl.ListGlobalSeasonsRequest(),
      options: bearerOptions(bearer),
    );
    return GlobalSeasons(
      currentSeasonId: resp.currentSeasonId,
      pastSeasonIds: resp.pastSeasonIds.toList(growable: false),
    );
  });

  GlobalLeaderboardEntry _toEntry(
    gl.GlobalLeaderboardEntry e,
  ) => GlobalLeaderboardEntry(
    rank: e.rank,
    userId: e.userId,
    // The wire uses empty strings for "absent" (proto3 has no null); map back.
    handle: e.handle.isEmpty ? null : e.handle,
    displayName: e.displayName.isEmpty ? null : e.displayName,
    score: e.score,
    contributingPieces: e.contributingPieces,
    reachedAtMs: e.reachedAtMs.toInt(),
  );
}

/// Production global-leaderboard-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
GlobalLeaderboardService globalLeaderboardService(Ref ref) =>
    GrpcGlobalLeaderboardService(
      channel: ref.watch(cymbraChannelProvider),
      authed: ref.watch(authedRunnerProvider),
    );
