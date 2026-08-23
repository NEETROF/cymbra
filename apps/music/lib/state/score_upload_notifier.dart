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

import '../analytics/usage_actions.dart';
import '../services/auth_service.dart' show AuthError, AuthException;
import '../services/file_picker_service.dart';
import '../services/notation_engine.dart';
import '../services/score_upload_service.dart';
import '../src/rust/api/musicxml.dart' show InstrumentKind, ScoreSummary;
import 'drums_access.dart';
import 'score_catalog.dart' show PracticeLevel;
import 'usage_tracking_notifier.dart';

part 'score_upload_notifier.freezed.dart';
part 'score_upload_notifier.g.dart';

/// Maps an upload failure to a short, user-facing French message (the raw
/// exception is never shown).
String uploadErrorMessage(Object error) {
  if (error is AuthException) {
    return switch (error.error) {
      // A fact, not a failure (change: add-client-transport-deadlines):
      // after an abandoned upload that in fact landed, re-submitting is the
      // expected path — the server dedups on (owner, sha256), so nothing was
      // duplicated and nothing went wrong.
      AuthError.alreadyExists =>
        'Cette partition est déjà dans votre bibliothèque.',
      // The refusal names whether a higher plan raises the limit (change:
      // add-premium-subscription) — the surface can then upsell.
      AuthError.rateLimited =>
        (error.message ?? '').contains('upgrade_raises_limit=true')
            ? 'Vous avez atteint votre quota d\'envois. Premium relève la limite.'
            : 'Vous avez atteint votre quota d\'envois. Réessayez plus tard.',
      AuthError.invalidArgument =>
        'Ce fichier n\'a pas pu être accepté (format ou contenu invalide).',
      AuthError.unauthenticated =>
        'Votre session a expiré. Reconnectez-vous et réessayez.',
      AuthError.unavailable =>
        'Serveur injoignable. Vérifiez votre connexion et réessayez.',
      _ => 'Une erreur est survenue lors de l\'envoi. Réessayez.',
    };
  }
  return 'Une erreur est survenue lors de l\'envoi. Réessayez.';
}

/// The three ordered steps of the contribution wizard.
enum UploadStep { upload, verify, confirm }

/// Immutable state of the contribution wizard (design 7). One value carries the
/// whole flow: the picked file, its client-side validation result, the rights
/// attestation, the chosen difficulty, and the submission status. Mutated only
/// via [ScoreUploadNotifier] (copyWith), never in place.
@freezed
abstract class ScoreUploadState with _$ScoreUploadState {
  const ScoreUploadState._();

  const factory ScoreUploadState({
    @Default(UploadStep.upload) UploadStep step,
    PickedScoreFile? file,
    @Default(false) bool validating,

    /// Server-parity summary of the validated file (metadata shown read-only).
    ScoreSummary? summary,

    /// Typed reject code when validation failed (`too_large`/`undecodable`/…).
    String? rejectCode,

    /// The rights attestation (design 2b).
    RightsBasis? rightsBasis,
    @Default(false) bool rightsAck,

    /// Fallback title/composer the user may type when the file carries none
    /// (server uses them only then; a parsed value always wins).
    String? fallbackTitle,
    String? fallbackComposer,

    /// The chosen difficulty (confirm step).
    PracticeLevel? level,
    @Default(false) bool submitting,

    /// Set on a successful upload.
    ContributedScore? result,

    /// A recoverable submission error message (inputs are kept).
    String? submitError,

    /// Typed code of a submission refusal the UI localizes itself (change:
    /// add-drums-access) — e.g. `drums_not_available`. Wins over [submitError]
    /// so no raw/pre-baked string is shown for a typed refusal.
    String? submitErrorCode,
  }) = _ScoreUploadState;

  /// A file passed client-side validation (has a summary, no reject).
  bool get isValidated => summary != null && rejectCode == null;

  /// The rights attestation is complete (basis chosen + checkbox ticked).
  bool get hasAttestation => rightsBasis != null && rightsAck;

  /// Whether the Upload step's requirements are met to advance to Verify:
  /// a validated file plus the mandatory rights attestation (spec).
  bool get canLeaveUpload => isValidated && hasAttestation;

  /// The score already carries a title, or the user supplied a fallback one.
  /// A title is mandatory (the server rejects an untitled upload).
  bool get hasTitle =>
      (summary?.title?.isNotEmpty ?? false) ||
      (fallbackTitle?.trim().isNotEmpty ?? false);

  /// Whether the Confirm step can be finalized: a difficulty is chosen and the
  /// score has a title (parsed or fallback).
  bool get canFinalize => level != null && hasTitle;

  /// The flow completed successfully.
  bool get isDone => result != null;
}

/// Drives the contribution wizard state machine
/// (`pickFile → validating → previewing → confirming → submitting → done/error`),
/// depending on the injectable picker / notation-engine / upload-service seams so
/// it is fully testable without the native library or a live backend.
@riverpod
class ScoreUploadNotifier extends _$ScoreUploadNotifier {
  /// Set when the user leaves the upload flow (the provider is autoDispose).
  /// Post-`await` state writes in [submit] check it so an abandoned upload's
  /// late result is discarded **explicitly** — under Riverpod 2.6.1 a write to
  /// a disposed notifier happens to be dropped silently, but that is an
  /// implementation detail (3.x throws), not a contract. `Ref.mounted` does
  /// not exist on this major. Abandoning claims nothing: no "cancelled", no
  /// "sent" — the request may already have been applied server-side, and
  /// `MyUploads` reports the truth on its next refresh (design D11).
  var _disposed = false;

