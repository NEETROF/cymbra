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
import 'play_session_envelope.dart';

part 'play_outbox_store.g.dart';

/// The durable play-activity outbox (change: add-play-activity-profile, D1/D7).
///
/// A persisted queue of [PlaySessionEnvelope]s that **survives app restarts** —
/// backed by the injectable [PreferencesService] seam (the same low-stakes,
/// test-fakeable store the session summary uses), so a captured session is safe
/// before any network attempt.
///
/// Invariants: [add] is idempotent by session id (a re-add is a no-op, so the
/// same session is never queued twice), and an entry is removed **only** once its
/// delivery has been acknowledged — no retention rule ever drops an un-acked
/// entry (the "no loss" guarantee).
@Riverpod(keepAlive: true)
PlayOutboxStore playOutboxStore(Ref ref) =>
    PlayOutboxStore(ref.watch(preferencesServiceProvider));

class PlayOutboxStore {
  PlayOutboxStore(this._prefs);

  final PreferencesService _prefs;

  static const String _key = 'playOutbox';

  /// All pending entries, oldest first (delivery order). Unreadable storage
  /// yields an empty list rather than throwing (never blocks capture/drain).
  Future<List<PlaySessionEnvelope>> all() async {
    final raw = await _prefs.getString(_key);
    if (raw == null || raw.isEmpty) return const [];
    try {
      final list = jsonDecode(raw) as List<dynamic>;
      return list
          .map((e) => PlaySessionEnvelope.fromJson(e as Map<String, dynamic>))
          .toList(growable: false);
    } catch (_) {
      return const [];
    }
  }

  /// Append [entry], **idempotently** by session id (a re-add of an id already
  /// queued is a no-op). Capture happens before any network attempt, so this is
  /// the durable point at which a session becomes safe.
  Future<void> add(PlaySessionEnvelope entry) async {
    final entries = List<PlaySessionEnvelope>.from(await all());
    if (entries.any((e) => e.sessionId == entry.sessionId)) return;
    entries.add(entry);
    await _write(entries);
  }

  /// Remove the entry with [sessionId] — called **only** after the server has
  /// acknowledged it. A no-op if absent (already removed).
  Future<void> remove(String sessionId) async {
    final entries = (await all())
        .where((e) => e.sessionId != sessionId)
        .toList(growable: false);
    await _write(entries);
  }

  Future<void> _write(List<PlaySessionEnvelope> entries) async {
    if (entries.isEmpty) {
      await _prefs.remove(_key);
      return;
    }
    await _prefs.setString(
      _key,
      jsonEncode(entries.map((e) => e.toJson()).toList(growable: false)),
    );
  }
}
