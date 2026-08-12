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

import 'dart:io' show Platform;

import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/foundation.dart' show debugPrint, kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'push_service.g.dart';

/// The OS notification-permission state (change: add-push-notifications).
/// Notifications are strictly **opt-in**: without [PushPermission.granted] the
/// device never registers a token and the user is never a push recipient.
enum PushPermission {
  /// The user has never been asked.
  notDetermined,

  /// The user allowed notifications.
  granted,

  /// The user refused (or the OS refuses on this device).
  denied,
}

/// The FCM-capable platforms (design D1). Windows and Linux are absent: neither
/// has a reliable app-closed push path, so those clients register nothing.
enum PushPlatform {
  ios('ios'),
  android('android'),
  macos('macos');

  const PushPlatform(this.wireName);

  /// The identifier sent to the backend's `RegisterPushToken`.
  final String wireName;
}

/// Resolve the running platform to its FCM platform, or `null` when this device
/// cannot hold a token (Windows, Linux, web).
PushPlatform? currentPushPlatform() {
  if (kIsWeb) return null;
  if (Platform.isIOS) return PushPlatform.ios;
  if (Platform.isAndroid) return PushPlatform.android;
  if (Platform.isMacOS) return PushPlatform.macos;
  return null;
}

/// Injectable seam over the native push SDK (change: add-push-notifications,
/// task 4.2).
///
/// Only notifiers talk to this — widgets go through
/// `pushRegistrationProvider` / `notificationPrefsProvider`. Behind a provider so
/// state and widgets are testable without Firebase (see the `flutter-testing`
/// skill).
abstract class PushService {
  /// The platform's FCM identity, or `null` on a platform that cannot register
  /// (Windows/Linux/web) **or** when no Firebase configuration is present.
  PushPlatform? get platform;

  /// Ask the OS for notification permission (a no-op returning the current state
  /// if already decided).
  Future<PushPermission> requestPermission();

  /// The current FCM registration token, or `null` when unavailable (permission
  /// refused, no Firebase config, unsupported platform).
  Future<String?> token();

  /// Tokens emitted whenever FCM rotates this install's registration.
  Stream<String> get tokenRefreshes;

  /// Drop the local registration so the device stops receiving pushes.
  Future<void> deleteToken();
}

/// Production [PushService] over `firebase_messaging`.
///
/// Initialization is **guarded**: `Firebase.initializeApp` throws when the native
/// Firebase configuration is absent (no `google-services.json` /
/// `GoogleService-Info.plist`), and every entry point below then behaves exactly
/// like an unsupported platform — the app starts and runs normally, it simply
/// never registers a token. See `docs/push-notifications-setup.md`.
class FirebasePushService implements PushService {
  FirebasePushService();

  /// `null` until [_ensureReady] has run; `false` once initialization failed.
  bool? _ready;

  @override
  PushPlatform? get platform => currentPushPlatform();

  /// Initialize Firebase once, tolerating a missing/incomplete configuration.
  Future<bool> _ensureReady() async {
    if (_ready != null) return _ready!;
    if (platform == null) return _ready = false;
    try {
      if (Firebase.apps.isEmpty) await Firebase.initializeApp();
      _ready = true;
    } catch (e) {
      // No Firebase project wired into this build: degrade to "no push".
      debugPrint(
        '[push] Firebase init failed ($e); push is unavailable. Is '
        'GoogleService-Info.plist / google-services.json in the bundle?',
      );
      _ready = false;
    }
    return _ready!;
  }

  @override
  Future<PushPermission> requestPermission() async {
    if (!await _ensureReady()) return PushPermission.denied;
    try {
      final settings = await FirebaseMessaging.instance.requestPermission();
      return switch (settings.authorizationStatus) {
        AuthorizationStatus.authorized ||
        AuthorizationStatus.provisional => PushPermission.granted,
        AuthorizationStatus.denied => PushPermission.denied,
        _ => PushPermission.notDetermined,
      };
    } catch (e) {
      debugPrint('[push] permission request failed ($e); treating as denied.');
      return PushPermission.denied;
    }
  }

  /// How long to wait for the APNs device token before giving up.
  static const _apnsTimeout = Duration(seconds: 10);
  static const _apnsPollInterval = Duration(milliseconds: 400);

  /// Wait for the APNs device token on Apple platforms.
  ///
  /// FCM derives its registration token from the APNs one, which the OS delivers
  /// **asynchronously** some time after `requestPermission` returns. Calling
  /// `getToken()` before it lands throws `apns-token-not-set` — so the first
  /// launch after the user allows notifications would silently fail to register,
  /// and (because registration runs once per signed-in account) the device would
  /// stay unregistered for the whole session.
  ///
  /// There is no event to await, hence the poll. It is bounded: a device that
  /// never registers with APNs — no capability on the profile, no network — must
  /// degrade to "no push", not hang. Android has no APNs and returns immediately.
  Future<bool> _awaitApnsToken() async {
    if (platform == PushPlatform.android) return true;
    final deadline = DateTime.now().add(_apnsTimeout);
    while (DateTime.now().isBefore(deadline)) {
      try {
        final apns = await FirebaseMessaging.instance.getAPNSToken();
        if (apns != null && apns.isNotEmpty) return true;
      } catch (e) {
        debugPrint('[push] getAPNSToken failed ($e).');
        return false;
      }
      await Future<void>.delayed(_apnsPollInterval);
    }
    debugPrint(
      '[push] no APNs token after ${_apnsTimeout.inSeconds}s. The provisioning '
      'profile may lack the Push Notifications capability, or the device is '
      'offline.',
    );
    return false;
  }

  @override
  Future<String?> token() async {
    if (!await _ensureReady()) return null;
    if (!await _awaitApnsToken()) return null;
    try {
      return await FirebaseMessaging.instance.getToken();
    } catch (e) {
      debugPrint('[push] getToken failed ($e).');
      return null;
    }
  }

  @override
  Stream<String> get tokenRefreshes =>
      FirebaseMessaging.instance.onTokenRefresh;

  @override
  Future<void> deleteToken() async {
    if (!await _ensureReady()) return;
    try {
      await FirebaseMessaging.instance.deleteToken();
    } catch (e) {
      // Best effort: the server-side unregister is what actually stops sends.
      debugPrint('[push] deleteToken failed ($e); continuing.');
    }
  }
}

/// Production push-service provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
PushService pushService(Ref ref) => FirebasePushService();
