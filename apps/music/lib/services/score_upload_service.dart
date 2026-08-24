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
import '../state/score_catalog.dart'
    show PracticeLevel, ScoreInstrument, scoreInstrumentFromWire;
import 'grpc_client.dart';
import 'rpc_deadlines.dart';
import 'score_bytes_result.dart';

export 'score_bytes_result.dart' show ScoreBytesResult;

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

  // Derived musical facets for the generated cover (null until known).
  final int? minNoteValue;
  final int? tempoBpm;
  final int noteCount;
  final int? lowestMidi;
  final int? highestMidi;

  /// Whether this upload is in the user's favorites (shown on the home screen).
  final bool favorite;

  /// Public-catalog proposal state (change: add-score-catalog-proposal): `null` when
  /// the score has not been proposed, else `pending` / `accepted` / `rejected`.
  final String? proposalStatus;

  /// The moderator's rejection reason, surfaced to the proposer when
  /// [proposalStatus] is `rejected` (null otherwise).
  final String? rejectionReason;

  /// The upload's stored instrument family (change: add-drums-access); `null`
  /// when recorded as `unknown` (no indication is shown).
  final ScoreInstrument? instrument;

  const ContributedScore({
    required this.id,
    required this.level,
    required this.createdAt,
    required this.measureCount,
    required this.timeSig,
    required this.keyFifths,
    this.title,
    this.composer,
    this.minNoteValue,
    this.tempoBpm,
    this.noteCount = 0,
    this.lowestMidi,
    this.highestMidi,
    this.favorite = true,
    this.proposalStatus,
    this.rejectionReason,
    this.instrument,
  });

  /// Whether this contribution has been proposed to the public catalog.
  bool get isProposed => proposalStatus != null;
}

/// Seam over the backend `ScoreService` — the app's contribution surface. Every
/// call is bearer-authenticated; the production impl refreshes transparently on
/// `UNAUTHENTICATED`. Tests override the provider with an in-memory fake.
/// Failures throw `AuthException` (see auth_service.dart).
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

  /// Propose one of the caller's private scores to the public catalog (change:
  /// add-score-catalog-proposal). Requires a licence declaration + right-to-distribute
  /// [attestation]. [resubmissionNote] is required only when re-proposing a score whose
  /// prior proposal was rejected (it reopens that catalog entry). Throws on a server
  /// refusal (duplicate/already-proposed/missing attestation) so the caller can localise.
  Future<void> propose({
    required String scoreId,
    required String license,
    required bool attestation,
    String attribution = '',
    String? resubmissionNote,
  });

  /// Delete one of the caller's contributed scores.
  Future<void> deleteScore(String id);

  /// Favorite / un-favorite one of the caller's uploads (home visibility).
  /// Un-favoriting never deletes the upload.
  Future<void> setFavorite(String id, bool favorite);

  /// Fetch a contributed score's bytes to open it in the player — the
  /// backend-backed byte source paralleling [ScoreAssetSource] (task 6.3). When
  /// [ifNoneMatch] still matches the stored content hash the backend returns
  /// `unchanged` with no payload (change: add-offline-score-cache).
  Future<ScoreBytesResult> fetchScoreBytes(String id, {String? ifNoneMatch});
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
    required AuthedRunner authed,
    RpcDeadlines deadlines = const RpcDeadlines(),
  }) : _client = score.ScoreServiceClient(channel, interceptors: [deadlines]),
       _authed = authed;

  final score.ScoreServiceClient _client;
  final AuthedRunner _authed;

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
    minNoteValue: r.hasMinNoteValue() ? r.minNoteValue : null,
    tempoBpm: r.hasTempoBpm() ? r.tempoBpm : null,
    noteCount: r.noteCount,
    lowestMidi: r.hasLowestMidi() ? r.lowestMidi : null,
    highestMidi: r.hasHighestMidi() ? r.highestMidi : null,
    favorite: r.favorite,
    proposalStatus: r.hasProposalStatus() ? r.proposalStatus : null,
    rejectionReason: r.hasRejectionReason() ? r.rejectionReason : null,
    instrument: scoreInstrumentFromWire(
      r.hasInstrument() ? r.instrument : null,
    ),
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
  Future<void> propose({
    required String scoreId,
    required String license,
    required bool attestation,
    String attribution = '',
    String? resubmissionNote,
  }) => _authed<void>((bearer) async {
    await _client.proposeScore(
      score.ProposeScoreRequest(
        scoreId: scoreId,
        license: license,
        rightsAck: attestation,
        attribution: attribution.isEmpty ? null : attribution,
        resubmissionNote: (resubmissionNote == null || resubmissionNote.isEmpty)
            ? null
            : resubmissionNote,
      ),
      options: bearerOptions(bearer),
    );
  });

  @override
  Future<void> deleteScore(String id) => _authed<void>((bearer) async {
    await _client.deleteScore(
      score.DeleteScoreRequest(id: id),
      options: bearerOptions(bearer),
    );
  });

  @override
  Future<void> setFavorite(String id, bool favorite) =>
      _authed<void>((bearer) async {
        await _client.setScoreFavorite(
          score.SetScoreFavoriteRequest(id: id, favorite: favorite),
          options: bearerOptions(bearer),
        );
      });

  @override
  Future<ScoreBytesResult> fetchScoreBytes(String id, {String? ifNoneMatch}) =>
      _authed((bearer) async {
        final resp = await _client.getScoreBytes(
          score.GetScoreBytesRequest(id: id, ifNoneMatch: ifNoneMatch),
          options: bearerOptions(bearer),
        );
        return ScoreBytesResult(
          data: resp.unchanged ? null : Uint8List.fromList(resp.data),
          etag: resp.etag,
          unchanged: resp.unchanged,
        );
      });
}

/// Production score-upload-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
ScoreUploadService scoreUploadService(Ref ref) => GrpcScoreUploadService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
  deadlines: ref.watch(rpcDeadlinesProvider),
);
