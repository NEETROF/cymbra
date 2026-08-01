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

import 'app_locale.dart';
import 'session_notifier.dart';
import 'session_state.dart';

/// Reconciles the account's stored language into the UI after sign-in (change:
/// sync-account-language-preference). A dedicated listener widget near the top of
/// the app subtree that isolates the `ref.listen` side effect (per the
/// flutter-riverpod-architecture rules) — it never calls a service directly, it
/// delegates the reconcile decision to [AppLocale.syncOnLogin].
///
/// It fires **once per signed-in account** (keyed by user id), so re-resolving the
/// account (e.g. after handle onboarding) does not re-run the reconcile; a fresh
/// sign-in after sign-out re-syncs.
class LanguageSyncListener extends ConsumerStatefulWidget {
  const LanguageSyncListener({required this.child, super.key});

  final Widget child;

  @override
  ConsumerState<LanguageSyncListener> createState() =>
      _LanguageSyncListenerState();
}

class _LanguageSyncListenerState extends ConsumerState<LanguageSyncListener> {
  /// The account already reconciled this session; guards against a repeat sync
  /// when the same account is re-resolved.
  String? _syncedUserId;

  @override
  Widget build(BuildContext context) {
    ref.listen<SessionState>(sessionNotifierProvider, (previous, next) {
      final account = next is SessionAuthenticated ? next.account : null;
      if (account == null) {
        // Signed out / guest / not yet resolved: forget the last synced account so
        // a later sign-in reconciles again.
        if (next is! SessionAuthenticated) _syncedUserId = null;
        return;
      }
      if (account.userId == _syncedUserId) return; // already reconciled
      _syncedUserId = account.userId;
      // Fire-and-forget: the notifier owns the reconcile decision + any push.
      unawaited(
        ref.read(appLocaleProvider.notifier).syncOnLogin(account.locale),
      );
    });
    return widget.child;
  }
}
