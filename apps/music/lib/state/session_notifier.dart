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
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/account_service.dart';
import '../services/auth_service.dart';
import '../services/grpc_client.dart';
import '../services/token_store.dart';
import 'session_state.dart';

part 'session_notifier.g.dart';

/// Single source of truth for the account session (design D2). Hydrates from
/// secure storage at startup and exposes the transitions the auth flows call.
/// `home` in [CymbraApp] switches on the resolved [SessionState].
@Riverpod(keepAlive: true)
class SessionNotifier extends _$SessionNotifier {
  TokenStore get _tokens => ref.read(tokenStoreProvider);
  AuthService get _auth => ref.read(authServiceProvider);
  AccountService get _account => ref.read(accountServiceProvider);

  @override
  SessionState build() {
    // Resolve after build returns — never touch `state` synchronously here.
    Future.microtask(_hydrate);
    return const SessionState.unknown();
  }

  /// Resolve the launch state: stored guest choice → guest; a stored token pair
  /// → fetch the account (refreshing silently if needed) → authenticated; no
  /// session → unauthenticated. A storage failure falls back to the entry
  /// screen rather than crashing (design risk: secure-storage availability).
  Future<void> _hydrate() async {
    try {
      if (await _tokens.isGuest()) {
        state = const SessionState.guest();
        return;
      }
      final stored = await _tokens.readTokens();
      if (stored == null) {
        state = const SessionState.unauthenticated();
        return;
      }
      await _resolveAuthenticated();
    } catch (_) {
      state = const SessionState.unauthenticated();
    }
  }

  /// Fetch the account for a token-bearing session. A refresh failure clears the
  /// store and routes to entry; a transient (offline) failure keeps the user in
  /// with an unknown account (handle onboarding is re-checked when back online).
  Future<void> _resolveAuthenticated() async {
    try {
      final account = await _account.getAccount();
      state = SessionState.authenticated(account: account);
    } on AuthException catch (e) {
      if (e.error == AuthError.unauthenticated) {
        await _tokens.clear();
        state = const SessionState.unauthenticated();
      } else {
        // Offline / transient: stay signed in, account unknown.
        state = const SessionState.authenticated();
      }
    }
  }

  /// Persist the guest choice and enter guest mode (no backend calls).
  Future<void> continueAsGuest() async {
    await _tokens.setGuest();
    state = const SessionState.guest();
  }

  /// Leave guest mode and return to the entry screen so the user can sign in.
  Future<void> leaveGuest() async {
    await _tokens.clear();
    state = const SessionState.unauthenticated();
  }

  /// Adopt a freshly-obtained session (from any sign-in path): store the tokens
  /// and resolve the account (which gates handle onboarding).
  Future<void> onSignedIn(AuthTokens tokens) async {
    await _tokens.writeTokens(tokens.toStored());
    state = const SessionState.unknown();
    await _resolveAuthenticated();
  }

  /// Replace the cached account after onboarding/profile changes (e.g. once a
  /// handle is chosen) so routing re-evaluates `needsHandle`.
  void setAccount(Account account) {
    state = SessionState.authenticated(account: account);
  }

  /// Sign out: best-effort `Logout`, then clear locally and return to entry —
  /// the local session is cleared even if the RPC cannot reach the backend.
  Future<void> signOut() async {
    final stored = await _tokens.readTokens();
    if (stored != null) {
      try {
        await _auth.logout(stored.refreshToken);
      } catch (_) {
        // Offline or already-revoked: fall through to local clear.
      }
    }
    await _tokens.clear();
    state = const SessionState.unauthenticated();
  }

  /// Local-only teardown after account deletion (the caller already invoked
  /// `DeleteAccount`).
  Future<void> onAccountDeleted() async {
    await _tokens.clear();
    state = const SessionState.unauthenticated();
  }

  /// Leave an in-progress sign-in from the handle gate. A **brand-new** account
  /// (just provisioned, no handle yet) is deleted so it does not linger as an
  /// orphan; an established account (already has a handle) is only signed out.
  /// The local session is cleared regardless — even if the backend call cannot
  /// be reached (offline), the server-side reaper purges any orphan left behind.
  Future<void> abandonOnboarding() async {
    final session = state;
    final brandNew =
        session is SessionAuthenticated &&
        (session.account?.needsHandle ?? false);
    if (!brandNew) {
      await signOut();
      return;
    }
    try {
      await _account.deleteAccount();
    } catch (_) {
      // Best-effort: leave even if DeleteAccount can't reach the backend.
    }
    await _tokens.clear();
    state = const SessionState.unauthenticated();
  }
}

/// Whether the current session is a guest (spec: guest gating). Online-bound
/// features watch this to prompt sign-in instead of calling the backend.
@riverpod
bool isGuestSession(Ref ref) =>
    ref.watch(sessionNotifierProvider) is SessionGuest;

/// Whether an online (backend-bound) service may run for the current session —
/// true only when authenticated. The `requiresAccount` guard for a feature is
/// `!ref.watch(canUseOnlineServicesProvider)`: when blocked, the UI offers to
/// sign in rather than reaching Cymbra ID (design D7).
@riverpod
bool canUseOnlineServices(Ref ref) =>
    ref.watch(sessionNotifierProvider) is SessionAuthenticated;
