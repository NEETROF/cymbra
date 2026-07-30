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
import '../services/oidc_token_source.dart';
import 'pending_social_link.dart';
import 'session_notifier.dart';
import 'session_state.dart';

part 'auth_flow.g.dart';

/// Imperative façade over the auth use-cases the screens drive (sign-up, verify,
/// the three sign-in methods, password reset). Each sign-in funnels through
/// [SessionNotifier.onSignedIn] so the post-auth handle-onboarding gate runs
/// once for every method (design D4). Failures throw [AuthException]; the OIDC
/// methods return `false` when the user cancels the native sheet.
class AuthFlow {
  AuthFlow(this._ref);

  final Ref _ref;

  AuthService get _auth => _ref.read(authServiceProvider);
  OidcTokenSource get _oidc => _ref.read(oidcTokenSourceProvider);
  SessionNotifier get _session => _ref.read(sessionNotifierProvider.notifier);
  PendingSocialLinkController get _pendingLink =>
      _ref.read(pendingSocialLinkControllerProvider.notifier);

  Future<void> signUp({
    required String email,
    required String password,
    String? locale,
  }) => _auth.signUpLocal(email: email, password: password, locale: locale);

  Future<void> verifyEmail(String code) => _auth.verifyEmail(code);

  Future<void> resendVerification(String email, {String? locale}) =>
      _auth.resendVerification(email, locale: locale);

  Future<void> requestPasswordReset(String email, {String? locale}) =>
      _auth.requestPasswordReset(email, locale: locale);

  Future<void> resetPassword({
    required String code,
    required String newPassword,
  }) => _auth.resetPassword(code: code, newPassword: newPassword);

  /// Sign in with email + password and adopt the session.
  Future<void> signInEmail({
    required String email,
    required String password,
  }) async {
    final tokens = await _auth.signInLocal(email: email, password: password);
    await _session.onSignedIn(tokens);
  }

  /// Sign in with Google. Returns false if the user dismissed the native sheet.
  /// [forceChooser] re-shows the account picker (re-authentication, e.g. delete).
  Future<bool> signInWithGoogle({bool forceChooser = false}) => _signInOidc(
    LinkedIdentity.providerGoogle,
    _oidc.googleIdToken(forceChooser: forceChooser),
  );

  /// Sign in with Apple. Returns false if the user cancelled.
  Future<bool> signInWithApple() =>
      _signInOidc(LinkedIdentity.providerApple, _oidc.appleIdToken());

  Future<bool> _signInOidc(
    String provider,
    Future<String?> idTokenFuture,
  ) async {
    final idToken = await idTokenFuture;
    if (idToken == null) return false; // user-cancel is a no-op
    final tokens = await _auth.signInOidc(idToken);
    await _session.onSignedIn(tokens);
    // If this social sign-in provisioned a brand-new account (it lands on handle
    // onboarding), remember its `id_token` so the user can instead link it to a
    // pre-existing account from the collision point (D7). An established account
    // clears any stale pending link.
    if (_isBrandNew) {
      _pendingLink.set(PendingSocialLink(idToken: idToken, provider: provider));
    } else {
      _pendingLink.clear();
    }
    return true;
  }

  /// Whether the resolved session is a just-provisioned account with no handle.
  bool get _isBrandNew {
    final session = _ref.read(sessionNotifierProvider);
    return session is SessionAuthenticated &&
        (session.account?.needsHandle ?? false);
  }

  /// Collision resolution (D7): link the pending social orphan onto an existing
  /// account the user proves ownership of via [authenticateExisting] (which
  /// returns that account's tokens). Ordering is critical: authenticate the
  /// existing account first (a wrong credential aborts with the orphan intact),
  /// then **delete the orphan** so its `(provider, subject)` is freed, adopt the
  /// existing session, and finally `LinkIdentity` — re-minting the social
  /// `id_token` if it expired in the meantime. On success the pending link is
  /// cleared and the user lands on their existing account.
  Future<void> linkExisting({
    required PendingSocialLink pending,
    required Future<AuthTokens> Function() authenticateExisting,
  }) async {
    final existing = await authenticateExisting();
    // Delete the orphan while it is still the active session (its bearer authorises
    // the delete), freeing the social identity before we try to attach it.
    await _session.deleteOrphanForLink();
    await _session.onSignedIn(existing);
    await _linkWithRemint(pending);
    _pendingLink.clear();
  }

  /// Attach [pending] to the current (existing) account, re-minting a fresh
  /// `id_token` once if the captured one is no longer valid by link time.
  Future<void> _linkWithRemint(PendingSocialLink pending) async {
    try {
      await _auth.linkIdentity(pending.idToken);
    } on AuthException catch (e) {
      // A stale/expired social token verifies as unauthenticated / invalid — the
      // only recoverable link failure. Re-mint from the same provider and retry.
      final recoverable =
          e.error == AuthError.unauthenticated ||
          e.error == AuthError.invalidArgument;
      if (!recoverable) rethrow;
      final fresh = await _mint(pending.provider);
      if (fresh == null) rethrow; // user cancelled the re-mint sheet
      await _auth.linkIdentity(fresh);
    }
  }

  Future<String?> _mint(String provider) =>
      provider == LinkedIdentity.providerApple
      ? _oidc.appleIdToken()
      : _oidc.googleIdToken();

  /// Authenticate the existing account with email + password (returns its tokens
  /// without adopting them — [linkExisting] sequences the session change).
  Future<AuthTokens> reauthEmail({
    required String email,
    required String password,
  }) => _auth.signInLocal(email: email, password: password);

  /// Authenticate the existing account with an OIDC [provider] (`google`/`apple`),
  /// or throw [AuthException] `unauthenticated` if the user cancels the sheet.
  Future<AuthTokens> reauthOidc(String provider) async {
    final idToken = await _mint(provider);
    if (idToken == null) {
      // Model a cancelled sheet as an auth failure so [linkExisting] aborts with
      // the orphan intact (no session change has happened yet).
      throw const AuthException(AuthError.unauthenticated);
    }
    return _auth.signInOidc(idToken);
  }
}

/// Provider for the [AuthFlow] façade.
@riverpod
AuthFlow authFlow(Ref ref) => AuthFlow(ref);
