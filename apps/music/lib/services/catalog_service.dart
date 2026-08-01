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

import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/grpc/score.pbgrpc.dart' as score;
import '../state/score_catalog.dart' show PracticeLevel;
import 'grpc_client.dart';
import 'score_upload_service.dart' show practiceLevelFromWire;

part 'catalog_service.g.dart';

/// One public-catalog score as surfaced by search or the saved list —
/// attribution-complete (licence + source), without bytes (fetched separately).
class CatalogHit {
  final String id;
  final String? title;
  final String? composer;

  /// Difficulty when the catalog carries one; `null` if unleveled.
  final PracticeLevel? level;
  final String license;
  final String source;

  // Attribution + musical facets for the generated cover (null until backfilled).
  final String? arranger;
  final int? minNoteValue;
  final int? tempoBpm;
  final int? noteCount;
  final int? lowestMidi;
  final int? highestMidi;
  final String timeSig;
  final int keyFifths;

  /// Moderation status (`pending` | `accepted`), populated for deck cards so a
  /// `pending` candidate can be labelled "potential new score" (change:
  /// rate-pending-scores). `null` when the surface doesn't carry it (e.g. hub
  /// search, which only ever returns `accepted`).
  final String? moderationStatus;

  /// Whether this is a `pending` (not-yet-validated) candidate the community is
  /// helping evaluate — drives the deck's "potential new score" framing.
  bool get isPending => moderationStatus == 'pending';

  const CatalogHit({
    required this.id,
    required this.license,
    required this.source,
    this.title,
    this.composer,
    this.level,
    this.arranger,
    this.minNoteValue,
    this.tempoBpm,
    this.noteCount,
    this.lowestMidi,
    this.highestMidi,
    this.timeSig = '',
    this.keyFifths = 0,
    this.moderationStatus,
  });
}

/// One page of catalog search results plus the offset to fetch the next page and
/// the total number of catalog scores matching the query+filters on the server
/// (independent of this page), so the hub can show the full match count.
class CatalogSearchPage {
  final List<CatalogHit> hits;
  final int nextOffset;
  final int total;

  const CatalogSearchPage({
    required this.hits,
    required this.nextOffset,
    required this.total,
  });
}

/// Seam over the backend `ScoreService`'s Score Hub surface — public-catalog
/// search + the per-user saved library. Every call is bearer-authenticated; the
/// production impl refreshes transparently on `UNAUTHENTICATED`. Tests override
/// the provider with an in-memory fake. Failures throw [AuthException].
/// Musical-facet filters for [CatalogService.search]. Each field `null` = no
/// constraint; a set filter excludes rows whose facet is unknown. Bundled into
/// one object so `search` stays within a sane parameter count.
class CatalogFilters {
  const CatalogFilters({
    this.isPiano,
    this.maxNoteValue,
    this.hasChords,
    this.hasTuplets,
    this.hasDotted,
    this.maxAmbitusSemitones,
    this.minBpm,
    this.maxBpm,
  });

  final bool? isPiano;
  final int? maxNoteValue;
  final bool? hasChords;
  final bool? hasTuplets;
  final bool? hasDotted;
  final int? maxAmbitusSemitones;
  final int? minBpm;
  final int? maxBpm;
}

abstract class CatalogService {
  /// Search the public catalog by free-text (title/composer), with optional
  /// [author] (composer) and [level] filters plus musical-facet [filters],
  /// paginated by [limit]/[offset].
  Future<CatalogSearchPage> search({
    String query,
    String? author,
    PracticeLevel? level,
    CatalogFilters filters,
    int limit,
    int offset,
  });

  /// Save a catalog score to the caller's library (idempotent).
  Future<void> save(String catalogId);

  /// Remove a saved catalog score from the caller's library (idempotent no-op).
  Future<void> remove(String catalogId);

  /// The caller's saved catalog scores, newest-saved first.
  Future<List<CatalogHit>> listSaved();

  /// Fetch a catalog score's bytes to open it in the player. Accepted-only for a
  /// normal caller (the moderation gate).
  Future<Uint8List> fetchBytes(String catalogId);

  /// Fetch a catalog score's bytes for the rating deck's **read-only preview**
  /// (change: rate-pending-scores). Unlike [fetchBytes] (player open), this serves
  /// a `pending` candidate too so a rater can hear it before rating; it never opens
  /// the full player and is not a library save.
  Future<Uint8List> ratingPreviewBytes(String catalogId);

  /// Source the swipe-rating deck (change: improve-rating-deck-sourcing): the
  /// caller's **un-rated** `accepted` scores, least-rated first, paginated. A
  /// score already rated by the caller is never returned, so the deck empties
  /// once everything is rated.
  Future<CatalogSearchPage> ratingDeck({int limit, int offset});

  /// The caller's per-user offline-cache secret — created on first request and
  /// stable thereafter (change: add-offline-score-cache). One input to the app's
  /// local offline-cache key derivation; the same value across the user's devices.
  Future<Uint8List> getOfflineCacheKey();
}

