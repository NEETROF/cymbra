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
import 'session_summary.dart';

part 'session_summary_store.g.dart';

/// Device-local persistence for the most recent [SessionResult].
///
/// Writes JSON through the injectable [PreferencesService] seam (not secure
/// storage — a score summary is low-stakes) so the last summary survives the
/// modal being closed and can be re-opened. This change persists locally only; a
/// later change adds the server upload that consumes the same JSON shape.
@riverpod
SessionSummaryStore sessionSummaryStore(Ref ref) =>
    SessionSummaryStore(ref.watch(preferencesServiceProvider));

class SessionSummaryStore {
  SessionSummaryStore(this._prefs);

  final PreferencesService _prefs;

  static const String _key = 'lastSessionResult';

  /// Persists [result] as the last session summary (device-local only).
  Future<void> save(SessionResult result) =>
      _prefs.setString(_key, jsonEncode(result.toJson()));

  /// Reads the last persisted summary, or null if none was ever stored (or the
  /// stored value is unreadable).
  Future<SessionResult?> load() async {
    final raw = await _prefs.getString(_key);
    if (raw == null) return null;
    try {
      return SessionResult.fromJson(jsonDecode(raw) as Map<String, dynamic>);
    } catch (_) {
      return null;
    }
  }
}
