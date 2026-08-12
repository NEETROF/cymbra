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

import '../src/grpc/notifications.pbgrpc.dart' as pb;
import 'grpc_client.dart';

part 'notification_service.g.dart';

/// The caller's notification settings as stored server-side (change:
/// add-push-notifications).
class NotificationSettings {
  const NotificationSettings({
    required this.prefs,
    this.timezone,
    this.hasRegisteredDevice = false,
  });

  /// Explicit per-category choices. A category **absent** from this map means the
  /// user never expressed one, so the category's product default applies.
  final Map<String, bool> prefs;

  /// The IANA timezone the server has on file, `null` when none.
  final String? timezone;

  /// Whether this account currently has at least one registered device.
  final bool hasRegisteredDevice;

  /// Empty settings — the state before anything has loaded.
  static const NotificationSettings empty = NotificationSettings(prefs: {});
}

/// Seam over the backend's `NotificationService` (change: add-push-notifications,
/// task 4.2). Only notifiers call this; behind a provider so state and widgets are
/// testable without the backend.
abstract class NotificationRegistryService {
  /// Register or refresh this device's FCM token.
  Future<void> registerToken({required String token, required String platform});

  /// Drop this device's token (sign-out).
  Future<void> unregisterToken(String token);

  /// Record the caller's choice for one notification category.
  Future<void> setPref({required String category, required bool enabled});

  /// Report the device's IANA timezone.
  Future<void> setTimezone(String timezone);

  /// Read the caller's current settings.
  Future<NotificationSettings> settings();
}

/// Production [NotificationRegistryService] over the generated
/// `NotificationServiceClient`. Runs through [AuthedRunner] so a stale access
/// token is refreshed once and retried — every method acts on the authenticated
/// caller's own account.
class GrpcNotificationRegistryService implements NotificationRegistryService {
  GrpcNotificationRegistryService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = pb.NotificationServiceClient(channel),
       _authed = authed;

  final pb.NotificationServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<void> registerToken({
    required String token,
    required String platform,
  }) => _authed((bearer) async {
    await _client.registerPushToken(
      pb.RegisterPushTokenRequest(token: token, platform: platform),
      options: bearerOptions(bearer),
    );
  });

  @override
  Future<void> unregisterToken(String token) => _authed((bearer) async {
    await _client.unregisterPushToken(
      pb.UnregisterPushTokenRequest(token: token),
      options: bearerOptions(bearer),
    );
  });

  @override
  Future<void> setPref({required String category, required bool enabled}) =>
      _authed((bearer) async {
        await _client.setNotificationPref(
          pb.SetNotificationPrefRequest(category: category, enabled: enabled),
          options: bearerOptions(bearer),
        );
      });

  @override
  Future<void> setTimezone(String timezone) => _authed((bearer) async {
    await _client.setTimezone(
      pb.SetTimezoneRequest(timezone: timezone),
      options: bearerOptions(bearer),
    );
  });

  @override
  Future<NotificationSettings> settings() => _authed((bearer) async {
    final res = await _client.getNotificationSettings(
      pb.GetNotificationSettingsRequest(),
      options: bearerOptions(bearer),
    );
    return NotificationSettings(
      prefs: {for (final p in res.prefs) p.category: p.enabled},
      timezone: res.hasTimezone() ? res.timezone : null,
      hasRegisteredDevice: res.hasRegisteredDevice,
    );
  });
}

/// Production notification-registry provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
NotificationRegistryService notificationRegistryService(Ref ref) =>
    GrpcNotificationRegistryService(
      channel: ref.watch(cymbraChannelProvider),
      authed: ref.watch(authedRunnerProvider),
    );
