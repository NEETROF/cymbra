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

import 'package:flutter/foundation.dart' show debugPrint;
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/notification_service.dart';
import '../services/push_service.dart';
import '../services/timezone_service.dart';

part 'push_registration.freezed.dart';
part 'push_registration.g.dart';

/// This device's push-registration state (change: add-push-notifications).
@freezed
sealed class PushRegistrationState with _$PushRegistrationState {
  const factory PushRegistrationState({
    /// The FCM platform this device reports as, `null` on Windows/Linux/web or
    /// when no Firebase configuration is present — those never register.
    PushPlatform? platform,
    @Default(PushPermission.notDetermined) PushPermission permission,

    /// The token currently registered server-side, `null` when none is.
    String? token,
  }) = _PushRegistrationState;

  const PushRegistrationState._();

  /// Whether this device can hold a push token at all.
  bool get supported => platform != null;

  /// Whether the device is registered with the backend right now.
  bool get registered => token != null;
}

/// Owns the device's push registration lifecycle (change: add-push-notifications,
/// task 4.3): request permission, obtain the FCM token, register it with the
/// backend, keep it fresh, report the timezone, and unregister on sign-out.
///
/// The UI never calls this directly on a schedule — [PushRegistrationListener]
/// drives it from the session state.
@Riverpod(keepAlive: true)
class PushRegistration extends _$PushRegistration {
  StreamSubscription<String>? _refreshes;

  @override
  PushRegistrationState build() {
    ref.onDispose(() {
      unawaited(_refreshes?.cancel());
      _refreshes = null;
    });
    return const PushRegistrationState();
  }

  /// Register this device for the signed-in user.
  ///
  /// Opt-in and best-effort: an unsupported platform, a refused permission or a
  /// failing backend all leave the app working with no notifications — nothing is
  /// surfaced to the user and no raw error escapes.
  ///
  /// Every early return is logged. Silence would be indistinguishable from
  /// success, and "the token never appeared" is otherwise undiagnosable — on a
  /// user's device as much as in local dev.
  Future<void> registerForCurrentUser() async {
    final push = ref.read(pushServiceProvider);
    final platform = push.platform;
    // Windows/Linux (and a build with no Firebase config) hold no token and are
    // never a recipient — stop before asking for anything.
    if (platform == null) {
      debugPrint(
        '[push] no registration: platform is not FCM-capable '
        '(Windows/Linux/web) or no Firebase configuration is bundled.',
      );
      state = const PushRegistrationState();
      return;
    }

    final permission = await push.requestPermission();
    state = state.copyWith(platform: platform, permission: permission);
    if (permission != PushPermission.granted) {
      debugPrint('[push] no registration: OS permission is $permission.');
      return;
    }

    final token = await push.token();
    if (token == null || token.isEmpty) {
      // The usual cause on Apple platforms is APNs registration failing — a
      // provisioning profile without the Push Notifications capability, or no
      // network at launch.
      debugPrint(
        '[push] no registration: FCM returned no token for '
        '${platform.wireName}.',
      );
      return;
    }

    await _register(token, platform);
    // Keep the server's copy current when FCM rotates the install's token.
    _refreshes ??= push.tokenRefreshes.listen((refreshed) {
      unawaited(_register(refreshed, platform));
    });
    await _reportTimezone();
  }

  /// Drop this device's registration (sign-out), server-side first so a failure
  /// to delete the local token still stops the sends.
  Future<void> unregister() async {
    final token = state.token;
    if (token != null) {
      try {
        await ref
            .read(notificationRegistryServiceProvider)
            .unregisterToken(token);
      } catch (e) {
        // Best effort: the token is also pruned server-side on the first send
        // that reports it invalid.
        debugPrint('[push] server-side unregister failed ($e); continuing.');
      }
    }
    try {
      await ref.read(pushServiceProvider).deleteToken();
    } catch (e) {
      debugPrint('[push] local token delete failed ($e); continuing.');
    }
    state = state.copyWith(token: null);
  }

  /// Report (or refresh) the device's IANA timezone so scheduled sends fire at
  /// the user's local hour.
  Future<void> _reportTimezone() async {
    final tz = await ref.read(timezoneServiceProvider).current();
    if (tz == null || tz.isEmpty) {
      debugPrint(
        '[push] timezone not reported: the platform gave no IANA name.',
      );
      return;
    }
    try {
      await ref.read(notificationRegistryServiceProvider).setTimezone(tz);
    } catch (e) {
      debugPrint('[push] timezone report failed ($e); keeping the stored one.');
    }
  }

  Future<void> _register(String token, PushPlatform platform) async {
    try {
      await ref
          .read(notificationRegistryServiceProvider)
          .registerToken(token: token, platform: platform.wireName);
      state = state.copyWith(platform: platform, token: token);
      debugPrint(
        '[push] registered ${platform.wireName} token with the server.',
      );
    } catch (e) {
      // Offline or backend down: leave the state unregistered so the next
      // sign-in / launch retries.
      debugPrint(
        '[push] server registration failed ($e); will retry on the '
        'next sign-in or launch.',
      );
    }
  }
}
