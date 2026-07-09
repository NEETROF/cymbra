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

// Pure, host-testable core for the desktop browser-loopback OAuth flow (design
// D1/D4). No `dart:io`, no browser, no network — just PKCE, the authorization-URL
// builder, `state` generation/validation, and redirect-query parsing, so the
// security-sensitive logic is covered by fast unit tests. The `dart:io`
// `HttpServer`, the browser launch, and the token HTTP exchange live in the
// adapter behind the seam (see desktop_oidc_token_source.dart).

import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';

/// Google's OAuth 2.0 authorization and token endpoints.
const String kGoogleAuthEndpoint =
    'https://accounts.google.com/o/oauth2/v2/auth';
const String kGoogleTokenEndpoint = 'https://oauth2.googleapis.com/token';

/// A desktop loopback flow that failed for a reason that is **not** a user
/// cancellation — a `state` mismatch, a missing `code`, a browser that wouldn't
/// open, or a rejected token exchange. Surfaced (not swallowed) so the UI shows
/// an error instead of looking like a silent no-op. [reason] is a short,
/// non-sensitive label — never the code, verifier, or token.
class DesktopOauthException implements Exception {
  final String reason;
  const DesktopOauthException(this.reason);

  @override
  String toString() => 'DesktopOauthException($reason)';
}

/// base64url **without** padding, per RFC 7636 (PKCE) and RFC 6749 `state`.
String base64UrlNoPad(List<int> bytes) =>
    base64Url.encode(bytes).replaceAll('=', '');

/// Draw [count] cryptographically random bytes from [rng].
Uint8List randomBytes(int count, Random rng) {
  final out = Uint8List(count);
  for (var i = 0; i < count; i++) {
    out[i] = rng.nextInt(256);
  }
  return out;
}

/// A PKCE verifier + its S256 challenge (RFC 7636).
class PkcePair {
  /// The high-entropy `code_verifier` (43–128 chars, base64url).
  final String verifier;

  /// The `code_challenge` = base64url(SHA-256(verifier)).
  final String challenge;

  const PkcePair({required this.verifier, required this.challenge});
}

/// The S256 `code_challenge` for a given `code_verifier`.
String pkceChallengeFor(String verifier) =>
    base64UrlNoPad(sha256.convert(ascii.encode(verifier)).bytes);

/// Build a [PkcePair] from raw verifier bytes (32 bytes ⇒ a 43-char verifier) —
/// separated from randomness so tests can pass fixed bytes and assert the
/// challenge deterministically.
PkcePair pkcePairFromBytes(List<int> verifierBytes) {
  final verifier = base64UrlNoPad(verifierBytes);
  return PkcePair(verifier: verifier, challenge: pkceChallengeFor(verifier));
}

/// Generate a PKCE pair using [rng] (default 32 bytes of entropy).
PkcePair generatePkcePair(Random rng, {int bytes = 32}) =>
    pkcePairFromBytes(randomBytes(bytes, rng));

/// Generate an opaque anti-CSRF `state` token from [rng].
String generateState(Random rng, {int bytes = 32}) =>
    base64UrlNoPad(randomBytes(bytes, rng));

/// Build Google's authorization URL for the loopback flow. [redirectUri] is the
/// app's `http://127.0.0.1:<port>` callback; [scopes] defaults to `openid email`.
/// When [forceChooser] is true, `prompt=select_account` makes Google show the
/// account picker (re-auth / switch account).
Uri buildAuthorizationUrl({
  required String clientId,
  required String redirectUri,
  required String state,
  required String codeChallenge,
  List<String> scopes = const ['openid', 'email'],
  bool forceChooser = false,
}) {
  final params = <String, String>{
    'client_id': clientId,
    'redirect_uri': redirectUri,
    'response_type': 'code',
    'scope': scopes.join(' '),
    'state': state,
    'code_challenge': codeChallenge,
    'code_challenge_method': 'S256',
  };
  if (forceChooser) params['prompt'] = 'select_account';
  return Uri.parse(kGoogleAuthEndpoint).replace(queryParameters: params);
}

/// The parsed outcome of the browser redirect hitting the loopback server.
sealed class RedirectResult {
  const RedirectResult();
}

/// A successful redirect carrying the authorization `code` (state already OK).
class RedirectSuccess extends RedirectResult {
  final String code;
  const RedirectSuccess(this.code);
}

/// The redirect reported an error, the `state` mismatched, or `code` was absent.
/// [reason] is a short, non-sensitive label (never the code/verifier/token).
class RedirectFailure extends RedirectResult {
  final String reason;
  const RedirectFailure(this.reason);
}

/// Parse the loopback redirect [uri] against the [expectedState]. Validates the
/// `state` first (CSRF), then surfaces an `error` param or the `code`.
RedirectResult parseRedirect(Uri uri, {required String expectedState}) {
  final q = uri.queryParameters;
  final state = q['state'];
  if (state == null || state != expectedState) {
    return const RedirectFailure('state_mismatch');
  }
  final error = q['error'];
  if (error != null && error.isNotEmpty) {
    return RedirectFailure(error);
  }
  final code = q['code'];
  if (code == null || code.isEmpty) {
    return const RedirectFailure('missing_code');
  }
  return RedirectSuccess(code);
}
