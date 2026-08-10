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
  Future<bool> isOnline();
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

  @override
  Future<bool> isOnline() async {
    try {
      final results = await _connectivity.checkConnectivity();
      return results.any((r) => r != ConnectivityResult.none);
    } catch (_) {
      // If the platform can't answer, assume offline so the byte cache is the
      // source of truth (never a false "online" that would suppress the cache).
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
