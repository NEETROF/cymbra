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

import 'session_summary.dart';

/// One durable outbox entry: a completed session captured at session end, ready
/// to be delivered to the server (change: add-play-activity-profile).
///
/// Carries a **client-generated UUID v7** [sessionId] — assigned before any
/// network attempt — so the server dedupes retried deliveries (the idempotency
/// key), and the [userId] of the account that produced it so, on a shared device,
/// one user's sessions are never delivered under another's identity.
///
/// Manual JSON (no codegen) to match the [SessionResult]/`Account` style: it is a
/// serialized transport record, not Riverpod state.
class PlaySessionEnvelope {
  const PlaySessionEnvelope({
    required this.sessionId,
    required this.userId,
    required this.scoreId,
    required this.playedAtMs,
    required this.tzOffsetMinutes,
    required this.overallSyncPct,
    required this.sessionResultJson,
  });

  /// Client-generated UUID v7 — the server's idempotency key.
  final String sessionId;

  /// The account that produced this session (per-user delivery).
  final String userId;

  /// The played score's identity (the piece id), or null if unknown.
  final String? scoreId;

  /// Wall-clock epoch (ms) when the session ended.
  final int playedAtMs;

  /// The client's UTC offset (minutes) at [playedAtMs], for local-day bucketing.
  final int tzOffsetMinutes;

  /// The success score (0..100) — the summary the server indexes.
  final double overallSyncPct;

  /// The full immutable session-result record as a JSON string (uploaded as-is
  /// for future replay/leaderboards).
  final String sessionResultJson;

  Map<String, dynamic> toJson() => {
    'sessionId': sessionId,
    'userId': userId,
    if (scoreId != null) 'scoreId': scoreId,
    'playedAtMs': playedAtMs,
    'tzOffsetMinutes': tzOffsetMinutes,
    'overallSyncPct': overallSyncPct,
    'sessionResultJson': sessionResultJson,
  };

  factory PlaySessionEnvelope.fromJson(Map<String, dynamic> json) =>
      PlaySessionEnvelope(
        sessionId: json['sessionId'] as String,
        userId: json['userId'] as String,
        scoreId: json['scoreId'] as String?,
        playedAtMs: (json['playedAtMs'] as num).toInt(),
        tzOffsetMinutes: (json['tzOffsetMinutes'] as num).toInt(),
        overallSyncPct: (json['overallSyncPct'] as num).toDouble(),
        sessionResultJson: json['sessionResultJson'] as String,
      );
}
