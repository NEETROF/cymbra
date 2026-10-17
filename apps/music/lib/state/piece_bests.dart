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

import 'dart:convert';

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';

part 'piece_bests.g.dart';

const String _kKey = 'piece.bests.v1';

/// The device's own record of the best overall synchronisation per piece
/// (change: add-music-piece-stars): what the stars read for a bundled piece,
/// which has no leaderboard, and the "before" a run is compared to the moment
/// it ends — before the server has even seen it. Monotonic, like the server's.
@Riverpod(keepAlive: true)
class LocalPieceBests extends _$LocalPieceBests {
  @override
  Map<String, double> build() {
    _restore();
    return const {};
  }

  Future<void> _restore() async {
    try {
      final raw = await ref.read(preferencesServiceProvider).getString(_kKey);
      if (raw == null) return;
      final decoded = jsonDecode(raw);
      if (decoded is! Map) return;
      final stored = <String, double>{
        for (final e in decoded.entries)
          if (e.key is String && e.value is num)
            e.key as String: (e.value as num).toDouble(),
      };
      // A best recorded before the restore finished wins over the stored one.
      state = {...stored, ...state}..updateAll((id, v) => _max(v, stored[id]));
    } catch (_) {}
  }

  static double _max(double a, double? b) => b == null || a >= b ? a : b;

  /// The best known here for [pieceId], or null when never scored.
  double? bestOf(String pieceId) => state[pieceId];

  /// Records a finished run; the best only rises.
  Future<void> record(String pieceId, double pct) async {
    final cur = state[pieceId];
    if (cur != null && cur >= pct) return;
    state = {...state, pieceId: pct};
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(_kKey, jsonEncode(state));
    } catch (_) {}
  }
}
