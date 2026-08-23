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

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../analytics/usage_actions.dart';
import '../services/auth_service.dart';
import '../services/catalog_service.dart';
import '../services/connectivity_service.dart';
import '../services/notation_engine.dart';
import '../services/offline_score_cache.dart';
import '../services/score_asset_source.dart';
import '../services/score_upload_service.dart';
import 'catalog_daily_access_notifier.dart';
import 'notation_data.dart';
import 'offline_race.dart';
import 'plan_notifier.dart';
import 'saved_catalog_scores.dart';
import 'score_catalog.dart';
import 'usage_tracking_notifier.dart';

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

  /// Whether this in-flight load no longer belongs to the current selection.
  /// Post-`await` continuations must consult THIS, never a bare
  /// `ref.read(selectedScoreProvider)`: when the selection changed (e.g. the
  /// user cancelled the open, which clears it), riverpod flags the element as
  /// dependency-outdated and a plain `ref.read` asserts in the window before
  /// the rebuild (change: add-client-transport-deadlines). An unusable ref
  /// means the selection changed, which means this load is abandoned.
  bool _abandoned(CatalogEntry entry) {
    try {
      return ref.read(selectedScoreProvider) != entry;
    } catch (_) {
      return true;
    }
  }

  ScoreAssetSource get _source => ref.read(scoreAssetSourceProvider);
  NotationEngine get _engine => ref.read(notationEngineProvider);

  Future<void> _load(CatalogEntry entry) async {
    final width = state.availableWidth > 0
        ? state.availableWidth
        : _initialWidth;
    final cacheKey = offlineCacheKeyFor(entry);
    // Captured BEFORE any await: after one, a selection change marks this
    // element dependency-outdated and a bare `ref.read` asserts — observed
    // live as an unhandled error when the read happened while evaluating the
    // race's arguments, so the orphaned work future had no listener yet.
    final connectivity = ref.read(connectivityServiceProvider);
    try {
      // Bytes come from a byte-sourced score (a user upload or a saved public-
      // catalog score), preferring a valid local encrypted copy so a favorited-
      // and-once-opened score plays offline; or from the asset bundle (a bundled
      // score). Same parse → layout path either way.
      final Uint8List bytes;
      CachedScore? cached;
      // Whether to run the post-render best-effort refresh (uploads only — a
      // catalog piece served from cache online has already been decided by the
      // server below, which doubles as its refresh).
      var refreshAfter = false;
      if (cacheKey != null) {
        final cache = ref.read(offlineScoreCacheProvider);
        cached = await cache.read(cacheKey);
        if (cached != null && entry.catalogId != null) {
          // A cached CATALOG piece (change: add-score-daily-access-rewards,
          // design D5): ONLINE the server decides first — the conditional fetch
          // is cheap (`unchanged` skips storage) and doubles as the access
          // decision; a locked answer never plays the cached copy (kept: access
          // is per-day). OFFLINE (or unreachable) the cached copy plays — the
          // documented offline grace.
          Uint8List? decided;
          try {
            decided = await raceAgainstOffline(
              _decideCachedCatalogOpen(entry, cached, connectivity),
              connectivity,
            );
          } on OfflineDuringLoad {
            // Connectivity dropped mid-decide: the offline grace applies — the
            // copy was legitimately obtained. The orphaned fetch dies on its
            // own deadline in the background.
            decided = cached.bytes;
          }
          if (decided == null) {
            // Locked — the failure is typed; the numbers went to the daily-access
            // provider. Nothing is played.
            if (_abandoned(entry)) return;
            state = NotationData(
              failure: ScoreLoadFailure.locked,
              availableWidth: width,
            );
            return;
          }
          bytes = decided;
        } else if (cached != null) {
          // A cached upload: the cache hit is authoritative and plays with no
          // network round-trip; refresh best-effort afterwards.
          bytes = cached.bytes;
          refreshAfter = true;
        } else {
          // Miss: the network fetch is the source of truth; store its bytes AND
          // its content hash (ETag) when the entry is a favorite, so a later open
          // can do a conditional refresh instead of re-downloading.
          //
          // Offline it cannot succeed (change: add-client-transport-deadlines):
          // pre-flight, don't even open a socket — but only on a POSITIVE
          // "offline" report. A captive portal still reads "online" (the call
          // then proceeds and fails on its deadline), and a plugin that cannot
          // answer must not gate the call at all. Mid-flight, the race aborts
          // at once on the connectivity transition instead of waiting out the
          // deadline.
          if (await connectivity.isDefinitelyOffline()) {
            throw const OfflineDuringLoad();
          }
          final fetched = await raceAgainstOffline(
            _fetchScoreBytes(entry),
            connectivity,
          );
          if (_abandoned(entry)) return;
          _reportAccess(entry, fetched);
          if (fetched.locked) {
            if (_abandoned(entry)) return;
            state = NotationData(
              failure: ScoreLoadFailure.locked,
              availableWidth: width,
            );
            return;
          }
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
      if (_abandoned(entry)) return;
      final systems = _engine.layout(document, width);
      state = NotationData(
        document: document,
        systems: systems,
        availableWidth: width,
      );
      // After serving an upload from cache, best-effort refresh from the network
      // — see [_refreshCachedEntry]. Never blocks or replaces the rendered view.
      if (refreshAfter && cached != null && cacheKey != null) {
        await _refreshCachedEntry(entry, cacheKey, cached.etag);
      }
    } catch (e) {
      if (_abandoned(entry)) return;
      // Keep the technical cause in the logs only — the UI shows a localized
      // message keyed off the typed [ScoreLoadFailure], never the raw
      // exception/gRPC text.
      debugPrint('Notation load failed for ${entry.id}: $e');
      final failure = await _classifyLoad(entry, cacheKey, e);
      if (_abandoned(entry)) return;
      state = NotationData(failure: failure, availableWidth: width);
    }
  }

  /// Decide a cached CATALOG open (design D5). Returns the bytes to play — the
  /// cached copy (offline / unreachable / server said unchanged) or the fresh
  /// server bytes (content changed; the cache is rewritten when favourited) — or
  /// `null` when the server refused the piece (locked).
  Future<Uint8List?> _decideCachedCatalogOpen(
    CatalogEntry entry,
    CachedScore cached,
    ConnectivityService connectivity,
  ) async {
    if (!await connectivity.isOnline()) {
      return cached.bytes;
    }
    final ScoreBytesResult fetched;
    try {
      fetched = await _fetchScoreBytes(
        entry,
        ifNoneMatch: cached.etag.isEmpty ? null : cached.etag,
      );
    } catch (e) {
      // Online per the connectivity probe but the server is unreachable: the
      // offline grace applies (the copy was legitimately obtained).
      debugPrint('Notation access check failed for ${entry.id}: $e');
      return cached.bytes;
    }
    // Abandoned mid-decide (selection changed): the caller discards the
    // result anyway, and any further ref use would assert — serve the copy.
    if (_abandoned(entry)) return cached.bytes;
    _reportAccess(entry, fetched);
    if (fetched.locked) return null;
    final data = fetched.data;
    if (fetched.unchanged || data == null) return cached.bytes;
    if (await _isFavorite(entry)) {
      final key = offlineCacheKeyFor(entry);
      if (key != null) {
        await ref
            .read(offlineScoreCacheProvider)
            .write(key, data, etag: fetched.etag);
      }
    }
    return data;
  }

  /// Push the access state a catalog fetch returned to the daily-access
  /// provider (the chip and the unlock sheet read it there), and record the
  /// quota-reached usage event when the piece was refused.
  void _reportAccess(CatalogEntry entry, ScoreBytesResult fetched) {
    final access = fetched.access;
    if (access == null) return;
    // Best-effort: after a cancel mid-load the ref may be dependency-outdated
    // (see [_abandoned]); an abandoned load has nothing to report.
    if (_abandoned(entry)) return;
    ref.read(catalogDailyAccessProvider.notifier).report(access);
    if (access.locked) {
      unawaited(
        ref
            .read(usageTrackingNotifierProvider.notifier)
            .record(
              UsageActions.catalogQuotaReached,
              subjectId: entry.catalogId,
            ),
      );
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
      if (_abandoned(entry)) return;
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
  /// saved-library list AND the plan allows caching catalog scores offline
  /// (spec `offline-score-cache`: a premium unlock once the plan system is on;
  /// own uploads are never gated — change: add-premium-subscription).
  Future<bool> _isFavorite(CatalogEntry entry) async {
    if (entry.contributedId != null) return entry.favorite;
    if (entry.catalogId != null) {
      if (!ref.read(catalogOfflineCacheAllowedProvider)) return false;
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
    // Aborted by the offline pre-flight or the mid-flight race: there is no
    // cached copy on this path, so the dedicated "not available offline"
    // message is the honest outcome.
    if (e is OfflineDuringLoad) return ScoreLoadFailure.offlineUnavailable;
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
