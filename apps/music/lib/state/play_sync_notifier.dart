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
import 'dart:convert';
import 'dart:math';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:uuid/uuid.dart';

import '../services/connectivity_service.dart';
import '../services/play_sync_service.dart';
import 'play_outbox_store.dart';
import 'play_session_envelope.dart';
import 'session_notifier.dart';
import 'session_summary.dart';

part 'play_sync_notifier.g.dart';

const _uuid = Uuid();

/// Exponential backoff (with full jitter) between failed outbox drains (change:
/// add-play-activity-profile). `attempt` starts at 1; the delay doubles each
/// attempt from 1s, capped at 5 minutes, then a uniform jitter in `[0, delay]` is
/// applied so many clients don't retry in lockstep. Pure + injectable RNG so it
/// is unit-testable.
Duration playOutboxBackoff(int attempt, {Random? random}) {
  final rng = random ?? Random();
  const baseMs = 1000;
  const capMs = 5 * 60 * 1000;
  final exp = (baseMs * pow(2, (attempt - 1).clamp(0, 20))).toInt();
  final capped = exp.clamp(baseMs, capMs);
  return Duration(milliseconds: (capped * rng.nextDouble()).toInt());
}

/// Seam for scheduling a delayed retry of a failed drain. Behind a provider so
/// tests can install a no-op scheduler and drive drains deterministically (no
/// real timers firing mid-test).
abstract class PlayRetryScheduler {
  void schedule(Duration delay, void Function() action);
  void cancel();
}

/// Production [PlayRetryScheduler] over a single reschedulable [Timer].
class TimerRetryScheduler implements PlayRetryScheduler {
  Timer? _timer;

  @override
  void schedule(Duration delay, void Function() action) {
    _timer?.cancel();
    _timer = Timer(delay, action);
  }

  @override
  void cancel() => _timer?.cancel();
}

@Riverpod(keepAlive: true)
PlayRetryScheduler playRetryScheduler(Ref ref) {
  final scheduler = TimerRetryScheduler();
  ref.onDispose(scheduler.cancel);
  return scheduler;
}

/// Reliable, no-loss delivery of end-of-session play stats (change: add-play-
/// activity-profile, D1/D2). Captures each session into the durable
/// [PlayOutboxStore] **before** any network attempt, then drains the outbox:
/// each entry is removed **only** on the server's persisted-ack; on failure it is
/// kept and retried with exponential backoff. Delivery is per-user (each entry is
/// sent only while its producing account is the signed-in one) and resumes on
/// **app launch**, **regained connectivity**, and **(re)authentication**.
///
/// The notifier state is the number of pending (un-acked) entries — a small
/// signal the UI can surface ("syncing…") and tests can assert against.
@Riverpod(keepAlive: true)
class PlaySyncNotifier extends _$PlaySyncNotifier {
  int _attempt = 0;
  // Serializes drains so concurrent triggers (launch, capture, connectivity)
  // never overlap and `await drain()` always includes the caller's own work.
  Future<void> _chain = Future<void>.value();

  PlayOutboxStore get _store => ref.read(playOutboxStoreProvider);
  PlaySyncService get _service => ref.read(playSyncServiceProvider);

  @override
  int build() {
    // Resume on (re)authentication: when an online session appears, flush.
    ref.listen(canUseOnlineServicesProvider, (_, online) {
      if (online) unawaited(drain());
    });
    // Resume on regained connectivity.
    final sub = ref.watch(connectivityServiceProvider).onOnline.listen((_) {
      unawaited(drain());
    });
    ref.onDispose(sub.cancel);
    // Resume on app launch — after build returns (never touch state early).
    Future.microtask(() async {
      state = (await _store.all()).length;
      await drain();
    });
    return 0;
  }

  /// Capture a completed session durably (before any network attempt), then try
  /// to deliver. Safe offline — the entry persists and a later drain sends it.
  /// A session produced with no signed-in account is not captured (it could not
  /// be attributed/delivered); the local session summary still records it.
  Future<void> captureSession(SessionResult result) async {
    final userId = ref.read(currentUserIdProvider);
    if (userId == null) return;
    final envelope = PlaySessionEnvelope(
      sessionId: _uuid.v7(),
      userId: userId,
      scoreId: result.pieceId.isEmpty ? null : result.pieceId,
      playedAtMs: result.playedAtMs,
      tzOffsetMinutes: DateTime.fromMillisecondsSinceEpoch(
        result.playedAtMs,
      ).timeZoneOffset.inMinutes,
      overallSyncPct: result.overallSyncPct,
      sessionResultJson: jsonEncode(result.toJson()),
    );
    await _store.add(envelope);
    state = (await _store.all()).length;
    await drain();
  }

  /// Deliver every pending entry for the signed-in user, removing each **only**
  /// on the server's ack. On the first failure the remaining entries are kept and
  /// a backoff retry is scheduled — nothing un-acked is ever dropped. Drains are
  /// serialized, so `await drain()` waits for any in-flight drain then its own.
  Future<void> drain() {
    final next = _chain.then((_) => _drainOnce());
    // Keep the chain alive even if a drain throws unexpectedly.
    _chain = next.catchError((_) {});
    return next;
  }

  Future<void> _drainOnce() async {
    final userId = ref.read(currentUserIdProvider);
    // Not authenticated (or account unresolved): entries wait for sign-in.
    if (userId == null || !ref.read(canUseOnlineServicesProvider)) return;
    try {
      final entries = await _store.all();
      for (final e in entries) {
        // Per-user delivery: never send another account's session.
        if (e.userId != userId) continue;
        try {
          await _service.recordSession(e);
          // Persisted-ack received → safe to drop the entry.
          await _store.remove(e.sessionId);
        } catch (_) {
          // Keep this and every remaining entry; retry with backoff.
          _scheduleRetry();
          return;
        }
      }
      _attempt = 0; // a clean pass resets the backoff
    } finally {
      state = (await _store.all()).length;
    }
  }

  void _scheduleRetry() {
    _attempt++;
    ref
        .read(playRetrySchedulerProvider)
        .schedule(playOutboxBackoff(_attempt), () => unawaited(drain()));
  }
}
