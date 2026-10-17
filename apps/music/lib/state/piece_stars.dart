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
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'my_standings_notifier.dart';
import 'piece_bests.dart';
import 'piece_stars_core.dart';
import 'session_notifier.dart';

part 'piece_stars.g.dart';

/// The stars a piece shows (change: add-music-piece-stars): from the best
/// overall synchronisation the server keeps for a catalog piece (the standings
/// read the score cards already make), merged with the device's own record —
/// which is all a bundled piece has, and what covers the moments before a run
/// is delivered. Null signed out: a guest's cards show no stars.
@riverpod
int? pieceStars(Ref ref, {required String pieceId, String? catalogId}) {
  if (!ref.watch(canUseOnlineServicesProvider)) return null;
  final local = ref.watch(localPieceBestsProvider.select((m) => m[pieceId]));
  double? server;
  if (catalogId != null) {
    ref.read(myStandingsProvider.notifier).request(catalogId);
    server = ref.watch(
      myStandingsProvider.select((m) => m[catalogId]?.bestOverallSyncPct),
    );
  }
  return starsFor(_best(local, server));
}

double? _best(double? a, double? b) {
  if (a == null) return b;
  if (b == null) return a;
  return a >= b ? a : b;
}
