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

import '../services/offline_score_cache.dart';
import 'contributed_scores.dart';
import 'favorites_index_store.dart';
import 'saved_catalog_scores.dart';
import 'score_catalog.dart';
import 'session_notifier.dart';

part 'favorite_scores.g.dart';

/// The signed-in user's favorites — the home screen's content: the catalog
/// scores they saved from the hub PLUS their own uploads that are favorited,
/// as one unified list (no distinction by origin). Empty when signed out (the
/// home then shows the bundled demo catalog instead). Invalidated by save/remove
/// (catalog) or upload/favorite-toggle (uploads).
///
/// Offline resilience (change: add-offline-score-cache, design D8): on a
/// successful online fetch the resolved list is persisted as a plaintext,
/// metadata-only snapshot (and orphaned cache files are swept); when the fetch
/// fails (e.g. no network at launch) the app falls back to that last-known-good
/// snapshot so the home still renders and cached favorites stay reachable.
@riverpod
Future<List<CatalogEntry>> favoriteScores(Ref ref) async {
  if (!ref.watch(canUseOnlineServicesProvider)) return const [];
  final userId = ref.watch(currentUserIdProvider);
  final store = ref.watch(favoritesIndexStoreProvider);
  try {
    final uploads = await ref.watch(myUploadsProvider.future);
    final saved = await ref.watch(savedCatalogScoresProvider.future);
    final handle = ref.watch(currentUserHandleProvider);
    final favorites = <CatalogEntry>[
      for (final s in uploads)
        if (s.favorite) contributedEntry(s, uploaderHandle: handle),
      ...saved,
    ];
    // Write-through the last-known-good snapshot and prune orphaned cache files
    // (a favorite removed on another device). Best-effort: storage hiccups never
    // break the home.
    if (userId != null) {
      try {
        await store.write(userId, favorites);
        final keep = {for (final e in favorites) ?offlineCacheKeyFor(e)};
        await ref.read(offlineScoreCacheProvider).sweep(keep);
      } catch (_) {}
    }
    return favorites;
  } catch (_) {
    // Offline / backend unreachable → the last-known-good snapshot so the home
    // renders and cached favorites remain openable. With no snapshot, surface the
    // original failure (nothing to show yet).
    if (userId != null) {
      final snapshot = await store.read(userId);
      if (snapshot.isNotEmpty) return snapshot;
    }
    rethrow;
  }
}

/// The subset of the current favorites whose bytes are cached locally (playable
/// offline), by `CatalogEntry.id` — drives the home's "not available offline"
/// marking (change: add-offline-score-cache). Bundled/uncached favorites are absent.
@riverpod
Future<Set<String>> offlinePlayableIds(Ref ref) async {
  final favorites = await ref.watch(favoriteScoresProvider.future);
  final cache = ref.watch(offlineScoreCacheProvider);
  final playable = <String>{};
  for (final e in favorites) {
    final key = offlineCacheKeyFor(e);
    if (key != null && await cache.has(key)) playable.add(e.id);
  }
  return playable;
}
