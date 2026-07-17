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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/catalog_service.dart';
import 'contributed_scores.dart';
import 'saved_catalog_scores.dart';
import 'score_catalog.dart';

part 'catalog_search_notifier.freezed.dart';
part 'catalog_search_notifier.g.dart';

/// Where the hub sources its results: the public catalog (searchable, savable)
/// or the signed-in user's own uploaded scores (the "mes partitions" filter).
enum CatalogSource { catalog, myUploads }

/// Immutable state of the Score Hub search: the source scope, the text query and
/// the author/difficulty filters, the loaded page of results, the set of
/// catalog ids currently in the user's library, and paging/loading status.
@freezed
abstract class CatalogSearchState with _$CatalogSearchState {
  const CatalogSearchState._();

  const factory CatalogSearchState({
    @Default(CatalogSource.catalog) CatalogSource source,
    @Default('') String query,
    @Default('') String author,
    PracticeLevel? level,
    // Advanced musical-facet filters (change: score-catalog-facets). Each null =
    // no constraint. Applied to the catalog source only (uploads carry no facet
    // data in the app).
    int? maxNoteValue,
    bool? hasChords,
    bool? hasTuplets,
    bool? hasDotted,
    int? maxAmbitusSemitones,
    int? minBpm,
    int? maxBpm,
    @Default(<CatalogEntry>[]) List<CatalogEntry> entries,

    /// Catalog ids the user has saved (drives the per-result saved indicator).
    @Default(<String>{}) Set<String> savedIds,
    @Default(false) bool loading,
    @Default(false) bool loadingMore,
    @Default(true) bool hasMore,
    @Default(0) int nextOffset,
    String? error,
  }) = _CatalogSearchState;

  /// Whether the hub is scoped to the user's own uploads ("mes partitions").
  bool get isMyUploads => source == CatalogSource.myUploads;

  /// Whether the current result list is empty after a completed (non-loading)
  /// query — the signal for the no-results / empty-uploads state.
  bool get isEmptyResult => !loading && entries.isEmpty && error == null;

  /// Whether a saved catalog entry is in the library (add/remove toggle state).
  bool isSaved(CatalogEntry e) =>
      e.catalogId != null && savedIds.contains(e.catalogId);

  /// Whether any advanced facet filter is active.
  bool get hasAdvancedFilters =>
      maxNoteValue != null ||
      hasChords != null ||
      hasTuplets != null ||
      hasDotted != null ||
      maxAmbitusSemitones != null ||
      minBpm != null ||
      maxBpm != null;
}

/// Drives the Score Hub search: a debounced text query, author + difficulty
/// filters, offset paging, and a source toggle between the public catalog and
/// the user's uploads. Depends on the injectable [catalogServiceProvider] and
/// [myContributedScoresProvider] so it is testable without a live backend.
@riverpod
class CatalogSearch extends _$CatalogSearch {
  static const int _pageSize = 20;
  static const Duration _debounce = Duration(milliseconds: 300);

  Timer? _debounceTimer;

  @override
  CatalogSearchState build() {
    ref.onDispose(() => _debounceTimer?.cancel());
    // Kick off the initial browse after build returns (never touch state here).
    Future.microtask(_reload);
    return const CatalogSearchState();
  }

  /// Update the text query and reload (debounced for search-as-you-type).
  void setQuery(String query) {
    state = state.copyWith(query: query);
    _debouncedReload();
  }

  /// Update the author (composer) filter and reload (debounced).
  void setAuthor(String author) {
    state = state.copyWith(author: author);
    _debouncedReload();
  }

  /// Update the difficulty filter and reload immediately.
  void setLevel(PracticeLevel? level) {
    state = state.copyWith(level: level);
    unawaited(_reload());
  }

  /// Switch the source scope (catalog ↔ my uploads) and reload immediately.
  void setSource(CatalogSource source) {
    if (source == state.source) return;
    state = state.copyWith(source: source);
    unawaited(_reload());
  }

  // --- advanced facet filters (change: score-catalog-facets) --------------
  // Applied to the catalog source; each reloads immediately.

  /// Fastest allowed note value (power-of-two denominator), or null for "tout".
  void setMaxNoteValue(int? denominator) {
    state = state.copyWith(maxNoteValue: denominator);
    unawaited(_reload());
  }

  void toggleChords(bool on) {
    state = state.copyWith(hasChords: on ? true : null);
    unawaited(_reload());
  }

  void toggleTuplets(bool on) {
    state = state.copyWith(hasTuplets: on ? true : null);
    unawaited(_reload());
  }

  void toggleDotted(bool on) {
    state = state.copyWith(hasDotted: on ? true : null);
    unawaited(_reload());
  }

  /// Maximum hand span in semitones (e.g. 12 = one octave), or null for "tout".
  void setMaxAmbitusSemitones(int? semitones) {
    state = state.copyWith(maxAmbitusSemitones: semitones);
    unawaited(_reload());
  }

  /// Tempo band as an inclusive BPM range (either bound null = open), or both
  /// null for "tout".
  void setTempoBand(int? minBpm, int? maxBpm) {
    state = state.copyWith(minBpm: minBpm, maxBpm: maxBpm);
    unawaited(_reload());
  }

