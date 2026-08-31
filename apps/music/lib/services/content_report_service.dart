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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/grpc/score.pbgrpc.dart' as score;
import 'grpc_client.dart';
import 'rpc_deadlines.dart';

part 'content_report_service.g.dart';

/// What a report is about (change: add-content-reporting). Mirrors the backend's
/// `target_kind` vocabulary; a value the server does not know is refused there
/// rather than silently stored.
enum ReportTarget {
  catalogScore('catalog_score'),
  soundFont('soundfont'),
  profile('profile');

  const ReportTarget(this.wire);

  /// The wire string the backend expects.
  final String wire;
}

/// Why the reporter is reporting. Kept short on purpose: a long list makes a
/// reporter hesitate, and the free-text note carries the detail.
enum ReportReason {
  copyright('copyright'),
  inappropriate('inappropriate'),
  wrongContent('wrong_content'),
  other('other');

  const ReportReason(this.wire);

  final String wire;
}

/// The longest note the backend accepts, mirrored client-side so the field can
/// stop the user at the limit instead of failing the call.
const int kReportNoteMaxLength = 2000;

/// Seam over the backend `ScoreService`'s reporting surface. Bearer-
/// authenticated; the production impl refreshes transparently on
/// `UNAUTHENTICATED`. Tests override the provider with a mock.
abstract class ContentReportService {
  /// Report [targetId] for [reason], with an optional free-text [note].
  ///
  /// Reporting the same item twice is **not** an error: the backend returns the
  /// first report's id, so the UI acknowledges either way. A reporter told off
  /// for pressing twice learns not to report.
  Future<void> report({
    required ReportTarget target,
    required String targetId,
    required ReportReason reason,
    String? note,
  });
}

/// Production [ContentReportService] over the generated `ScoreServiceClient`.
class GrpcContentReportService implements ContentReportService {
  GrpcContentReportService({
    required ClientChannel channel,
    required AuthedRunner authed,
    RpcDeadlines deadlines = const RpcDeadlines(),
  }) : _client = score.ScoreServiceClient(channel, interceptors: [deadlines]),
       _authed = authed;

  final score.ScoreServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<void> report({
    required ReportTarget target,
    required String targetId,
    required ReportReason reason,
    String? note,
  }) => _authed((bearer) async {
    final trimmed = note?.trim();
    await _client.reportContent(
      score.ReportContentRequest(
        targetKind: target.wire,
        targetId: targetId,
        reason: reason.wire,
        note: trimmed == null || trimmed.isEmpty ? null : trimmed,
      ),
      options: bearerOptions(bearer),
    );
  });
}

/// Production content-report provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
ContentReportService contentReportService(Ref ref) => GrpcContentReportService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
  deadlines: ref.watch(rpcDeadlinesProvider),
);
