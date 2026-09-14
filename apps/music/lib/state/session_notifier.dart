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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../analytics/usage_actions.dart';
import '../services/account_service.dart';
import '../services/auth_service.dart';
import '../services/grpc_client.dart';
import '../services/offline_score_cache.dart';
import '../services/offline_server_secret_service.dart';
import '../services/oidc_token_source.dart';
import '../services/token_store.dart';
import 'favorites_index_store.dart';
import 'session_state.dart';
import 'usage_tracking_notifier.dart';

part 'session_notifier.g.dart';

/// Backoff for re-resolving a degraded session (change: fix-session-account-
/// retry). The first re-attempt waits [kAccountRetryInitialDelay]; each
/// subsequent one doubles, capped at [kAccountRetryMaxDelay]. At the cap that is
/// one `GetAccount` per minute — each already bounded by the interactive RPC
/// deadline — and only while the app is in the foreground.
const Duration kAccountRetryInitialDelay = Duration(seconds: 2);

/// Ceiling for the retry backoff. Deliberately not "give up": an app left open
/// on a desktop for hours must still recover on its own.
const Duration kAccountRetryMaxDelay = Duration(seconds: 60);

/// Single source of truth for the account session (design D2). Hydrates from
/// secure storage at startup and exposes the transitions the auth flows call.
/// `home` in [CymbraApp] switches on the resolved [SessionState].
@Riverpod(keepAlive: true)
class SessionNotifier extends _$SessionNotifier {
  TokenStore get _tokens => ref.read(tokenStoreProvider);
  AuthService get _auth => ref.read(authServiceProvider);
  AccountService get _account => ref.read(accountServiceProvider);
  OidcTokenSource get _oidc => ref.read(oidcTokenSourceProvider);

  /// The pending re-attempt for a degraded session, or null when none is armed.
  /// A [Timer] (not `Future.delayed`) precisely because it can be cancelled —
  /// a delayed future's continuation would still fire after backgrounding.
  Timer? _accountRetry;

  /// The delay the *next* re-attempt will wait; doubles on each failure and is
  /// reset by [_stopAccountRetry].
  Duration _retryDelay = kAccountRetryInitialDelay;

  /// The in-flight account resolution, shared by every concurrent trigger
  /// (timer, foreground, user-tapped retry) so they never issue a second
  /// `GetAccount` — same single-flight shape as [CoordinatedTokenRefresher].
  Future<void>? _resolving;

  /// Whether the app is in the foreground. Retries are armed only while true:
  /// a backgrounded desktop app keeps executing (unlike a frozen mobile
  /// process), so an ungated loop would spend battery and RPCs unseen.
  bool _foreground = true;