  /// Clear every advanced facet filter (keeps text/author/level/source).
  void clearAdvancedFilters() {
    state = state.copyWith(
      maxNoteValue: null,
      hasChords: null,
      hasTuplets: null,
      hasDotted: null,
      maxAmbitusSemitones: null,
      minBpm: null,
      maxBpm: null,
    );
    unawaited(_reload());
  }

  /// Reload the current view (e.g. after deleting one of the user's uploads).
  Future<void> refresh() => _reload();

  void _debouncedReload() {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(_debounce, () => unawaited(_reload()));
  }

  /// Load the first page for the current source/query/filters.
  Future<void> _reload() async {
    state = state.copyWith(
      loading: true,
      error: null,
      entries: const [],
      nextOffset: 0,
      hasMore: true,
    );
    try {
      if (state.source == CatalogSource.catalog) {
        final saved = await _loadSavedIds();
        final page = await ref
            .read(catalogServiceProvider)
            .search(
              query: state.query,
              author: state.author.isEmpty ? null : state.author,
              level: state.level,
              // Corpus is piano-only for now: always constrain to piano.
              isPiano: true,
              maxNoteValue: state.maxNoteValue,
              hasChords: state.hasChords,
              hasTuplets: state.hasTuplets,
              hasDotted: state.hasDotted,
              maxAmbitusSemitones: state.maxAmbitusSemitones,
              minBpm: state.minBpm,
              maxBpm: state.maxBpm,
              limit: _pageSize,
              offset: 0,
            );
        state = state.copyWith(
          loading: false,
          savedIds: saved,
          entries: [for (final h in page.hits) catalogEntryFromHit(h)],
          nextOffset: page.nextOffset,
          hasMore: page.hits.length >= _pageSize,
        );
      } else {
        final uploads = await ref.read(myContributedScoresProvider.future);
        state = state.copyWith(
          loading: false,
          entries: uploads.where(_matchesFilters).toList(),
          hasMore: false,
        );
      }
    } catch (e) {
      state = state.copyWith(loading: false, error: e.toString());
    }
  }

  /// Fetch the next page (catalog source only) and append it. No-op while a load
  /// is in flight, when exhausted, or in the (unpaged) my-uploads source.
  Future<void> loadMore() async {
    if (state.source != CatalogSource.catalog) return;
    if (state.loading || state.loadingMore || !state.hasMore) return;
    state = state.copyWith(loadingMore: true);
    try {
      final page = await ref
          .read(catalogServiceProvider)
          .search(
            query: state.query,
            author: state.author.isEmpty ? null : state.author,
            level: state.level,
            isPiano: true,
            maxNoteValue: state.maxNoteValue,
            hasChords: state.hasChords,
            hasTuplets: state.hasTuplets,
            hasDotted: state.hasDotted,
            maxAmbitusSemitones: state.maxAmbitusSemitones,
            minBpm: state.minBpm,
            maxBpm: state.maxBpm,
            limit: _pageSize,
            offset: state.nextOffset,
          );
      state = state.copyWith(
        loadingMore: false,
        entries: [
          ...state.entries,
          for (final h in page.hits) catalogEntryFromHit(h),
        ],
        nextOffset: page.nextOffset,
        hasMore: page.hits.length >= _pageSize,
      );
    } catch (e) {
      state = state.copyWith(loadingMore: false, error: e.toString());
    }
  }

  /// Add or remove a catalog result from the user's library (optimistic), then
  /// persist through the backend and refresh the saved-scores provider.
  Future<void> toggleSave(CatalogEntry entry) async {
    final id = entry.catalogId;
    if (id == null) return;
    final wasSaved = state.savedIds.contains(id);
    final next = {...state.savedIds};
    wasSaved ? next.remove(id) : next.add(id);
    state = state.copyWith(savedIds: next); // optimistic
    try {
      final service = ref.read(catalogServiceProvider);
      wasSaved ? await service.remove(id) : await service.save(id);
      ref.invalidate(savedCatalogScoresProvider);
    } catch (_) {
      // Revert the optimistic toggle on failure.
      final reverted = {...state.savedIds};
      wasSaved ? reverted.add(id) : reverted.remove(id);
      state = state.copyWith(savedIds: reverted);
    }
  }

  /// The user's saved catalog ids (for the per-result saved indicator).
  Future<Set<String>> _loadSavedIds() async {
    final saved = await ref.read(savedCatalogScoresProvider.future);
    return {
      for (final e in saved)
        if (e.catalogId != null) e.catalogId!,
    };
  }

  /// Client-side filter for the my-uploads source (the backend search covers the
  /// catalog; uploads are a small, already-owned set).
  bool _matchesFilters(CatalogEntry e) {
    final q = state.query.trim().toLowerCase();
    final a = state.author.trim().toLowerCase();
    final title = e.title.toLowerCase();
    final composer = e.composer.toLowerCase();
    final queryOk = q.isEmpty || title.contains(q) || composer.contains(q);
    final authorOk = a.isEmpty || composer.contains(a);
    final levelOk = state.level == null || e.level == state.level;
    return queryOk && authorOk && levelOk;
  }
}
