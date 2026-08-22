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

import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/grpc/play.pbgrpc.dart' as play;
import '../state/play_activity.dart';
import '../state/play_session_envelope.dart';
import 'grpc_client.dart';
import 'rpc_deadlines.dart';

part 'play_sync_service.g.dart';

/// Seam for reliable play-stat delivery + the per-day activity read (change:
/// add-play-activity-profile). Behind a provider so the outbox sender and the
/// heatmap are testable without the backend.
abstract class PlaySyncService {
  /// Deliver one captured session. Returns normally **only** on the server's
  /// persisted-ack (the signal to drop the outbox entry); throws on any failure
  /// (offline, unauthenticated, server error) so the entry is retried. Idempotent
  /// server-side by the session id, so a re-delivery is safe.
  ///
  /// Returns the **points the session earned** (change: add-play-rewards), which
  /// the ack carries back so the summary can show its "+N" without a second read.
  /// 0 when nothing was awarded — below the quality floor, this piece already paid
  /// out, the daily cap reached, or a re-delivery of a session that already paid.
  Future<int> recordSession(PlaySessionEnvelope envelope);

  /// Deliver one captured **practice** session (change: add-measure-range-
  /// practice, D4) — a scoreless activity record. Same ack/retry contract as
  /// [recordSession]; the server stores it apart from the scored sessions, so a
  /// practice never produces a grade or a leaderboard entry. Returns the points
  /// the practice earned (the once-a-day showing-up award, 0 thereafter).
  Future<int> recordPractice(PlaySessionEnvelope envelope);

  /// Read a user's per-day activity (the heatmap). Honors the target's visibility
  /// server-side (a non-public target is refused there).
  Future<PlayActivity> getPlayActivity(String userId);
}

/// Production [PlaySyncService] over the generated `PlayServiceClient`. Protected
/// calls run through [authedCall] so a stale access token is refreshed once and
/// retried transparently (mirrors the account/auth adapters).
class GrpcPlaySyncService implements PlaySyncService {
  GrpcPlaySyncService({
    required ClientChannel channel,
    required AuthedRunner authed,
    RpcDeadlines deadlines = const RpcDeadlines(),
  }) : _client = play.PlayServiceClient(channel, interceptors: [deadlines]),
       _authed = authed;

  final play.PlayServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<int> recordSession(PlaySessionEnvelope e) => _authed((bearer) async {
    final resp = await _client.recordPlaySession(
      play.RecordPlaySessionRequest(
        sessionId: e.sessionId,
        scoreId: e.scoreId,
        playedAtMs: Int64(e.playedAtMs),
        tzOffsetMinutes: e.tzOffsetMinutes,
        overallSyncPct: e.overallSyncPct,
        sessionResultJson: e.sessionResultJson,
      ),
      options: bearerOptions(bearer),
    );
    return resp.pointsAwarded;
  });

  @override
  Future<int> recordPractice(PlaySessionEnvelope e) => _authed((bearer) async {
    final resp = await _client.recordPractice(
      play.RecordPracticeRequest(
        sessionId: e.sessionId,
        scoreId: e.scoreId,
        practicedAtMs: Int64(e.playedAtMs),
        tzOffsetMinutes: e.tzOffsetMinutes,
      ),
      options: bearerOptions(bearer),
    );
    return resp.pointsAwarded;
  });

  @override
  Future<PlayActivity> getPlayActivity(String userId) =>
      _authed((bearer) async {
        final resp = await _client.getPlayActivity(
          play.GetPlayActivityRequest(userId: userId),
          options: bearerOptions(bearer),
        );
        return PlayActivity(
          days: resp.days
              .map(
                (d) => DayActivity(
                  day: DateTime.parse(d.day),
                  count: d.count,
                  avgSyncPct: d.avgSyncPct,
                  practiceCount: d.practiceCount,
                ),
              )
              .toList(growable: false),
          totalSessions: resp.totalSessions,
          totalPractices: resp.totalPractices,
        );
      });
}

/// Production play-sync-service provider. Override in tests with a fake/mock.
@Riverpod(keepAlive: true)
PlaySyncService playSyncService(Ref ref) => GrpcPlaySyncService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
  deadlines: ref.watch(rpcDeadlinesProvider),
);
