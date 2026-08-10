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
import 'score_catalog.dart';

part 'favorites_index_store.g.dart';

/// A last-known-good snapshot of the authenticated user's favorites index
/// (change: add-offline-score-cache, design D8), so the home renders offline.
///
/// Stores **metadata only** — id, kind (catalog/contributed), title, composer,
/// level — and **never score bytes**. Because it carries no licensed bytes it is
/// deliberately kept in **plaintext** local storage via the [PreferencesService]
/// seam, **decoupled from the secure keystore**: a user must never lose their
/// favorites list offline just because the platform has no usable keystore. Scoped
/// by `userId`; a guest / signed-out session has no snapshot.
@Riverpod(keepAlive: true)
FavoritesIndexStore favoritesIndexStore(Ref ref) =>
    FavoritesIndexStore(ref.watch(preferencesServiceProvider));

class FavoritesIndexStore {
  FavoritesIndexStore(this._prefs);

  final PreferencesService _prefs;

  String _key(String userId) => 'favorites-index:$userId';

  /// The saved snapshot for [userId], or an empty list when absent/unreadable
  /// (never throws — a corrupt snapshot degrades to "no offline favorites").
  Future<List<CatalogEntry>> read(String userId) async {
    final raw = await _prefs.getString(_key(userId));
    if (raw == null || raw.isEmpty) return const [];
    try {
      final list = jsonDecode(raw) as List<dynamic>;
      return list
          .map((e) => _fromJson(e as Map<String, dynamic>))
          .toList(growable: false);
    } catch (_) {
      return const [];
    }
  }

  /// Persist [entries] as the snapshot for [userId] (metadata only). An empty list
  /// clears the key.
  Future<void> write(String userId, List<CatalogEntry> entries) async {
    if (entries.isEmpty) {
      await _prefs.remove(_key(userId));
      return;
    }
    await _prefs.setString(
      _key(userId),
      jsonEncode(entries.map(_toJson).toList(growable: false)),
    );
  }

  /// Drop the snapshot for [userId] (sign-out / account deletion).
  Future<void> clear(String userId) => _prefs.remove(_key(userId));

  Map<String, dynamic> _toJson(CatalogEntry e) => {
    'id': e.id,
    'title': e.title,
    'composer': e.composer,
    'level': e.level.name,
    if (e.catalogId != null) 'catalogId': e.catalogId,
    if (e.contributedId != null) 'contributedId': e.contributedId,
    if (e.uploaderHandle != null) 'uploaderHandle': e.uploaderHandle,
  };

  CatalogEntry _fromJson(Map<String, dynamic> j) => CatalogEntry(
    id: j['id'] as String,
    title: (j['title'] as String?) ?? '',
    composer: (j['composer'] as String?) ?? '',
    level: PracticeLevel.values.firstWhere(
      (l) => l.name == j['level'],
      orElse: () => PracticeLevel.beginner,
    ),
    catalogId: j['catalogId'] as String?,
    contributedId: j['contributedId'] as String?,
    uploaderHandle: j['uploaderHandle'] as String?,
  );
}
