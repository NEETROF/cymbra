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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'play_reward_cue.freezed.dart';
part 'play_reward_cue.g.dart';

/// What the run the player just finished earned (change: add-play-rewards).
///
/// The amount is not known at session end: the run is captured into the durable
/// outbox first and the server decides what it is worth, so the number arrives on
/// the **ack** — which may land before the summary modal opens or well after it.
/// Holding it here rather than in the summary's own state lets the modal render
/// the "+N" whenever it arrives, and lets it show nothing at all when the ack
/// never comes (offline) instead of a stale number from an earlier run.
@freezed
abstract class PlayRewardCueState with _$PlayRewardCueState {
  const factory PlayRewardCueState({
    /// Points the awaited run earned; `0` means "nothing to show" — a run below
    /// the quality floor, a piece that has already paid out, the daily cap
    /// reached, or simply no ack yet.
    @Default(0) int points,

    /// Increments once per award reported, so a listener fires even for two
    /// consecutive runs that earned the same amount.
    @Default(0) int seq,
  }) = _PlayRewardCueState;
}

/// The transient "this run earned N" cue, published by the play-sync outbox as
/// each ack lands and consumed by the session summary.
///
/// Deliberately its own provider rather than a field on [`PlaySyncNotifier`]:
/// that notifier's state is the pending-entry count, which three leaderboard
/// providers already react to, and reward feedback has nothing to do with how
/// many entries are queued. This mirrors `rewardRevisionProvider` — a producer
/// publishes into a dedicated signal, and consumers react to it; nobody
/// invalidates a sibling (architecture rule 2).
@Riverpod(keepAlive: true)
class PlayRewardCue extends _$PlayRewardCue {
  /// The session whose ack we are waiting for. Only that one may set the cue, so
  /// an older queued entry draining in the background (after a spell offline)
  /// can never attribute its award to the run on screen.
  String? _awaited;

  @override
  PlayRewardCueState build() => const PlayRewardCueState();

  /// Arm for the run just captured, clearing any previous amount — so the
  /// summary that is about to open starts from "nothing yet" rather than the
  /// last run's number.
  void arm(String sessionId) {
    _awaited = sessionId;
    state = state.copyWith(points: 0);
  }

  /// Report what the ack said [sessionId] earned. Ignored unless it is the run
  /// currently armed, and ignored for a zero award (there is nothing to show).
  void report(String sessionId, int points) {
    if (sessionId != _awaited || points <= 0) return;
    state = state.copyWith(points: points, seq: state.seq + 1);
  }
}
