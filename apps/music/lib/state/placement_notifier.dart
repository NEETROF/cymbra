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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/course_catalog_service.dart';
import '../services/preferences_service.dart';
import 'placement_core.dart';
import 'rating_activity_notifier.dart' show nowFnProvider;

part 'placement_notifier.freezed.dart';
part 'placement_notifier.g.dart';

/// Where the path starts for this device (change: add-music-placement-test,
/// D3/D4): a pointer, never a completion. Device-local by design.
@freezed
sealed class PlacementState with _$PlacementState {
  const PlacementState._();

  const factory PlacementState({
    /// The starting unit's slug, or null for the first unit.
    String? startUnit,

    /// When the placement was last taken; null when never. Clearing the start
    /// keeps it: the offer is for a device that never took the placement.
    DateTime? takenAt,

    /// False until the persisted pointer was read.
    @Default(false) bool loaded,
  }) = _PlacementState;

  bool get taken => takenAt != null;
}

const String _kKey = 'placement.v1';

/// Owns the starting unit: restores it, replaces it on a retake, clears it on
/// "start from the beginning". Persisted through the preferences seam.
@Riverpod(keepAlive: true)
class Placement extends _$Placement {
  @override
  PlacementState build() {
    _restore();
    return const PlacementState();
  }

  Future<void> _restore() async {
    String? start;
    DateTime? taken;
    try {
      final raw = await ref.read(preferencesServiceProvider).getString(_kKey);
      if (raw != null) {
        final decoded = jsonDecode(raw);
        if (decoded is Map) {
          if (decoded['startUnit'] case final String s when s.isNotEmpty) {
            start = s;
          }
          if (decoded['takenAtMs'] case final int ms) {
            taken = DateTime.fromMillisecondsSinceEpoch(ms);
          }
        }
      }
    } catch (_) {}
    // A pointer set before the restore finished wins over the stored one.
    state = PlacementState(
      startUnit: state.taken ? state.startUnit : start,
      takenAt: state.takenAt ?? taken,
      loaded: true,
    );
  }

  /// The placement was taken (or retaken) and confirmed on [unit].
  Future<void> set(String unit) => _write(
    PlacementState(
      startUnit: unit,
      takenAt: ref.read(nowFnProvider)(),
      loaded: true,
    ),
  );

  /// Start from the beginning: no unit skipped. The placement stays "taken".
  Future<void> clear() => _write(
    PlacementState(
      startUnit: null,
      takenAt: state.takenAt ?? ref.read(nowFnProvider)(),
      loaded: true,
    ),
  );

  Future<void> _write(PlacementState next) async {
    state = next;
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(
            _kKey,
            jsonEncode({
              'startUnit': next.startUnit,
              'takenAtMs': next.takenAt?.millisecondsSinceEpoch,
            }),
          );
    } catch (_) {}
  }
}

/// "Later" on the offer card: hidden for this app session only (D5 — the
/// offer returns until a lesson is completed or the placement taken).
@Riverpod(keepAlive: true)
class PlacementOfferDismissed extends _$PlacementOfferDismissed {
  @override
  bool build() => false;

  void dismiss() => state = true;
}

/// The path's units skipped by placement, from the listing and the pointer.
@riverpod
Set<String> skippedUnits(Ref ref) {
  final listings = ref.watch(coursesProvider).valueOrNull ?? const [];
  final start = ref.watch(placementProvider.select((p) => p.startUnit));
  return skippedUnitsOf(unitOrder(listings), start);
}
