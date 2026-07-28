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
}

/// Production connectivity-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
ConnectivityService connectivityService(Ref ref) => ConnectivityPlusService();
