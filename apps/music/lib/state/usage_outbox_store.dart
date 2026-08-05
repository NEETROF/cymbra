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

import '../analytics/usage_event_record.dart';
import '../services/preferences_service.dart';

part 'usage_outbox_store.g.dart';

/// The durable feature-usage event buffer (change: add-feature-usage-analytics,
/// task 6.2). A persisted queue of [UsageEventRecord]s that **survives app
/// restarts**, backed by the injectable [PreferencesService] seam — so events
/// produced offline are retained and delivered on a later flush (no data loss, no
/// user-facing error). A soft cap bounds the buffer if delivery is unavailable for
/// a very long time (telemetry must never grow unbounded on disk).
@Riverpod(keepAlive: true)
UsageOutboxStore usageOutboxStore(Ref ref) =>
    UsageOutboxStore(ref.watch(preferencesServiceProvider));

class UsageOutboxStore {
  UsageOutboxStore(this._prefs);

  final PreferencesService _prefs;

  static const String _key = 'usageOutbox';

  /// Bound on buffered events; oldest are dropped past this (telemetry is
  /// best-effort, so a permanently offline device never fills the disk).
  static const int maxBuffered = 500;

  /// All pending events, oldest first. Unreadable storage yields an empty list
  /// rather than throwing (never blocks recording/flushing).
  Future<List<UsageEventRecord>> all() async {
    final raw = await _prefs.getString(_key);
    if (raw == null || raw.isEmpty) return const [];
    try {
      final list = jsonDecode(raw) as List<dynamic>;
      return list
          .map((e) => UsageEventRecord.fromJson(e as Map<String, dynamic>))
          .toList(growable: false);
    } catch (_) {
      return const [];
    }
  }

  /// Append [event]; drops the oldest entries past [maxBuffered].
  Future<void> add(UsageEventRecord event) async {
    final entries = List<UsageEventRecord>.from(await all())..add(event);
    final trimmed = entries.length > maxBuffered
        ? entries.sublist(entries.length - maxBuffered)
        : entries;
    await _write(trimmed);
  }

  /// Remove the events with these [ids] — called after a successful flush drops
  /// exactly the delivered batch (events added meanwhile are kept).
  Future<void> removeIds(Set<String> ids) async {
    if (ids.isEmpty) return;
    final entries = (await all())
        .where((e) => !ids.contains(e.id))
        .toList(growable: false);
    await _write(entries);
  }

  /// Drop every buffered event (used when the user opts out of collection).
  Future<void> clear() async {
    await _prefs.remove(_key);
  }

  Future<void> _write(List<UsageEventRecord> entries) async {
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
