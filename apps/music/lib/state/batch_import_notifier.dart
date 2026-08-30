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

import '../services/auth_service.dart';
import '../services/file_picker_service.dart';
import '../services/score_upload_service.dart';
import 'score_catalog.dart' show PracticeLevel;

part 'batch_import_notifier.freezed.dart';
part 'batch_import_notifier.g.dart';

/// What became of one file in a batch (change: add-private-score-catalog).
/// Each is a per-file fact: one file's outcome never decides another's.
enum BatchOutcome {
  /// Stored — the score is now in the private library.
  imported,

  /// The owner already had this exact content (server dedups on owner+sha256).
  duplicate,

  /// The server refused the bytes (not a score, unreadable, too large).
  invalid,

  /// The plan's rolling upload quota is used up.
  quotaExceeded,

  /// Anything else (transport, session, server) — retryable, unlike the above.
  failed,
}

/// One file's result in the batch.
@freezed
abstract class BatchFileResult with _$BatchFileResult {
  const factory BatchFileResult({
    required String name,
    required BatchOutcome outcome,
  }) = _BatchFileResult;
}

/// Immutable state of a batch import. One attestation and one difficulty govern
/// the whole batch (design D2); the per-file outcomes accumulate as it runs.
@freezed
abstract class BatchImportState with _$BatchImportState {
  const BatchImportState._();

  const factory BatchImportState({
    @Default(<PickedScoreFile>[]) List<PickedScoreFile> files,

    /// The rights attestation, applied to every file in the batch.
    RightsBasis? rightsBasis,
    @Default(false) bool rightsAck,

    /// The difficulty, applied to every file in the batch.
    PracticeLevel? level,

    /// The caller's remaining allowance, once read. `null` = not read yet (the
    /// warning simply is not shown rather than guessing a number).
    UploadAllowance? allowance,
    @Default(false) bool running,

    /// Index of the file currently uploading, for progress.
    @Default(0) int currentIndex,
    @Default(<BatchFileResult>[]) List<BatchFileResult> results,
    @Default(false) bool done,
  }) = _BatchImportState;

  /// The attestation is complete (basis chosen + checkbox ticked).
  bool get hasAttestation => rightsBasis != null && rightsAck;

  /// The batch may start: files selected, attestation complete, difficulty
  /// chosen, and not already running or finished.
  bool get canStart =>
      files.isNotEmpty && hasAttestation && level != null && !running && !done;

  /// Whether the selection is larger than what the plan still allows, so the
  /// flow must say so BEFORE any upload runs. Unknown allowance ⇒ no warning.
  bool get exceedsAllowance =>
      allowance != null && files.length > allowance!.remaining;

  /// How many files cannot land under the current allowance (0 when it fits or
  /// is unknown).
  int get overAllowanceCount =>
      exceedsAllowance ? files.length - allowance!.remaining : 0;

  int get importedCount =>
      results.where((r) => r.outcome == BatchOutcome.imported).length;
}

/// Drives a batch import: collect one attestation + one difficulty, then upload
/// the selection **one file at a time** through the same single-file upload
/// operation the wizard uses (design D1). A failing file is recorded and the run
/// continues — the batch is a sequence of independent outcomes, never
/// all-or-nothing.
@riverpod
class BatchImportNotifier extends _$BatchImportNotifier {
  bool _disposed = false;

  @override
  BatchImportState build() {
    ref.onDispose(() => _disposed = true);
    return const BatchImportState();
  }

  /// Seed the batch with a selection (replaces any previous one).
  void setFiles(List<PickedScoreFile> files) =>
      state = BatchImportState(files: List.unmodifiable(files));

  void setRightsBasis(RightsBasis basis) =>
      state = state.copyWith(rightsBasis: basis);

  void setRightsAck(bool ack) => state = state.copyWith(rightsAck: ack);

  void setLevel(PracticeLevel level) => state = state.copyWith(level: level);

  /// Read the caller's remaining allowance so the flow can warn up front. A
  /// failure here is deliberately silent: not knowing the allowance must not
  /// block an import the server may well accept.
  Future<void> loadAllowance() async {
    try {
      final a = await ref.read(scoreUploadServiceProvider).uploadAllowance();
      if (_disposed) return;
      state = state.copyWith(allowance: a);
    } catch (_) {
      // Leave `allowance` null — no warning, no obstacle.
    }
  }

  /// Run the batch. Uploads sequentially, recording one outcome per file; a
  /// per-file failure never aborts the remaining files.
  Future<void> run() async {
    if (!state.canStart) return;
    final basis = state.rightsBasis!;
    final level = state.level!;
    state = state.copyWith(running: true, results: const [], currentIndex: 0);
    final service = ref.read(scoreUploadServiceProvider);
    final results = <BatchFileResult>[];
    for (var i = 0; i < state.files.length; i++) {
      final file = state.files[i];
      if (_disposed) return; // Abandoned: say nothing, change nothing.
      state = state.copyWith(currentIndex: i);
      BatchOutcome outcome;
      try {
        await service.upload(
          data: file.bytes,
          filename: file.name,
          level: level,
          rightsBasis: basis,
          rightsAck: true,
        );
        outcome = BatchOutcome.imported;
      } catch (e) {
        outcome = _outcomeOf(e);
      }
      results.add(BatchFileResult(name: file.name, outcome: outcome));
      if (_disposed) return;
      state = state.copyWith(results: List.unmodifiable(results));
    }
    if (_disposed) return;
    state = state.copyWith(running: false, done: true);
  }

  /// Classify a per-file failure. The server's typed refusals map 1:1 onto the
  /// outcomes the result board shows; anything else is the retryable bucket.
  static BatchOutcome _outcomeOf(Object error) {
    if (error is AuthException) {
      return switch (error.error) {
        AuthError.alreadyExists => BatchOutcome.duplicate,
        AuthError.invalidArgument => BatchOutcome.invalid,
        AuthError.rateLimited => BatchOutcome.quotaExceeded,
        _ => BatchOutcome.failed,
      };
    }
    return BatchOutcome.failed;
  }
}