  @override
  SessionState build() {
    // A keepAlive notifier outlives every screen, so the timer it owns must die
    // with it — the backstop behind the per-teardown cancellations below.
    ref.onDispose(_stopAccountRetry);
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

  /// Fetch the account for a token-bearing session, coalescing concurrent
  /// callers into one `GetAccount` (change: fix-session-account-retry). A
  /// revoked session clears the store and routes to entry; a transient failure
  /// keeps the user signed in with an unresolved account **and schedules a
  /// re-attempt** — that degraded state is never a resting place.
  Future<void> _resolveAuthenticated() {
    final existing = _resolving;
    if (existing != null) return existing;
    final started = _runResolve();
    _resolving = started;
    started.whenComplete(() {
      if (identical(_resolving, started)) _resolving = null;
    });
    return started;
  }

  Future<void> _runResolve() async {
    try {
      final account = await _account.getAccount();
      _stopAccountRetry(); // resolved: cancel the loop and reset the backoff
      state = SessionState.authenticated(account: account);
    } on AuthException catch (e) {
      // A revoked session (`unauthenticated`) or a deleted account (`notFound`:
      // the token still verifies by signature/expiry, but the user row is gone)
      // are both terminal — clear the local session and route to entry. This
      // holds inside a re-attempt too: the loop can never keep a dead session
      // alive. Any other error is offline/transient: stay signed in with an
      // unresolved account so a flaky network doesn't sign the user out, and
      // retry until it resolves.
      if (e.error == AuthError.unauthenticated ||
          e.error == AuthError.notFound) {
        _stopAccountRetry();
        await _tokens.clear();
        state = const SessionState.unauthenticated();
      } else {
        state = const SessionState.authenticated();
        _scheduleAccountRetry();
      }
    }
  }

  /// Re-resolve the account of a degraded session. The public entry point for
  /// both the lifecycle hooks and the user-facing retry on the profile screen;
  /// a no-op unless the session is [SessionState.accountUnresolved], so callers
  /// can invoke it without inspecting the session shape.
  Future<void> refreshAccount() async {
    if (!state.accountUnresolved) return;
    await _resolveAuthenticated();
  }

  /// Arm the next re-attempt, doubling the backoff. Never arms while the app is
  /// out of the foreground — [onForeground] restarts the loop on return.
  void _scheduleAccountRetry() {
    if (!_foreground) return;
    _accountRetry?.cancel();
    final delay = _retryDelay;
    _retryDelay = _nextRetryDelay(delay);
    _accountRetry = Timer(delay, () {
      _accountRetry = null;
      // The session may have resolved or ended while the timer was pending.
      if (!state.accountUnresolved) return;
      unawaited(_resolveAuthenticated());
    });
  }

  /// Cancel any pending re-attempt and reset the backoff to its first delay.
  /// Called from every session teardown and from `ref.onDispose`.
  void _stopAccountRetry() {
    _accountRetry?.cancel();
    _accountRetry = null;
    _retryDelay = kAccountRetryInitialDelay;
  }

  Duration _nextRetryDelay(Duration current) {
    final doubled = current * 2;
    return doubled > kAccountRetryMaxDelay ? kAccountRetryMaxDelay : doubled;
  }

  /// The app left the foreground: stop retrying. A resolution already in flight
  /// runs to completion (it is bounded by the RPC deadline and cannot be
  /// cancelled mid-call), but it will not arm a successor. The session itself is
  /// untouched — backgrounding never signs anyone out.
  void onBackground() {
    _foreground = false;
    _stopAccountRetry();
  }

  /// The app returned to the foreground: if the session is still degraded, retry
  /// **immediately** and reset the backoff. A user coming back after an hour
  /// must not inherit the delay accumulated before they left, and a foreground
  /// return is the likeliest moment for connectivity to have changed.
  Future<void> onForeground() async {
    _foreground = true;
    if (!state.accountUnresolved) return;
    _stopAccountRetry();
    await _resolveAuthenticated();
  }

  /// Persist the guest choice and enter guest mode (no backend calls).
  Future<void> continueAsGuest() async {
    _stopAccountRetry();
    await _tokens.setGuest();
    state = const SessionState.guest();
  }

  /// Leave guest mode and return to the entry screen so the user can sign in.
  Future<void> leaveGuest() async {
    _stopAccountRetry();
    await _tokens.clear();
    state = const SessionState.unauthenticated();
  }

  /// Adopt a freshly-obtained session (from any sign-in path): store the tokens
  /// and resolve the account (which gates handle onboarding).
  Future<void> onSignedIn(AuthTokens tokens) async {
    // A new session never inherits the previous one's backoff.
    _stopAccountRetry();
    await _tokens.writeTokens(tokens.toStored());
    state = const SessionState.unknown();
    await _resolveAuthenticated();
    // Usage telemetry (change: add-feature-usage-analytics): every sign-in funnels
    // here, so this is the single point that distinguishes a brand-new account
    // (its first authenticated appearance = a sign-up) from a returning one.
    final resolved = state;
    if (resolved is SessionAuthenticated) {
      final isNew = resolved.account?.needsHandle ?? false;
      unawaited(
        ref
            .read(usageTrackingNotifierProvider.notifier)
            .record(isNew ? UsageActions.authSignUp : UsageActions.authSignIn),
      );
    }
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
    await _endLocalSession();
  }

  /// Sign out of **all** the account's devices. Revokes every session
  /// server-side, then — **only on success** — tears down the local session
  /// (the current device's session is revoked too). Unlike [signOut]'s
  /// best-effort logout, a failed revoke rethrows and leaves the user signed in:
  /// we don't sign the current device out locally while other devices stay live.
  Future<void> signOutEverywhere() async {
    await _auth.revokeAllSessions();
    await _endLocalSession();
  }

  /// Shared local teardown: forget the cached OIDC account (best-effort, never
  /// blocks on the native SDK), clear the stored tokens, and return to entry.
  Future<void> _endLocalSession() async {
    _stopAccountRetry();
    try {
      await _oidc.signOut();
    } catch (_) {
      // Best-effort: never block local sign-out on the native SDK.
    }
    await _purgeOfflineData();
    await _tokens.clear();
    state = const SessionState.unauthenticated();
  }

  /// Purge the offline score cache + its key material and clear this user's
  /// plaintext favorites-index snapshot and cached server secret (change:
  /// add-offline-score-cache). Read the user id *before* the session is cleared.
  /// Best-effort: a storage hiccup never blocks sign-out.
  Future<void> _purgeOfflineData() async {
    try {
      final userId = ref.read(currentUserIdProvider);
      await ref.read(offlineScoreCacheProvider).purgeAll();
      if (userId != null) {
        await ref.read(favoritesIndexStoreProvider).clear(userId);
        await ref.read(offlineServerSecretServiceProvider).clear(userId);
      }
    } catch (_) {
      // Never block teardown on cache cleanup.
    }
  }

  /// Local-only teardown after account deletion (the caller already invoked
  /// `DeleteAccount`).
  /// Delete the caller's account (server-side) and end the local session. Owns the
  /// account-service call so the UI never touches the service directly; the caller
  /// re-authenticates first (deletion is destructive).
  Future<void> deleteAccount() async {
    await _account.deleteAccount();
    await onAccountDeleted();
  }

  Future<void> onAccountDeleted() async {
    _stopAccountRetry();
    await _purgeOfflineData();
    await _tokens.clear();
    state = const SessionState.unauthenticated();
  }

  /// Delete the just-created social **orphan** during the collision-link flow
  /// (change: add-account-identity-linking, D7): removes the brand-new account
  /// server-side — freeing its `(provider, subject)` before `LinkIdentity` — and
  /// clears the local session. Unlike [abandonOnboarding] this is **not**
  /// best-effort: a failed delete rethrows so the caller aborts the link (the
  /// social identity would still be owned by the orphan → `ALREADY_EXISTS`). The
  /// caller immediately adopts the existing account via [onSignedIn], so this
  /// deliberately does not route to the entry screen.
  Future<void> deleteOrphanForLink() async {
    _stopAccountRetry();
    await _account.deleteAccount();
    await _tokens.clear();
    state = const SessionState.unknown();
  }

  /// Leave an in-progress sign-in from the handle gate. A **brand-new** account
  /// (just provisioned, no handle yet) is deleted so it does not linger as an
  /// orphan; an established account (already has a handle) is only signed out.
  /// The local session is cleared regardless — even if the backend call cannot
  /// be reached (offline), the server-side reaper purges any orphan left behind.
  Future<void> abandonOnboarding() async {
    _stopAccountRetry();
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

/// The signed-in user's handle (for upload attribution), or `null` when guest /
/// signed out / not yet chosen.
@riverpod
String? currentUserHandle(Ref ref) {
  final session = ref.watch(sessionNotifierProvider);
  return session is SessionAuthenticated ? session.account?.handle : null;
}

/// The signed-in user's account id, or `null` when guest / signed out / the
/// account is not yet resolved. Used for per-user play-outbox delivery (change:
/// add-play-activity-profile): a captured session is tied to this id and only
/// delivered while it is the signed-in one.
@riverpod
String? currentUserId(Ref ref) {
  final session = ref.watch(sessionNotifierProvider);
  return session is SessionAuthenticated ? session.account?.userId : null;
}
