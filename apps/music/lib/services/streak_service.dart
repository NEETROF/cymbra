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
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/grpc/play.pbgrpc.dart' as play;
import 'grpc_client.dart';
import 'rpc_deadlines.dart';

part 'streak_service.freezed.dart';
part 'streak_service.g.dart';

/// The practice-streak standing (change: add-practice-streak). The **server**
/// owns every number here — the app displays them and never computes a streak
/// itself, so multiple devices agree and the counter cannot be faked from the
/// client.
@freezed
abstract class StreakView with _$StreakView {
  const StreakView._();

  const factory StreakView({
    /// Consecutive local days ending at the last play.
    required int current,

    /// The all-time best run — what the streak badges are measured against.
    required int longest,

    /// Whether today is already secured (a play happened on the local day).
    required bool playedToday,

    /// Whether a broken streak can be bought back right now: inside the grace
    /// window AND affordable. The single condition for showing the offer.
    required bool recoverable,

    /// What a freeze costs in points. Reported even when [recoverable] is false
    /// because the balance is short, so the UI can say why rather than hide it.
    required int recoverCost,

    /// The streak a confirmed recovery would restore.
    required int recoverableStreak,
  }) = _StreakView;

  /// An empty standing — the pre-load/never-played state the chip renders muted.
  static const StreakView none = StreakView(
    current: 0,
    longest: 0,
    playedToday: false,
    recoverable: false,
    recoverCost: 0,
    recoverableStreak: 0,
  );

  /// Whether there is a live streak to show at all.
  bool get hasStreak => current > 0;

  /// A live streak with no play yet today — the in-app "don't lose it" cue for
  /// platforms that get no push (design D4).
  bool get atRisk => hasStreak && !playedToday && !recoverable;
}

/// The outcome of a confirmed freeze: the restored standing + the fresh balance.
@freezed
abstract class StreakRecoveryView with _$StreakRecoveryView {
  const factory StreakRecoveryView({
    required StreakView streak,
    required int newBalance,
  }) = _StreakRecoveryView;
}

StreakView _toStreak(play.StreakStanding s) => StreakView(
  current: s.current,
  longest: s.longest,
  playedToday: s.playedToday,
  recoverable: s.recoverable,
  recoverCost: s.recoverCost,
  recoverableStreak: s.recoverableStreak,
);

/// Seam over the backend `PlayService`'s practice-streak surface. Behind a
/// provider so the chip and the recovery flow are testable without a backend.
///
/// Both calls carry the device's **current UTC offset**: that is what "today"
/// means for this caller, and it is the same convention the play ingest buckets
/// by — so the chip and the server can never disagree about whether today is
/// already secured.
abstract class StreakService {
  /// The caller's streak standing as of their current local day.
  Future<StreakView> getStreak();

  /// Spend points to restore a broken streak. Only ever called after the user
  /// confirms; the server refuses an intact streak, one past the grace window,
  /// and an unaffordable one.
  Future<StreakRecoveryView> recover();
}

/// Production [StreakService] over the generated `PlayServiceClient`. Protected
/// calls run through [AuthedRunner] so a stale access token is refreshed once and
/// the call retried transparently (mirrors [GrpcPlaySyncService]).
class GrpcStreakService implements StreakService {
  GrpcStreakService({
    required ClientChannel channel,
    required AuthedRunner authed,
    RpcDeadlines deadlines = const RpcDeadlines(),
  }) : _client = play.PlayServiceClient(channel, interceptors: [deadlines]),
       _authed = authed;

  final play.PlayServiceClient _client;
  final AuthedRunner _authed;

  /// The device's UTC offset right now, in minutes — re-read per call so a
  /// travelling player (or a DST switch) is handled without restarting the app.
  int get _tzOffsetMinutes => DateTime.now().timeZoneOffset.inMinutes;

  @override
  Future<StreakView> getStreak() => _authed((bearer) async {
    final resp = await _client.getStreak(
      play.GetStreakRequest(tzOffsetMinutes: _tzOffsetMinutes),
      options: bearerOptions(bearer),
    );
    return _toStreak(resp.standing);
  });

  @override
  Future<StreakRecoveryView> recover() => _authed((bearer) async {
    final resp = await _client.recoverStreak(
      play.RecoverStreakRequest(tzOffsetMinutes: _tzOffsetMinutes),
      options: bearerOptions(bearer),
    );
    return StreakRecoveryView(
      streak: _toStreak(resp.standing),
      newBalance: resp.newBalance.toInt(),
    );
  });
}

/// Production streak-service provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
StreakService streakService(Ref ref) => GrpcStreakService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
  deadlines: ref.watch(rpcDeadlinesProvider),
);
