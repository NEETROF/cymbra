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

import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'notification_prefs.dart';
import 'push_registration.dart';
import 'session_notifier.dart';
import 'session_state.dart';

/// Drives the device's push registration from the session (change:
/// add-push-notifications, task 4.3).
///
/// A dedicated listener widget near the top of the app subtree that isolates the
/// `ref.listen` side effect (per the flutter-riverpod-architecture rules): it
/// never calls a service, it delegates to [PushRegistration]. Registration runs
/// **once per signed-in account** (keyed by user id) so re-resolving the same
/// account does not re-register; signing out unregisters the device so the next
/// user of this install is not messaged in the previous one's place.
class PushRegistrationListener extends ConsumerStatefulWidget {
  const PushRegistrationListener({required this.child, super.key});

  final Widget child;

  @override
  ConsumerState<PushRegistrationListener> createState() =>
      _PushRegistrationListenerState();
}

class _PushRegistrationListenerState
    extends ConsumerState<PushRegistrationListener> {
  /// The account already registered on this device this session.
  String? _registeredUserId;

  @override
  Widget build(BuildContext context) {
    ref.listen<SessionState>(sessionNotifierProvider, (previous, next) {
      final account = next is SessionAuthenticated ? next.account : null;
      if (account == null) {
        // Guest / signed out / not yet resolved. Only an actual departure from a
        // registered account unregisters — `unknown` is the transient startup
        // state and must not tear a registration down.
        if (next is! SessionAuthenticated && next is! SessionUnknown) {
          if (_registeredUserId != null) {
            _registeredUserId = null;
            unawaited(ref.read(pushRegistrationProvider.notifier).unregister());
          }
        }
        return;
      }
      if (account.userId == _registeredUserId) return; // already registered
      _registeredUserId = account.userId;
      // Fire-and-forget: the notifier owns permission, token and timezone.
      unawaited(
        ref.read(pushRegistrationProvider.notifier).registerForCurrentUser(),
      );
      // The new account's stored preferences replace the previous ones.
      ref.invalidate(notificationPrefsProvider);
    });
    return widget.child;
  }
}
