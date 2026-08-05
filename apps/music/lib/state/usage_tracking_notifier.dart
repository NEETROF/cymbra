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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:uuid/uuid.dart';

import '../analytics/usage_environment.dart';
import '../analytics/usage_event_record.dart';
import '../services/connectivity_service.dart';
import '../services/usage_tracking_service.dart';
import 'app_locale.dart';
import 'session_notifier.dart';
import 'usage_consent.dart';
import 'usage_outbox_store.dart';

part 'usage_tracking_notifier.g.dart';

const _uuid = Uuid();

/// The collection kill-switch flag key (mirrors the backend registry
/// `analytics.collection.enabled`, default ON). Off ⇒ no client emits events.
const String kAnalyticsCollectionFlag = 'analytics.collection.enabled';

/// The remote collection kill-switch, as a bool the tracker reads. Defaults to
/// **on** and — deliberately — has NO dependency on the feature-flag client here,
/// so building it (from an instrumented call site under a test harness) never
/// spins up the flag notifier / shared_preferences. `main.dart` overrides it to
/// read the real `analytics.collection.enabled` flag; tests get the safe default
/// (or override this provider directly).
@riverpod
bool usageCollectionKillSwitch(Ref ref) => true;

/// How often the buffer is flushed on a best-effort basis. `null` disables the
/// periodic timer (tests override it so no real timer fires mid-test).
@Riverpod(keepAlive: true)
Duration? usageFlushInterval(Ref ref) => const Duration(minutes: 2);

/// The background flush loop (change: add-feature-usage-analytics). Deliberately
/// SEPARATE from [UsageTrackingNotifier] so the tracker itself stays free of native
/// plugins (connectivity) and periodic timers — instrumented notifiers build the
/// tracker on every tracked action, and it must be cheap + test-safe. This
/// scheduler is warmed ONLY at app launch (`main.dart`), so a widget/unit test that
/// merely exercises an instrumented method never spins up a real timer or plugin.
///
/// It flushes on a periodic tick and on regained connectivity; the tracker itself
/// also flushes on (re)authentication, on launch, and right after each recorded
/// event, so no delivery signal is lost.
@Riverpod(keepAlive: true)
class UsageFlushScheduler extends _$UsageFlushScheduler {
  @override
  void build() {
    // Regained connectivity → flush (native plugin; kept out of the tracker).
    final sub = ref.watch(connectivityServiceProvider).onOnline.listen((_) {
      unawaited(ref.read(usageTrackingNotifierProvider.notifier).flush());
    });
    ref.onDispose(sub.cancel);

    final interval = ref.watch(usageFlushIntervalProvider);
    if (interval != null) {
      final timer = Timer.periodic(interval, (_) {
        unawaited(ref.read(usageTrackingNotifierProvider.notifier).flush());
      });
      ref.onDispose(timer.cancel);
    }
  }
}

/// First-party feature-usage tracking (change: add-feature-usage-analytics, tasks
/// 6.1–6.3, design D5).
///
/// [record] stamps an event with the on-device environment and appends it to the
/// durable [UsageOutboxStore], then triggers a best-effort flush. Emission is
/// gated on BOTH the user's [UsageConsent] and the remote collection kill-switch
/// (and requires an authenticated account — ingestion is authenticated; the guest
/// slice is deferred, design D9). The buffer is flushed periodically and on
/// regained connectivity / (re)authentication / launch; a failed flush is silent
/// and retried — no data loss, no user-facing error. Opting out clears the buffer.
///
/// Only notifiers call [record] — never widgets directly (the UI-never-calls-a-
/// service rule): a feature notifier fires the action; nothing awaits it.
@Riverpod(keepAlive: true)
class UsageTrackingNotifier extends _$UsageTrackingNotifier {
  String _appVersion = '';
  // Serializes flushes so concurrent triggers never overlap.
  Future<void> _chain = Future<void>.value();
  // Set on dispose so the post-build warm-up microtask (which runs later, possibly
  // after a short-lived test container is torn down) never reads a disposed ref.
  bool _disposed = false;