/// Wire form of a [PracticeLevel] for the backend's `level` filter.
String? _levelWire(PracticeLevel? level) => level?.name;

/// Production [CatalogService] over the generated `ScoreServiceClient`. Protected
/// calls run through [authedCall] so a stale access token is refreshed once and
/// the call retried transparently (mirrors [GrpcScoreUploadService]).
class GrpcCatalogService implements CatalogService {
  GrpcCatalogService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = score.ScoreServiceClient(channel),
       _authed = authed;

  final score.ScoreServiceClient _client;
  final AuthedRunner _authed;

  CatalogHit _toHit(score.CatalogHit h) => CatalogHit(
    id: h.id,
    title: h.hasTitle() ? h.title : null,
    composer: h.hasComposer() ? h.composer : null,
    level: h.hasLevel() ? practiceLevelFromWire(h.level) : null,
    license: h.license,
    source: h.source,
    arranger: h.hasArranger() ? h.arranger : null,
    minNoteValue: h.hasMinNoteValue() ? h.minNoteValue : null,
    tempoBpm: h.hasTempoBpm() ? h.tempoBpm : null,
    noteCount: h.hasNoteCount() ? h.noteCount : null,
    lowestMidi: h.hasLowestMidi() ? h.lowestMidi : null,
    highestMidi: h.hasHighestMidi() ? h.highestMidi : null,
    timeSig: h.timeSig,
    keyFifths: h.keyFifths,
    moderationStatus: h.hasModerationStatus() ? h.moderationStatus : null,
  );

  @override
  Future<CatalogSearchPage> search({
    String query = '',
    String? author,
    PracticeLevel? level,
    CatalogFilters filters = const CatalogFilters(),
    int limit = 20,
    int offset = 0,
  }) => _authed((bearer) async {
    final resp = await _client.searchCatalog(
      score.SearchCatalogRequest(
        query: query,
        author: author,
        level: _levelWire(level),
        isPiano: filters.isPiano,
        maxNoteValue: filters.maxNoteValue,
        hasChords: filters.hasChords,
        hasTuplets: filters.hasTuplets,
        hasDotted: filters.hasDotted,
        maxAmbitusSemitones: filters.maxAmbitusSemitones,
        minBpm: filters.minBpm,
        maxBpm: filters.maxBpm,
        limit: limit,
        offset: offset,
      ),
      options: bearerOptions(bearer),
    );
    return CatalogSearchPage(
      hits: resp.hits.map(_toHit).toList(),
      nextOffset: resp.nextOffset,
      total: resp.total,
    );
  });

  @override
  Future<void> save(String catalogId) => _authed<void>((bearer) async {
    await _client.saveCatalogScore(
      score.SaveCatalogScoreRequest(catalogId: catalogId),
      options: bearerOptions(bearer),
    );
  });

  @override
  Future<void> remove(String catalogId) => _authed<void>((bearer) async {
    await _client.removeSavedCatalogScore(
      score.RemoveSavedCatalogScoreRequest(catalogId: catalogId),
      options: bearerOptions(bearer),
    );
  });

  @override
  Future<List<CatalogHit>> listSaved() => _authed((bearer) async {
    final resp = await _client.listSavedCatalogScores(
      score.ListSavedCatalogScoresRequest(),
      options: bearerOptions(bearer),
    );
    return resp.hits.map(_toHit).toList();
  });

  @override
  Future<Uint8List> fetchBytes(String catalogId) => _authed((bearer) async {
    final resp = await _client.getCatalogScoreBytes(
      score.GetCatalogScoreBytesRequest(catalogId: catalogId),
      options: bearerOptions(bearer),
    );
    return Uint8List.fromList(resp.data);
  });

  @override
  Future<Uint8List> ratingPreviewBytes(String catalogId) =>
      _authed((bearer) async {
        final resp = await _client.getRatingPreviewBytes(
          score.GetRatingPreviewBytesRequest(catalogId: catalogId),
          options: bearerOptions(bearer),
        );
        return Uint8List.fromList(resp.data);
      });

  @override
  Future<CatalogSearchPage> ratingDeck({int limit = 20, int offset = 0}) =>
      _authed((bearer) async {
        final resp = await _client.listRatingDeck(
          score.ListRatingDeckRequest(limit: limit, offset: offset),
          options: bearerOptions(bearer),
        );
        return CatalogSearchPage(
          hits: resp.hits.map(_toHit).toList(),
          nextOffset: resp.nextOffset,
          total:
              resp.hits.length, // no separate total; the page length suffices
        );
      });

  @override
  Future<Uint8List> getOfflineCacheKey() => _authed((bearer) async {
    final resp = await _client.getOfflineCacheKey(
      score.GetOfflineCacheKeyRequest(),
      options: bearerOptions(bearer),
    );
    return Uint8List.fromList(resp.secret);
  });
}

/// Production catalog-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
CatalogService catalogService(Ref ref) => GrpcCatalogService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
);
