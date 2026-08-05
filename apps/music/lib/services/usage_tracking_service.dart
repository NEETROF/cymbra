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

import 'package:fixnum/fixnum.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../analytics/usage_event_record.dart';
import '../src/grpc/usage.pbgrpc.dart' as usage;
import 'grpc_client.dart';

part 'usage_tracking_service.g.dart';

/// Seam for delivering a batch of buffered feature-usage events (change:
/// add-feature-usage-analytics, task 6.1). Behind a provider so the tracking
/// notifier + offline buffer are testable without the backend. Best-effort:
/// `report` throws on any failure so the buffer keeps the batch for a later flush.
abstract class UsageTrackingService {
  /// Deliver a batch via `ReportEvents`. Returns normally on any server response
  /// (the signal to drop the flushed events — ingestion is best-effort, so
  /// server-skipped malformed events are not retried); throws on transport failure.
  Future<void> report(List<UsageEventRecord> events);
}

/// Production [UsageTrackingService] over the generated `UsageServiceClient`. Runs
/// through [AuthedRunner] so a stale access token is refreshed once and retried
/// (ingestion is authenticated — the anti-spam guarantee, design D9).
class GrpcUsageTrackingService implements UsageTrackingService {
  GrpcUsageTrackingService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = usage.UsageServiceClient(channel),
       _authed = authed;

  final usage.UsageServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<void> report(List<UsageEventRecord> events) => _authed((bearer) async {
    await _client.reportEvents(
      usage.ReportEventsRequest(
        events: events.map(
          (e) => usage.UsageEvent(
            action: e.action,
            variant: e.variant,
            subjectId: e.subjectId,
            platform: e.platform,
            deviceClass: e.deviceClass,
            appVersion: e.appVersion,
            locale: e.locale,
            occurredAtMs: Int64(e.occurredAtMs),
          ),
        ),
      ),
      options: bearerOptions(bearer),
    );
  });
}

/// Production usage-tracking-service provider. Override in tests with a fake/mock.
@Riverpod(keepAlive: true)
UsageTrackingService usageTrackingService(Ref ref) => GrpcUsageTrackingService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
);
