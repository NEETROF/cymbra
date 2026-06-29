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

import '../services/auth_service.dart';
import '../services/grpc_client.dart';
import '../services/oidc_token_source.dart';
import 'session_notifier.dart';

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

  Future<void> signUp({required String email, required String password}) =>
      _auth.signUpLocal(email: email, password: password);

  Future<void> verifyEmail(String code) => _auth.verifyEmail(code);

  Future<void> resendVerification(String email) =>
      _auth.resendVerification(email);

  Future<void> requestPasswordReset(String email) =>
      _auth.requestPasswordReset(email);

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
  Future<bool> signInWithGoogle({bool forceChooser = false}) =>
      _signInOidc(_oidc.googleIdToken(forceChooser: forceChooser));

  /// Sign in with Apple. Returns false if the user cancelled.
  Future<bool> signInWithApple() => _signInOidc(_oidc.appleIdToken());

  Future<bool> _signInOidc(Future<String?> idTokenFuture) async {
    final idToken = await idTokenFuture;
    if (idToken == null) return false; // user-cancel is a no-op
    final tokens = await _auth.signInOidc(idToken);
    await _session.onSignedIn(tokens);
    return true;
  }
}

/// Provider for the [AuthFlow] façade.
@riverpod
AuthFlow authFlow(Ref ref) => AuthFlow(ref);
