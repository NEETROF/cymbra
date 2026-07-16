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
import 'auth_service.dart';
import 'grpc_client.dart';
import 'score_upload_service.dart' show practiceLevelFromWire;
import 'token_store.dart';

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

  const CatalogHit({
    required this.id,
    required this.license,
    required this.source,
    this.title,
    this.composer,
    this.level,
  });
}

/// One page of catalog search results plus the offset to fetch the next page.
class CatalogSearchPage {
  final List<CatalogHit> hits;
  final int nextOffset;

  const CatalogSearchPage({required this.hits, required this.nextOffset});
}

/// Seam over the backend `ScoreService`'s Score Hub surface — public-catalog
/// search + the per-user saved library. Every call is bearer-authenticated; the
/// production impl refreshes transparently on `UNAUTHENTICATED`. Tests override
/// the provider with an in-memory fake. Failures throw [AuthException].
abstract class CatalogService {
  /// Search the public catalog by free-text (title/composer), with optional
  /// [author] (composer) and [level] filters, paginated by [limit]/[offset].
  Future<CatalogSearchPage> search({
    String query,
    String? author,
    PracticeLevel? level,
    int limit,
    int offset,
  });

  /// Save a catalog score to the caller's library (idempotent).
  Future<void> save(String catalogId);

  /// Remove a saved catalog score from the caller's library (idempotent no-op).
  Future<void> remove(String catalogId);

  /// The caller's saved catalog scores, newest-saved first.
  Future<List<CatalogHit>> listSaved();

  /// Fetch a catalog score's bytes to open it in the player.
  Future<Uint8List> fetchBytes(String catalogId);
}

/// Wire form of a [PracticeLevel] for the backend's `level` filter.
String? _levelWire(PracticeLevel? level) => level?.name;

/// Production [CatalogService] over the generated `ScoreServiceClient`. Protected
/// calls run through [authedCall] so a stale access token is refreshed once and
/// the call retried transparently (mirrors [GrpcScoreUploadService]).
class GrpcCatalogService implements CatalogService {
  GrpcCatalogService({
    required ClientChannel channel,
    required TokenStore tokenStore,
    required AuthService authService,
  }) : _client = score.ScoreServiceClient(channel),
       _tokenStore = tokenStore,
       _authService = authService;

  final score.ScoreServiceClient _client;
  final TokenStore _tokenStore;
  final AuthService _authService;

  Future<String?> _accessToken() async =>
      (await _tokenStore.readTokens())?.accessToken;

  Future<String?> _refreshAccess() async {
    final stored = await _tokenStore.readTokens();
    if (stored == null) return null;
    try {
      final fresh = await _authService.refresh(stored.refreshToken);
      await _tokenStore.writeTokens(fresh.toStored());
      return fresh.accessToken;
    } catch (_) {
      await _tokenStore.clear();
      return null;
    }
  }

  Future<T> _authed<T>(Future<T> Function(String? bearer) call) async {
    try {
      return await authedCall(
        call,
        accessToken: _accessToken,
        refreshAccessToken: _refreshAccess,
        onExpired: () {},
      );
    } on GrpcError catch (e) {
      throw authExceptionFromGrpc(e);
    }
  }

  CatalogHit _toHit(score.CatalogHit h) => CatalogHit(
    id: h.id,
    title: h.hasTitle() ? h.title : null,
    composer: h.hasComposer() ? h.composer : null,
    level: h.hasLevel() ? practiceLevelFromWire(h.level) : null,
    license: h.license,
    source: h.source,
  );

  @override
  Future<CatalogSearchPage> search({
    String query = '',
    String? author,
    PracticeLevel? level,
    int limit = 20,
    int offset = 0,
  }) => _authed((bearer) async {
    final resp = await _client.searchCatalog(
      score.SearchCatalogRequest(
        query: query,
        author: author,
        level: _levelWire(level),
        limit: limit,
        offset: offset,
      ),
      options: bearerOptions(bearer),
    );
    return CatalogSearchPage(
      hits: resp.hits.map(_toHit).toList(),
      nextOffset: resp.nextOffset,
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
}

/// Production catalog-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
CatalogService catalogService(Ref ref) => GrpcCatalogService(
  channel: ref.watch(cymbraChannelProvider),
  tokenStore: ref.watch(tokenStoreProvider),
  authService: ref.watch(authServiceProvider),
);
