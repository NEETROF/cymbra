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

import 'package:flutter/foundation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/auth_service.dart';
import '../services/catalog_service.dart';
import '../services/connectivity_service.dart';
import '../services/notation_engine.dart';
import '../services/offline_score_cache.dart';
import '../services/score_asset_source.dart';
import '../services/score_upload_service.dart';
import 'notation_data.dart';
import 'saved_catalog_scores.dart';
import 'score_catalog.dart';

part 'notation_notifier.g.dart';

/// Loads the selected score's MusicXML asset, parses it via the Rust bridge, and
/// lays it out into systems — exposing the result as immutable [NotationData].
///
/// Re-builds whenever [SelectedScore] changes. Layout is recomputed cheaply (a
/// synchronous bridge call) when the viewport width changes; the document is
/// only re-parsed when the selected score changes.
@riverpod
class Notation extends _$Notation {
  /// Width used for the first layout, before the screen reports its real size.
  static const double _initialWidth = 800;

  /// Minimum width change (px) that triggers a re-layout, to avoid thrashing.
  static const double _relayoutThreshold = 8;

  @override
  NotationData build() {
    final entry = ref.watch(selectedScoreProvider);
    if (entry != null) {
      // `_load` reads `state`, which is not set during build(); defer it to a
      // microtask so it runs after this build returns and sets state when it
      // resolves.
      Future.microtask(() => _load(entry));
    }
    return const NotationData();
  }

  ScoreAssetSource get _source => ref.read(scoreAssetSourceProvider);
  NotationEngine get _engine => ref.read(notationEngineProvider);

  Future<void> _load(CatalogEntry entry) async {
    final width = state.availableWidth > 0
        ? state.availableWidth
        : _initialWidth;
    final cacheKey = offlineCacheKeyFor(entry);
    try {
      // Bytes come from a byte-sourced score (a user upload or a saved public-
      // catalog score), preferring a valid local encrypted copy so a favorited-
      // and-once-opened score plays offline; or from the asset bundle (a bundled
      // score). Same parse → layout path either way.
      final Uint8List bytes;
      CachedScore? cached;
      if (cacheKey != null) {
        final cache = ref.read(offlineScoreCacheProvider);
        cached = await cache.read(cacheKey);
        if (cached != null) {
          // A cache hit is authoritative and plays with no network round-trip.
          bytes = cached.bytes;
        } else {
          // Miss: the network fetch is the source of truth; store its bytes AND
          // its content hash (ETag) when the entry is a favorite, so a later open
          // can do a conditional refresh instead of re-downloading.
          final fetched = await _fetchScoreBytes(entry);
          bytes = fetched.data!;
          if (await _isFavorite(entry)) {
            await cache.write(cacheKey, bytes, etag: fetched.etag);
          }
        }
      } else {
        bytes = await _source.load(entry.assetPath);
      }
      final document = await _engine.parse(bytes);
      // Guard against a selection change while we were loading.
      if (ref.read(selectedScoreProvider) != entry) return;
      final systems = _engine.layout(document, width);
      state = NotationData(
        document: document,
        systems: systems,
        availableWidth: width,
      );
      // After serving from cache, best-effort refresh from the network — see
      // [_refreshCachedEntry]. Never blocks or replaces the rendered view.
      if (cached != null && cacheKey != null) {
        await _refreshCachedEntry(entry, cacheKey, cached.etag);
      }
    } catch (e) {
      if (ref.read(selectedScoreProvider) != entry) return;
      // Keep the technical cause in the logs only — the UI shows a localized
      // message keyed off the typed [ScoreLoadFailure], never the raw
      // exception/gRPC text.
      debugPrint('Notation load failed for ${entry.id}: $e');
      final failure = await _classifyLoad(entry, cacheKey, e);
      if (ref.read(selectedScoreProvider) != entry) return;
      state = NotationData(failure: failure, availableWidth: width);
    }
  }