  @override
  ScoreUploadState build() {
    ref.onDispose(() => _disposed = true);
    return const ScoreUploadState();
  }

  /// Pick a file and validate it locally. A cancelled pick is a no-op.
  Future<void> pickAndValidate() async {
    final picked = await ref.read(filePickerProvider).pickScore();
    if (picked == null) return;
    // Reset any prior validation, keep the wizard on the Upload step.
    state = ScoreUploadState(file: picked, validating: true);
    final outcome = await ref
        .read(notationEngineProvider)
        .validate(picked.bytes);
    var rejectCode = outcome.rejectCode;
    // Drum gate (change: add-drums-access): a percussion score is valid, but
    // uploading one requires the drum feature — refuse locally, like any other
    // validation refusal, when it is not visible to this caller. The backend
    // refuses the same upload independently (defence in depth).
    if (rejectCode == null &&
        outcome.summary?.instrument == InstrumentKind.percussion &&
        !ref.read(drumsEnabledProvider)) {
      rejectCode = kDrumsNotAvailableCode;
    }
    state = state.copyWith(
      validating: false,
      summary: outcome.summary,
      rejectCode: rejectCode,
    );
  }

  /// Set the declared rights basis (author / public domain).
  void setRightsBasis(RightsBasis basis) =>
      state = state.copyWith(rightsBasis: basis);

  /// Toggle the authorship/rights confirmation checkbox.
  void setRightsAck(bool ack) => state = state.copyWith(rightsAck: ack);

  /// Advance Upload → Verify once the file is validated and the attestation is
  /// complete. Gated: a no-op otherwise.
  void goToVerify() {
    if (state.canLeaveUpload) state = state.copyWith(step: UploadStep.verify);
  }

  /// Advance Verify → Confirm (the preview is review-only, always allowed).
  void goToConfirm() {
    if (state.step == UploadStep.verify) {
      state = state.copyWith(step: UploadStep.confirm);
    }
  }

  /// Return to a previous step to change an input.
  void backToUpload() => state = state.copyWith(step: UploadStep.upload);
  void backToVerify() => state = state.copyWith(step: UploadStep.verify);

  /// Choose the difficulty (confirm step).
  void setLevel(PracticeLevel level) => state = state.copyWith(level: level);

  /// Set the fallback title/composer (only meaningful when the file has none).
  void setFallbackTitle(String v) => state = state.copyWith(fallbackTitle: v);
  void setFallbackComposer(String v) =>
      state = state.copyWith(fallbackComposer: v);

  /// Finalize: submit the file bytes + level + rights attestation to the backend.
  /// Gated on [ScoreUploadState.canFinalize]; keeps the user's inputs on a
  /// recoverable error so they can retry.
  Future<void> submit() async {
    final file = state.file;
    final basis = state.rightsBasis;
    final level = state.level;
    if (file == null ||
        basis == null ||
        level == null ||
        !state.rightsAck ||
        !state.hasTitle) {
      return;
    }
    state = state.copyWith(
      submitting: true,
      submitError: null,
      submitErrorCode: null,
    );
    try {
      final record = await ref
          .read(scoreUploadServiceProvider)
          .upload(
            data: file.bytes,
            filename: file.name,
            level: level,
            rightsBasis: basis,
            rightsAck: true,
            fallbackTitle: state.fallbackTitle,
            fallbackComposer: state.fallbackComposer,
          );
      if (_disposed) return; // Abandoned: discard the late result, say nothing.
      // Setting `result` is the signal: `MyUploads` listens for this transition
      // and refreshes itself (which cascades to myContributedScores +
      // favoriteScores). The uploader does NOT invalidate a sibling provider.
      state = state.copyWith(submitting: false, result: record);
      // Usage telemetry (change: add-feature-usage-analytics).
      unawaited(
        ref
            .read(usageTrackingNotifierProvider.notifier)
            .record(UsageActions.scoreUpload, subjectId: record.id),
      );
    } catch (e) {
      if (_disposed) return; // Abandoned: a late failure is nobody's news.
      // The backend's typed drum refusal (change: add-drums-access) maps to the
      // same localized reason as the local one — never the raw gRPC string.
      if (_isDrumsRefusal(e)) {
        state = state.copyWith(
          submitting: false,
          submitErrorCode: kDrumsNotAvailableCode,
        );
        return;
      }
      state = state.copyWith(
        submitting: false,
        submitError: uploadErrorMessage(e),
      );
    }
  }

  /// Whether [e] is the backend's typed drum-gate refusal: a
  /// `PERMISSION_DENIED` whose message starts with `drums_not_available`.
  static bool _isDrumsRefusal(Object e) =>
      e is AuthException &&
      e.error == AuthError.permissionDenied &&
      (e.message ?? '').startsWith(kDrumsNotAvailableCode);

  /// Reset the whole flow (e.g. after a successful upload, to contribute again).
  void reset() => state = const ScoreUploadState();
}
