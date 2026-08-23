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

import '../services/connectivity_service.dart';

/// Thrown when connectivity is lost while awaited network work is in flight,
/// or when work is refused pre-flight because the device is already offline
/// (change: add-client-transport-deadlines). Callers map it to their own
/// offline outcome (cached bytes, a dedicated offline message).
class OfflineDuringLoad implements Exception {
  const OfflineDuringLoad();

  @override
  String toString() => 'OfflineDuringLoad';
}

/// Races [work] against the device reporting a transition to offline.
///
/// The device knows it went offline within milliseconds; without this, the app
/// finds out only when the call exhausts its deadline. First to resolve wins:
/// if the offline **transition** wins, an [OfflineDuringLoad] is thrown and the
/// orphaned [work] is left to die on its own deadline in the background (its
/// late result or error is swallowed).
///
/// Two deliberate constraints:
/// - The subscription is owned explicitly and cancelled in a `finally` —
///   [ConnectivityService.onlineStatus] is a broadcast stream, so an
///   unsatisfied listener would leak one subscription per call.
/// - Only the transition **event** counts; the current value is never re-read
///   here, so this cannot double-fire with a caller's pre-flight `isOnline()`
///   check.
Future<T> raceAgainstOffline<T>(
  Future<T> work,
  ConnectivityService connectivity,
) async {
  final completer = Completer<T>();
  final sub = connectivity.onlineStatus.listen(
    (online) {
      if (!online && !completer.isCompleted) {
        completer.completeError(const OfflineDuringLoad());
      }
    },
    // A connectivity stream that cannot deliver (platform plugin missing or
    // failing) is NO signal, not an offline signal — the race degrades to the
    // plain awaited work, which stays bounded by its own deadline (D8: no
    // reading is not a negative reading).
    onError: (Object _, StackTrace _) {},
    cancelOnError: true,
  );
  // Route the work through the completer so a late result (or error) after an
  // offline abort is swallowed instead of surfacing as an unhandled error.
  unawaited(
    work.then(
      (value) {
        if (!completer.isCompleted) completer.complete(value);
      },
      onError: (Object e, StackTrace st) {
        if (!completer.isCompleted) completer.completeError(e, st);
      },
    ),
  );
  try {
    return await completer.future;
  } finally {
    // Not awaited: on a misbehaving platform stream (missing plugin, failing
    // event channel) the cancel future itself can hang or error, and the
    // caller's result must never be held hostage by subscription teardown.
    unawaited(sub.cancel().catchError((_) {}));
  }
}