  /// Fetch a byte-sourced score's bytes (upload or saved catalog), optionally
  /// conditional on [ifNoneMatch] so an unchanged copy comes back without a
  /// payload (change: add-offline-score-cache).
  Future<ScoreBytesResult> _fetchScoreBytes(
    CatalogEntry entry, {
    String? ifNoneMatch,
  }) {
    if (entry.contributedId != null) {
      return ref
          .read(scoreUploadServiceProvider)
          .fetchScoreBytes(entry.contributedId!, ifNoneMatch: ifNoneMatch);
    }
    return ref
        .read(catalogServiceProvider)
        .fetchScoreBytes(entry.catalogId!, ifNoneMatch: ifNoneMatch);
  }

  /// Best-effort freshness/integrity guard after serving a favorite from cache
  /// (design D7): only when online, send the cached ETag and rewrite the local
  /// copy **only** if the server reports a different content hash. Score bytes are
  /// immutable under a stable id, so this normally reports "unchanged" and skips
  /// the re-download and re-encrypt entirely. Never throws to the caller — a
  /// refresh failure leaves the already-rendered cache view untouched.
  Future<void> _refreshCachedEntry(
    CatalogEntry entry,
    String cacheKey,
    String cachedEtag,
  ) async {
    try {
      if (!await ref.read(connectivityServiceProvider).isOnline()) return;
      final fetched = await _fetchScoreBytes(
        entry,
        ifNoneMatch: cachedEtag.isEmpty ? null : cachedEtag,
      );
      final data = fetched.data;
      // Hash matched (or no payload) → the cache is already fresh, keep it.
      if (fetched.unchanged || data == null) return;
      if (await _isFavorite(entry)) {
        await ref
            .read(offlineScoreCacheProvider)
            .write(cacheKey, data, etag: fetched.etag);
      }
    } catch (e) {
      debugPrint('Notation cache refresh failed for ${entry.id}: $e');
    }
  }

  /// Whether [entry] is currently in the user's favorites (the cache-write gate).
  /// Uploads carry their own flag; a catalog entry is a favorite iff it is in the
  /// saved-library list.
  Future<bool> _isFavorite(CatalogEntry entry) async {
    if (entry.contributedId != null) return entry.favorite;
    if (entry.catalogId != null) {
      final saved =
          ref.read(savedCatalogScoresProvider).valueOrNull ?? const [];
      return saved.any((e) => e.catalogId == entry.catalogId);
    }
    return false;
  }

  /// Classify a load failure. A byte-sourced favorite with no cached copy while
  /// the device is offline gets the dedicated "not available offline" message
  /// instead of the generic server-unavailable one.
  Future<ScoreLoadFailure> _classifyLoad(
    CatalogEntry entry,
    String? cacheKey,
    Object e,
  ) async {
    if (cacheKey != null &&
        e is AuthException &&
        e.error == AuthError.unavailable) {
      final online = await ref.read(connectivityServiceProvider).isOnline();
      if (!online) return ScoreLoadFailure.offlineUnavailable;
    }
    return _classify(e);
  }

  /// Maps a load exception to a typed [ScoreLoadFailure] so the UI can show a
  /// specific localized message. Backend failures arrive as an [AuthException]
  /// carrying a gRPC-derived [AuthError].
  static ScoreLoadFailure _classify(Object e) {
    if (e is AuthException) {
      return switch (e.error) {
        AuthError.notFound => ScoreLoadFailure.notFound,
        // The row exists but its bytes aren't ready (not synced yet / gated
        // pending review) — the backend reports this as FAILED_PRECONDITION.
        AuthError.failedPrecondition => ScoreLoadFailure.notAvailableYet,
        AuthError.unavailable => ScoreLoadFailure.unavailable,
        // Per-user catalog access limit hit (change: add-catalog-access-limits).
        AuthError.rateLimited => ScoreLoadFailure.rateLimited,
        _ => ScoreLoadFailure.generic,
      };
    }
    return ScoreLoadFailure.generic;
  }

  /// Updates the viewport width and re-lays-out the cached document. No-op when
  /// the change is below [_relayoutThreshold] or no document is loaded yet.
  void setAvailableWidth(double width) {
    if (width <= 0) return;
    if ((width - state.availableWidth).abs() < _relayoutThreshold) return;
    final document = state.document;
    if (document == null) {
      state = state.copyWith(availableWidth: width);
      return;
    }
    state = state.copyWith(
      availableWidth: width,
      systems: _engine.layout(document, width),
    );
  }
}
