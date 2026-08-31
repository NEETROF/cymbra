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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/content_report_service.dart';

part 'content_report_notifier.freezed.dart';
part 'content_report_notifier.g.dart';

/// Transient outcome of a report submission (change: add-content-reporting).
///
/// The sequence counter exists so a listener fires again for a second report of
/// the same kind — two consecutive successes would otherwise be one unchanged
/// state and show no acknowledgement the second time.
@freezed
abstract class ContentReportState with _$ContentReportState {
  const factory ContentReportState({
    @Default(false) bool sending,
    @Default(0) int sentSeq,
    @Default(0) int errorSeq,
  }) = _ContentReportState;
}

/// Owns the report mutation. The sheet fires [submit] and never awaits its
/// return; a dedicated listener reacts to [ContentReportState] (architecture
/// rules 1 and 3).
@riverpod
class ContentReport extends _$ContentReport {
  @override
  ContentReportState build() => const ContentReportState();

  /// Report [targetId]. Fire-and-observe: a failure lands in the state as an
  /// `errorSeq` bump, never thrown — the caller is a button, not an exception
  /// handler.
  Future<void> submit({
    required ReportTarget target,
    required String targetId,
    required ReportReason reason,
    String? note,
  }) async {
    if (state.sending) return;
    state = state.copyWith(sending: true);
    try {
      await ref
          .read(contentReportServiceProvider)
          .report(
            target: target,
            targetId: targetId,
            reason: reason,
            note: note,
          );
      state = state.copyWith(sending: false, sentSeq: state.sentSeq + 1);
    } catch (_) {
      // The cause is logged by the transport; the UI shows a localized message
      // (never a raw gRPC string).
      state = state.copyWith(sending: false, errorSeq: state.errorSeq + 1);
    }
  }
}
