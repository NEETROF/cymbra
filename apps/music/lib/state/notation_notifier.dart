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
    final cacheKey = _cacheKey(entry);
    try {
      // Bytes come from: a byte-sourced score (a user upload or a saved public-
      // catalog score) via the offline-preferred cache path, or the asset bundle
      // (a bundled score). Same parse → layout path either way.
      final Uint8List bytes = cacheKey != null
          ? await _loadByteSourced(entry, cacheKey)
          : await _source.load(entry.assetPath);
      final document = await _engine.parse(bytes);
      // Guard against a selection change while we were loading.
      if (ref.read(selectedScoreProvider) != entry) return;
      final systems = _engine.layout(document, width);
      state = NotationData(
        document: document,
        systems: systems,
        availableWidth: width,
      );
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

  /// Stable cache key for a byte-sourced entry, or `null` for a bundled asset
  /// (bundled scores are already local and public — never cached).
  static String? _cacheKey(CatalogEntry entry) {
    if (entry.contributedId != null) return 'contributed:${entry.contributedId}';
    if (entry.catalogId != null) return 'catalog:${entry.catalogId}';
    return null;
  }

  /// Load a byte-sourced score, preferring a valid local encrypted copy so a
  /// favorited-and-once-opened score plays offline. On a cache miss the network
  /// fetch is the source of truth, and its bytes are cached when the entry is a
  /// favorite (the "opened once while favorited" write). Caching is a no-op when
  /// unavailable (guest / no keystore / offline).
  Future<Uint8List> _loadByteSourced(CatalogEntry entry, String cacheKey) async {
    final cache = ref.read(offlineScoreCacheProvider);
    final cached = await cache.read(cacheKey);
    if (cached != null) {
      // Content is immutable under a stable id, so a cache hit is authoritative;
      // no online round-trip is needed to play it.
      return cached.bytes;
    }
    final bytes = await _fetchBytes(entry);
    if (await _isFavorite(entry)) {
      // The ETag round-trip is a separate optimization; the plaintext-hash
      // integrity check inside the cache guards the entry regardless.
      await cache.write(cacheKey, bytes, etag: '');
    }
    return bytes;
  }

  Future<Uint8List> _fetchBytes(CatalogEntry entry) {
    if (entry.contributedId != null) {
      return ref.read(scoreUploadServiceProvider).fetchBytes(entry.contributedId!);
    }
    return ref.read(catalogServiceProvider).fetchBytes(entry.catalogId!);
  }

  /// Whether [entry] is currently in the user's favorites (the cache-write gate).
  /// Uploads carry their own flag; a catalog entry is a favorite iff it is in the
  /// saved-library list.
  Future<bool> _isFavorite(CatalogEntry entry) async {
    if (entry.contributedId != null) return entry.favorite;
    if (entry.catalogId != null) {
      final saved = ref.read(savedCatalogScoresProvider).valueOrNull ?? const [];
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
