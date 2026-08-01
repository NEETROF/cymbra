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

import '../src/grpc/leaderboard.pbgrpc.dart' as lb;
import '../state/leaderboard.dart';
import 'grpc_client.dart';

part 'leaderboard_service.g.dart';

/// Seam for reading a piece's leaderboards (change: add-play-leaderboards).
/// Behind a provider so the views are testable with a fake — no native library,
/// no live backend.
abstract class LeaderboardService {
  /// Read one board `(scoreId, mode)`: the ranked public entries (page) plus the
  /// caller's own standing. The public/eligible listing gate + own-rank are
  /// enforced server-side.
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  });
}

/// Production [LeaderboardService] over the generated `LeaderboardServiceClient`.
/// Protected calls run through [authedCall] so a stale access token is refreshed
/// once and retried transparently (mirrors the play/account adapters).
class GrpcLeaderboardService implements LeaderboardService {
  GrpcLeaderboardService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = lb.LeaderboardServiceClient(channel),
       _authed = authed;

  final lb.LeaderboardServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<Leaderboard> getLeaderboard({
    required String scoreId,
    required LeaderboardMode mode,
    int offset = 0,
    int limit = 50,
  }) => _authed((bearer) async {
    final resp = await _client.getLeaderboard(
      lb.GetLeaderboardRequest(
        scoreId: scoreId,
        mode: mode.wire,
        offset: offset,
        limit: limit,
      ),
      options: bearerOptions(bearer),
    );
    return Leaderboard(
      entries: resp.entries.map(_toEntry).toList(growable: false),
      total: resp.total,
      // `own` is an optional message: its presence is the "has own standing" flag.
      own: resp.hasOwn() ? _toEntry(resp.own) : null,
    );
  });

  LeaderboardEntry _toEntry(lb.LeaderboardEntry e) => LeaderboardEntry(
    rank: e.rank,
    userId: e.userId,
    // The wire uses empty strings for "absent" (proto3 has no null); map back.
    handle: e.handle.isEmpty ? null : e.handle,
    displayName: e.displayName.isEmpty ? null : e.displayName,
    subscore: e.subscore,
    tiebreakMetric: e.tiebreakMetric,
    achievedAtMs: e.achievedAtMs.toInt(),
  );
}

/// Production leaderboard-service provider. Override in tests with a fake/mock.
@Riverpod(keepAlive: true)
LeaderboardService leaderboardService(Ref ref) => GrpcLeaderboardService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
);
