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

import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'connectivity_service.g.dart';

/// Seam over network reachability (change: add-play-activity-profile). Behind a
/// provider so the outbox sender's "resume on regained connectivity" trigger is
/// testable with a controllable stream (no native plugin).
abstract class ConnectivityService {
  /// Emits an event whenever connectivity is (re)gained (a transition to any
  /// non-`none` transport). The sender re-drains the outbox on each event.
  Stream<void> get onOnline;

  /// Emits the current online-ness (`true`/`false`) on **every** connectivity
  /// transition, in both directions (change: add-offline-score-cache). Lets the
  /// UI react live when Wi-Fi drops or comes back — unlike [onOnline], which fires
  /// on regain only.
  Stream<bool> get onlineStatus;

  /// A point-in-time reachability check (change: add-offline-score-cache): `true`
  /// when any non-`none` transport is present. Used to tell "app is offline" from
  /// "online but the backend failed" when classifying a score-load failure.
  /// Fail-closed: an unanswerable platform reads as offline, so the byte cache
  /// stays the source of truth.
  Future<bool> isOnline();

  /// `true` only when the platform POSITIVELY reports no usable transport
  /// (change: add-client-transport-deadlines). Unlike [isOnline]'s fail-closed
  /// bias — right for preferring a cache — a gate that refuses to even issue a
  /// network call must fire only on a definitive negative: a plugin that cannot
  /// answer is NOT a report of "offline", and treating it as one would convert
  /// a broken plugin into "nothing loads".
  Future<bool> isDefinitelyOffline();
}

/// Production [ConnectivityService] over `connectivity_plus`.
class ConnectivityPlusService implements ConnectivityService {
  ConnectivityPlusService([Connectivity? connectivity])
    : _connectivity = connectivity ?? Connectivity();

  final Connectivity _connectivity;

  @override
  Stream<void> get onOnline => _connectivity.onConnectivityChanged
      .where((results) => results.any((r) => r != ConnectivityResult.none))
      .map((_) {});

  @override
  Stream<bool> get onlineStatus => _connectivity.onConnectivityChanged
      .map((results) => results.any((r) => r != ConnectivityResult.none))
      .distinct();

  /// Bound on the platform probe itself (change:
  /// add-client-transport-deadlines): a reachability check that can hang is
  /// the very disease this change treats, and it sits on the score-open hot
  /// path. Real platforms answer in milliseconds; past this, "no reading"
  /// applies (each caller keeps its own bias).
  static const Duration _probeTimeout = Duration(milliseconds: 500);

  Future<List<ConnectivityResult>> _probe() =>
      _connectivity.checkConnectivity().timeout(_probeTimeout);

  @override
  Future<bool> isOnline() async {
    try {
      final results = await _probe();
      return results.any((r) => r != ConnectivityResult.none);
    } catch (_) {
      // If the platform can't answer, assume offline so the byte cache is the
      // source of truth (never a false "online" that would suppress the cache).
      return false;
    }
  }

  @override
  Future<bool> isDefinitelyOffline() async {
    try {
      final results = await _probe();
      return results.every((r) => r == ConnectivityResult.none);
    } catch (_) {
      // No reading is not a negative reading: never gate a call on it.
      return false;
    }
  }
}

/// Production connectivity-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
ConnectivityService connectivityService(Ref ref) => ConnectivityPlusService();

/// Live "is the device online right now?" for the UI (change:
/// add-offline-score-cache): the current reachability, then every subsequent
/// change so the home re-marks favorites the moment Wi-Fi drops or returns.
/// Callers default to "online" until the first value resolves. Auto-disposed.
@riverpod
Stream<bool> isOnlineNow(Ref ref) async* {
  final connectivity = ref.watch(connectivityServiceProvider);
  yield await connectivity.isOnline();
  yield* connectivity.onlineStatus;
}
