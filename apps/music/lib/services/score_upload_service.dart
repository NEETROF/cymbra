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
import 'token_store.dart';

part 'score_upload_service.g.dart';

/// The basis on which a user may contribute a score (design 2b). The wire form
/// matches the backend's `rights_basis` check.
enum RightsBasis { author, publicDomain }

extension RightsBasisWire on RightsBasis {
  String get wire => switch (this) {
    RightsBasis.author => 'own_work',
    RightsBasis.publicDomain => 'public_domain',
  };
}

/// A score the signed-in user has contributed. All descriptive metadata is
/// **server-derived** (design 2b) — the app never sends it.
class ContributedScore {
  final String id;
  final String? title;
  final String? composer;
  final PracticeLevel level;
  final DateTime createdAt;
  final int measureCount;
  final String timeSig;
  final int keyFifths;

  const ContributedScore({
    required this.id,
    required this.level,
    required this.createdAt,
    required this.measureCount,
    required this.timeSig,
    required this.keyFifths,
    this.title,
    this.composer,
  });
}

/// Seam over the backend `ScoreService` — the app's contribution surface. Every
/// call is bearer-authenticated; the production impl refreshes transparently on
/// `UNAUTHENTICATED`. Tests override the provider with an in-memory fake.
/// Failures throw [AuthException] (see auth_service.dart).
abstract class ScoreUploadService {
  /// Upload a contribution. The bytes, chosen [level], and rights attestation are
  /// sent; [fallbackTitle]/[fallbackComposer] are used by the server ONLY when the
  /// file itself carries none (a parsed value always wins). The returned record
  /// carries the effective server-side metadata.
  Future<ContributedScore> upload({
    required Uint8List data,
    required String filename,
    required PracticeLevel level,
    required RightsBasis rightsBasis,
    required bool rightsAck,
    String? fallbackTitle,
    String? fallbackComposer,
  });

  /// The caller's own contributed scores, newest first.
  Future<List<ContributedScore>> listMyScores();

  /// Delete one of the caller's contributed scores.
  Future<void> deleteScore(String id);

  /// Fetch a contributed score's bytes to open it in the player — the
  /// backend-backed byte source paralleling [ScoreAssetSource] (task 6.3).
  Future<Uint8List> fetchBytes(String id);
}

/// Maps a backend level string to [PracticeLevel]; defaults to beginner.
PracticeLevel practiceLevelFromWire(String s) => switch (s) {
  'intermediate' => PracticeLevel.intermediate,
  'advanced' => PracticeLevel.advanced,
  _ => PracticeLevel.beginner,
};

/// Production [ScoreUploadService] over the generated `ScoreServiceClient`.
/// Protected calls run through [authedCall] so a stale access token is refreshed
/// once and the call retried transparently (mirrors [GrpcAccountService]).
class GrpcScoreUploadService implements ScoreUploadService {
  GrpcScoreUploadService({
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

  ContributedScore _toScore(score.ScoreRecord r) => ContributedScore(
    id: r.id,
    title: r.hasTitle() ? r.title : null,
    composer: r.hasComposer() ? r.composer : null,
    level: practiceLevelFromWire(r.level),
    createdAt: DateTime.fromMillisecondsSinceEpoch(
      r.createdAt.toInt() * 1000,
      isUtc: true,
    ),
    measureCount: r.measureCount,
    timeSig: r.timeSig,
    keyFifths: r.keyFifths,
  );

  @override
  Future<ContributedScore> upload({
    required Uint8List data,
    required String filename,
    required PracticeLevel level,
    required RightsBasis rightsBasis,
    required bool rightsAck,
    String? fallbackTitle,
    String? fallbackComposer,
  }) => _authed(
    (bearer) async => _toScore(
      await _client.uploadScore(
        score.UploadScoreRequest(
          data: data,
          filename: filename,
          level: level.name,
          rightsBasis: rightsBasis.wire,
          rightsAck: rightsAck,
          fallbackTitle: fallbackTitle,
          fallbackComposer: fallbackComposer,
        ),
        options: bearerOptions(bearer),
      ),
    ),
  );

  @override
  Future<List<ContributedScore>> listMyScores() => _authed((bearer) async {
    final resp = await _client.listMyScores(
      score.ListMyScoresRequest(),
      options: bearerOptions(bearer),
    );
    return resp.scores.map(_toScore).toList();
  });

  @override
  Future<void> deleteScore(String id) => _authed<void>((bearer) async {
    await _client.deleteScore(
      score.DeleteScoreRequest(id: id),
      options: bearerOptions(bearer),
    );
  });

  @override
  Future<Uint8List> fetchBytes(String id) => _authed((bearer) async {
    final resp = await _client.getScoreBytes(
      score.GetScoreBytesRequest(id: id),
      options: bearerOptions(bearer),
    );
    return Uint8List.fromList(resp.data);
  });
}

/// Production score-upload-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
ScoreUploadService scoreUploadService(Ref ref) => GrpcScoreUploadService(
  channel: ref.watch(cymbraChannelProvider),
  tokenStore: ref.watch(tokenStoreProvider),
  authService: ref.watch(authServiceProvider),
);
