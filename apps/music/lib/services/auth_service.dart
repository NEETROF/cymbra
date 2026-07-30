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

import 'token_store.dart';

/// The audience every Cymbra ID sign-in is scoped to (spec: "Sign in scoped to
/// the music audience"). The app never requests another audience.
const String kMusicAudience = 'music';

/// An access/refresh token pair returned by a successful sign-in or refresh.
class AuthTokens {
  final String accessToken;
  final String refreshToken;

  const AuthTokens({required this.accessToken, required this.refreshToken});

  StoredTokens toStored() =>
      StoredTokens(accessToken: accessToken, refreshToken: refreshToken);
}

/// Categories of auth/account failure the UI distinguishes, decoupled from the
/// gRPC status codes (so screens, notifiers, and fakes never import grpc).
enum AuthError {
  /// Wrong credentials or an invalid/expired token (gRPC `UNAUTHENTICATED`).
  unauthenticated,

  /// Email or handle already taken (gRPC `ALREADY_EXISTS`).
  alreadyExists,

  /// Rate limit or account lockout (gRPC `RESOURCE_EXHAUSTED`).
  rateLimited,

  /// State precondition unmet, e.g. email not yet verified (`FAILED_PRECONDITION`).
  failedPrecondition,

  /// Malformed input or an invalid/expired OTP code (gRPC `INVALID_ARGUMENT`).
  invalidArgument,

  /// Optimistic-concurrency conflict, e.g. a handle claimed mid-flight (`ABORTED`).
  conflict,

  /// Resource missing (gRPC `NOT_FOUND`).
  notFound,

  /// Backend unreachable / offline (gRPC `UNAVAILABLE`).
  unavailable,

  /// Anything else.
  unknown,
}

/// Maps a gRPC status code (stable integer) to an [AuthError]. Pure, so it is
/// unit-tested without a grpc dependency; the gRPC adapters call it.
AuthError authErrorFromCode(int grpcCode) {
  switch (grpcCode) {
    case 3:
      return AuthError.invalidArgument;
    case 5:
      return AuthError.notFound;
    case 6:
      return AuthError.alreadyExists;
    case 8:
      return AuthError.rateLimited;
    case 9:
      return AuthError.failedPrecondition;
    case 10:
      return AuthError.conflict;
    case 14:
      return AuthError.unavailable;
    case 16:
      return AuthError.unauthenticated;
    default:
      return AuthError.unknown;
  }
}

/// A categorized auth/account failure surfaced to the UI.
class AuthException implements Exception {
  final AuthError error;
  final String? message;

  const AuthException(this.error, [this.message]);

  @override
  String toString() =>
      'AuthException($error${message == null ? '' : ': $message'})';
}

/// Seam over Cymbra ID's `AuthService` (task 3.2). All sign-in calls are scoped
/// to the `music` audience. The production implementation talks gRPC; tests
/// override the provider with an in-memory fake. Failures throw [AuthException].
abstract class AuthService {
  /// Create a local (email + password) account. Advances to verification.
  /// [locale] (e.g. `"fr"`) localizes the verification email; null = server default.
  Future<void> signUpLocal({
    required String email,
    required String password,
    String? locale,
  });

  /// Verify a newly-registered email with the emailed OTP [code].
  Future<void> verifyEmail(String code);

  /// Re-send the verification code to [email]. [locale] localizes the email.
  Future<void> resendVerification(String email, {String? locale});

  /// Sign in with email + password (audience `music`). Returns the session.
  Future<AuthTokens> signInLocal({
    required String email,
    required String password,
  });

  /// Exchange a provider [idToken] (Google/Apple) for a session (audience
  /// `music`). New accounts are auto-provisioned on first sign-in.
  Future<AuthTokens> signInOidc(String idToken);

  /// Revoke the refresh token server-side (best-effort on sign-out).
  Future<void> logout(String refreshToken);

  /// Revoke **all** of the caller's sessions server-side ("sign out from all
  /// devices"). Authenticated by the access token — no refresh token argument.
  /// This kills the current device's session too, so the caller tears down the
  /// local session on success.
  Future<void> revokeAllSessions();

  /// Begin a password reset for [email] (no account enumeration). [locale]
  /// localizes the reset email.
  Future<void> requestPasswordReset(String email, {String? locale});

  /// Complete a password reset with the emailed [code] and a new password.
  Future<void> resetPassword({
    required String code,
    required String newPassword,
  });

  /// Attach an additional sign-in identity (Google/Apple) to the **current**
  /// account (change: add-account-identity-linking). [idToken] is a fresh
  /// provider `id_token`; the identity binds to the caller's session (bearer),
  /// not a new account. A social identity that already belongs to a *different*
  /// account surfaces as [AuthError.alreadyExists] (the app does not merge).
  Future<void> linkIdentity(String idToken);

  /// Remove a sign-in identity from the current account. The backend refuses to
  /// unlink the **last** remaining identity (anti-lockout) with
  /// [AuthError.failedPrecondition]. [provider]/[subject] come from
  /// [AccountService.listIdentities]; the subject is opaque and never displayed.
  Future<void> unlinkIdentity({
    required String provider,
    required String subject,
  });

  /// Add a local (email + password) credential to the **current** account so it
  /// can also sign in with email. The credential is created **unverified** and a
  /// verification email is sent — the password is usable only after the email is
  /// confirmed (mirrors sign-up). [AuthError.alreadyExists] when the account
  /// already has a password or the email belongs to another account.
  Future<void> setLocalCredential({
    required String email,
    required String password,
    String? locale,
  });
}
