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
import '../services/rating_service.dart';
import 'rating_activity_notifier.dart';
import 'saved_catalog_scores.dart' show catalogEntryFromHit;
import 'score_catalog.dart';

part 'rating_deck_notifier.freezed.dart';
part 'rating_deck_notifier.g.dart';

/// Immutable state of the swipe-rating deck (change: add-app-score-rating): the
/// queue of sourced `accepted` cards, a cursor to the current top card, paging /
/// loading status, and the ids the user has already judged this session (so a
/// re-query never re-shows them — "prioritise un-rated").
@freezed
abstract class RatingDeckState with _$RatingDeckState {
  const RatingDeckState._();

  const factory RatingDeckState({
    /// The sourced cards, front of the deck first. Cards the user has judged are
    /// consumed by advancing [cursor]; new pages append here.
    @Default(<CatalogEntry>[]) List<CatalogEntry> cards,

    /// Index of the current top card into [cards]; advances past judged cards.
    @Default(0) int cursor,

    /// Catalog ids judged (rated or skipped) this session — excluded from later
    /// pages so the deck never repeats a card.
    @Default(<String>{}) Set<String> seenIds,

    /// Catalog ids whose in-card preview has played past the unlock threshold, so
    /// their rating controls are enabled ("listen before rating"). Sticky per
    /// card for the session.
    @Default(<String>{}) Set<String> unlockedIds,
    @Default(false) bool loading,
    @Default(false) bool loadingMore,
    @Default(true) bool hasMore,
    @Default(0) int nextOffset,
    String? error,
  }) = _RatingDeckState;

  /// The card currently on top of the deck, or `null` when the deck is exhausted
  /// or still loading its first page.
  CatalogEntry? get topCard =>
      cursor >= 0 && cursor < cards.length ? cards[cursor] : null;

  /// The next card behind the top one (for a peek/stacked visual), or `null`.
  CatalogEntry? get nextCard =>
      cursor + 1 < cards.length ? cards[cursor + 1] : null;

  /// Whether the top card's rating controls are unlocked — i.e. its preview has
  /// been heard past the threshold. `Skip` is always allowed; verdict/star rating
  /// is gated on this.
  bool get topUnlocked {
    final id = topCard?.catalogId;
    return id != null && unlockedIds.contains(id);
  }

  /// Whether the deck has run out of cards after a completed load — the signal
  /// for the empty/last-card state (spec: "deck empties when all sourced scores
  /// are rated").
  bool get isExhausted =>
      !loading && !loadingMore && !hasMore && topCard == null;
}

/// Drives the swipe-rating deck: sources `accepted` piano scores through the
/// injectable [catalogServiceProvider] (the backend gate guarantees only
/// validated scores are returned) and records verdict/star ratings through the
/// [ratingServiceProvider]. Modeled on [CatalogSearch]; testable without a live
/// backend or the native library via provider overrides.
@riverpod
class RatingDeck extends _$RatingDeck {
  static const int _pageSize = 20;

  /// Re-query when fewer than this many un-judged cards remain ahead of the
  /// cursor, so the top of the deck rarely stalls waiting on the network.
  static const int _prefetchThreshold = 5;

  /// Fraction of a card's preview that must play before its rating unlocks
  /// ("listen before rating" — the user's chosen gate: a share of the piece).
  static const double previewUnlockFraction = 0.25;

  @override
  RatingDeckState build() {
    // Kick off the initial load after build returns (never touch state here).
    Future.microtask(_loadFirstPage);
    return const RatingDeckState();
  }

  /// The piano-only catalog filter (the corpus is piano-only, matching the hub).
  CatalogFilters get _filters => const CatalogFilters(isPiano: true);

  /// Load the first page of accepted cards, replacing any existing deck.
  Future<void> _loadFirstPage() async {
    state = state.copyWith(
      loading: true,
      error: null,
      cards: const [],
      cursor: 0,
      nextOffset: 0,
      hasMore: true,
    );
    try {
      final page = await ref
          .read(catalogServiceProvider)
          .search(filters: _filters, limit: _pageSize, offset: 0);
      final fresh = _newCards(page.hits, const <String>{});
      state = state.copyWith(
        loading: false,
        cards: fresh,
        nextOffset: page.nextOffset,
        hasMore: page.hits.length >= _pageSize,
      );
    } catch (e) {
      state = state.copyWith(loading: false, error: e.toString());
    }
  }

