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

/// One buffered feature-usage event (change: add-feature-usage-analytics). A plain
/// JSON-serialisable value (like `PlaySessionEnvelope`) so the offline outbox can
/// persist it via the preferences seam without codegen. The `id` is a
/// client-generated UUID used only for buffer de-dup / removal, never sent.
class UsageEventRecord {
  const UsageEventRecord({
    required this.id,
    required this.action,
    required this.platform,
    required this.deviceClass,
    required this.appVersion,
    required this.locale,
    required this.occurredAtMs,
    this.variant,
    this.subjectId,
  });

  final String id;
  final String action;
  final String? variant;
  final String? subjectId;
  final String platform;
  final String deviceClass;
  final String appVersion;
  final String locale;
  final int occurredAtMs;

  Map<String, dynamic> toJson() => {
    'id': id,
    'action': action,
    if (variant != null) 'variant': variant,
    if (subjectId != null) 'subjectId': subjectId,
    'platform': platform,
    'deviceClass': deviceClass,
    'appVersion': appVersion,
    'locale': locale,
    'occurredAtMs': occurredAtMs,
  };

  static UsageEventRecord fromJson(Map<String, dynamic> j) => UsageEventRecord(
    id: j['id'] as String,
    action: j['action'] as String,
    variant: j['variant'] as String?,
    subjectId: j['subjectId'] as String?,
    platform: j['platform'] as String,
    deviceClass: j['deviceClass'] as String,
    appVersion: j['appVersion'] as String,
    locale: j['locale'] as String,
    occurredAtMs: j['occurredAtMs'] as int,
  );
}