  UsageOutboxStore get _store => ref.read(usageOutboxStoreProvider);
  UsageTrackingService get _service => ref.read(usageTrackingServiceProvider);

  @override
  void build() {
    ref.onDispose(() => _disposed = true);

    // Resume flushing on (re)authentication. Cheap + plugin-free, so building the
    // tracker from an instrumented call site stays test-safe (the periodic /
    // connectivity triggers live in the separately-warmed UsageFlushScheduler).
    ref.listen(canUseOnlineServicesProvider, (_, online) {
      if (online) unawaited(flush());
    });

    // Opting out stops emission AND drops anything already buffered.
    ref.listen(usageConsentProvider, (_, consent) {
      if (!consent) unawaited(_store.clear());
    });

    // After build returns: warm the app version + flush anything left from a
    // previous run (never touch state synchronously in build). Guarded so a
    // missing native plugin (tests) never escapes, and skipped entirely if the
    // container was disposed before the microtask ran.
    Future.microtask(() async {
      if (_disposed) return;
      try {
        _appVersion = await ref.read(appInfoServiceProvider).version();
      } catch (_) {}
      if (_disposed) return;
      await flush();
    });
  }

  /// True when collection may emit: there is an authenticated account to attribute
  /// events to, the user consents, and the remote kill-switch is on. Auth is
  /// checked first (the cheap, plugin-free gate) so an unauthenticated caller never
  /// touches the flag/consent stores — the common case, incl. most test harnesses.
  bool get _enabled {
    if (!ref.read(canUseOnlineServicesProvider)) return false;
    if (!ref.read(usageConsentProvider)) return false;
    return ref.read(usageCollectionKillSwitchProvider);
  }

  /// Record a taxonomy [action] (optionally with a low-cardinality [variant] and a
  /// high-cardinality [subjectId], e.g. a score id). A no-op — silently — when
  /// collection is disabled. Fire-and-forget: callers never await a result.
  Future<void> record(
    String action, {
    String? variant,
    String? subjectId,
  }) async {
    if (!_enabled) return;
    // Telemetry must NEVER break a call site: swallow any failure (local storage
    // unavailable, etc.) — a lost event is acceptable; a thrown one is not.
    try {
      final event = UsageEventRecord(
        id: _uuid.v7(),
        action: action,
        variant: variant,
        subjectId: subjectId,
        platform: usagePlatform(),
        deviceClass: usageDeviceClass(),
        appVersion: _appVersion.isEmpty ? 'unknown' : _appVersion,
        locale: ref.read(appLocaleProvider).languageCode,
        occurredAtMs: DateTime.now().millisecondsSinceEpoch,
      );
      await _store.add(event);
      // Attempt delivery now; callers fire-and-forget (they wrap `record` in
      // `unawaited`), so awaiting the best-effort flush here never blocks the UI.
      await flush();
    } catch (_) {}
  }

  /// Best-effort delivery of the buffered events. Serialized so concurrent
  /// triggers never overlap. A failure keeps the buffer for the next trigger.
  Future<void> flush() {
    final next = _chain.then((_) => _flushOnce());
    _chain = next.catchError((_) {});
    return next;
  }

  Future<void> _flushOnce() async {
    if (_disposed) return;
    // Only send while authenticated + online; otherwise events wait, buffered.
    if (!ref.read(canUseOnlineServicesProvider)) return;
    try {
      final batch = await _store.all();
      if (batch.isEmpty) return;
      await _service.report(batch);
      // Delivered → drop exactly this batch (events added meanwhile are kept).
      await _store.removeIds(batch.map((e) => e.id).toSet());
    } catch (_) {
      // Silent: keep the buffer, retry on the next trigger (no user-facing error;
      // also covers a test harness with no local-storage plugin).
    }
  }
}
