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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../analytics/usage_actions.dart';
import '../services/auth_service.dart';
import '../services/offline_score_cache.dart';
import '../services/score_upload_service.dart';
import 'score_catalog.dart';
import 'score_upload_notifier.dart';
import 'session_notifier.dart';
import 'usage_tracking_notifier.dart';

part 'contributed_scores.g.dart';

/// Outcome of a public-catalog proposal, as the UI needs to phrase it (change:
/// add-score-catalog-proposal). Deliberately typed rather than an exception or a
/// raw status: the listener maps it to a localized message, never to a technical
/// string, and the refusals the server can return are each their own case.
enum ScoreProposalOutcome {
  /// Accepted for review — the contribution now shows `pending`.
  submitted,

  /// Refused: this content is already in the catalog (`ALREADY_EXISTS`) — either
  /// the same score was proposed before, or someone proposed identical bytes.
  alreadyInCatalog,

  /// Any other refusal (missing attestation/justification, network, …).
  failed,
}

/// The last proposal's outcome, or `null` when there is nothing to report.
///
/// A dedicated one-shot channel so a **refusal never poisons the uploads list**:
/// a refused proposal changes nothing server-side, so [MyUploads] keeps its data
/// and reports here instead. The library listener widget watches this, shows the
/// localized message and clears it.
@riverpod
class ScoreProposalFeedback extends _$ScoreProposalFeedback {
  @override
  ScoreProposalOutcome? build() => null;

  void report(ScoreProposalOutcome outcome) => state = outcome;

  /// Consume the outcome so the same message is not shown twice.
  void clear() => state = null;
}

/// The signed-in user's raw uploads (all of them, favorite or not). Empty when
/// signed out.
///
/// Owns the upload mutations (favorite toggle, delete) so widgets never call the
/// upload service directly — they call this notifier, which reloads itself
/// (`AsyncValue.guard` keeps failures in the state, never thrown). It also
/// *listens* for a completed upload and refreshes itself, rather than the uploader
/// reaching across to invalidate it (architecture rules: no sibling invalidation,
/// UI never calls services, actions surface via state not awaited returns).
@riverpod
class MyUploads extends _$MyUploads {
  @override
  Future<List<ContributedScore>> build() {
    // The dependent listens to what it depends on: a new successful upload
    // (`result` transitions non-null) refreshes the list.
    ref.listen(scoreUploadNotifierProvider.select((s) => s.result), (
      prev,
      next,
    ) {
      if (next != null && next != prev) ref.invalidateSelf();
    });
    // `watch` so sign-in/out rebuilds the list; mutations reload via `_fetch`.
    if (!ref.watch(canUseOnlineServicesProvider)) {
      return Future.value(const <ContributedScore>[]);
    }
    return ref.read(scoreUploadServiceProvider).listMyScores();
  }

  Future<List<ContributedScore>> _fetch() {
    if (!ref.read(canUseOnlineServicesProvider)) {
      return Future.value(const <ContributedScore>[]);
    }
    return ref.read(scoreUploadServiceProvider).listMyScores();
  }

  /// Re-fetch the caller's uploads, keeping the current list visible while the fetch
  /// is in flight (no loading flicker). Used to pick up server-side changes the app
  /// can't observe locally — e.g. a moderator changing a contribution's proposal
  /// status in the back office (change: add-score-catalog-proposal). A failure lands
  /// in the state (surfaced by the listener), never thrown.
  Future<void> refresh() async {
    state = await AsyncValue.guard(_fetch);
  }

  /// Favorite / un-favorite one of the caller's uploads, then reload. A failure
  /// lands in the state (surfaced by a listener), never thrown to the caller.
  Future<void> toggleFavorite(
    String contributedId, {
    required bool favorite,
  }) async {
    state = await AsyncValue.guard(() async {
      await ref
          .read(scoreUploadServiceProvider)
          .setFavorite(contributedId, favorite);
      unawaited(
        ref
            .read(usageTrackingNotifierProvider.notifier)
            .record(
              favorite ? UsageActions.favoriteAdd : UsageActions.favoriteRemove,
              subjectId: contributedId,
            ),
      );
      // Un-favoriting removes it from the home → drop the offline cache file
      // (change: add-offline-score-cache). Favoriting keeps any existing copy.
      if (!favorite) {
        await ref
            .read(offlineScoreCacheProvider)
            .evict('contributed:$contributedId');
      }
      return _fetch();
    });
  }

