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

/// Immutable state of the Score Hub search: the "mes partitions" quick-filter,
/// the text query and the author/difficulty/facet filters, the loaded page of
/// results (the user's uploads plus the public catalog, mixed), the set of
/// catalog ids currently in the library, and paging/loading status.
@freezed
abstract class CatalogSearchState with _$CatalogSearchState {
  const CatalogSearchState._();

  const factory CatalogSearchState({
    /// The "mes partitions" quick-filter: when true, only the user's own uploads
    /// are shown; when false, uploads are mixed in with the public catalog.
    @Default(false) bool myScoresOnly,
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
  bool get isMyUploads => myScoresOnly;

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

  /// Toggle the "mes partitions" quick-filter and reload immediately. When on,
  /// only the user's uploads show; when off, uploads are mixed with the catalog.
  void setMyScoresOnly(bool value) {
    if (value == state.myScoresOnly) return;
    state = state.copyWith(myScoresOnly: value);
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

  /// Load the first page. The user's matching uploads always lead the list; the
  /// public catalog follows unless the "mes partitions" quick-filter is on.
  Future<void> _reload() async {
    state = state.copyWith(
      loading: true,
      error: null,
      entries: const [],
      nextOffset: 0,
      hasMore: true,
    );
    try {
      final uploads = await _matchingUploads();
      if (state.myScoresOnly) {
        state = state.copyWith(
          loading: false,
          entries: uploads,
          hasMore: false,
        );
        return;
      }
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
        entries: [
          ...uploads,
          for (final h in page.hits) catalogEntryFromHit(h),
        ],
        nextOffset: page.nextOffset,
        hasMore: page.hits.length >= _pageSize,
      );
    } catch (e) {
      state = state.copyWith(loading: false, error: e.toString());
    }
  }

  /// The user's uploads that match the current filters (leads the result list).
  Future<List<CatalogEntry>> _matchingUploads() async {
    final uploads = await ref.read(myContributedScoresProvider.future);
    return uploads.where(_matchesFilters).toList();
  }

  /// Fetch the next catalog page and append it. No-op while a load is in flight,
  /// when exhausted, or under the "mes partitions" quick-filter (uploads only).
  Future<void> loadMore() async {
    if (state.myScoresOnly) return;
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

  /// Client-side filter for the user's uploads (the backend search covers the
  /// catalog; uploads are a small set filtered here by the same query/author/
  /// level and the facet filters the entry carries — chords/tuplets/dotted are
  /// not available client-side, so they don't constrain uploads).
  bool _matchesFilters(CatalogEntry e) {
    final q = state.query.trim().toLowerCase();
    final a = state.author.trim().toLowerCase();
    final title = e.title.toLowerCase();
    final composer = e.composer.toLowerCase();
    if (!(q.isEmpty || title.contains(q) || composer.contains(q))) return false;
    if (!(a.isEmpty || composer.contains(a))) return false;
    if (state.level != null && e.level != state.level) return false;
    if (state.maxNoteValue case final max?) {
      if (e.minNoteValue == null || e.minNoteValue! > max) return false;
    }
    if (state.maxAmbitusSemitones case final span?) {
      final lo = e.lowestMidi, hi = e.highestMidi;
      if (lo == null || hi == null || (hi - lo) > span) return false;
    }
    if (state.minBpm != null || state.maxBpm != null) {
      final t = e.tempoBpm;
      if (t == null) return false;
      if (state.minBpm != null && t < state.minBpm!) return false;
      if (state.maxBpm != null && t > state.maxBpm!) return false;
    }
    return true;
  }
}
