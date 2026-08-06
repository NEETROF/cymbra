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
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import 'player_data.dart';

part 'practice_settings_store.g.dart';

/// The practice settings saved for one score (change: add-measure-range-
/// practice, D7): the measure range, so reopening a piece pre-fills the passage
/// the player was last drilling. A practice run always loops, endlessly, so
/// there is nothing else to remember.
///
/// Manual JSON (no codegen) like the other stored transport records — it is a
/// small serialized preference, not Riverpod state.
class PracticeSettings {
  const PracticeSettings({
    required this.startMeasure,
    required this.endMeasure,
  });

  /// Inclusive 0-based measure bounds of the saved range.
  final int startMeasure;
  final int endMeasure;

  Map<String, dynamic> toJson() => {
    'startMeasure': startMeasure,
    'endMeasure': endMeasure,
  };

  // Entries written before the loop settings were dropped carry extra keys;
  // they are simply ignored, so an old preference still restores its range.
  factory PracticeSettings.fromJson(Map<String, dynamic> json) =>
      PracticeSettings(
        startMeasure: (json['startMeasure'] as num).toInt(),
        endMeasure: (json['endMeasure'] as num).toInt(),
      );

  /// This selection normalized against a piece of [measureCount] measures, or
  /// null when it cannot be salvaged (the piece has no measures, or the saved
  /// range now covers the whole piece — which is a full run, not a practice
  /// selection). A score re-imported with a different measure count therefore
  /// falls back to the whole piece rather than pointing at bars that no longer
  /// exist.
  PracticeSettings? clampedTo(int measureCount) {
    final range = normalizePracticeRange(
      start: startMeasure,
      end: endMeasure,
      measureCount: measureCount,
    );
    if (range == null) return null;
    if (range.start == 0 && range.end == measureCount - 1) return null;
    return PracticeSettings(startMeasure: range.start, endMeasure: range.end);
  }
}

/// Device-local, per-score persistence of the practice settings, over the same
/// injectable [PreferencesService] seam the session summary uses (low-stakes,
/// offline, test-fakeable). Keyed by score id, so each piece remembers its own
/// selection.
@riverpod
PracticeSettingsStore practiceSettingsStore(Ref ref) =>
    PracticeSettingsStore(ref.watch(preferencesServiceProvider));

class PracticeSettingsStore {
  PracticeSettingsStore(this._prefs);

  final PreferencesService _prefs;

  static const String _key = 'practiceSettings';

  /// Serializes the mutations. Every write is a read-modify-write of the single
  /// stored map, and the player fires several in a row without awaiting them
  /// (applying the range then the loop settings), so overlapping them would let
  /// one clobber another. Chaining makes the **last requested** write the last
  /// one to land.
  Future<void> _chain = Future<void>.value();

  Future<void> _enqueue(Future<void> Function() op) {
    final next = _chain.then((_) => op());
    // Keep the chain alive even if one write throws.
    _chain = next.catchError((_) {});
    return next;
  }

  /// All saved settings, keyed by score id. Unreadable storage yields an empty
  /// map rather than throwing (a lost preference must never block playing).
  Future<Map<String, PracticeSettings>> _all() async {
    final raw = await _prefs.getString(_key);
    if (raw == null || raw.isEmpty) return const {};
    try {
      final map = jsonDecode(raw) as Map<String, dynamic>;
      return {
        for (final e in map.entries)
          e.key: PracticeSettings.fromJson(e.value as Map<String, dynamic>),
      };
    } catch (_) {
      return const {};
    }
  }

  /// The settings saved for [scoreId], or null if none (or unreadable).
  Future<PracticeSettings?> load(String scoreId) async =>
      (await _all())[scoreId];

  /// Saves [settings] for [scoreId], replacing any previous selection.
  Future<void> save(String scoreId, PracticeSettings settings) =>
      _enqueue(() async {
        final all = Map<String, PracticeSettings>.from(await _all());
        all[scoreId] = settings;
        await _write(all);
      });

  /// Forgets [scoreId]'s selection (the player went back to a full run).
  Future<void> clear(String scoreId) => _enqueue(() async {
    final all = Map<String, PracticeSettings>.from(await _all());
    if (all.remove(scoreId) == null) return;
    await _write(all);
  });

  Future<void> _write(Map<String, PracticeSettings> all) async {
    if (all.isEmpty) {
      await _prefs.remove(_key);
      return;
    }
    await _prefs.setString(
      _key,
      jsonEncode({for (final e in all.entries) e.key: e.value.toJson()}),
    );
  }
}