  /// Delete one of the caller's uploads (destructive), then reload. Also evicts
  /// its offline cache file (change: add-offline-score-cache).
  Future<void> delete(String contributedId) async {
    state = await AsyncValue.guard(() async {
      await ref.read(scoreUploadServiceProvider).deleteScore(contributedId);
      await ref
          .read(offlineScoreCacheProvider)
          .evict('contributed:$contributedId');
      return _fetch();
    });
  }

  /// Propose one of the caller's uploads to the public catalog (change: add-score-
  /// catalog-proposal), then reload so the new `pending` proposal status is reflected.
  /// [resubmissionNote] is required only when re-proposing a rejected score.
  ///
  /// Never thrown to the caller — widgets fire this and react to state. A **refusal**
  /// (duplicate content, already proposed, missing attestation) leaves the server
  /// untouched, so it is reported through [ScoreProposalFeedback] and the uploads list
  /// is left exactly as it was: turning the list into an `AsyncError` would empty
  /// "mes partitions" over a message that only concerns one card.
  Future<void> proposeToPublicCatalog(
    String contributedId, {
    required String license,
    required bool attestation,
    String attribution = '',
    String? resubmissionNote,
  }) async {
    final feedback = ref.read(scoreProposalFeedbackProvider.notifier);
    try {
      await ref
          .read(scoreUploadServiceProvider)
          .propose(
            scoreId: contributedId,
            license: license,
            attestation: attestation,
            attribution: attribution,
            resubmissionNote: resubmissionNote,
          );
    } on AuthException catch (e) {
      feedback.report(
        e.error == AuthError.alreadyExists
            ? ScoreProposalOutcome.alreadyInCatalog
            : ScoreProposalOutcome.failed,
      );
      return;
    } catch (_) {
      feedback.report(ScoreProposalOutcome.failed);
      return;
    }
    unawaited(
      ref
          .read(usageTrackingNotifierProvider.notifier)
          .record(UsageActions.scorePropose, subjectId: contributedId),
    );
    feedback.report(ScoreProposalOutcome.submitted);
    // Reload so the card picks up its `pending` tag. A reload failure is a plain
    // list failure (the proposal itself went through) — it belongs in the state.
    state = await AsyncValue.guard(_fetch);
  }
}

/// The signed-in user's contributed scores, as [CatalogEntry]s (byte-sourced from
/// the backend) so they slot into the same player path as bundled scores. Used by
/// the hub's "mes partitions" (all uploads). Empty when signed out.
@riverpod
Future<List<CatalogEntry>> myContributedScores(Ref ref) async {
  final scores = await ref.watch(myUploadsProvider.future);
  final handle = ref.watch(currentUserHandleProvider);
  return [for (final s in scores) contributedEntry(s, uploaderHandle: handle)];
}

/// Maps an upload to a [CatalogEntry] (byte-sourced, facets carried through).
/// [uploaderHandle] drives the "{handle} · Cymbra" attribution on the card.
CatalogEntry contributedEntry(ContributedScore s, {String? uploaderHandle}) {
  final hasTitle = s.title != null && s.title!.isNotEmpty;
  final hasComposer = s.composer != null && s.composer!.isNotEmpty;
  var composer = '';
  if (hasComposer) {
    composer = s.composer!;
  } else if (!hasTitle) {
    composer = _shortDate(s.createdAt);
  }

  return CatalogEntry(
    id: 'contrib-${s.id}',
    title: hasTitle ? s.title! : 'Sans titre',
    // Fall back to the upload date so multiple untitled uploads stay
    // distinguishable in the list (option A).
    composer: composer,
    level: s.level,
    contributedId: s.id,
    // Facets so the contributed cover is as faithful as a catalog one.
    minNoteValue: s.minNoteValue,
    tempoBpm: s.tempoBpm,
    noteCount: s.noteCount,
    lowestMidi: s.lowestMidi,
    highestMidi: s.highestMidi,
    timeSig: s.timeSig,
    keyFifths: s.keyFifths,
    favorite: s.favorite,
    uploaderHandle: uploaderHandle,
    proposalStatus: s.proposalStatus,
    proposalRejectionReason: s.rejectionReason,
  );
}

String _shortDate(DateTime d) =>
    '${d.day.toString().padLeft(2, '0')}/${d.month.toString().padLeft(2, '0')}/${d.year}';