  /// Fetch the next page and append the still-un-judged cards. No-op while a load
  /// is in flight or when the catalog is exhausted.
  Future<void> loadMore() async {
    if (state.loading || state.loadingMore || !state.hasMore) return;
    state = state.copyWith(loadingMore: true, error: null);
    try {
      final page = await ref
          .read(catalogServiceProvider)
          .search(
            filters: _filters,
            limit: _pageSize,
            offset: state.nextOffset,
          );
      state = state.copyWith(
        loadingMore: false,
        cards: [...state.cards, ..._newCards(page.hits, state.seenIds)],
        nextOffset: page.nextOffset,
        hasMore: page.hits.length >= _pageSize,
      );
    } catch (e) {
      state = state.copyWith(loadingMore: false, error: e.toString());
    }
  }

  /// Map hits to entries, dropping any whose catalog id is already sourced or
  /// judged (so the deck never shows a duplicate or an already-rated card).
  List<CatalogEntry> _newCards(
    List<CatalogHit> hits,
    Set<String> alreadyJudged,
  ) {
    final present = {
      for (final c in state.cards)
        if (c.catalogId != null) c.catalogId!,
    };
    return [
      for (final h in hits)
        if (!alreadyJudged.contains(h.id) && !present.contains(h.id))
          catalogEntryFromHit(h),
    ];
  }

  /// Record a swipe/tap-button rating for the top card, then advance. Optimistic:
  /// the deck advances immediately (pattern from `toggleSave`); a persistence
  /// failure reverts the advance and surfaces an error so the card can be
  /// re-rated. `dislike` is a negative verdict and still counts.
  Future<void> rate(RatingVerdict verdict) => _submit(verdict, null);

  /// Record an explicit 1–5 star rating for the top card, deriving the verdict so
  /// swipe and stars fold into one rating (design D2): 5 → love, 3–4 → like,
  /// 1–2 → dislike.
  Future<void> rateStars(int stars) => _submit(_verdictForStars(stars), stars);

  /// Advance past the top card WITHOUT recording any rating (spec: Skip).
  void skip() {
    final card = state.topCard;
    if (card == null) return;
    _advance(card);
  }

  static RatingVerdict _verdictForStars(int stars) {
    if (stars >= 5) return RatingVerdict.love;
    if (stars >= 3) return RatingVerdict.like;
    return RatingVerdict.dislike;
  }

  /// Report how far the top card's in-card preview has played (0..1). Once it
  /// crosses [previewUnlockFraction] the card's rating unlocks (sticky). A no-op
  /// once already unlocked or below the threshold, so it is cheap to call every
  /// preview frame.
  void markPreviewed(String catalogId, double fraction) {
    if (fraction < previewUnlockFraction) return;
    if (state.unlockedIds.contains(catalogId)) return;
    state = state.copyWith(unlockedIds: {...state.unlockedIds, catalogId});
  }

  Future<void> _submit(RatingVerdict verdict, int? stars) async {
    final card = state.topCard;
    final id = card?.catalogId;
    if (card == null || id == null) return;
    // Gated on "listened enough": a verdict/star rating is ignored until the
    // card's preview has played past the unlock threshold (Skip stays allowed).
    if (!state.unlockedIds.contains(id)) return;
    final fromCursor = state.cursor;
    _advance(card); // optimistic
    try {
      await ref
          .read(ratingServiceProvider)
          .submit(catalogId: id, verdict: verdict, stars: stars);
      // Record the activity so the library's "rate some scores" nudge resets.
      unawaited(ref.read(ratingActivityProvider.notifier).markRatedNow());
    } catch (e) {
      // Revert the optimistic advance and drop the card from "seen" so it can be
      // re-rated (mirrors the hub's toggleSave revert).
      final reverted = {...state.seenIds}..remove(id);
      state = state.copyWith(
        cursor: fromCursor,
        seenIds: reverted,
        error: e.toString(),
      );
    }
  }

  /// Advance the cursor past [card], mark it seen, and prefetch when the deck runs
  /// low. Shared by rate/skip so both consume a card identically.
  void _advance(CatalogEntry card) {
    final id = card.catalogId;
    state = state.copyWith(
      cursor: state.cursor + 1,
      seenIds: id == null ? state.seenIds : {...state.seenIds, id},
      error: null,
    );
    final remaining = state.cards.length - state.cursor;
    if (remaining <= _prefetchThreshold && state.hasMore) {
      unawaited(loadMore());
    }
  }

  /// Reload the deck from the first page (e.g. pull-to-refresh or retry after an
  /// error), clearing the judged set so the full accepted corpus is offered again.
  Future<void> refresh() {
    state = state.copyWith(seenIds: const <String>{});
    return _loadFirstPage();
  }
}
